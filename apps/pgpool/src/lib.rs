// SPEC-MANAGED: apps/pgpool/tech-design/semantic/source/apps-pgpool-src-lib-rs.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-bootstrap" tracker="#pgpool-bootstrap" reason="Initial working-name app scaffold before generated source ownership lands.">
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use server_core::{BindConfig, ConnectionBudget};
use tcp_server::TcpSocketOptions;

pub mod spec;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoolMode {
    Session,
    Transaction,
}

#[derive(Clone, Debug)]
pub struct RuntimePlan {
    pub frontend_bind: BindConfig,
    pub admin_bind: BindConfig,
    pub frontend_socket: TcpSocketOptions,
    pub max_frontend_connections: usize,
    pub max_backend_connections: usize,
    pub pool_mode: PoolMode,
    pub admin_h2c: http_server::H2cServerOptions,
}

impl RuntimePlan {
    pub fn frontend_budget(&self) -> ConnectionBudget {
        ConnectionBudget::new(self.max_frontend_connections)
    }

    pub fn admin_drain_timeout(&self) -> Duration {
        self.admin_h2c.drain_timeout
    }

    pub fn to_json(&self) -> Value {
        json!({
            "app_id": "pgpool",
            "name_status": "working-name",
            "frontend": {
                "protocol": "postgresql-wire",
                "bind": self.frontend_bind.socket_addr().to_string(),
                "socket": {
                    "backlog": self.frontend_socket.backlog,
                    "reuse_addr": self.frontend_socket.reuse_addr,
                    "nodelay": self.frontend_socket.nodelay
                },
                "max_connections": self.max_frontend_connections
            },
            "backend_pool": {
                "max_connections": self.max_backend_connections,
                "mode": match self.pool_mode {
                    PoolMode::Session => "session",
                    PoolMode::Transaction => "transaction",
                }
            },
            "admin": {
                "protocol": "http1+h2c",
                "bind": self.admin_bind.socket_addr().to_string(),
                "max_concurrent_streams": self.admin_h2c.max_concurrent_streams,
                "drain_timeout_ms": self.admin_h2c.drain_timeout.as_millis()
            },
            "shared_libs": ["server-core", "tcp-server", "http-server"]
        })
    }
}

impl Default for RuntimePlan {
    fn default() -> Self {
        Self {
            frontend_bind: BindConfig::any(6432),
            admin_bind: BindConfig::any(9080),
            frontend_socket: TcpSocketOptions::default(),
            max_frontend_connections: 10_000,
            max_backend_connections: 512,
            pool_mode: PoolMode::Transaction,
            admin_h2c: http_server::H2cServerOptions::default(),
        }
    }
}

pub fn default_runtime_plan() -> RuntimePlan {
    RuntimePlan::default()
}

pub fn runtime_plan_json() -> String {
    serde_json::to_string_pretty(&default_runtime_plan().to_json())
        .expect("runtime plan serializes")
}
// </HANDWRITE>

// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#logic
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
pub mod wire;
// </HANDWRITE>

// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
pub mod proxy;
// </HANDWRITE>

// SPEC-MANAGED: apps/pgpool/tech-design/logic/backend-pool-connection-reuse-and-transaction-session-pool-modes.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-backend-pool" tracker="#1289" reason="Backend pool needs generator primitives that do not exist yet.">
pub mod pool;
// </HANDWRITE>

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_plan_uses_shared_server_substrate() {
        let plan = default_runtime_plan();
        assert_eq!(plan.frontend_bind.port, 6432);
        assert_eq!(plan.admin_bind.port, 9080);
        assert_eq!(plan.frontend_budget().max(), 10_000);
        assert!(plan.frontend_socket.nodelay);
        assert_eq!(plan.admin_h2c.max_concurrent_streams, 4096);
    }

    #[test]
    fn runtime_plan_names_working_app_id() {
        let json = default_runtime_plan().to_json();
        assert_eq!(json["app_id"], "pgpool");
        assert_eq!(json["name_status"], "working-name");
        assert_eq!(json["shared_libs"][0], "server-core");
    }
}
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
