// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Typed decode/read error taxonomy for the pgpool wire codec.
//!
//! The codec never panics on malformed or oversized input; every failure
//! path returns one of these variants instead.

use thiserror::Error;

/// Decode/read error taxonomy for frontend/backend message framing.
///
/// The codec never panics on malformed or oversized input (R10/R11); every
/// failure path returns one of these variants instead.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum FrameError {
    /// Declared frame length exceeds the configured bound.
    #[error("frame declared length {declared} exceeds configured maximum {max}")]
    Oversized { declared: usize, max: usize },

    /// Frame parsed under the declared length but a field was structurally
    /// invalid (bad UTF-8, truncated array, unknown enum discriminant, ...).
    #[error("malformed frame (tag={tag:?}): {reason}")]
    Malformed { tag: Option<u8>, reason: String },

    /// Tag byte does not match any known frontend/backend message in this
    /// slice's scope.
    #[error("unknown message tag byte {tag:#04x}")]
    UnknownTag { tag: u8 },

    /// Underlying stream I/O error surfaced from the reader.
    #[error("wire I/O error: {0}")]
    Io(String),
}
// </HANDWRITE>
