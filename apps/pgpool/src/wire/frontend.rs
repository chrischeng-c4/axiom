// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Client-to-server (frontend) PostgreSQL protocol 3.0 message variants this
//! slice decodes/encodes.

use bytes::{Bytes, BytesMut};

use crate::wire::codec::{write_cstr, write_tagged, write_untagged, Cursor};
use crate::wire::config::WireCodecConfig;
use crate::wire::error::FrameError;
use crate::wire::frame::Frame;

pub const TAG_PASSWORD: u8 = b'p';
pub const TAG_QUERY: u8 = b'Q';
pub const TAG_PARSE: u8 = b'P';
pub const TAG_BIND: u8 = b'B';
pub const TAG_DESCRIBE: u8 = b'D';
pub const TAG_EXECUTE: u8 = b'E';
pub const TAG_SYNC: u8 = b'S';
pub const TAG_TERMINATE: u8 = b'X';

const STARTUP_PROTOCOL_MAJOR: i32 = 3;
const STARTUP_PROTOCOL_MINOR: i32 = 0;
const SSL_REQUEST_CODE: i32 = 80_877_103;

/// Client-to-server message variants this slice decodes/encodes.
#[derive(Debug, Clone, PartialEq)]
pub enum FrontendMessage {
    Startup(StartupMessage),
    Ssl(SslRequest),
    Password(PasswordMessage),
    SaslInitialResponse(SaslInitialResponse),
    SaslResponse(SaslResponse),
    Query(Query),
    Parse(Parse),
    Bind(Bind),
    Describe(Describe),
    Execute(Execute),
    Sync(Sync),
    Terminate(Terminate),
}

impl FrontendMessage {
    /// Serializes the message's tag byte (if any) + i32 length + field
    /// payload into a caller-supplied `BytesMut` per protocol 3.0 layout.
    pub fn encode(&self, buf: &mut BytesMut) {
        match self {
            FrontendMessage::Startup(m) => m.encode(buf),
            FrontendMessage::Ssl(m) => m.encode(buf),
            FrontendMessage::Password(m) => m.encode(buf),
            FrontendMessage::SaslInitialResponse(m) => m.encode(buf),
            FrontendMessage::SaslResponse(m) => m.encode(buf),
            FrontendMessage::Query(m) => m.encode(buf),
            FrontendMessage::Parse(m) => m.encode(buf),
            FrontendMessage::Bind(m) => m.encode(buf),
            FrontendMessage::Describe(m) => m.encode(buf),
            FrontendMessage::Execute(m) => m.encode(buf),
            FrontendMessage::Sync(m) => m.encode(buf),
            FrontendMessage::Terminate(m) => m.encode(buf),
        }
    }

    /// Decodes a fully-buffered `Frame` into a typed `FrontendMessage` by
    /// tag byte (or, untagged, by protocol version code for
    /// StartupMessage/SSLRequest).
    ///
    /// A tagged frame with tag `'p'` always decodes as [`PasswordMessage`]
    /// (the common cleartext/MD5-auth case): the wire format cannot
    /// distinguish `PasswordMessage`/`SaslInitialResponse`/`SaslResponse` by
    /// bytes alone since all three share tag `'p'` — a caller that knows the
    /// session is mid-SASL-handshake should call
    /// [`SaslInitialResponse::decode`]/[`SaslResponse::decode`] directly on
    /// `frame.payload` instead of going through this generic dispatch.
    pub fn decode(frame: &Frame, config: &WireCodecConfig) -> Result<FrontendMessage, FrameError> {
        match frame.tag {
            None => decode_untagged(&frame.payload, config),
            Some(TAG_PASSWORD) => Ok(FrontendMessage::Password(PasswordMessage::decode(
                &frame.payload,
            ))),
            Some(TAG_QUERY) => Query::decode(&frame.payload).map(FrontendMessage::Query),
            Some(TAG_PARSE) => Parse::decode(&frame.payload, config).map(FrontendMessage::Parse),
            Some(TAG_BIND) => Bind::decode(&frame.payload, config).map(FrontendMessage::Bind),
            Some(TAG_DESCRIBE) => Describe::decode(&frame.payload).map(FrontendMessage::Describe),
            Some(TAG_EXECUTE) => Execute::decode(&frame.payload).map(FrontendMessage::Execute),
            Some(TAG_SYNC) => {
                Cursor::new(&frame.payload).expect_end(Some(TAG_SYNC))?;
                Ok(FrontendMessage::Sync(Sync))
            }
            Some(TAG_TERMINATE) => {
                Cursor::new(&frame.payload).expect_end(Some(TAG_TERMINATE))?;
                Ok(FrontendMessage::Terminate(Terminate))
            }
            Some(other) => Err(FrameError::UnknownTag { tag: other }),
        }
    }
}

