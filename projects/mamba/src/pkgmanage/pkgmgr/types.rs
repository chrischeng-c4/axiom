// REQ: FileHash, ReleaseFile, PackageMetadata, IndexClient structs
// REQ: IndexError enum variants per spec §Error Types

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Cryptographic digest of a distribution file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileHash {
    /// Hash algorithm identifier (e.g. "sha256").
    pub algorithm: String,
    /// Lowercase hex digest.
    pub digest: String,
}

impl Default for FileHash {
    fn default() -> Self {
        FileHash {
            algorithm: String::new(),
            digest: String::new(),
        }
    }
}

/// One distribution artifact (wheel or sdist) within a release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseFile {
    pub filename: String,
    pub url: String,
    /// Cryptographic hash of this file.
    pub hash: FileHash,
    /// PEP 440 version specifier, e.g. ">=3.8".
    pub requires_python: Option<String>,
    /// Size in bytes (not always present in all API responses).
    #[serde(default)]
    pub size: Option<u64>,
    /// Upload timestamp (ISO 8601).
    #[serde(default)]
    pub upload_time: Option<String>,
    /// Whether this release file has been yanked.
    #[serde(default)]
    pub yanked: bool,
    /// Reason for yanking, if any.
    #[serde(default)]
    pub yanked_reason: Option<String>,
    /// PEP 658 metadata availability flag or hash object.
    #[serde(default)]
    pub dist_info_metadata: serde_json::Value,
    /// Which index protocol produced this record ("json-api" or "simple-api").
    #[serde(default)]
    pub source: Option<String>,
}

/// Aggregated metadata for a package fetched from the index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    /// Normalized package name (PEP 503).
    pub name: String,
    /// All available version strings, newest first (lexicographic descending for shard 2a).
    pub versions: Vec<String>,
    /// Map of version string to list of distribution files.
    pub releases: BTreeMap<String, Vec<ReleaseFile>>,
    /// Minimum Python version constraint from the latest release.
    pub requires_python: Option<String>,
    /// Which index protocol produced this record.
    pub source: String,
}

/// Top-level client handle. Constructed once; shared across concurrent fetches.
///
/// HTTP methods are added in shard-2. Concurrency primitives are added in shard-3.
#[derive(Debug, Clone)]
pub struct IndexClient {
    /// Resolved base URL of the package index.
    pub index_url: String,
    /// Root of the local artifact cache (~/.cache/mamba by default).
    pub cache_dir: String,
    /// Semaphore bound for simultaneous in-flight HTTP requests.
    pub max_concurrent: u32,
    /// Per-request HTTP timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum retry attempts with exponential backoff.
    pub retry_max: u32,
}

/// Errors produced by the index client.
///
/// Variant names match the spec §Error Types exactly.
#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    /// Package name returned HTTP 404 from all strategies.
    #[error("package not found: {name}")]
    NotFound { name: String },

    /// Downloaded artifact digest does not match index-declared hash.
    #[error("hash mismatch for {filename}: expected {expected}, got {actual}")]
    HashMismatch {
        filename: String,
        expected: String,
        actual: String,
    },

    /// Failed to deserialize JSON API or Simple API response.
    #[error("parse error for {url}: {detail}")]
    ParseError { url: String, detail: String },

    /// reqwest transport error after all retries exhausted.
    #[error("network error for {url}: {detail}")]
    NetworkError { url: String, detail: String },

    /// Request did not complete within IndexClient.timeout_secs.
    #[error("request timed out for {url} after {timeout_secs}s")]
    Timeout { url: String, timeout_secs: u64 },

    /// Disk read/write error for metadata or artifact cache.
    #[error("cache I/O error for {path}: {detail}")]
    CacheIo { path: String, detail: String },

    /// Requested version is yanked on the index and no other satisfying version exists.
    #[error("version {version} of {name} is yanked")]
    YankedRelease { name: String, version: String },
}
