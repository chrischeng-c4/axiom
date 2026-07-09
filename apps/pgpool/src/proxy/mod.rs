// SPEC-MANAGED: apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md#logic
// <HANDWRITE gap="missing-generator:logic:pgpool-session-proxy" tracker="#1288" reason="Session-mode proxy needs generator primitives that do not exist yet.">
//! Session-mode 1:1 PostgreSQL proxy: one backend connection per accepted
//! frontend client, admission-gated by its own `ConnectionBudget`, with
//! frame-aware auth passthrough (credential bytes relayed opaquely, never
//! persisted) and bidirectional relay until `Terminate`/EOF/`FrameError`.
//! See the TD at
//! `apps/pgpool/tech-design/logic/session-mode-proxy-with-auth-passthrough-and-serve-entrypoint.md`.

mod config;
mod error;
mod handler;
mod session;

pub use config::{BackendEndpointConfig, SessionProxyConfig};
pub use error::{ProxyError, RejectionReason, SessionOutcome};
pub use handler::SessionHandler;
pub use session::run_session;
// </HANDWRITE>
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
