// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
//! `TransactionHandler`: transaction-mode pooling per the TD Logic
//! flowchart and Pool Lease State Machine — frontend admission, a one-time
//! real startup+auth handshake against a freshly acquired backend
//! (immediately reset and returned to idle, so the client holds no backend
//! lease between transactions), then a loop that leases a backend per
//! non-`Terminate` frontend frame and relays verbatim until the backend
//! reports `ReadyForQuery(Idle)` (reset + return to idle, loop back) or the
//! leg ends via `Terminate`/EOF/`FrameError` (release `Close`, close the
//! client). Drain is honored transparently by `server_tcp::serve_arc`'s
//! outer accept loop + bounded `drain_timeout` task abandonment — this
//! handler references `cx.drain` nowhere, exactly like `SessionHandler`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use bytes::BytesMut;
use server_lifecycle::ConnectionBudget;
use server_tcp::{ConnectionContext, TcpHandler};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::pool::backend_pool::{BackendLease, BackendPool, StartupAdmission};
use crate::pool::telemetry::{TransactionPhase, TransactionPhaseOutcome};
use crate::pool::types::{BackendConnectionId, LeaseDisposition, PoolRejectionReason};
use crate::pool::TransactionReactor;
use crate::proxy::{
    forward_backend, forward_backend_batch, forward_frontend, forward_raw,
    read_backend_relay_batch_with_raw, read_relay_frame_with_raw, read_startup, relay_until_ready,
    HandshakeOutcome, RejectionReason,
};
use crate::wire::{
    FrameReader, FrontendMessage, RelayFrame, RelayFrameKind, Role, TransactionStatus,
    WireCodecConfig,
};

/// Full configuration for one `TransactionHandler`, per the TD Schema
/// section: its own frontend admission budget (mirroring
/// `SessionProxyConfig.frontend_budget`, deliberately not wired into
/// `server_tcp::TcpServerConfig.connection_budget` for the same reason —
/// see `crate::proxy::SessionProxyConfig`), the shared backend pool, wire
/// bounds, and the drain timeout `pgpool serve` also feeds into
/// `TcpServerConfig` (not referenced by `TransactionHandler` itself; drain
/// is purely a `server_tcp::serve_arc`-level concern).
#[derive(Debug, Clone)]
pub struct TransactionProxyConfig {
    pub frontend_budget: ConnectionBudget,
    pub backend_pool: BackendPool,
    pub wire: WireCodecConfig,
    pub drain_timeout: Duration,
}

/// Transaction-mode `server_tcp::TcpHandler` impl `pgpool serve` binds to
/// its listener when `RuntimePlan::pool_mode` is `PoolMode::Transaction`.
/// Private field, constructed via `TransactionHandler::new(config)`;
/// mirrors `SessionHandler`'s shape.
#[derive(Debug, Clone)]
pub struct TransactionHandler {
    config: TransactionProxyConfig,
    engine: TransactionEngine,
}

#[derive(Debug, Clone)]
enum TransactionEngine {
    Legacy,
    Reactor(TransactionReactor),
}

impl TransactionHandler {
    // @spec apps/pgpool/tech-design/logic/p0-dense-buffer-readiness-reactor.md#logic
    pub fn new(config: TransactionProxyConfig) -> Self {
        if let Some(policy) = config.backend_pool.reserve_policy() {
            tracing::info!(
                reserve_pool_timeout_seconds = policy.reserve_pool_timeout_seconds,
                queue_wait_timeout_seconds = policy.queue_wait_timeout_seconds,
                reserve_idle_timeout_seconds = policy.reserve_idle_timeout_seconds,
                "pgpool reserve lease cache enabled; control-plane exchange stays off the relay path"
            );
        }
        // The single-owner readiness reactor is the production transaction
        // data path. Keep an explicit legacy escape hatch for operational
        // rollback while the session-mode Tokio handler remains unchanged.
        let engine = if std::env::var("PGPOOL_TRANSACTION_ENGINE").as_deref() == Ok("legacy") {
            TransactionEngine::Legacy
        } else {
            match TransactionReactor::start(config.backend_pool.clone()) {
                Ok(reactor) => TransactionEngine::Reactor(reactor),
                Err(error) => {
                    tracing::warn!(%error, "pgpool transaction reactor unavailable; using legacy handler");
                    TransactionEngine::Legacy
                }
            }
        };
        Self { config, engine }
    }

