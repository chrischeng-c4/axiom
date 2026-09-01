//! Shared object-store boundary for local durable storage and cloud archives.

#[cfg(feature = "gcs")]
mod gcs;
mod local;
#[cfg(feature = "s3")]
mod s3;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "gcs")]
pub use gcs::GcsObjectStore;
pub use local::LocalObjectStore;
#[cfg(feature = "s3")]
pub use s3::S3ObjectStore;

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ObjectVersion(String);

impl ObjectVersion {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObjectMeta {
    /// Key relative to the store's configured prefix.
    pub key: String,
    pub size: u64,
    pub content_type: String,
    pub version: ObjectVersion,
    pub etag: Option<String>,
    pub updated: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Object {
    pub meta: ObjectMeta,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PutCondition {
    Any,
    IfAbsent,
    IfVersion(ObjectVersion),
}

#[derive(Debug, Error)]
pub enum ObjectStoreError {
    #[error("object {key} was not found")]
    NotFound { key: String },
    #[error("object {key} did not satisfy the write precondition")]
    PreconditionFailed { key: String },
    #[error("object key is invalid: {key}")]
    InvalidKey { key: String },
    #[error("object path is unsafe: {path}")]
    UnsafePath { path: String },
    #[error("object-store authorization failed")]
    Unauthorized,
    #[error("object store is temporarily unavailable: {message}")]
    Unavailable { message: String },
    #[error("object-store response is corrupt: {message}")]
    Corrupt { message: String },
    #[error("object-store I/O failed: {message}")]
    Io { message: String },
}

pub type Result<T> = std::result::Result<T, ObjectStoreError>;

/// Object I/O boundary. Implementations own conditional-write mechanics.
pub trait ObjectStore: Send + Sync + 'static {
    fn put(
        &self,
        key: &str,
        bytes: &[u8],
        content_type: &str,
        condition: PutCondition,
    ) -> Result<ObjectMeta>;

    fn get(&self, key: &str) -> Result<Object>;
    fn head(&self, key: &str) -> Result<ObjectMeta>;
    fn list(&self, prefix: &str) -> Result<Vec<ObjectMeta>>;
    fn delete(&self, key: &str) -> Result<()>;
}

pub(crate) fn validate_key(key: &str) -> Result<&str> {
    let key = key.trim_matches('/');
    if key.is_empty()
        || key.contains('\0')
        || key.contains('\\')
        || key
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(ObjectStoreError::InvalidKey {
            key: key.to_string(),
        });
    }
    Ok(key)
}
