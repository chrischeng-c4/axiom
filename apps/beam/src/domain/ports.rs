//! Domain ports (interfaces) for Infrastructure abstraction.

use std::future::Future;

// <HANDWRITE gap="missing-generator:logic" tracker="#2151" reason="logic section in ports.rs is hand-written pending codegen support">
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Storage file is missing: {path}")]
    FileNotFound { path: String },
    #[error("Invalid offset alignment: offset {offset} must be aligned to {alignment} bytes")]
    InvalidAlignment { offset: u64, alignment: usize },
    #[error("Out of range offset: offset {offset} (size {size} bytes) exceeds file size {file_size}")]
    OutOfRange { offset: u64, size: usize, file_size: u64 },
    #[error("Short read at offset {offset}: expected {expected} bytes, read {got} bytes")]
    ShortRead { offset: u64, expected: usize, got: usize },
    #[error("Underlying I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Hexagonal port for fetching uncompressed raw vectors from storage.
pub trait VectorRepository: Send + Sync {
    /// Asynchronously fetch raw vector data from NVMe disk given their physical offsets.
    /// Returns a flat vector of floats representing the fetched records.
    fn fetch_async(&self, offsets: &[u64], vector_bytes: usize) -> impl Future<Output = anyhow::Result<Vec<u8>>> + Send;
}
// </HANDWRITE>

use crate::collection::Metric;

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },
    #[error("Byte alignment mismatch: raw bytes length {length} is not a multiple of {dim} * 4")]
    ByteAlignmentMismatch { length: usize, dim: usize },
    #[error("Vector count mismatch: expected {expected}, got {got}")]
    VectorCountMismatch { expected: usize, got: usize },
    #[error("Score count mismatch: expected {expected}, got {got}")]
    ScoreCountMismatch { expected: usize, got: usize },
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2145" reason="logic section in ports.rs is hand-written pending codegen support">
/// Hexagonal port for batched vector distance calculation.
pub trait DistanceCalculator: Send + Sync {
    /// Asynchronously calculate distance metrics between a batch of queries and target candidate vectors on GPU.
    /// Returns a flat list of scores.
    fn compute_batched(
        &self,
        queries: &[f32],
        targets: &[f32],
        dim: usize,
        metric: Metric,
    ) -> impl Future<Output = anyhow::Result<Vec<f32>>> + Send;
}
// </HANDWRITE>
