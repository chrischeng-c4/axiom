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
