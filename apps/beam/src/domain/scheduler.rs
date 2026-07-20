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
        let mut results = Vec::with_capacity(batch.queries.len());

        // Process queries in parallel/pipelined fashion to maximize throughput.
        for query in &batch.queries {
            // Phase 1: CPU HNSW Graph Traversal (RAM)
            let candidates = collection.navigator.find_candidates(query, batch.k);
            let offsets = collection.payload.resolve_offsets(&candidates);

            if offsets.is_empty() {
                results.push(Vec::new());
                continue;
            }

            // Phase 2: Async NVMe Fetch (io_uring)
            // Calculate size: dim * sizeof(f32)
            let vector_bytes = collection.dim * std::mem::size_of::<f32>();
            let raw_bytes = self.repo.fetch_async(&offsets, vector_bytes).await?;

            // Convert raw bytes to f32 vectors
            let mut targets = Vec::with_capacity(raw_bytes.len() / 4);
            for chunk in raw_bytes.chunks_exact(4) {
                let val = f32::from_le_bytes(chunk.try_into().unwrap());
                targets.push(val);
            }

            // Phase 3: GPU Batch Computation (WGPU GEMM)
            let scores = self.calc.compute_batched(query, &targets, collection.dim).await?;

            // Phase 4: CPU Top-K Reduction
            let mut candidate_scores: Vec<(String, f32)> = candidates
                .into_iter()
                .zip(scores.into_iter())
                .collect();

            // Sort ascending for L2 (smaller distance is better), descending for Inner Product
            if collection.metric == crate::collection::Metric::Cosine
                || collection.metric == crate::collection::Metric::Dot
            {
                candidate_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            } else {
                candidate_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            }

            candidate_scores.truncate(batch.k);
            results.push(candidate_scores);
        }

        Ok(results)
    }
}
