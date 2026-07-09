// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// <HANDWRITE gap="missing-generator:logic:pg-wire-codec" tracker="#1287" reason="Wire protocol codec needs generator primitives that do not exist yet.">
//! Server-to-client (backend) PostgreSQL protocol 3.0 message variants this
//! slice decodes/encodes, plus `TransactionStatus` (see the TD's
//! Transaction Status Tracking state machine).

use bytes::{Bytes, BytesMut};

use crate::wire::codec::{write_cstr, write_tagged, Cursor};
use crate::wire::config::WireCodecConfig;
use crate::wire::error::FrameError;
use crate::wire::frame::Frame;

pub const TAG_AUTHENTICATION: u8 = b'R';
pub const TAG_PARAMETER_STATUS: u8 = b'S';
pub const TAG_BACKEND_KEY_DATA: u8 = b'K';
pub const TAG_READY_FOR_QUERY: u8 = b'Z';
pub const TAG_ROW_DESCRIPTION: u8 = b'T';
pub const TAG_DATA_ROW: u8 = b'D';
pub const TAG_COMMAND_COMPLETE: u8 = b'C';
pub const TAG_ERROR_RESPONSE: u8 = b'E';
pub const TAG_NOTICE_RESPONSE: u8 = b'N';

const AUTH_OK: i32 = 0;
const AUTH_CLEARTEXT_PASSWORD: i32 = 3;
const AUTH_MD5_PASSWORD: i32 = 5;
const AUTH_SASL: i32 = 10;
const AUTH_SASL_CONTINUE: i32 = 11;
const AUTH_SASL_FINAL: i32 = 12;

/// Server-to-client message variants this slice decodes/encodes.
#[derive(Debug, Clone, PartialEq)]
pub enum BackendMessage {
    AuthenticationOk(AuthenticationOk),
    AuthenticationCleartextPassword(AuthenticationCleartextPassword),
    AuthenticationMd5Password(AuthenticationMd5Password),
    AuthenticationSasl(AuthenticationSasl),
    AuthenticationSaslContinue(AuthenticationSaslContinue),
    AuthenticationSaslFinal(AuthenticationSaslFinal),
    ParameterStatus(ParameterStatus),
    BackendKeyData(BackendKeyData),
    ReadyForQuery(ReadyForQuery),
    RowDescription(RowDescription),
    DataRow(DataRow),
    CommandComplete(CommandComplete),
    ErrorResponse(ErrorResponse),
    NoticeResponse(NoticeResponse),
}

impl BackendMessage {
    /// Serializes the message's tag byte + i32 length + field payload into a
    /// caller-supplied `BytesMut` per protocol 3.0 layout.
    pub fn encode(&self, buf: &mut BytesMut) {
        match self {
            BackendMessage::AuthenticationOk(m) => m.encode(buf),
            BackendMessage::AuthenticationCleartextPassword(m) => m.encode(buf),
            BackendMessage::AuthenticationMd5Password(m) => m.encode(buf),
            BackendMessage::AuthenticationSasl(m) => m.encode(buf),
            BackendMessage::AuthenticationSaslContinue(m) => m.encode(buf),
            BackendMessage::AuthenticationSaslFinal(m) => m.encode(buf),
            BackendMessage::ParameterStatus(m) => m.encode(buf),
            BackendMessage::BackendKeyData(m) => m.encode(buf),
            BackendMessage::ReadyForQuery(m) => m.encode(buf),
            BackendMessage::RowDescription(m) => m.encode(buf),
            BackendMessage::DataRow(m) => m.encode(buf),
            BackendMessage::CommandComplete(m) => m.encode(buf),
            BackendMessage::ErrorResponse(m) => m.encode(buf),
            BackendMessage::NoticeResponse(m) => m.encode(buf),
        }
    }

