//! KV store error types

use thiserror::Error;

/// Errors specific to KV store operations
#[derive(Error, Debug)]
pub enum KvError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Key too long: {0} characters (max 256)")]
    KeyTooLong(usize),

    #[error("Empty key not allowed")]
    EmptyKey,

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("CAS conflict: expected version {expected}, current version {current}")]
    CasConflict { expected: u64, current: u64 },

    #[error("Lock not held or expired")]
    LockNotHeld,

    #[error("Lock held by different owner")]
    LockOwnerMismatch { expected: String, actual: String },

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Out of memory: maxmemory limit reached and eviction failed")]
    OutOfMemory,

    #[error("Index out of range: {index}, list length is {len}")]
    IndexOutOfRange { index: i64, len: usize },
}
