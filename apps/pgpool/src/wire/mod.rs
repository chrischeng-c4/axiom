// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#logic
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! PostgreSQL wire protocol 3.0 message codec: frontend/backend message
//! types, encode/decode over `bytes::BytesMut`/`bytes::Bytes`, the
//! incremental bounded `FrameReader`, and `ReadyForQuery`-driven
//! `TransactionStatus` tracking. No external Postgres protocol crate is
//! used; see the TD at
//! `apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md`.

mod codec;

pub mod backend;
pub mod config;
pub mod error;
pub mod frame;
pub mod frontend;
pub mod reader;

pub use backend::{
    AuthenticationCleartextPassword, AuthenticationMd5Password, AuthenticationOk,
    AuthenticationSasl, AuthenticationSaslContinue, AuthenticationSaslFinal, BackendKeyData,
    BackendMessage, CommandComplete, DataRow, ErrorResponse, FieldDescription, NoticeResponse,
    ParameterStatus, ReadyForQuery, RowDescription, TransactionStatus,
};
pub use config::WireCodecConfig;
pub use error::FrameError;
pub use frame::Frame;
pub use frontend::{
    Bind, Describe, DescribeTarget, Execute, FrontendMessage, Parse, PasswordMessage, Query,
    SaslInitialResponse, SaslResponse, SslRequest, StartupMessage, Sync, Terminate,
};
pub use reader::{FrameReader, Role, WireMessage};
// </HANDWRITE>
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