    /// Decodes a fully-buffered `Frame` into a typed `BackendMessage` by tag
    /// byte (and, for tag `'R'`, by the 4-byte authentication sub-code).
    pub fn decode(frame: &Frame, config: &WireCodecConfig) -> Result<BackendMessage, FrameError> {
        let tag = frame.tag.ok_or(FrameError::Malformed {
            tag: None,
            reason: "backend frame missing required tag byte".to_string(),
        })?;
        match tag {
            TAG_AUTHENTICATION => decode_authentication(&frame.payload),
            TAG_PARAMETER_STATUS => {
                ParameterStatus::decode(&frame.payload).map(BackendMessage::ParameterStatus)
            }
            TAG_BACKEND_KEY_DATA => {
                BackendKeyData::decode(&frame.payload).map(BackendMessage::BackendKeyData)
            }
            TAG_READY_FOR_QUERY => {
                ReadyForQuery::decode(&frame.payload).map(BackendMessage::ReadyForQuery)
            }
            TAG_ROW_DESCRIPTION => {
                RowDescription::decode(&frame.payload, config).map(BackendMessage::RowDescription)
            }
            TAG_DATA_ROW => DataRow::decode(&frame.payload, config).map(BackendMessage::DataRow),
            TAG_COMMAND_COMPLETE => {
                CommandComplete::decode(&frame.payload).map(BackendMessage::CommandComplete)
            }
            TAG_ERROR_RESPONSE => {
                ErrorResponse::decode(&frame.payload).map(BackendMessage::ErrorResponse)
            }
            TAG_NOTICE_RESPONSE => {
                NoticeResponse::decode(&frame.payload).map(BackendMessage::NoticeResponse)
            }
            other => Err(FrameError::UnknownTag { tag: other }),
        }
    }
}

fn decode_authentication(payload: &Bytes) -> Result<BackendMessage, FrameError> {
    let mut cur = Cursor::new(payload);
    let code = cur.read_i32(Some(TAG_AUTHENTICATION))?;
    match code {
        AUTH_OK => {
            cur.expect_end(Some(TAG_AUTHENTICATION))?;
            Ok(BackendMessage::AuthenticationOk(AuthenticationOk))
        }
        AUTH_CLEARTEXT_PASSWORD => {
            cur.expect_end(Some(TAG_AUTHENTICATION))?;
            Ok(BackendMessage::AuthenticationCleartextPassword(
                AuthenticationCleartextPassword,
            ))
        }
        AUTH_MD5_PASSWORD => {
            let salt_bytes = cur.read_exact(4, Some(TAG_AUTHENTICATION))?;
            let mut salt = [0u8; 4];
            salt.copy_from_slice(salt_bytes);
            cur.expect_end(Some(TAG_AUTHENTICATION))?;
            Ok(BackendMessage::AuthenticationMd5Password(
                AuthenticationMd5Password { salt },
            ))
        }
        AUTH_SASL => {
            let mut mechanisms = Vec::new();
            loop {
                let mechanism = cur.read_cstr(Some(TAG_AUTHENTICATION))?;
                if mechanism.is_empty() {
                    break;
                }
                mechanisms.push(mechanism);
            }
            cur.expect_end(Some(TAG_AUTHENTICATION))?;
            Ok(BackendMessage::AuthenticationSasl(AuthenticationSasl {
                mechanisms,
            }))
        }
        AUTH_SASL_CONTINUE => {
            let payload = cur.read_remaining_bytes();
            Ok(BackendMessage::AuthenticationSaslContinue(
                AuthenticationSaslContinue { payload },
            ))
        }
        AUTH_SASL_FINAL => {
            let payload = cur.read_remaining_bytes();
            Ok(BackendMessage::AuthenticationSaslFinal(
                AuthenticationSaslFinal { payload },
            ))
        }
        other => Err(FrameError::Malformed {
            tag: Some(TAG_AUTHENTICATION),
            reason: format!("unknown authentication sub-code {other}"),
        }),
    }
}

/// Tag `'R'`, auth type code 0: authentication succeeded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticationOk;

impl AuthenticationOk {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| buf.put_i32(AUTH_OK));
    }
}

/// Tag `'R'`, auth type code 3: client must send a cleartext
/// `PasswordMessage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuthenticationCleartextPassword;

impl AuthenticationCleartextPassword {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| {
            buf.put_i32(AUTH_CLEARTEXT_PASSWORD)
        });
    }
}

/// Tag `'R'`, auth type code 5: client must send an MD5-hashed
/// `PasswordMessage` using this 4-byte salt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationMd5Password {
    pub salt: [u8; 4],
}

