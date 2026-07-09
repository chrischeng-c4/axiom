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
    // `ReadyForQuery(Idle)` or the leg ends.
    loop {
        let first_frame = match read_frame(&mut client_read, &mut frontend_reader).await {
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
            TxnLegOutcome::ReadyIdle => {
                release_backend(
                    config,
                    txn_id,
                    txn_backend_read,
                    txn_backend_write,
                    LeaseDisposition::ReturnToIdle,
                )
                .await;
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
    /// back to `await_client_activity`.
    ReadyIdle,
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

    let client_to_backend = async {
        loop {
            match read_frame(client_read, frontend_reader).await {
                Ok(Some(WireMessage::Frontend(FrontendMessage::Terminate(terminate)))) => {
                    let _ = forward_frontend(backend_write, &FrontendMessage::Terminate(terminate))
                        .await;
                    return;
                }
                Ok(Some(WireMessage::Frontend(msg))) => {
                    if forward_frontend(backend_write, &msg).await.is_err() {
                        return;
                    }
                }
                Ok(Some(WireMessage::Backend(_))) => {
                    unreachable!("frontend-role reader only emits Frontend frames")
                }
                Ok(None) | Err(_) => return,
            }
        }
    };

    let backend_to_client = async {
        loop {
            match read_frame(backend_read, backend_reader).await {
                Ok(Some(WireMessage::Backend(BackendMessage::ReadyForQuery(ready)))) => {
                    let is_idle = matches!(&ready.status, TransactionStatus::Idle);
                    if forward_backend(client_write, &BackendMessage::ReadyForQuery(ready))
                        .await
                        .is_err()
                    {
                        return false;
                    }
                    if is_idle {
                        return true;
                    }
                    // Still `InTransaction`/`Failed`: keep relaying.
                }
                Ok(Some(WireMessage::Backend(msg))) => {
                    if forward_backend(client_write, &msg).await.is_err() {
                        return false;
                    }
                }
                Ok(Some(WireMessage::Frontend(_))) => {
                    unreachable!("backend-role reader only emits Backend frames")
                }
                Ok(None) | Err(_) => return false,
            }
        }
    };

    tokio::select! {
        _ = client_to_backend => TxnLegOutcome::Ended,
        ready_idle = backend_to_client => {
            if ready_idle {
                TxnLegOutcome::ReadyIdle
            } else {
                TxnLegOutcome::Ended
            }
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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// CODEGEN-BEGIN
pub fn handle_accept() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Decision: ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget, same primitive WI #1288 already uses) succeeds for this connection?
    if todo!("decision: ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget, same primitive WI #1288 already uses) succeeds for this connection?") /* budget exhausted */ {
        return Err(todo!("error: Frontend saturated: write BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) to the client and close the socket — unchanged from WI #1288, shared by both pool modes"));
    } else { /* permit acquired */
        // Decision: RuntimePlan::PoolMode (fixed for the process)
        if todo!("decision: RuntimePlan::PoolMode (fixed for the process)") /* PoolMode::Session */ {
            todo!("terminal: Session mode: delegates to the unchanged WI #1288 SessionHandler::run_session pipeline, except connect_backend now calls BackendPool::acquire_fresh() (capacity-bounded by max_backend_connections, R1) instead of a raw TcpStream::connect, and teardown calls BackendPool::release(id, stream, LeaseDisposition::Close) instead of just dropping the socket; the per-message auth-passthrough/relay steps are unchanged and are documented in apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md, not redrawn here");
        } else { /* PoolMode::Transaction */
            // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-txn_admit_handshake
            // TODO: Implement process step: Transaction mode: BackendPool::acquire_fresh() dials a brand-new backend connection for this client's own one-time real startup+auth relay (reusing the frame-aware relay_startup/relay_until_ready mechanism from the session-mode proxy), bounded by the shared max_backend_connections capacity (R1)
            todo!("process: Transaction mode: BackendPool::acquire_fresh() dials a brand-new backend connection for this client's own one-time real startup+auth relay (reusing the frame-aware relay_startup/relay_until_ready mechanism from the session-mode proxy), bounded by the shared max_backend_connections capacity (R1)");
            // Decision: Backend connect succeeded and AuthenticationOk + ReadyForQuery(Idle) were forwarded to the client before any ErrorResponse?
            if todo!("decision: Backend connect succeeded and AuthenticationOk + ReadyForQuery(Idle) were forwarded to the client before any ErrorResponse?") /* connect failed/timed out */ {
                return Err(todo!("error: acquire_fresh() connect failed/timed out: write ErrorResponse (SQLSTATE 08006 connection_failure), release the frontend permit, close the client socket — same RejectionReason::BackendUnreachable mapping as session mode"));
            } else if todo!("decision branch: {}", "backend ErrorResponse before ReadyForQuery") { /* backend ErrorResponse before ReadyForQuery */
                return Err(todo!("error: Backend emitted ErrorResponse during the admission handshake: forward it to the client verbatim, release the frontend permit, close both sides — same RejectionReason::BackendAuthFailed mapping as session mode"));
            } else { /* AuthenticationOk + ReadyForQuery forwarded */
                // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-release_after_handshake
                // TODO: Implement process step: Handshake succeeded: this client is now admitted and vouched-for; the handshake backend is immediately reset (DISCARD ALL) and returned to the shared idle pool via BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle) — the client now holds NO backend lease (R1, R2)
                todo!("process: Handshake succeeded: this client is now admitted and vouched-for; the handshake backend is immediately reset (DISCARD ALL) and returned to the shared idle pool via BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle) — the client now holds NO backend lease (R1, R2)");
                loop {
                    // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-await_client_activity
                    // TODO: Implement process step: Client holds no backend lease; a frontend-role FrameReader fed from the client stream waits for the client's next frame
                    todo!("process: Client holds no backend lease; a frontend-role FrameReader fed from the client stream waits for the client's next frame");
                    // Decision: Is the observed frontend frame the start of a transaction/first query after ReadyForQuery-idle (any frame other than Terminate), or Terminate/clean EOF?
                    if todo!("decision: Is the observed frontend frame the start of a transaction/first query after ReadyForQuery-idle (any frame other than Terminate), or Terminate/clean EOF?") /* Terminate/clean EOF */ {
                        break;
                    } else { /* any other frame (transaction/query start) */
                        // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-acquire_txn_backend
                        // TODO: Implement process step: BackendPool::acquire(): pop an idle already-authenticated connection and liveness-check it (non-blocking peek read), else fresh-connect if capacity remains, else wait up to acquire_timeout (R1, R3)
                        todo!("process: BackendPool::acquire(): pop an idle already-authenticated connection and liveness-check it (non-blocking peek read), else fresh-connect if capacity remains, else wait up to acquire_timeout (R1, R3)");
                        // Decision: A lease was returned within acquire_timeout, or the wait timed out?
                        if todo!("decision: A lease was returned within acquire_timeout, or the wait timed out?") /* acquire_timeout elapsed, no lease */ {
                            break;
                        } else { /* lease acquired */
                            // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-relay_transaction
                            // TODO: Implement process step: Relay frontend<->backend frames verbatim on the leased connection (same decode/re-encode primitives as session mode's bidirectional relay) until the backend's ReadyForQuery reports Idle again, or Terminate/EOF/FrameError ends the leg (R2)
                            todo!("process: Relay frontend<->backend frames verbatim on the leased connection (same decode/re-encode primitives as session mode's bidirectional relay) until the backend's ReadyForQuery reports Idle again, or Terminate/EOF/FrameError ends the leg (R2)");
                            // Decision: How did the leased transaction's relay end?
                            if todo!("decision: How did the leased transaction's relay end?") /* backend ReadyForQuery(Idle) */ {
                                // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-release_after_transaction
                                // TODO: Implement process step: Backend reported ReadyForQuery(Idle): reset (DISCARD ALL) and BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle); the client returns to holding no lease and loops back to await_client_activity (R1, R2, AC1, AC2)
                                todo!("process: Backend reported ReadyForQuery(Idle): reset (DISCARD ALL) and BackendPool::release(id, stream, LeaseDisposition::ReturnToIdle); the client returns to holding no lease and loops back to await_client_activity (R1, R2, AC1, AC2)");
                                continue;
                            } else { /* EOF/FrameError mid-transaction */
                                break;
                            }
                        }
                    }
                }
                todo!("terminal: Client sent Terminate (or closed cleanly) while holding no lease: nothing to release, session ends cleanly");
                return Err(todo!("error: Backend pool exhausted for longer than acquire_timeout: write a synthesized PoolRejectionReason::BackendPoolSaturated ErrorResponse (SQLSTATE 53300) to this client only and close its socket; every other admitted client and in-flight transaction lease is unaffected (R3, AC3)"));
                return Err(todo!("error: Backend/client EOF or FrameError mid-transaction: the lease is released as BackendPool::release(id, stream, LeaseDisposition::Close) — never returned to idle, since its session state is unknown/unsafe to reuse — and the client socket is closed"));
            }
            // SPEC-REF: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#pgpool-backend-pool-logic-flow-drain_interaction
            // TODO: Implement process step: Concurrently, DrainSignal flips to Draining on SIGTERM/SIGINT (unchanged tcp-server mechanism): the accept loop stops admitting new connections immediately, while an in-flight handshake or transaction lease keeps running unaffected until it ends or TcpServerConfig.drain_timeout elapses, at which point the task is abandoned
            todo!("process: Concurrently, DrainSignal flips to Draining on SIGTERM/SIGINT (unchanged tcp-server mechanism): the accept loop stops admitting new connections immediately, while an in-flight handshake or transaction lease keeps running unaffected until it ends or TcpServerConfig.drain_timeout elapses, at which point the task is abandoned");
        }
    }
    // Terminal: client_terminate_idle -> Client sent Terminate (or closed cleanly) while holding no lease: nothing to release, session ends cleanly
    // Terminal: reject_auth_failed -> Backend emitted ErrorResponse during the admission handshake: forward it to the client verbatim, release the frontend permit, close both sides — same RejectionReason::BackendAuthFailed mapping as session mode
    // Terminal: reject_backend_unreachable -> acquire_fresh() connect failed/timed out: write ErrorResponse (SQLSTATE 08006 connection_failure), release the frontend permit, close the client socket — same RejectionReason::BackendUnreachable mapping as session mode
    // Terminal: reject_frontend_saturated -> Frontend saturated: write BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) to the client and close the socket — unchanged from WI #1288, shared by both pool modes
    // Terminal: reject_pool_saturated -> Backend pool exhausted for longer than acquire_timeout: write a synthesized PoolRejectionReason::BackendPoolSaturated ErrorResponse (SQLSTATE 53300) to this client only and close its socket; every other admitted client and in-flight transaction lease is unaffected (R3, AC3)
    // Terminal: relay_error_or_eof -> Backend/client EOF or FrameError mid-transaction: the lease is released as BackendPool::release(id, stream, LeaseDisposition::Close) — never returned to idle, since its session state is unknown/unsafe to reuse — and the client socket is closed
    // Terminal: session_mode_delegate -> Session mode: delegates to the unchanged WI #1288 SessionHandler::run_session pipeline, except connect_backend now calls BackendPool::acquire_fresh() (capacity-bounded by max_backend_connections, R1) instead of a raw TcpStream::connect, and teardown calls BackendPool::release(id, stream, LeaseDisposition::Close) instead of just dropping the socket; the per-message auth-passthrough/relay steps are unchanged and are documented in apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md, not redrawn here
}
// CODEGEN-END
