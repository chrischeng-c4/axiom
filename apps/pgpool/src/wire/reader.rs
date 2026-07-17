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

use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::{AsyncRead, AsyncReadExt};

use crate::wire::backend::{
    BackendMessage, ReadyForQuery, TransactionStatus, TAG_COMMAND_COMPLETE, TAG_DATA_ROW,
    TAG_ERROR_RESPONSE, TAG_NOTICE_RESPONSE, TAG_READY_FOR_QUERY, TAG_ROW_DESCRIPTION,
};
use crate::wire::codec::Cursor;
use crate::wire::config::WireCodecConfig;
use crate::wire::error::FrameError;
use crate::wire::frame::Frame;
use crate::wire::frontend::{FrontendMessage, TAG_QUERY, TAG_TERMINATE};

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

/// A fully validated decoded message together with the exact bytes read from
/// the transport. Relay paths can inspect `message` for control flow while
/// writing `bytes` unchanged, avoiding an allocation and re-encode for every
/// steady-state frame.
#[derive(Debug, Clone, PartialEq)]
pub struct WireFrame {
    pub message: WireMessage,
    pub bytes: Bytes,
}

/// The only per-frame facts that transaction pooling needs after full wire
/// validation. Every other accepted frame is forwarded as its exact raw bytes
/// without constructing an owned message model that the relay would discard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayFrameKind {
    Other,
    FrontendTerminate,
    BackendReady(TransactionStatus),
}