fn decode_untagged(
    payload: &Bytes,
    config: &WireCodecConfig,
) -> Result<FrontendMessage, FrameError> {
    let mut cur = Cursor::new(payload);
    let code = cur.read_i32(None)?;
    if code == SSL_REQUEST_CODE {
        cur.expect_end(None)?;
        return Ok(FrontendMessage::Ssl(SslRequest));
    }
    let major = (code >> 16) & 0xFFFF;
    let minor = code & 0xFFFF;
    if major != STARTUP_PROTOCOL_MAJOR || minor != STARTUP_PROTOCOL_MINOR {
        return Err(FrameError::Malformed {
            tag: None,
            reason: format!("unsupported startup protocol version {major}.{minor}"),
        });
    }
    let mut parameters = Vec::new();
    loop {
        if parameters.len() >= config.max_bind_params {
            return Err(FrameError::Malformed {
                tag: None,
                reason: format!(
                    "startup parameter count exceeds configured maximum {}",
                    config.max_bind_params
                ),
            });
        }
        let key = cur.read_cstr(None)?;
        if key.is_empty() {
            break;
        }
        let value = cur.read_cstr(None)?;
        parameters.push((key, value));
    }
    cur.expect_end(None)?;
    Ok(FrontendMessage::Startup(StartupMessage {
        protocol_major: major,
        protocol_minor: minor,
        parameters,
    }))
}

/// Untagged startup packet (no leading tag byte): 4-byte length, 4-byte
/// protocol version, then a null-terminated key/value parameter list
/// terminated by an empty string.
///
/// TD deviation: the schema types `parameters` as a JSON-Schema `object`
/// (map-shaped), but its description calls out "ordered key/value pairs" and
/// R1 requires a byte-for-byte round trip. A `BTreeMap`/`HashMap` would
/// re-sort or reorder keys on re-encode and silently break that byte-exact
/// guarantee, so this implements `parameters` as `Vec<(String, String)>` to
/// preserve wire order losslessly.
#[derive(Debug, Clone, PartialEq)]
pub struct StartupMessage {
    pub protocol_major: i32,
    pub protocol_minor: i32,
    pub parameters: Vec<(String, String)>,
}

impl StartupMessage {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_untagged(buf, |buf| {
            let code = (self.protocol_major << 16) | (self.protocol_minor & 0xFFFF);
            buf.put_i32(code);
            for (k, v) in &self.parameters {
                write_cstr(buf, k);
                write_cstr(buf, v);
            }
            buf.put_u8(0); // empty-string terminator
        });
    }
}

/// Untagged 8-byte SSLRequest packet (length=8, request code 80877103); no
/// fields beyond identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SslRequest;

impl SslRequest {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_untagged(buf, |buf| buf.put_i32(SSL_REQUEST_CODE));
    }
}

/// Tag `'p'`. Carries either a cleartext/MD5 password string or a raw SASL
/// frame's opaque bytes (SCRAM crypto is out of scope for this slice; bytes
/// are parsed/relayed only).
#[derive(Debug, Clone, PartialEq)]
pub struct PasswordMessage {
    pub payload: Bytes,
}

impl PasswordMessage {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_PASSWORD, |buf| buf.put_slice(&self.payload));
    }

    pub fn decode(payload: &Bytes) -> PasswordMessage {
        PasswordMessage {
            payload: payload.clone(),
        }
    }
}

/// Tag `'p'` variant carrying SASL mechanism name + initial response bytes
/// (null-terminated mechanism, then i32 length + opaque bytes, -1 length =
/// no response).
#[derive(Debug, Clone, PartialEq)]
pub struct SaslInitialResponse {
    pub mechanism: String,
    pub response: Option<Bytes>,
}

