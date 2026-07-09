// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Raw wire `Frame` envelope: a fully-buffered frame as read off the stream,
//! before typed decode into a `FrontendMessage`/`BackendMessage`.

use bytes::Bytes;

/// One fully-buffered wire frame as read off the stream, before typed
/// decode: optional tag byte + declared length + raw payload bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    /// Tag byte for tagged frames; `None` for the untagged
    /// StartupMessage/SSLRequest/CancelRequest family.
    pub tag: Option<u8>,
    /// Raw payload bytes after the tag+length header, exactly
    /// `declared_length - 4` bytes (or `-4` from the untagged length field).
    pub payload: Bytes,
}
// </HANDWRITE>