/// A fully validated transaction-relay frame. Unlike [`WireFrame`], this
/// carries only the control information needed to preserve transaction lease
/// boundaries, avoiding allocations for ordinary result-set frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayFrame {
    pub kind: RelayFrameKind,
    pub bytes: Bytes,
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

    /// Appends directly from an async transport to the parser buffer. This
    /// preserves `feed`'s append semantics while avoiding an intermediate
    /// stack buffer and copy on the relay hot path.
    pub async fn read_from(
        &mut self,
        stream: &mut (impl AsyncRead + Unpin),
    ) -> std::io::Result<usize> {
        stream.read_buf(&mut self.buf).await
    }

    /// Appends directly from a synchronous nonblocking transport. The parser
    /// owns the receive allocation, so readiness reactors avoid copying every
    /// socket read through an intermediate scratch buffer.
    // @spec apps/pgpool/tech-design/logic/p0-dense-buffer-readiness-reactor.md#logic
    pub(crate) fn read_from_sync(
        &mut self,
        stream: &mut impl std::io::Read,
    ) -> std::io::Result<usize> {
        const READ_RESERVE: usize = 16 * 1024;
        self.buf.reserve(READ_RESERVE);
        let destination = self.buf.chunk_mut();
        // SAFETY: `chunk_mut` exposes `destination.len()` writable,
        // uninitialized bytes. `Read` initializes exactly the returned prefix
        // and `advance_mut` publishes only that initialized prefix.
        let destination =
            unsafe { std::slice::from_raw_parts_mut(destination.as_mut_ptr(), destination.len()) };
        let read = std::io::Read::read(stream, destination)?;
        // SAFETY: `read` cannot exceed the slice passed to `Read::read`, and
        // that prefix was initialized by the successful call above.
        unsafe { self.buf.advance_mut(read) };
        Ok(read)
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
        self.next_frame_with_raw()
            .map(|frame| frame.map(|frame| frame.message))
    }

    /// Attempts to read and fully validate the next frame while preserving
    /// the exact wire bytes that carried it. This has identical framing and
    /// typed-decoding guarantees to [`Self::next_frame`]; the additional raw
    /// bytes are for a trusted relay write, not an unchecked bypass.
    pub fn next_frame_with_raw(&mut self) -> Result<Option<WireFrame>, FrameError> {
        let Some((frame, frame_bytes, untagged)) = self.take_frame()? else {
            return Ok(None);
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
                Ok(Some(WireFrame {
                    message: WireMessage::Frontend(message),
                    bytes: frame_bytes,
                }))
            }
            Role::Backend => {
                let message = BackendMessage::decode(&frame, &self.config)?;
                if let BackendMessage::ReadyForQuery(ready) = &message {
                    self.tx_status = ready.status;
                }
                Ok(Some(WireFrame {
                    message: WireMessage::Backend(message),
                    bytes: frame_bytes,
                }))
            }
        }
    }

    /// Decodes a frame for the transaction relay's ownership state machine.
    /// Common simple-query frames are structurally validated without building
    /// owned `String`, `Vec`, or `Bytes` fields; uncommon frames fall back to
    /// the established typed decoder. No raw frame reaches the relay without
    /// the same bounds and structural checks as [`Self::next_frame_with_raw`].
    pub fn next_relay_frame_with_raw(&mut self) -> Result<Option<RelayFrame>, FrameError> {
        let Some((frame, frame_bytes, untagged)) = self.take_frame()? else {
            return Ok(None);
        };

        let kind = match self.role {
            Role::Frontend if untagged => {
                // Transaction relay is normally entered only after startup.
                // Preserve the complete startup behavior if this API is used
                // earlier, where allocating the startup model is not hot.
                let message = FrontendMessage::decode(&frame, &self.config)?;
                if matches!(message, FrontendMessage::Startup(_)) {
                    self.awaiting_untagged_startup = false;
                }
                RelayFrameKind::Other
            }
            Role::Frontend => validate_frontend_relay(&frame, &self.config)?,
            Role::Backend => {
                let kind = validate_backend_relay(&frame, &self.config)?;
                if let RelayFrameKind::BackendReady(status) = kind {
                    self.tx_status = status;
                }
                kind
            }
        };

        Ok(Some(RelayFrame {
            kind,
            bytes: frame_bytes,
        }))
    }

    fn take_frame(&mut self) -> Result<Option<(Frame, Bytes, bool)>, FrameError> {
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
        Ok(Some((frame, frame_bytes, untagged)))
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#1876" reason="logic section in reader.rs is hand-written pending codegen support">
<!-- aw:adopt-existing -->
fn validate_frontend_relay(
    frame: &Frame,
    config: &WireCodecConfig,
) -> Result<RelayFrameKind, FrameError> {
    match frame.tag {
        Some(TAG_QUERY) => {
            let mut cur = Cursor::new(&frame.payload);
            cur.skip_cstr(Some(TAG_QUERY))?;
            cur.expect_end(Some(TAG_QUERY))?;
            Ok(RelayFrameKind::Other)
        }
        Some(TAG_TERMINATE) => {
            Cursor::new(&frame.payload).expect_end(Some(TAG_TERMINATE))?;
            Ok(RelayFrameKind::FrontendTerminate)
        }
        Some(tag @ (b'P' | b'B' | b'D' | b'E' | b'H' | b'C' | b'S')) => {
            validate_extended_query_frame(frame, config)?;
            debug_assert!(is_extended_query_tag(tag));
            Ok(RelayFrameKind::Other)
        }
        _ => {
            let _ = FrontendMessage::decode(frame, config)?;
            Ok(RelayFrameKind::Other)
        }
    }
}

impl RelayFrame {
    /// Whether this frontend relay frame begins an extended-query exchange.
    /// Transaction pooling cannot safely relay the exchange until full
    /// extended-protocol support lands, so both engines reject it before it
    /// can be queued behind a `ReadyForQuery` lease boundary.
    pub fn is_extended_query(&self) -> bool {
        self.bytes
            .first()
            .is_some_and(|tag| is_extended_query_tag(*tag))
    }
}

fn is_extended_query_tag(tag: u8) -> bool {
    matches!(tag, b'P' | b'B' | b'D' | b'E' | b'H' | b'C' | b'S')
}

fn validate_extended_query_frame(frame: &Frame, config: &WireCodecConfig) -> Result<(), FrameError> {
    match frame.tag {
        Some(b'H') => Cursor::new(&frame.payload).expect_end(Some(b'H')),
        Some(b'C') => {
            let mut cur = Cursor::new(&frame.payload);
            let target = cur.read_u8(Some(b'C'))?;
            if !matches!(target, b'S' | b'P') {
                return Err(FrameError::Malformed {
                    tag: Some(b'C'),
                    reason: format!("invalid Close target {target:?}"),
                });
            }
            cur.skip_cstr(Some(b'C'))?;
            cur.expect_end(Some(b'C'))
        }
        _ => {
            let _ = FrontendMessage::decode(frame, config)?;
            Ok(())
        }
    }
}
// </HANDWRITE>

fn validate_backend_relay(
    frame: &Frame,
    config: &WireCodecConfig,
) -> Result<RelayFrameKind, FrameError> {
    let tag = frame.tag.ok_or(FrameError::Malformed {
        tag: None,
        reason: "backend frame missing required tag byte".to_string(),
    })?;
    match tag {
        TAG_READY_FOR_QUERY => {
            let ready = ReadyForQuery::decode(&frame.payload)?;
            Ok(RelayFrameKind::BackendReady(ready.status))
        }
        TAG_DATA_ROW => {
            let mut cur = Cursor::new(&frame.payload);
            let count = read_bounded_count(&mut cur, tag, config.max_row_columns)?;
            for _ in 0..count {
                let len = cur.read_i32(Some(tag))?;
                if len == -1 {
                    continue;
                }
                if len < 0 {
                    return Err(FrameError::Malformed {
                        tag: Some(tag),
                        reason: format!("negative column length {len}"),
                    });
                }
                cur.read_exact(len as usize, Some(tag))?;
            }
            cur.expect_end(Some(tag))?;
            Ok(RelayFrameKind::Other)
        }
        TAG_ROW_DESCRIPTION => {
            let mut cur = Cursor::new(&frame.payload);
            let count = read_bounded_count(&mut cur, tag, config.max_row_columns)?;
            for _ in 0..count {
                cur.skip_cstr(Some(tag))?;
                cur.read_i32(Some(tag))?;
                cur.read_i16(Some(tag))?;
                cur.read_i32(Some(tag))?;
                cur.read_i16(Some(tag))?;
                cur.read_i32(Some(tag))?;
                cur.read_i16(Some(tag))?;
            }
            cur.expect_end(Some(tag))?;
            Ok(RelayFrameKind::Other)
        }
        TAG_COMMAND_COMPLETE => {
            let mut cur = Cursor::new(&frame.payload);
            cur.skip_cstr(Some(tag))?;
            cur.expect_end(Some(tag))?;
            Ok(RelayFrameKind::Other)
        }
        TAG_ERROR_RESPONSE | TAG_NOTICE_RESPONSE => {
            let mut cur = Cursor::new(&frame.payload);
            loop {
                let field_code = cur.read_u8(Some(tag))?;
                if field_code == 0 {
                    break;
                }
                cur.skip_cstr(Some(tag))?;
            }
            cur.expect_end(Some(tag))?;
            Ok(RelayFrameKind::Other)
        }
        // Authentication and startup-adjacent backend messages are not the
        // transaction relay's steady-state result path. Keep their existing
        // typed decoder as the compatibility fallback.
        _ => {
            let _ = BackendMessage::decode(frame, config)?;
            Ok(RelayFrameKind::Other)
        }
    }
}

fn read_bounded_count(cur: &mut Cursor<'_>, tag: u8, max: usize) -> Result<usize, FrameError> {
    let count = cur.read_i16(Some(tag))?;
    if count < 0 {
        return Err(FrameError::Malformed {
            tag: Some(tag),
            reason: format!("negative field count {count}"),
        });
    }
    let count = count as usize;
    if count > max {
        return Err(FrameError::Malformed {
            tag: Some(tag),
            reason: format!("field count {count} exceeds configured maximum {max}"),
        });
    }
    Ok(count)
}
// </HANDWRITE>
