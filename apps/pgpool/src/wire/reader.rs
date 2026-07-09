// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#logic
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! `FrameReader`: an incremental reader that handles split/partial reads,
//! length-prefix validation, and bounded frame size from the TD Config
//! section, emitting typed messages and never panicking on malformed input.
//!
//! This slice reads from an in-memory `BytesMut` fed by the caller via
//! [`FrameReader::feed`] rather than polling an async stream directly — the
//! actual stream/`TcpHandler` seam lands in the next slice (#1288); the
//! logic here (header/length parsing, bounds, split-read reassembly) is
//! identical either way.

use bytes::BytesMut;

use crate::wire::backend::{BackendMessage, TransactionStatus};
use crate::wire::config::WireCodecConfig;
use crate::wire::error::FrameError;
use crate::wire::frame::Frame;
use crate::wire::frontend::FrontendMessage;

/// Which side of the connection this `FrameReader` is decoding: the
/// frontend (client-to-pool) direction expects an untagged
/// StartupMessage/SSLRequest frame first; the backend (pool-to-Postgres)
/// direction is tagged frames only and drives `TransactionStatus` tracking
/// off `ReadyForQuery`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Frontend,
    Backend,
}

/// A decoded frame, tagged by which message family it came from.
#[derive(Debug, Clone, PartialEq)]
pub enum WireMessage {
    Frontend(FrontendMessage),
    Backend(BackendMessage),
}

/// Incremental, bounded, non-panicking frame reader over a byte stream fed
/// via [`FrameReader::feed`].
pub struct FrameReader {
    buf: BytesMut,
    role: Role,
    config: WireCodecConfig,
    /// True only while the frontend reader is still awaiting its untagged
    /// startup packet (SSLRequest may legitimately precede StartupMessage,
    /// so this only flips off once an actual StartupMessage is decoded).
    awaiting_untagged_startup: bool,
    tx_status: TransactionStatus,
}

impl FrameReader {
    pub fn new(role: Role, config: &WireCodecConfig) -> Self {
        Self {
            buf: BytesMut::with_capacity(config.read_buffer_initial_capacity),
            role,
            config: *config,
            awaiting_untagged_startup: role == Role::Frontend,
            tx_status: config.initial_transaction_status,
        }
    }

    /// Appends freshly-read bytes from the transport into the internal
    /// buffer. Safe to call with any chunk size, including 0 or 1 bytes,
    /// to simulate split/partial reads.
    pub fn feed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    /// The `TransactionStatus` last observed via a backend `ReadyForQuery`
    /// message (or `config.initial_transaction_status` before the first one
    /// arrives).
    pub fn transaction_status(&self) -> TransactionStatus {
        self.tx_status
    }

    /// Attempts to read and decode the next frame from the buffered bytes.
    ///
    /// Returns `Ok(None)` when the buffer doesn't yet hold a full frame
    /// (header or body still incomplete — the split/partial-read boundary);
    /// the caller should await more bytes and call again. Returns
    /// `Err(FrameError::Oversized)` as soon as a declared length exceeding
    /// the configured bound is known, without waiting for (or buffering)
    /// the oversized body.
    pub fn next_frame(&mut self) -> Result<Option<WireMessage>, FrameError> {
        let untagged = self.role == Role::Frontend && self.awaiting_untagged_startup;
        let header_len = if untagged { 4 } else { 5 };

        if self.buf.len() < header_len {
            return Ok(None);
        }

        let declared_length = if untagged {
            i32::from_be_bytes(self.buf[0..4].try_into().expect("checked len"))
        } else {
            i32::from_be_bytes(self.buf[1..5].try_into().expect("checked len"))
        };

        if declared_length < 4 {
            let tag = if untagged { None } else { Some(self.buf[0]) };
            return Err(FrameError::Malformed {
                tag,
                reason: format!(
                    "declared frame length {declared_length} is below the minimum of 4"
                ),
            });
        }

        let max = if untagged {
            self.config.max_startup_bytes
        } else {
            self.config.max_frame_bytes
        };
        if declared_length as usize > max {
            return Err(FrameError::Oversized {
                declared: declared_length as usize,
                max,
            });
        }

        let total_len = if untagged {
            declared_length as usize
        } else {
            1 + declared_length as usize
        };
        if self.buf.len() < total_len {
            return Ok(None);
        }

        let frame_bytes = self.buf.split_to(total_len).freeze();
        let frame = if untagged {
            Frame {
                tag: None,
                payload: frame_bytes.slice(4..),
            }
        } else {
            Frame {
                tag: Some(frame_bytes[0]),
                payload: frame_bytes.slice(5..),
            }
        };

        match self.role {
            Role::Frontend => {
                let message = FrontendMessage::decode(&frame, &self.config)?;
                if untagged {
                    if let FrontendMessage::Startup(_) = &message {
                        self.awaiting_untagged_startup = false;
                    }
                    // SSLRequest leaves awaiting_untagged_startup set: the
                    // client still owes an untagged StartupMessage next.
                }
                Ok(Some(WireMessage::Frontend(message)))
            }
            Role::Backend => {
                let message = BackendMessage::decode(&frame, &self.config)?;
                if let BackendMessage::ReadyForQuery(ready) = &message {
                    self.tx_status = ready.status;
                }
                Ok(Some(WireMessage::Backend(message)))
            }
        }
    }
}
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
