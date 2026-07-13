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
//! client). Drain is honored transparently by `tcp_server::serve_arc`'s
//! outer accept loop + bounded `drain_timeout` task abandonment — this
//! handler references `cx.drain` nowhere, exactly like `SessionHandler`.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use anyhow::Result;
use bytes::BytesMut;
use server_core::ConnectionBudget;
use tcp_server::{ConnectionContext, TcpHandler};
use tokio::io::AsyncWriteExt;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;

use crate::pool::backend_pool::{BackendLease, BackendPool};
use crate::pool::types::{BackendConnectionId, LeaseDisposition, PoolRejectionReason};
use crate::proxy::{
    forward_backend, forward_frontend, read_frame, read_startup, relay_until_ready,
    HandshakeOutcome, RejectionReason,
};
use crate::wire::{
    BackendMessage, FrameReader, FrontendMessage, Role, TransactionStatus, WireCodecConfig,
    WireMessage,
};

/// Full configuration for one `TransactionHandler`, per the TD Schema
/// section: its own frontend admission budget (mirroring
/// `SessionProxyConfig.frontend_budget`, deliberately not wired into
/// `tcp_server::TcpServerConfig.connection_budget` for the same reason —
/// see `crate::proxy::SessionProxyConfig`), the shared backend pool, wire
/// bounds, and the drain timeout `pgpool serve` also feeds into
/// `TcpServerConfig` (not referenced by `TransactionHandler` itself; drain
/// is purely a `tcp_server::serve_arc`-level concern).
#[derive(Debug, Clone)]
pub struct TransactionProxyConfig {
    pub frontend_budget: ConnectionBudget,
    pub backend_pool: BackendPool,
    pub wire: WireCodecConfig,
    pub drain_timeout: Duration,
}

/// Transaction-mode `tcp_server::TcpHandler` impl `pgpool serve` binds to
/// its listener when `RuntimePlan::pool_mode` is `PoolMode::Transaction`.
/// Private field, constructed via `TransactionHandler::new(config)`;
/// mirrors `SessionHandler`'s shape.
#[derive(Debug, Clone)]
pub struct TransactionHandler {
    config: TransactionProxyConfig,
}