impl AuthenticationMd5Password {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| {
            buf.put_i32(AUTH_MD5_PASSWORD);
            buf.put_slice(&self.salt);
        });
    }
}

/// Tag `'R'`, auth type code 10: server offers a list of SASL mechanism
/// names, null-terminated list terminated by an empty string.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticationSasl {
    pub mechanisms: Vec<String>,
}

impl AuthenticationSasl {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| {
            buf.put_i32(AUTH_SASL);
            for mechanism in &self.mechanisms {
                write_cstr(buf, mechanism);
            }
            buf.put_u8(0);
        });
    }
}

/// Tag `'R'`, auth type code 11: opaque SASL continuation bytes
/// (server-first-message).
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticationSaslContinue {
    pub payload: Bytes,
}

impl AuthenticationSaslContinue {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| {
            buf.put_i32(AUTH_SASL_CONTINUE);
            buf.put_slice(&self.payload);
        });
    }
}

/// Tag `'R'`, auth type code 12: opaque SASL final bytes
/// (server-final-message).
#[derive(Debug, Clone, PartialEq)]
pub struct AuthenticationSaslFinal {
    pub payload: Bytes,
}

impl AuthenticationSaslFinal {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_AUTHENTICATION, |buf| {
            buf.put_i32(AUTH_SASL_FINAL);
            buf.put_slice(&self.payload);
        });
    }
}

/// Tag `'S'`. Runtime parameter report (`server_version`, `client_encoding`,
/// ...).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterStatus {
    pub name: String,
    pub value: String,
}

impl ParameterStatus {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_tagged(buf, TAG_PARAMETER_STATUS, |buf| {
            write_cstr(buf, &self.name);
            write_cstr(buf, &self.value);
        });
    }

    pub fn decode(payload: &Bytes) -> Result<ParameterStatus, FrameError> {
        let mut cur = Cursor::new(payload);
        let name = cur.read_cstr(Some(TAG_PARAMETER_STATUS))?;
        let value = cur.read_cstr(Some(TAG_PARAMETER_STATUS))?;
        cur.expect_end(Some(TAG_PARAMETER_STATUS))?;
        Ok(ParameterStatus { name, value })
    }
}

/// Tag `'K'`. Cancellation key data for this backend connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackendKeyData {
    pub process_id: i32,
    pub secret_key: i32,
}

impl BackendKeyData {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_BACKEND_KEY_DATA, |buf| {
            buf.put_i32(self.process_id);
            buf.put_i32(self.secret_key);
        });
    }

    pub fn decode(payload: &Bytes) -> Result<BackendKeyData, FrameError> {
        let mut cur = Cursor::new(payload);
        let process_id = cur.read_i32(Some(TAG_BACKEND_KEY_DATA))?;
        let secret_key = cur.read_i32(Some(TAG_BACKEND_KEY_DATA))?;
        cur.expect_end(Some(TAG_BACKEND_KEY_DATA))?;
        Ok(BackendKeyData {
            process_id,
            secret_key,
        })
    }
}

/// Tag `'Z'`. Marks the backend ready for a new query cycle; status drives
/// `TransactionStatus` tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadyForQuery {
    pub status: TransactionStatus,
}

impl ReadyForQuery {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_READY_FOR_QUERY, |buf| {
            buf.put_u8(self.status.to_status_byte())
        });
    }

    pub fn decode(payload: &Bytes) -> Result<ReadyForQuery, FrameError> {
        let mut cur = Cursor::new(payload);
        let byte = cur.read_u8(Some(TAG_READY_FOR_QUERY))?;
        let status = TransactionStatus::from_status_byte(byte)?;
        cur.expect_end(Some(TAG_READY_FOR_QUERY))?;
        Ok(ReadyForQuery { status })
    }
}

/// Decoded from the ReadyForQuery status byte: `'I'` -> idle, `'T'` ->
/// in_transaction, `'E'` -> failed. See the Transaction Status Tracking
/// state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionStatus {
    Idle,
    InTransaction,
    Failed,
}

