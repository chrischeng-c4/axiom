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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#logic
// CODEGEN-BEGIN
pub fn poll_stream() -> std::result::Result<(), Box<dyn std::error::Error>> {
    loop {
        // Decision: Buffer holds enough bytes for a frame header (1 tag byte + 4-byte length for tagged frames; 4-byte length only for the untagged StartupMessage/SSLRequest)?
        if todo!("decision: Buffer holds enough bytes for a frame header (1 tag byte + 4-byte length for tagged frames; 4-byte length only for the untagged StartupMessage/SSLRequest)?") /* header incomplete */ {
            // SPEC-REF: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#pgpool-wire-codec-logic-flow-need_more_header
            // TODO: Implement process step: Not enough header bytes yet: keep the partial bytes in BytesMut and return Pending (split/partial-read boundary)
            todo!("process: Not enough header bytes yet: keep the partial bytes in BytesMut and return Pending (split/partial-read boundary)");
            continue;
        } else { /* header complete */
            // SPEC-REF: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#pgpool-wire-codec-logic-flow-read_length
            // TODO: Implement process step: Parse the declared i32 frame length from the header
            todo!("process: Parse the declared i32 frame length from the header");
            // Decision: declared length <= configured max_frame_bytes (bounded frame size)?
            if todo!("decision: declared length <= configured max_frame_bytes (bounded frame size)?") /* exceeds max_frame_bytes */ {
                break;
            } else { /* within bound */
                // Decision: Buffer already holds the full declared frame length?
                if todo!("decision: Buffer already holds the full declared frame length?") /* body incomplete */ {
                    // SPEC-REF: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#pgpool-wire-codec-logic-flow-need_more_body
                    // TODO: Implement process step: Frame body still incomplete: keep bytes in BytesMut and return Pending (handles split/partial reads across buffer boundaries)
                    todo!("process: Frame body still incomplete: keep bytes in BytesMut and return Pending (handles split/partial reads across buffer boundaries)");
                    continue;
                } else { /* full frame buffered */
                    break;
                }
            }
        }
    }
    return Err(todo!("error: Return FrameError::Oversized{declared,max} — typed error, connection is closed by the caller, no panic"));
    // SPEC-REF: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#pgpool-wire-codec-logic-flow-decode_message
    // TODO: Implement process step: Decode the frame payload into a typed FrontendMessage or BackendMessage variant by tag byte (or, untagged, by protocol version code for StartupMessage/SSLRequest)
    todo!("process: Decode the frame payload into a typed FrontendMessage or BackendMessage variant by tag byte (or, untagged, by protocol version code for StartupMessage/SSLRequest)");
    // Decision: Decode succeeded: known tag, well-formed fixed/variable fields for that message type?
    if todo!("decision: Decode succeeded: known tag, well-formed fixed/variable fields for that message type?") /* unknown tag or bad field */ {
        return Err(todo!("error: Return FrameError::Malformed{tag,reason} — typed error, no panic, buffer cursor still advances past the bad frame"));
    } else { /* well-formed */
        // SPEC-REF: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#pgpool-wire-codec-logic-flow-update_tx_status
        // TODO: Implement process step: If the decoded message is ReadyForQuery, update TransactionStatus from the status byte (I/T/E -> Idle/InTransaction/Failed)
        todo!("process: If the decoded message is ReadyForQuery, update TransactionStatus from the status byte (I/T/E -> Idle/InTransaction/Failed)");
        todo!("terminal: Advance the buffer cursor past the consumed frame and emit the typed Frame to the caller (tcp-server TcpHandler seam)");
    }
    // Terminal: emit_frame -> Advance the buffer cursor past the consumed frame and emit the typed Frame to the caller (tcp-server TcpHandler seam)
    // Terminal: encode_done -> Return the encoded byte range ready for the transport write path
    // Terminal: reject_malformed -> Return FrameError::Malformed{tag,reason} — typed error, no panic, buffer cursor still advances past the bad frame
    // Terminal: reject_oversized -> Return FrameError::Oversized{declared,max} — typed error, connection is closed by the caller, no panic
}
// CODEGEN-END