impl SaslInitialResponse {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_PASSWORD, |buf| {
            write_cstr(buf, &self.mechanism);
            match &self.response {
                Some(bytes) => {
                    buf.put_i32(bytes.len() as i32);
                    buf.put_slice(bytes);
                }
                None => buf.put_i32(-1),
            }
        });
    }

    pub fn decode(payload: &Bytes) -> Result<SaslInitialResponse, FrameError> {
        let mut cur = Cursor::new(payload);
        let mechanism = cur.read_cstr(Some(TAG_PASSWORD))?;
        let len = cur.read_i32(Some(TAG_PASSWORD))?;
        let response = if len == -1 {
            None
        } else if len < 0 {
            return Err(FrameError::Malformed {
                tag: Some(TAG_PASSWORD),
                reason: format!("negative SASL initial response length {len}"),
            });
        } else {
            Some(Bytes::copy_from_slice(
                cur.read_exact(len as usize, Some(TAG_PASSWORD))?,
            ))
        };
        cur.expect_end(Some(TAG_PASSWORD))?;
        Ok(SaslInitialResponse {
            mechanism,
            response,
        })
    }
}

/// Tag `'p'` variant carrying a SASL continuation frame's opaque bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct SaslResponse {
    pub payload: Bytes,
}

impl SaslResponse {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_PASSWORD, |buf| buf.put_slice(&self.payload));
    }

    pub fn decode(payload: &Bytes) -> SaslResponse {
        SaslResponse {
            payload: payload.clone(),
        }
    }
}

/// Tag `'Q'`. Simple query protocol: a single null-terminated SQL string.
#[derive(Debug, Clone, PartialEq)]
pub struct Query {
    pub sql: String,
}

impl Query {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_tagged(buf, TAG_QUERY, |buf| write_cstr(buf, &self.sql));
    }

    pub fn decode(payload: &Bytes) -> Result<Query, FrameError> {
        let mut cur = Cursor::new(payload);
        let sql = cur.read_cstr(Some(TAG_QUERY))?;
        cur.expect_end(Some(TAG_QUERY))?;
        Ok(Query { sql })
    }
}

/// Tag `'P'`. Extended query: prepare a statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Parse {
    pub statement_name: String,
    pub sql: String,
    pub param_type_oids: Vec<i32>,
}

impl Parse {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_PARSE, |buf| {
            write_cstr(buf, &self.statement_name);
            write_cstr(buf, &self.sql);
            buf.put_i16(self.param_type_oids.len() as i16);
            for oid in &self.param_type_oids {
                buf.put_i32(*oid);
            }
        });
    }

    pub fn decode(payload: &Bytes, config: &WireCodecConfig) -> Result<Parse, FrameError> {
        let mut cur = Cursor::new(payload);
        let statement_name = cur.read_cstr(Some(TAG_PARSE))?;
        let sql = cur.read_cstr(Some(TAG_PARSE))?;
        let count = read_bounded_count(&mut cur, Some(TAG_PARSE), config.max_bind_params)?;
        let mut param_type_oids = Vec::with_capacity(count);
        for _ in 0..count {
            param_type_oids.push(cur.read_i32(Some(TAG_PARSE))?);
        }
        cur.expect_end(Some(TAG_PARSE))?;
        Ok(Parse {
            statement_name,
            sql,
            param_type_oids,
        })
    }
}

/// Tag `'B'`. Extended query: bind a portal to a prepared statement with
/// parameter values.
#[derive(Debug, Clone, PartialEq)]
pub struct Bind {
    pub portal_name: String,
    pub statement_name: String,
    pub param_formats: Vec<i16>,
    /// `None` = SQL NULL parameter.
    pub param_values: Vec<Option<Bytes>>,
    pub result_formats: Vec<i16>,
}

