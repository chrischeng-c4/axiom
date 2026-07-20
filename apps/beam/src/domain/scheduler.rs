// SPEC-MANAGED: apps/beam/tech-design/interfaces/rest/wire-the-high-throughput-pipeline-into-production-query-serving.md#changes
use std::sync::Arc;
use crate::domain::collection::Collection;
use crate::domain::ports::{DistanceCalculator, VectorRepository};

/// Bounded Context Aggregate representing a batch of vector queries.
pub struct QueryBatch {
    pub id: String,
    pub queries: Vec<Vec<f32>>,
    pub k: usize,
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2153" reason="logic section in scheduler.rs is hand-written pending codegen support">
/// Domain Service that pipelines candidate retrieval, NVMe fetch, and GPU distance calculation.
pub struct PipelineScheduler<R, C>
where
    R: VectorRepository,
    C: DistanceCalculator,
{
    repo: Arc<R>,
    calc: Arc<C>,
}

impl<R, C> PipelineScheduler<R, C>
where
    R: VectorRepository + 'static,
    C: DistanceCalculator + 'static,
{
    pub fn new(repo: R, calc: C) -> Self {
        Self {
            repo: Arc::new(repo),
            calc: Arc::new(calc),
        }
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

        // Phase 1: CPU HNSW Graph Traversal (RAM) - Pre-resolve all candidates
        let mut candidate_states = Vec::with_capacity(batch.queries.len());
        for query in &batch.queries {
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
            candidate_states.push((query.clone(), resolved_candidates, offsets));
        }

        let mut results = Vec::with_capacity(batch.queries.len());
        let mut next_fetch = None;

        for i in 0..batch.queries.len() {
            let (query, resolved_candidates, offsets) = &candidate_states[i];
            
            let mut fetch_elapsed = std::time::Duration::ZERO;
            let mut queue_wait_elapsed = std::time::Duration::ZERO;

            // Phase 2: Async NVMe Fetch
            let raw_bytes = if i == 0 {
                let fetch_start = std::time::Instant::now();
                let vector_bytes = collection.dim * std::mem::size_of::<f32>();
                let res = if offsets.is_empty() {
                    Vec::new()
                } else {
                    self.repo.fetch_async(offsets, vector_bytes).await?
                };
                fetch_elapsed = fetch_start.elapsed();
                res
            } else {
                let queue_wait_start = std::time::Instant::now();
                let res = next_fetch.take().unwrap().await??;
                queue_wait_elapsed = queue_wait_start.elapsed();
                res
            };

            // Prefetch next batch's candidates concurrently if there are more
            if i + 1 < batch.queries.len() {
                let next_offsets = candidate_states[i + 1].2.clone();
                let vector_bytes = collection.dim * std::mem::size_of::<f32>();
                let repo = Arc::clone(&self.repo);

                next_fetch = Some(tokio::spawn(async move {
                    if next_offsets.is_empty() {
                        Ok(Vec::new())
                    } else {
                        repo.fetch_async(&next_offsets, vector_bytes).await
                    }
                }));
            }

            if raw_bytes.is_empty() {
                results.push(Vec::new());
                continue;
            }

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

            // Phase 3: GPU Batch Computation
            let compute_start = std::time::Instant::now();
            let scores = self.calc.compute_batched(query, &targets, collection.dim, collection.metric).await?;
            let compute_elapsed = compute_start.elapsed();

            // Validate score count
            if scores.len() != num_targets {
                return Err(crate::domain::ports::PipelineError::ScoreCountMismatch {
                    expected: num_targets,
                    got: scores.len(),
                }
                .into());
            }

            // Phase 4: CPU Top-K Reduction
            let topk_start = std::time::Instant::now();
            let mut candidate_scores: Vec<(String, f32)> = resolved_candidates
                .iter()
                .cloned()
                .zip(scores)
                .collect();

            // Sort ascending for L2 (smaller distance is better), descending for Inner Product / Cosine.
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
            let topk_elapsed = topk_start.elapsed();

            results.push(candidate_scores);

            eprintln!(
                "beam query serve: fetch_ms={:.2}, queue_wait_ms={:.2}, compute_ms={:.2}, topk_ms={:.2}",
                fetch_elapsed.as_secs_f64() * 1000.0,
                queue_wait_elapsed.as_secs_f64() * 1000.0,
                compute_elapsed.as_secs_f64() * 1000.0,
                topk_elapsed.as_secs_f64() * 1000.0
            );
        }

        Ok(results)
    }
}
// </HANDWRITE>
