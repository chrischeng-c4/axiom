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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// CODEGEN-BEGIN
pub fn cli_serve_entry() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-tcp_accept
    // TODO: Implement process step: tcp-server's serve_arc accepts a raw TCP connection and invokes SessionHandler::handle(stream, ConnectionContext) for every accepted socket
    todo!("process: tcp-server's serve_arc accepts a raw TCP connection and invokes SessionHandler::handle(stream, ConnectionContext) for every accepted socket");
    // Decision: SessionHandler's own ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget) succeeds for this connection (R1)?
    if todo!("decision: SessionHandler's own ConnectionBudget::try_acquire() (RuntimePlan::frontend_budget) succeeds for this connection (R1)?") /* budget exhausted */ {
        return Err(todo!("error: Saturated: encode BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) and write it directly to the client stream, then close the socket without touching the backend or any other session (AC3)"));
    } else { /* permit acquired */
        // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-connect_backend
        // TODO: Implement process step: Admitted, permit held for the session lifetime: TCP-connect to the configured backend endpoint (PGPOOL_BACKEND_ADDR/--backend-addr host:port, bounded by PGPOOL_BACKEND_CONNECT_TIMEOUT_MS/--backend-connect-timeout-ms) (R3)
        todo!("process: Admitted, permit held for the session lifetime: TCP-connect to the configured backend endpoint (PGPOOL_BACKEND_ADDR/--backend-addr host:port, bounded by PGPOOL_BACKEND_CONNECT_TIMEOUT_MS/--backend-connect-timeout-ms) (R3)");
        // Decision: Backend TCP connect succeeded within the connect timeout?
        if todo!("decision: Backend TCP connect succeeded within the connect timeout?") /* connect failed/timed out */ {
            return Err(todo!("error: Unreachable/timed out: encode BackendMessage::ErrorResponse (SQLSTATE 08006 connection_failure) to the client, release the ConnectionBudget permit, close the client socket"));
        } else { /* backend socket established */
            // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_startup
            // TODO: Implement process step: Frame-aware startup relay: a frontend-role FrameReader fed from the client stream decodes the client's untagged StartupMessage (or SSLRequest, rejected — TLS is out of scope), which is re-encoded and forwarded byte-identically to the backend stream (R2)
            todo!("process: Frame-aware startup relay: a frontend-role FrameReader fed from the client stream decodes the client's untagged StartupMessage (or SSLRequest, rejected — TLS is out of scope), which is re-encoded and forwarded byte-identically to the backend stream (R2)");
            // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_auth
            // TODO: Implement process step: Auth passthrough loop: alternately decode backend-role BackendMessage frames from the backend stream and frontend-role FrontendMessage frames from the client stream, re-encoding and forwarding every Authentication*/PasswordMessage/SaslInitialResponse/SaslResponse frame verbatim in both directions; pgpool treats password/SASL payload bytes as opaque relay data only and never persists them (R2, AC2)
            todo!("process: Auth passthrough loop: alternately decode backend-role BackendMessage frames from the backend stream and frontend-role FrontendMessage frames from the client stream, re-encoding and forwarding every Authentication*/PasswordMessage/SaslInitialResponse/SaslResponse frame verbatim in both directions; pgpool treats password/SASL payload bytes as opaque relay data only and never persists them (R2, AC2)");
            // Decision: Backend emits AuthenticationOk before any ErrorResponse?
            if todo!("decision: Backend emits AuthenticationOk before any ErrorResponse?") /* ErrorResponse (auth failed) */ {
                return Err(todo!("error: Backend emits ErrorResponse during startup/auth (bad credentials, SCRAM failure, ...): forward it to the client verbatim, release the permit, close both sides (no retry, no credential caching)"));
            } else { /* AuthenticationOk */
                // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-relay_ready
                // TODO: Implement process step: Forward remaining backend startup messages (ParameterStatus*, BackendKeyData, ReadyForQuery) to the client; the backend-role FrameReader's transaction_status() records the initial TransactionStatus
                todo!("process: Forward remaining backend startup messages (ParameterStatus*, BackendKeyData, ReadyForQuery) to the client; the backend-role FrameReader's transaction_status() records the initial TransactionStatus");
                // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-bidi_relay
                // TODO: Implement process step: Bidirectional relay: two concurrent tasks each decode WireMessage frames from one leg (frontend FrameReader on client->backend, backend FrameReader on backend->client) and re-encode+forward them to the other leg, until a frontend Terminate message, clean EOF, or a FrameError ends that leg (R2)
                todo!("process: Bidirectional relay: two concurrent tasks each decode WireMessage frames from one leg (frontend FrameReader on client->backend, backend FrameReader on backend->client) and re-encode+forward them to the other leg, until a frontend Terminate message, clean EOF, or a FrameError ends that leg (R2)");
                // Decision: Which condition ended the bidirectional relay?
                if todo!("decision: Which condition ended the bidirectional relay?") /* Terminate or clean client EOF */ {
                    todo!("terminal: Client sent Terminate (or closed cleanly): forward Terminate to the backend if not already sent, close the backend connection, release the ConnectionBudget permit — a clean session end");
                } else { /* backend EOF or FrameError */
                    return Err(todo!("error: Backend closed the connection, or a leg hit FrameError (malformed/oversized frame): close the client socket, release the permit; a FrameError never forwards the offending bytes, it only ends that leg"));
                }
            }
        }
        // SPEC-REF: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#pgpool-session-proxy-logic-flow-drain_interaction
        // TODO: Implement process step: Concurrently, DrainSignal (from ConnectionContext.drain) flips to Draining when the process receives SIGTERM/SIGINT: tcp-server's accept loop stops taking new connections immediately, while this session's bidi_relay keeps running unaffected until the client/backend end it or tcp_server's TcpServerConfig.drain_timeout elapses, at which point the task is abandoned (R4, AC4)
        todo!("process: Concurrently, DrainSignal (from ConnectionContext.drain) flips to Draining when the process receives SIGTERM/SIGINT: tcp-server's accept loop stops taking new connections immediately, while this session's bidi_relay keeps running unaffected until the client/backend end it or tcp_server's TcpServerConfig.drain_timeout elapses, at which point the task is abandoned (R4, AC4)");
    }
    // Terminal: backend_closed_or_error -> Backend closed the connection, or a leg hit FrameError (malformed/oversized frame): close the client socket, release the permit; a FrameError never forwards the offending bytes, it only ends that leg
    // Terminal: client_terminate -> Client sent Terminate (or closed cleanly): forward Terminate to the backend if not already sent, close the backend connection, release the ConnectionBudget permit — a clean session end
    // Terminal: reject_backend_unreachable -> Unreachable/timed out: encode BackendMessage::ErrorResponse (SQLSTATE 08006 connection_failure) to the client, release the ConnectionBudget permit, close the client socket
    // Terminal: reject_saturated -> Saturated: encode BackendMessage::ErrorResponse (SQLSTATE 53300 too_many_connections) and write it directly to the client stream, then close the socket without touching the backend or any other session (AC3)
    // Terminal: relay_error_from_backend -> Backend emits ErrorResponse during startup/auth (bad credentials, SCRAM failure, ...): forward it to the client verbatim, release the permit, close both sides (no retry, no credential caching)
}
// CODEGEN-END