impl TransactionStatus {
    pub fn from_status_byte(byte: u8) -> Result<TransactionStatus, FrameError> {
        match byte {
            b'I' => Ok(TransactionStatus::Idle),
            b'T' => Ok(TransactionStatus::InTransaction),
            b'E' => Ok(TransactionStatus::Failed),
            other => Err(FrameError::Malformed {
                tag: Some(TAG_READY_FOR_QUERY),
                reason: format!("unknown transaction status byte {other:#04x}"),
            }),
        }
    }

    pub fn to_status_byte(self) -> u8 {
        match self {
            TransactionStatus::Idle => b'I',
            TransactionStatus::InTransaction => b'T',
            TransactionStatus::Failed => b'E',
        }
    }
}

/// Tag `'T'`. Column metadata for a result set.
#[derive(Debug, Clone, PartialEq)]
pub struct RowDescription {
    pub fields: Vec<FieldDescription>,
}

impl RowDescription {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_ROW_DESCRIPTION, |buf| {
            buf.put_i16(self.fields.len() as i16);
            for field in &self.fields {
                write_cstr(buf, &field.name);
                buf.put_i32(field.table_oid);
                buf.put_i16(field.column_attr);
                buf.put_i32(field.type_oid);
                buf.put_i16(field.type_size);
                buf.put_i32(field.type_modifier);
                buf.put_i16(field.format);
            }
        });
    }

    pub fn decode(payload: &Bytes, config: &WireCodecConfig) -> Result<RowDescription, FrameError> {
        let mut cur = Cursor::new(payload);
        let count =
            read_bounded_count(&mut cur, Some(TAG_ROW_DESCRIPTION), config.max_row_columns)?;
        let mut fields = Vec::with_capacity(count);
        for _ in 0..count {
            let name = cur.read_cstr(Some(TAG_ROW_DESCRIPTION))?;
            let table_oid = cur.read_i32(Some(TAG_ROW_DESCRIPTION))?;
            let column_attr = cur.read_i16(Some(TAG_ROW_DESCRIPTION))?;
            let type_oid = cur.read_i32(Some(TAG_ROW_DESCRIPTION))?;
            let type_size = cur.read_i16(Some(TAG_ROW_DESCRIPTION))?;
            let type_modifier = cur.read_i32(Some(TAG_ROW_DESCRIPTION))?;
            let format = cur.read_i16(Some(TAG_ROW_DESCRIPTION))?;
            fields.push(FieldDescription {
                name,
                table_oid,
                column_attr,
                type_oid,
                type_size,
                type_modifier,
                format,
            });
        }
        cur.expect_end(Some(TAG_ROW_DESCRIPTION))?;
        Ok(RowDescription { fields })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDescription {
    pub name: String,
    pub table_oid: i32,
    pub column_attr: i16,
    pub type_oid: i32,
    pub type_size: i16,
    pub type_modifier: i32,
    pub format: i16,
}

/// Tag `'D'`. One result row; each column is either raw bytes or SQL NULL.
#[derive(Debug, Clone, PartialEq)]
pub struct DataRow {
    pub columns: Vec<Option<Bytes>>,
}

impl DataRow {
    pub fn encode(&self, buf: &mut BytesMut) {
        use bytes::BufMut;
        write_tagged(buf, TAG_DATA_ROW, |buf| {
            buf.put_i16(self.columns.len() as i16);
            for column in &self.columns {
                match column {
                    Some(bytes) => {
                        buf.put_i32(bytes.len() as i32);
                        buf.put_slice(bytes);
                    }
                    None => buf.put_i32(-1),
                }
            }
        });
    }

    pub fn decode(payload: &Bytes, config: &WireCodecConfig) -> Result<DataRow, FrameError> {
        let mut cur = Cursor::new(payload);
        let count = read_bounded_count(&mut cur, Some(TAG_DATA_ROW), config.max_row_columns)?;
        let mut columns = Vec::with_capacity(count);
        for _ in 0..count {
            let len = cur.read_i32(Some(TAG_DATA_ROW))?;
            if len == -1 {
                columns.push(None);
            } else if len < 0 {
                return Err(FrameError::Malformed {
                    tag: Some(TAG_DATA_ROW),
                    reason: format!("negative column length {len}"),
                });
            } else {
                columns.push(Some(Bytes::copy_from_slice(
                    cur.read_exact(len as usize, Some(TAG_DATA_ROW))?,
                )));
            }
        }
        cur.expect_end(Some(TAG_DATA_ROW))?;
        Ok(DataRow { columns })
    }
}

/// Tag `'C'`. Command completion tag string (e.g. `"SELECT 3"`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandComplete {
    pub tag: String,
}