impl TransactionHandler {
    pub fn new(config: TransactionProxyConfig) -> Self {
        Self { config }
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
        Box::pin(async move {
            run_transaction_client(stream, &config, cx.peer_addr).await;
            Ok(())
        })
    }
}

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

    // Admission handshake: acquire_fresh() always dials brand-new (Pool
    // Lease State Machine's `admitting` state); its own `PoolError` maps to
    // `rejected_saturated` (pool momentarily full even for admission) or
    // `rejected_backend_unreachable` per the Lease FSM's two admission
    // rejection edges.
    let lease = match config.backend_pool.acquire_fresh().await {
        Ok(lease) => lease,
        Err(crate::pool::types::PoolError::Saturated { .. }) => {
            let mut client = client;
            write_pool_rejection(&mut client, PoolRejectionReason::BackendPoolSaturated).await;
            drop(permit);
            tracing::info!(
                peer = %peer_addr,
                outcome = "rejected_pool_saturated_admission",
                "pgpool transaction admission rejected"
            );
            return;
        }
        Err(crate::pool::types::PoolError::BackendUnreachable(_)) => {
            let mut client = client;
            write_rejection(&mut client, RejectionReason::BackendUnreachable).await;
            drop(permit);
            tracing::info!(
                peer = %peer_addr,
                outcome = "rejected_backend_unreachable",
                "pgpool transaction admission rejected"
            );
            return;
        }
    };

    let (mut client_read, mut client_write) = client.into_split();
    let mut frontend_reader = FrameReader::new(Role::Frontend, &config.wire);

    let BackendLease {
        id: handshake_id,
        stream: handshake_backend,
        ..
    } = lease;
    let (mut backend_read, mut backend_write) = handshake_backend.into_split();
    let mut backend_reader = FrameReader::new(Role::Backend, &config.wire);

    let startup =
        match read_startup(&mut client_read, &mut client_write, &mut frontend_reader).await {
            Ok(startup) => startup,
            Err(_) => {
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
        };
    if forward_frontend(&mut backend_write, &FrontendMessage::Startup(startup))
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
        Ok(HandshakeOutcome::Rejected) => {
            // The backend's own `ErrorResponse` was already forwarded
            // verbatim by `relay_until_ready`.
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
        Ok(HandshakeOutcome::Ready) => {}
        Err(_) => {
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

    // Handshake succeeded: reset immediately and return to idle. The client
    // holds NO backend lease at this point (Lease FSM's `idle_no_lease`).
    release_backend(
        config,
        handshake_id,
        backend_read,
        backend_write,
        LeaseDisposition::ReturnToIdle,
    )
    .await;

    // `await_client_activity` loop: each non-Terminate frontend frame
    // acquires a per-transaction lease and relays until that transaction's
    // `ReadyForQuery(Idle)` or the leg ends. `pending_first_frame` carries
    // a frame the client had already pipelined ahead of the previous leg's
    // `ReadyForQuery` (captured by `relay_one_transaction`, never lost),
    // standing in for a fresh frontend read on this iteration.
    let mut pending_first_frame: Option<FrontendMessage> = None;
    loop {
        let first_frame = match pending_first_frame.take() {
            Some(msg) => msg,
            None => match read_frame(&mut client_read, &mut frontend_reader).await {
                Ok(Some(WireMessage::Frontend(FrontendMessage::Terminate(_)))) | Ok(None) => {
                    drop(permit);
                    return;
                }
                Ok(Some(WireMessage::Frontend(msg))) => msg,
                Ok(Some(WireMessage::Backend(_))) => {
                    unreachable!("frontend-role reader only emits Frontend frames")
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
        let lease = match config.backend_pool.acquire().await {
            Ok(lease) => lease,
            Err(_) => {
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

enum TxnLegOutcome {
    /// Backend reported `ReadyForQuery(Idle)`: reset + return to idle, loop
    /// back to `await_client_activity`. Carries a frontend message if the
    /// client had already pipelined its next request ahead of observing
    /// this leg's `ReadyForQuery` -- captured (never forwarded to this
    /// now-being-reset backend) so it becomes the next lease's
    /// `first_frame` without a redundant socket read.
    ReadyIdle(Option<FrontendMessage>),
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
    first_frame: FrontendMessage,
) -> TxnLegOutcome {
    let is_terminate = matches!(first_frame, FrontendMessage::Terminate(_));
    if forward_frontend(backend_write, &first_frame).await.is_err() {
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
    let mut pending_frontend: Option<FrontendMessage> = None;

    loop {
        let is_idle = loop {
            if pending_frontend.is_some() {
                // Already have a pending frame; only await the backend now.
                match read_frame(backend_read, backend_reader).await {
                    Ok(Some(WireMessage::Backend(BackendMessage::ReadyForQuery(ready)))) => {
                        let idle = matches!(&ready.status, TransactionStatus::Idle);
                        if forward_backend(client_write, &BackendMessage::ReadyForQuery(ready))
                            .await
                            .is_err()
                        {
                            return TxnLegOutcome::Ended;
                        }
                        break idle;
                    }
                    Ok(Some(WireMessage::Backend(msg))) => {
                        if forward_backend(client_write, &msg).await.is_err() {
                            return TxnLegOutcome::Ended;
                        }
                    }
                    Ok(Some(WireMessage::Frontend(_))) => {
                        unreachable!("backend-role reader only emits Backend frames")
                    }
                    Ok(None) | Err(_) => return TxnLegOutcome::Ended,
                }
            } else {
                tokio::select! {
                    backend_result = read_frame(backend_read, backend_reader) => {
                        match backend_result {
                            Ok(Some(WireMessage::Backend(BackendMessage::ReadyForQuery(ready)))) => {
                                let idle = matches!(&ready.status, TransactionStatus::Idle);
                                if forward_backend(client_write, &BackendMessage::ReadyForQuery(ready))
                                    .await
                                    .is_err()
                                {
                                    return TxnLegOutcome::Ended;
                                }
                                break idle;
                            }
                            Ok(Some(WireMessage::Backend(msg))) => {
                                if forward_backend(client_write, &msg).await.is_err() {
                                    return TxnLegOutcome::Ended;
                                }
                            }
                            Ok(Some(WireMessage::Frontend(_))) => {
                                unreachable!("backend-role reader only emits Backend frames")
                            }
                            Ok(None) | Err(_) => return TxnLegOutcome::Ended,
                        }
                    }
                    frontend_result = read_frame(client_read, frontend_reader) => {
                        match frontend_result {
                            Ok(Some(WireMessage::Frontend(FrontendMessage::Terminate(_))))
                            | Ok(None)
                            | Err(_) => return TxnLegOutcome::Ended,
                            Ok(Some(WireMessage::Frontend(msg))) => {
                                pending_frontend = Some(msg);
                            }
                            Ok(Some(WireMessage::Backend(_))) => {
                                unreachable!("frontend-role reader only emits Frontend frames")
                            }
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
            None => match read_frame(client_read, frontend_reader).await {
                Ok(Some(WireMessage::Frontend(msg))) => msg,
                Ok(Some(WireMessage::Backend(_))) => {
                    unreachable!("frontend-role reader only emits Frontend frames")
                }
                Ok(None) | Err(_) => return TxnLegOutcome::Ended,
            },
        };
        if let FrontendMessage::Terminate(terminate) = msg {
            let _ = forward_frontend(backend_write, &FrontendMessage::Terminate(terminate)).await;
            return TxnLegOutcome::Ended;
        }
        if forward_frontend(backend_write, &msg).await.is_err() {
            return TxnLegOutcome::Ended;
        }
    }
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