impl Bind {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_BIND, |buf| {
            write_cstr(buf, &self.portal_name);
            write_cstr(buf, &self.statement_name);
            buf.put_i16(self.param_formats.len() as i16);
            for format in &self.param_formats {
                buf.put_i16(*format);
            }
            buf.put_i16(self.param_values.len() as i16);
            for value in &self.param_values {
                match value {
                    Some(bytes) => {
                        buf.put_i32(bytes.len() as i32);
                        buf.put_slice(bytes);
                    }
                    None => buf.put_i32(-1),
                }
            }
            buf.put_i16(self.result_formats.len() as i16);
            for format in &self.result_formats {
                buf.put_i16(*format);
            }
        });
    }

    pub fn decode(payload: &Bytes, config: &WireCodecConfig) -> Result<Bind, FrameError> {
        let mut cur = Cursor::new(payload);
        let portal_name = cur.read_cstr(Some(TAG_BIND))?;
        let statement_name = cur.read_cstr(Some(TAG_BIND))?;

        let format_count = read_bounded_count(&mut cur, Some(TAG_BIND), config.max_bind_params)?;
        let mut param_formats = Vec::with_capacity(format_count);
        for _ in 0..format_count {
            param_formats.push(cur.read_i16(Some(TAG_BIND))?);
        }

        let value_count = read_bounded_count(&mut cur, Some(TAG_BIND), config.max_bind_params)?;
        let mut param_values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            let len = cur.read_i32(Some(TAG_BIND))?;
            if len == -1 {
                param_values.push(None);
            } else if len < 0 {
                return Err(FrameError::Malformed {
                    tag: Some(TAG_BIND),
                    reason: format!("negative bind parameter length {len}"),
                });
            } else {
                param_values.push(Some(Bytes::copy_from_slice(
                    cur.read_exact(len as usize, Some(TAG_BIND))?,
                )));
            }
        }

        let result_format_count =
            read_bounded_count(&mut cur, Some(TAG_BIND), config.max_bind_params)?;
        let mut result_formats = Vec::with_capacity(result_format_count);
        for _ in 0..result_format_count {
            result_formats.push(cur.read_i16(Some(TAG_BIND))?);
        }

        cur.expect_end(Some(TAG_BIND))?;
        Ok(Bind {
            portal_name,
            statement_name,
            param_formats,
            param_values,
            result_formats,
        })
    }
}

/// TD deviation: the schema types `target_kind` as a string enum
/// (`"statement"`/`"portal"`); this implements it as a closed Rust enum
/// (rather than a raw `String`) so an invalid target byte on the wire is
/// rejected by the type system at construction, not just at decode time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DescribeTarget {
    Statement,
    Portal,
}

/// Tag `'D'`. Extended query: describe a statement or portal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Describe {
    pub target_kind: DescribeTarget,
    pub name: String,
}

impl Describe {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_DESCRIBE, |buf| {
            let kind_byte = match self.target_kind {
                DescribeTarget::Statement => b'S',
                DescribeTarget::Portal => b'P',
            };
            buf.put_u8(kind_byte);
            write_cstr(buf, &self.name);
        });
    }

    pub fn decode(payload: &Bytes) -> Result<Describe, FrameError> {
        let mut cur = Cursor::new(payload);
        let kind_byte = cur.read_u8(Some(TAG_DESCRIBE))?;
        let target_kind = match kind_byte {
            b'S' => DescribeTarget::Statement,
            b'P' => DescribeTarget::Portal,
            other => {
                return Err(FrameError::Malformed {
                    tag: Some(TAG_DESCRIBE),
                    reason: format!("unknown describe target byte {other:#04x}"),
                })
            }
        };
        let name = cur.read_cstr(Some(TAG_DESCRIBE))?;
        cur.expect_end(Some(TAG_DESCRIBE))?;
        Ok(Describe { target_kind, name })
    }
}

/// Tag `'E'`. Extended query: execute a bound portal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execute {
    pub portal_name: String,
    /// 0 = no limit.
    pub max_rows: i32,
}

impl Execute {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_EXECUTE, |buf| {
            write_cstr(buf, &self.portal_name);
            buf.put_i32(self.max_rows);
        });
    }

    pub fn decode(payload: &Bytes) -> Result<Execute, FrameError> {
        let mut cur = Cursor::new(payload);
        let portal_name = cur.read_cstr(Some(TAG_EXECUTE))?;
        let max_rows = cur.read_i32(Some(TAG_EXECUTE))?;
        cur.expect_end(Some(TAG_EXECUTE))?;
        Ok(Execute {
            portal_name,
            max_rows,
        })
    }
}

/// Tag `'S'`. Extended query: sync, closing the current extended-query
/// message stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sync;

impl Sync {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_tagged(buf, TAG_SYNC, |_buf| {});
    }
}

/// Tag `'X'`. Graceful client-initiated close.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Terminate;

impl Terminate {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_tagged(buf, TAG_TERMINATE, |_buf| {});
    }
}

fn read_bounded_count(
    cur: &mut Cursor<'_>,
    tag: Option<u8>,
    max: usize,
) -> Result<usize, FrameError> {
    let count = cur.read_i16(tag)?;
    if count < 0 {
        return Err(FrameError::Malformed {
            tag,
            reason: format!("negative field count {count}"),
        });
    }
    let count = count as usize;
    if count > max {
        return Err(FrameError::Malformed {
            tag,
            reason: format!("field count {count} exceeds configured maximum {max}"),
        });
    }
    Ok(count)
}
// </HANDWRITE>