impl CommandComplete {
    pub fn encode(&self, buf: &mut BytesMut) {
        write_tagged(buf, TAG_COMMAND_COMPLETE, |buf| write_cstr(buf, &self.tag));
    }

    pub fn decode(payload: &Bytes) -> Result<CommandComplete, FrameError> {
        let mut cur = Cursor::new(payload);
        let tag = cur.read_cstr(Some(TAG_COMMAND_COMPLETE))?;
        cur.expect_end(Some(TAG_COMMAND_COMPLETE))?;
        Ok(CommandComplete { tag })
    }
}

/// Tag `'E'`. Backend error; a set of typed one-byte-code fields terminated
/// by a null byte.
///
/// TD deviation: the schema types `fields` as a JSON-Schema `object` keyed
/// by the field-code character (`additionalProperties: {type: string}`).
/// The wire format's field code is fundamentally a single byte (`'S'`
/// severity, `'C'` sqlstate, `'M'` message, ...), not a `String`, so this
/// implements `fields` as `Vec<(u8, String)>` — preserving both the byte
/// typing and the on-wire field order for a faithful round trip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorResponse {
    pub fields: Vec<(u8, String)>,
}

impl ErrorResponse {
    pub fn encode(&self, buf: &mut BytesMut) {
        encode_notice_fields(buf, TAG_ERROR_RESPONSE, &self.fields);
    }

    pub fn decode(payload: &Bytes) -> Result<ErrorResponse, FrameError> {
        Ok(ErrorResponse {
            fields: decode_notice_fields(payload, TAG_ERROR_RESPONSE)?,
        })
    }
}

/// Tag `'N'`. Same field structure as `ErrorResponse`, non-fatal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoticeResponse {
    pub fields: Vec<(u8, String)>,
}

impl NoticeResponse {
    pub fn encode(&self, buf: &mut BytesMut) {
        encode_notice_fields(buf, TAG_NOTICE_RESPONSE, &self.fields);
    }

    pub fn decode(payload: &Bytes) -> Result<NoticeResponse, FrameError> {
        Ok(NoticeResponse {
            fields: decode_notice_fields(payload, TAG_NOTICE_RESPONSE)?,
        })
    }
}

fn encode_notice_fields(buf: &mut BytesMut, tag: u8, fields: &[(u8, String)]) {
    use bytes::BufMut;
    write_tagged(buf, tag, |buf| {
        for (code, value) in fields {
            buf.put_u8(*code);
            write_cstr(buf, value);
        }
        buf.put_u8(0);
    });
}

fn decode_notice_fields(payload: &Bytes, tag: u8) -> Result<Vec<(u8, String)>, FrameError> {
    let mut cur = Cursor::new(payload);
    let mut fields = Vec::new();
    loop {
        let code = cur.read_u8(Some(tag))?;
        if code == 0 {
            break;
        }
        let value = cur.read_cstr(Some(tag))?;
        fields.push((code, value));
    }
    cur.expect_end(Some(tag))?;
    Ok(fields)
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
// SPEC-MANAGED: apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
// CODEGEN-BEGIN
use serde::{Deserialize, Serialize};

/// Tag 'R', auth type code 5: client must send an MD5-hashed PasswordMessage using this 4-byte salt.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationMd5Password {
    pub salt: Vec<i64>,
}

/// Tag 'R', auth type code 10: server offers a list of SASL mechanism names, null-terminated list terminated by an empty string.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationSasl {
    pub mechanisms: Vec<String>,
}

/// Tag 'R', auth type code 11: opaque SASL continuation bytes (server-first-message).
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationSaslContinue {
    pub payload: bytes::Bytes,
}

/// Tag 'R', auth type code 12: opaque SASL final bytes (server-final-message).
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationSaslFinal {
    pub payload: bytes::Bytes,
}

/// Tag 'K'. Cancellation key data for this backend connection.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackendKeyData {
    pub process_id: i64,
    pub secret_key: i64,
}