    pub fn config(&self) -> &TransactionProxyConfig {
        &self.config
    }
}

impl TcpHandler for TransactionHandler {
    // Mirrors `SessionHandler`'s boxed-future choice: a plain, nameable
    // handler type per the TD Schema section instead of an unstable `impl
    // Trait` associated type.
    type Future = Pin<Box<dyn Future<Output = Result<()>> + Send>>;

    fn handle(&self, stream: TcpStream, cx: ConnectionContext) -> Self::Future {
        let config = self.config.clone();
        let engine = self.engine.clone();
        Box::pin(async move {
            match engine {
                TransactionEngine::Legacy => {
                    run_transaction_client(stream, &config, cx.peer_addr).await;
                }
                TransactionEngine::Reactor(reactor) => {
                    let permit = match config.frontend_budget.try_acquire() {
                        Ok(permit) => permit,
                        Err(_) => {
                            let mut client = stream;
                            write_rejection(&mut client, RejectionReason::FrontendBudgetExhausted)
                                .await;
                            return Ok(());
                        }
                    };
                    match reactor.handoff(stream, permit) {
                        Ok(done) => {
                            // Keep the tcp-server task alive for the reactor
                            // client's real lifetime. Drain therefore stops
                            // accepting new frontends but still waits for an
                            // already-open transaction to commit/close.
                            let _ = done.await;
                        }
                        Err(_) => {
                            tracing::info!(
                                peer = %cx.peer_addr,
                                outcome = "reactor_handoff_failed",
                                "pgpool transaction reactor handoff failed"
                            );
                        }
                    }
                }
            }
            Ok(())
        })
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1882" reason="logic section in transaction.rs is hand-written pending codegen support">
/// One accepted frontend connection's full transaction-mode lifecycle.
/// Never panics: every rejection/error path writes (or forwards) the
/// appropriate wire frame, releases the frontend permit, and releases any
/// held backend lease before returning.
async fn run_transaction_client(
    client: TcpStream,
    config: &TransactionProxyConfig,
    peer_addr: std::net::SocketAddr,
) {
    let permit = match config.frontend_budget.try_acquire() {
        Ok(permit) => permit,
        Err(_) => {
            let mut client = client;
            write_rejection(&mut client, RejectionReason::FrontendBudgetExhausted).await;
            tracing::info!(
                peer = %peer_addr,
                outcome = "rejected_saturated_frontend",
                "pgpool transaction admission rejected"
            );
            return;
        }
    };

    let (mut client_read, mut client_write) = client.into_split();
    let mut frontend_reader = FrameReader::new(Role::Frontend, &config.wire);

    // @spec apps/pgpool/tech-design/logic/trust-startup-replay-for-capped-transaction-pooling.md#logic
    // Decode startup before asking the pool for capacity. A matching,
    // challenge-free reply can therefore establish a frontend without taking
    // a physical backend from the capped pool.
    let startup =
        match read_startup(&mut client_read, &mut client_write, &mut frontend_reader).await {
            Ok(startup) => startup,
            Err(_) => {
                drop(permit);
                return;
            }
        };
    let startup = config.backend_pool.normalize_backend_startup(startup);

    let mut replay_safe_startup = false;
    match config.backend_pool.acquire_for_startup(&startup).await {
        Ok(StartupAdmission::Replay(messages)) => {
            replay_safe_startup = true;
            for message in messages {
                if forward_backend(&mut client_write, &message).await.is_err() {
                    drop(permit);
                    return;
                }
            }
        }
        Ok(StartupAdmission::Fresh(lease)) => {
            let BackendLease {
                id: handshake_id,
                stream: handshake_backend,
                ..
            } = lease;
            let (mut backend_read, mut backend_write) = handshake_backend.into_split();
            let mut backend_reader = FrameReader::new(Role::Backend, &config.wire);

            if forward_frontend(
                &mut backend_write,
                &FrontendMessage::Startup(startup.clone()),
            )
            .await
            .is_err()
            {
                release_backend(
                    config,
                    handshake_id,
                    backend_read,
                    backend_write,
                    LeaseDisposition::Close,
                )
                .await;
                drop(permit);
                return;
            }

            match relay_until_ready(
                &mut client_read,
                &mut client_write,
                &mut backend_read,
                &mut backend_write,
                &mut frontend_reader,
                &mut backend_reader,
            )
            .await
            {
                Ok(HandshakeOutcome::Ready { startup_replay }) => {
                    if let Some(messages) = startup_replay {
                        replay_safe_startup = true;
                        config
                            .backend_pool
                            .publish_startup_replay(startup.clone(), messages);
                    }
                }
                Ok(HandshakeOutcome::Rejected) | Err(_) => {
                    // The backend's ErrorResponse, when present, was already
                    // forwarded verbatim by relay_until_ready.
                    release_backend(
                        config,
                        handshake_id,
                        backend_read,
                        backend_write,
                        LeaseDisposition::Close,
                    )
                    .await;
                    drop(permit);
                    return;
                }
            }

            // The fresh handshake backend must be reset before later
            // transaction leases can reuse it; replay clients hold none.
            release_backend(
                config,
                handshake_id,
                backend_read,
                backend_write,
                LeaseDisposition::ReturnToIdle,
            )
            .await;
        }
        Err(crate::pool::types::PoolError::Saturated { .. }) => {
            write_pool_rejection(&mut client_write, PoolRejectionReason::BackendPoolSaturated)
                .await;
            drop(permit);
            tracing::info!(
                peer = %peer_addr,
                outcome = "rejected_pool_saturated_admission",
                "pgpool transaction admission rejected"
            );
            return;
        }
        Err(crate::pool::types::PoolError::BackendUnreachable(_)) => {
            write_rejection(&mut client_write, RejectionReason::BackendUnreachable).await;
            drop(permit);
            tracing::info!(
                peer = %peer_addr,
                outcome = "rejected_backend_unreachable",
                "pgpool transaction admission rejected"
            );
            return;
        }
    }

    // `await_client_activity` loop: each non-Terminate frontend frame
    // acquires a per-transaction lease and relays until that transaction's
    // `ReadyForQuery(Idle)` or the leg ends. `pending_first_frame` carries
    // a frame the client had already pipelined ahead of the previous leg's
    // `ReadyForQuery` (captured by `relay_one_transaction`, never lost),
    // standing in for a fresh frontend read on this iteration.
    let mut pending_first_frame: Option<RelayFrame> = None;
    loop {
        let first_frame = match pending_first_frame.take() {
            Some(msg) => msg,
            None => match read_relay_frame_with_raw(&mut client_read, &mut frontend_reader).await {
                Ok(Some(frame)) => match frame.kind {
                    RelayFrameKind::FrontendTerminate => {
                        drop(permit);
                        return;
                    }
                    RelayFrameKind::Other => frame,
                    RelayFrameKind::BackendReady(_) => {
                        unreachable!("frontend-role reader only emits Frontend frames")
                    }
                },
                Ok(None) => {
                    drop(permit);
                    return;
                }
                Err(_) => {
                    drop(permit);
                    return;
                }
            },
        };

        // Per the Pool Lease State Machine, `acquiring_transaction` has
        // exactly one rejection edge (`rejected_pool_saturated`): any
        // `PoolError` here (saturation timeout, or a fresh-connect failure
        // inside `acquire()`) is reported to this client as the synthesized
        // pool-saturated rejection.
        let acquire_started = config.backend_pool.transaction_phase_started_at();
        let acquisition = if replay_safe_startup {
            config
                .backend_pool
                .acquire_for_replayed_startup(&startup)
                .await
        } else {
            config.backend_pool.acquire().await
        };
        let lease = match acquisition {
            Ok(lease) => {
                config.backend_pool.record_transaction_phase_started(
                    TransactionPhase::Acquire,
                    TransactionPhaseOutcome::Success,
                    acquire_started,
                );
                lease
            }
            Err(_) => {
                config.backend_pool.record_transaction_phase_started(
                    TransactionPhase::Acquire,
                    TransactionPhaseOutcome::Failure,
                    acquire_started,
                );
                write_pool_rejection(&mut client_write, PoolRejectionReason::BackendPoolSaturated)
                    .await;
                drop(permit);
                return;
            }
        };

        let BackendLease {
            id: txn_id,
            stream: txn_backend,
            ..
        } = lease;
        let (mut txn_backend_read, mut txn_backend_write) = txn_backend.into_split();
        let mut txn_backend_reader = FrameReader::new(Role::Backend, &config.wire);

        let relay_started = config.backend_pool.transaction_phase_started_at();
        let outcome = relay_one_transaction(
            &mut client_read,
            &mut client_write,
            &mut txn_backend_read,
            &mut txn_backend_write,
            &mut frontend_reader,
            &mut txn_backend_reader,
            first_frame,
        )
        .await;
        let relay_outcome = match outcome {
            TxnLegOutcome::ReadyIdle(_) => TransactionPhaseOutcome::Success,
            TxnLegOutcome::Ended => TransactionPhaseOutcome::Failure,
        };
        config.backend_pool.record_transaction_phase_started(
            TransactionPhase::Relay,
            relay_outcome,
            relay_started,
        );

        match outcome {
            TxnLegOutcome::ReadyIdle(pending) => {
                release_backend(
                    config,
                    txn_id,
                    txn_backend_read,
                    txn_backend_write,
                    LeaseDisposition::ReturnToIdle,
                )
                .await;
                pending_first_frame = pending;
                // Loop back to `await_client_activity`.
            }
            TxnLegOutcome::Ended => {
                release_backend(
                    config,
                    txn_id,
                    txn_backend_read,
                    txn_backend_write,
                    LeaseDisposition::Close,
                )
                .await;
                drop(permit);
                return;
            }
        }
    }
}
// </HANDWRITE>

enum TxnLegOutcome {
    /// Backend reported `ReadyForQuery(Idle)`: reset + return to idle, loop
    /// back to `await_client_activity`. Carries a frontend message if the
    /// client had already pipelined its next request ahead of observing
    /// this leg's `ReadyForQuery` -- captured (never forwarded to this
    /// now-being-reset backend) so it becomes the next lease's
    /// `first_frame` without a redundant socket read.
    ReadyIdle(Option<RelayFrame>),
    /// `Terminate`/EOF/`FrameError` on either leg: release `Close`, close
    /// the client.
    Ended,
}

/// Relays one transaction's frames verbatim (having already read
/// `first_frame` off the frontend to trigger this lease), ending when the
/// backend reports `ReadyForQuery(Idle)` or either leg ends.
async fn relay_one_transaction(
    client_read: &mut OwnedReadHalf,
    client_write: &mut OwnedWriteHalf,
    backend_read: &mut OwnedReadHalf,
    backend_write: &mut OwnedWriteHalf,
    frontend_reader: &mut FrameReader,
    backend_reader: &mut FrameReader,
    first_frame: RelayFrame,
) -> TxnLegOutcome {
    let is_terminate = matches!(first_frame.kind, RelayFrameKind::FrontendTerminate);
    if forward_raw(backend_write, &first_frame.bytes)
        .await
        .is_err()
    {
        return TxnLegOutcome::Ended;
    }
    if is_terminate {
        return TxnLegOutcome::Ended;
    }

    // A client is free to pipeline its next simple-query message
    // immediately behind this one without waiting to observe
    // `ReadyForQuery` (tokio-postgres and other async clients do this).
    // Forwarding that next message onto this (about-to-be-reset) backend
    // before this leg's `ReadyForQuery` is confirmed would let
    // `reset_connection`'s wait-for-`ReadyForQuery` logic mistake it for
    // `DISCARD ALL`'s own response -- silently discarding the pipelined
    // query's real response and permanently desyncing the connection. So
    // a decoded-but-not-yet-actionable frontend frame is only ever
    // stashed in `pending_frontend`, never forwarded, until this leg's
    // outcome is known.
    //
    // Frontend EOF/error/`Terminate` carry no such hazard -- there is
    // nothing to misattribute -- so they are still watched for
    // concurrently with the backend read below, letting an abrupt client
    // disconnect end (and free/close) this leg promptly even while the
    // backend is still mid-response, instead of only being noticed after
    // the backend eventually replies.
    let mut pending_frontend: Option<RelayFrame> = None;

    loop {
        let is_idle = loop {
            if pending_frontend.is_some() {
                // Already have a pending frame; only await the backend now.
                match relay_backend_batch(backend_read, client_write, backend_reader).await {
                    Ok(Some(status)) => break matches!(status, TransactionStatus::Idle),
                    Ok(None) => {}
                    Err(()) => return TxnLegOutcome::Ended,
                }
            } else {
                tokio::select! {
                    backend_result = relay_backend_batch(backend_read, client_write, backend_reader) => {
                        match backend_result {
                            Ok(Some(status)) => break matches!(status, TransactionStatus::Idle),
                            Ok(None) => {}
                            Err(()) => return TxnLegOutcome::Ended,
                        }
                    }
                    frontend_result = read_relay_frame_with_raw(client_read, frontend_reader) => {
                        match frontend_result {
                            Ok(Some(frame)) => match frame.kind {
                                RelayFrameKind::FrontendTerminate => {
                                    return TxnLegOutcome::Ended;
                                }
                                RelayFrameKind::Other => {
                                    pending_frontend = Some(frame);
                                }
                                RelayFrameKind::BackendReady(_) => {
                                    unreachable!("frontend-role reader only emits Frontend frames")
                                }
                            },
                            Ok(None) => return TxnLegOutcome::Ended,
                            Err(_) => return TxnLegOutcome::Ended,
                        }
                    }
                }
            }
        };

        if is_idle {
            return TxnLegOutcome::ReadyIdle(pending_frontend);
        }

        // Backend reported still-in-transaction: the next frame (already
        // pipelined, or read fresh now) belongs to this same lease (e.g.
        // the next statement in an explicit BEGIN...COMMIT).
        let msg = match pending_frontend.take() {
            Some(msg) => msg,
            None => match read_relay_frame_with_raw(client_read, frontend_reader).await {
                Ok(Some(frame)) => match frame.kind {
                    RelayFrameKind::FrontendTerminate | RelayFrameKind::Other => frame,
                    RelayFrameKind::BackendReady(_) => {
                        unreachable!("frontend-role reader only emits Frontend frames")
                    }
                },
                Ok(None) | Err(_) => return TxnLegOutcome::Ended,
            },
        };
        if matches!(msg.kind, RelayFrameKind::FrontendTerminate) {
            let _ = forward_raw(backend_write, &msg.bytes).await;
            return TxnLegOutcome::Ended;
        }
        if forward_raw(backend_write, &msg.bytes).await.is_err() {
            return TxnLegOutcome::Ended;
        }
    }
}

/// Relays a single immediately available backend batch. The batch helper
/// stops before awaiting another socket read and at `ReadyForQuery`, so its
/// status has the same ownership meaning as the former one-frame loop.
async fn relay_backend_batch(
    backend_read: &mut OwnedReadHalf,
    client_write: &mut OwnedWriteHalf,
    backend_reader: &mut FrameReader,
) -> Result<Option<TransactionStatus>, ()> {
    let batch = read_backend_relay_batch_with_raw(backend_read, backend_reader)
        .await
        .map_err(|_| ())?
        .ok_or(())?;
    forward_backend_batch(client_write, &batch)
        .await
        .map_err(|_| ())?;
    if batch.terminal_error {
        return Err(());
    }
    Ok(batch.ready)
}

async fn release_backend(
    config: &TransactionProxyConfig,
    id: BackendConnectionId,
    backend_read: OwnedReadHalf,
    backend_write: OwnedWriteHalf,
    disposition: LeaseDisposition,
) {
    if let Ok(stream) = backend_read.reunite(backend_write) {
        config.backend_pool.release(id, stream, disposition).await;
    }
}

async fn write_rejection(
    write: &mut (impl tokio::io::AsyncWrite + Unpin),
    reason: RejectionReason,
) {
    if let Some(message) = reason.synthesized_error_response() {
        let mut buf = BytesMut::new();
        message.encode(&mut buf);
        let _ = write.write_all(&buf).await;
    }
    let _ = write.shutdown().await;
}

async fn write_pool_rejection(
    write: &mut (impl tokio::io::AsyncWrite + Unpin),
    reason: PoolRejectionReason,
) {
    let message = reason.synthesized_error_response();
    let mut buf = BytesMut::new();
    message.encode(&mut buf);
    let _ = write.write_all(&buf).await;
    let _ = write.shutdown().await;
}
// </HANDWRITE>
