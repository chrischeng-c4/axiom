// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#logic
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Shared byte-level read/write primitives used by `frontend`/`backend`
//! message encode/decode. Internal to the `wire` module; never panics —
//! every read that could run past the end of the payload returns a typed
//! `FrameError::Malformed` instead.

use bytes::{Bytes, BytesMut};

use crate::wire::error::FrameError;

/// A cursor over one already-length-bounded message payload.
pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(crate) fn remaining(&self) -> usize {
        self.bytes.len() - self.pos
    }

    pub(crate) fn read_u8(&mut self, tag: Option<u8>) -> Result<u8, FrameError> {
        Ok(self.read_exact(1, tag)?[0])
    }

    pub(crate) fn read_i16(&mut self, tag: Option<u8>) -> Result<i16, FrameError> {
        let s = self.read_exact(2, tag)?;
        Ok(i16::from_be_bytes([s[0], s[1]]))
    }

    pub(crate) fn read_i32(&mut self, tag: Option<u8>) -> Result<i32, FrameError> {
        let s = self.read_exact(4, tag)?;
        Ok(i32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }

    pub(crate) fn read_exact(&mut self, n: usize, tag: Option<u8>) -> Result<&'a [u8], FrameError> {
        if self.remaining() < n {
            return Err(FrameError::Malformed {
                tag,
                reason: format!(
                    "unexpected end of frame reading {n} byte(s), {} remaining",
                    self.remaining()
                ),
            });
        }
        let s = &self.bytes[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    /// Reads a null-terminated string field, validating UTF-8.
    pub(crate) fn read_cstr(&mut self, tag: Option<u8>) -> Result<String, FrameError> {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != 0 {
            self.pos += 1;
        }
        if self.pos >= self.bytes.len() {
            return Err(FrameError::Malformed {
                tag,
                reason: "unterminated string field (missing null terminator)".to_string(),
            });
        }
        let raw = &self.bytes[start..self.pos];
        self.pos += 1; // consume the null terminator
        std::str::from_utf8(raw)
            .map(str::to_string)
            .map_err(|_| FrameError::Malformed {
                tag,
                reason: "invalid UTF-8 in string field".to_string(),
            })
    }

    /// Consumes and returns every remaining byte in the payload.
    pub(crate) fn read_remaining_bytes(&mut self) -> Bytes {
        let s = Bytes::copy_from_slice(&self.bytes[self.pos..]);
        self.pos = self.bytes.len();
        s
    }

    /// Asserts the cursor consumed the whole payload; any leftover bytes are
    /// a malformed/trailing-garbage frame.
    pub(crate) fn expect_end(&self, tag: Option<u8>) -> Result<(), FrameError> {
        if self.remaining() != 0 {
            return Err(FrameError::Malformed {
                tag,
                reason: format!(
                    "{} trailing byte(s) after expected fields",
                    self.remaining()
                ),
            });
        }
        Ok(())
    }
}

/// Writes a tagged message: tag byte, i32 length (payload len + 4), then the
/// payload produced by `body`.
pub(crate) fn write_tagged(buf: &mut BytesMut, tag: u8, body: impl FnOnce(&mut BytesMut)) {
    use bytes::BufMut;
    buf.put_u8(tag);
    let length_pos = buf.len();
    buf.put_i32(0); // placeholder, backpatched below
    let body_start = buf.len();
    body(buf);
    let body_len = buf.len() - body_start;
    let length = (body_len + 4) as i32;
    buf[length_pos..length_pos + 4].copy_from_slice(&length.to_be_bytes());
}

/// Writes an untagged message: i32 length (payload len + 4), then the
/// payload produced by `body`.
pub(crate) fn write_untagged(buf: &mut BytesMut, body: impl FnOnce(&mut BytesMut)) {
    use bytes::BufMut;
    let length_pos = buf.len();
    buf.put_i32(0); // placeholder, backpatched below
    let body_start = buf.len();
    body(buf);
    let body_len = buf.len() - body_start;
    let length = (body_len + 4) as i32;
    buf[length_pos..length_pos + 4].copy_from_slice(&length.to_be_bytes());
}

/// Writes a null-terminated string field.
pub(crate) fn write_cstr(buf: &mut BytesMut, s: &str) {
    use bytes::BufMut;
    buf.put_slice(s.as_bytes());
    buf.put_u8(0);
}
// </HANDWRITE>