/// Tag 'B'. Extended query: bind a portal to a prepared statement with parameter values.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bind {
    pub portal_name: String,
    pub statement_name: String,
    pub param_formats: Vec<i64>,
    pub param_values: Vec<Option<bytes::Bytes>>,
    pub result_formats: Vec<i64>,
}

/// Tag 'C'. Command completion tag string (e.g. "SELECT 3").
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandComplete {
    pub tag: String,
}

/// Tag 'D'. One result row; each column is either raw bytes or SQL NULL.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataRow {
    pub columns: Vec<Option<bytes::Bytes>>,
}

/// Tag 'D'. Extended query: describe a statement or portal.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Describe {
    pub target_kind: String,
    pub name: String,
}

/// Tag 'E'. Backend error; a set of typed one-byte-code fields terminated by a null byte.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Field code ('S' severity, 'C' sqlstate, 'M' message, ...) to value.
    pub fields: serde_json::Value,
}

/// Tag 'E'. Extended query: execute a bound portal.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Execute {
    pub portal_name: String,
    /// 0 = no limit.
    pub max_rows: i64,
}

/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldDescription {
    pub name: String,
    pub table_oid: i64,
    pub column_attr: i64,
    pub type_oid: i64,
    pub type_size: i64,
    pub type_modifier: i64,
    pub format: i64,
}

/// One fully-buffered wire frame as read off the stream, before typed decode: optional tag byte + declared length + raw payload bytes.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Frame {
    /// Tag byte for tagged frames; null for the untagged StartupMessage/SSLRequest/CancelRequest family.
    #[serde(default)]
    pub tag: Option<usize>,
    /// Raw payload bytes after the tag+length header, exactly `declared_length - 4` bytes (or `- 4` from the untagged length field).
    pub payload: bytes::Bytes,
}

/// Tag 'N'. Same field structure as ErrorResponse, non-fatal.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NoticeResponse {
    pub fields: serde_json::Value,
}

/// Tag 'S'. Runtime parameter report (server_version, client_encoding, ...).
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterStatus {
    pub name: String,
    pub value: String,
}

/// Tag 'P'. Extended query: prepare a statement.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parse {
    pub statement_name: String,
    pub sql: String,
    pub param_type_oids: Vec<i64>,
}

/// Tag 'p'. Carries either a cleartext/MD5 password string or a raw SASL frame's opaque bytes (SCRAM crypto is out of scope for this slice; bytes are parsed/relayed only).
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PasswordMessage {
    pub payload: bytes::Bytes,
}

/// Tag 'Q'. Simple query protocol: a single null-terminated SQL string.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Query {
    pub sql: String,
}

/// Tag 'Z'. Marks the backend ready for a new query cycle; status drives TransactionStatus tracking.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadyForQuery {
    pub status: TransactionStatus,
}

/// Tag 'T'. Column metadata for a result set.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RowDescription {
    pub fields: Vec<FieldDescription>,
}

/// Tag 'p' variant carrying SASL mechanism name + initial response bytes (null-terminated mechanism, then i32 length + opaque bytes, -1 length = no response).
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaslInitialResponse {
    pub mechanism: String,
    pub response: Option<bytes::Bytes>,
}

/// Tag 'p' variant carrying a SASL continuation frame's opaque bytes.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaslResponse {
    pub payload: bytes::Bytes,
}

/// Untagged startup packet (no leading tag byte): 4-byte length, 4-byte protocol version, then a null-terminated key/value parameter list terminated by an empty string.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StartupMessage {
    /// Protocol major version supported by this slice (3.0).
    pub protocol_major: i64,
    pub protocol_minor: i64,
    /// Startup parameters (user, database, application_name, ...) as ordered key/value pairs.
    pub parameters: serde_json::Value,
}

/// Decoded from the ReadyForQuery status byte: 'I' -> idle, 'T' -> in_transaction, 'E' -> failed. See the Transaction Status Tracking state-machine.
/// @spec apps/pgpool/tech-design/logic/pg-wire-message-codec-frontend-backend-frames.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TransactionStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "in_transaction")]
    InTransaction,
    #[serde(rename = "failed")]
    Failed,
}
// CODEGEN-END
