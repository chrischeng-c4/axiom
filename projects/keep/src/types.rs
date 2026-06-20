//! KV store data types
//!
//! Supports high-precision numerics and Redis-compatible collections.

use std::collections::{BTreeMap, HashSet};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Key type for KV store (max 256 UTF-8 characters)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KvKey(String);

impl KvKey {
    /// Create a new key, validating length constraints
    pub fn new(key: impl Into<String>) -> Result<Self, crate::error::KvError> {
        let key = key.into();
        if key.len() > 256 {
            return Err(crate::error::KvError::KeyTooLong(key.len()));
        }
        if key.is_empty() {
            return Err(crate::error::KvError::EmptyKey);
        }
        Ok(Self(key))
    }

    /// Get the key as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Value types supported by the KV store
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KvValue {
    /// 64-bit signed integer
    Int(i64),
    /// 64-bit floating point
    Float(f64),
    /// 128-bit fixed-point decimal for financial precision
    Decimal(Decimal),
    /// UTF-8 string
    String(String),
    /// Binary data
    Bytes(Vec<u8>),
    /// Ordered list of values
    List(Vec<KvValue>),
    /// Key-value map (Hash in Redis)
    Map(std::collections::HashMap<String, KvValue>),
    /// Unordered set of unique strings
    Set(HashSet<String>),
    /// Sorted set with scores (member -> score)
    SortedSet(BTreeMap<String, f64>),
    /// Null/None value
    Null,
}
