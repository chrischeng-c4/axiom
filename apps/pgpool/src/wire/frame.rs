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
