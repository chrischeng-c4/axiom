//! Domain service orchestrating the high-throughput query pipeline.

use crate::domain::collection::Collection;
use crate::domain::ports::{DistanceCalculator, VectorRepository};

/// Bounded Context Aggregate representing a batch of vector queries.
pub struct QueryBatch {
    pub id: String,
    pub queries: Vec<Vec<f32>>,
    pub k: usize,
}

// <HANDWRITE gap="missing-generator:logic" tracker="pending-tracker" reason="logic section in scheduler.rs is hand-written pending codegen support">
/// Domain Service that pipelines candidate retrieval, NVMe fetch, and GPU distance calculation.
pub struct PipelineScheduler<R, C>
where
    R: VectorRepository,
    C: DistanceCalculator,
{
    repo: R,
    calc: C,
}
// </HANDWRITE>

impl<R, C> PipelineScheduler<R, C>
where
    R: VectorRepository,
    C: DistanceCalculator,
{
    pub fn new(repo: R, calc: C) -> Self {
        Self { repo, calc }
    }

    /// Execute a QueryBatch against a Collection Aggregate in a high-throughput async pipeline.
    pub async fn execute_batch(
        &self,
        collection: &Collection,
        batch: &QueryBatch,
    ) -> anyhow::Result<Vec<Vec<(String, f32)>>> {
        // Validate query dimensions
        for query in &batch.queries {
            if query.len() != collection.dim {
                return Err(crate::domain::ports::PipelineError::DimensionMismatch {
                    expected: collection.dim,
                    got: query.len(),
                }
                .into());
            }
        }

        let mut results = Vec::with_capacity(batch.queries.len());

        // Process queries in parallel/pipelined fashion to maximize throughput.
        for query in &batch.queries {
            // Phase 1: CPU HNSW Graph Traversal (RAM)
            let candidates = collection.navigator.find_candidates(query, batch.k);
            
            // Resolve candidates to (id, offset) pairs, filtering out missing offsets.
            let mut resolved_candidates = Vec::with_capacity(candidates.len());
            let mut offsets = Vec::with_capacity(candidates.len());
            for id in candidates {
                if let Some(&offset) = collection.payload.offsets.get(&id) {
                    resolved_candidates.push(id);
                    offsets.push(offset);
                }
            }

            if offsets.is_empty() {
                results.push(Vec::new());
                continue;
            }

            // Phase 2: Async NVMe Fetch (io_uring)
            // Calculate size: dim * sizeof(f32)
            let vector_bytes = collection.dim * std::mem::size_of::<f32>();
            let raw_bytes = self.repo.fetch_async(&offsets, vector_bytes).await?;

            // Validate vector-byte alignment
            if raw_bytes.len() % (collection.dim * 4) != 0 {
                return Err(crate::domain::ports::PipelineError::ByteAlignmentMismatch {
                    length: raw_bytes.len(),
                    dim: collection.dim,
                }
                .into());
            }

            // Validate decoded vector count
            let num_targets = raw_bytes.len() / (collection.dim * 4);
            if num_targets != offsets.len() {
                return Err(crate::domain::ports::PipelineError::VectorCountMismatch {
                    expected: offsets.len(),
                    got: num_targets,
                }
                .into());
            }

            // Convert raw bytes to f32 vectors
            let mut targets = Vec::with_capacity(raw_bytes.len() / 4);
            for chunk in raw_bytes.chunks_exact(4) {
                let val = f32::from_le_bytes(chunk.try_into().unwrap());
                targets.push(val);
            }

            // Phase 3: GPU Batch Computation (WGPU GEMM fallback)
            let scores = self.calc.compute_batched(query, &targets, collection.dim, collection.metric).await?;

            // Validate score count
            if scores.len() != num_targets {
                return Err(crate::domain::ports::PipelineError::ScoreCountMismatch {
                    expected: num_targets,
                    got: scores.len(),
                }
                .into());
            }

            // Phase 4: CPU Top-K Reduction
            let mut candidate_scores: Vec<(String, f32)> = resolved_candidates
                .into_iter()
                .zip(scores)
                .collect();

            // Sort ascending for L2 (smaller distance is better), descending for Inner Product / Cosine.
            // Tie-break with lexicographical ID to keep output ordering deterministic.
            if collection.metric == crate::collection::Metric::Cosine
                || collection.metric == crate::collection::Metric::Dot
            {
                candidate_scores.sort_by(|a, b| {
                    b.1.partial_cmp(&a.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.0.cmp(&b.0))
                });
            } else {
                candidate_scores.sort_by(|a, b| {
                    a.1.partial_cmp(&b.1)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.0.cmp(&b.0))
                });
            }

            candidate_scores.truncate(batch.k);
            results.push(candidate_scores);
        }

        Ok(results)
    }
}
