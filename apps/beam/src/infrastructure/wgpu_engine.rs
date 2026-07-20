//! Infrastructure implementation of DistanceCalculator.

use std::sync::Arc;
use crate::gpu::GpuContext;
use crate::domain::ports::DistanceCalculator;

/// Infrastructure Adapter implementing DistanceCalculator using wgpu / CPU fallback.
pub struct WgpuDistanceEngine {
    #[allow(dead_code)]
    ctx: Option<Arc<GpuContext>>,
}

impl WgpuDistanceEngine {
    /// Create a new Distance Engine wrapping the wgpu context.
    pub fn new(ctx: Option<Arc<GpuContext>>) -> Self {
        Self { ctx }
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2145" reason="logic section in wgpu_engine.rs is hand-written pending codegen support">
impl DistanceCalculator for WgpuDistanceEngine {
    async fn compute_batched(
        &self,
        queries: &[f32],
        targets: &[f32],
        dim: usize,
        metric: crate::collection::Metric,
    ) -> anyhow::Result<Vec<f32>> {
        if dim == 0 {
            return Err(crate::domain::ports::PipelineError::DimensionMismatch {
                expected: dim,
                got: queries.len(),
            }
            .into());
        }
        if queries.len() % dim != 0 {
            return Err(crate::domain::ports::PipelineError::DimensionMismatch {
                expected: dim,
                got: queries.len(),
            }
            .into());
        }
        if !targets.len().is_multiple_of(dim) {
            return Err(crate::domain::ports::PipelineError::VectorCountMismatch {
                expected: targets.len() / dim,
                got: targets.len(),
            }
            .into());
        }

        let num_queries = queries.len() / dim;
        let num_targets = targets.len() / dim;

        if let Some(ref ctx) = self.ctx {
            // GPU Path!
            let (backend, adapter_name) = ctx.adapter_info();
            println!(
                "{{\"backend\":\"GPU\",\"adapter\":\"{}\",\"backend_api\":\"{}\",\"batch_size\":{},\"queries\":{},\"targets\":{}}}",
                adapter_name, backend, num_queries, num_queries, num_targets
            );

            // Construct queries packed buffer, L2-normalizing for Cosine
            let mut packed_queries = Vec::with_capacity(queries.len());
            for q_idx in 0..num_queries {
                let query_vector = &queries[q_idx * dim..(q_idx + 1) * dim];
                match metric {
                    crate::collection::Metric::Cosine => {
                        packed_queries.extend_from_slice(&crate::collection::l2_normalize(query_vector));
                    }
                    _ => {
                        packed_queries.extend_from_slice(query_vector);
                    }
                }
            }

            // Construct dummy Collection from targets
            let mut col = crate::collection::Collection::new("temp", dim, metric);
            for i in 0..num_targets {
                let target = &targets[i * dim..(i + 1) * dim];
                col.add(format!("{}", i), target)?;
            }

            // Construct flat index
            let index = crate::gpu::GpuFlatIndex::new(ctx, &col);

            // Calculate distances
            let scores = ctx.compute_distances_batch(&index, &packed_queries, num_queries);
            Ok(scores)
        } else {
            // CPU Path!
            println!(
                "{{\"backend\":\"CPU\",\"adapter\":\"none\",\"backend_api\":\"none\",\"batch_size\":{},\"queries\":{},\"targets\":{}}}",
                num_queries, num_queries, num_targets
            );

            let mut scores = Vec::with_capacity(num_queries * num_targets);
            for q_idx in 0..num_queries {
                let query_vector = &queries[q_idx * dim..(q_idx + 1) * dim];
                let q = match metric {
                    crate::collection::Metric::Cosine => crate::collection::l2_normalize(query_vector),
                    _ => query_vector.to_vec(),
                };

                for i in 0..num_targets {
                    let target_vector = &targets[i * dim..(i + 1) * dim];
                    let score = match metric {
                        crate::collection::Metric::L2 => {
                            let mut dist = 0.0;
                            for j in 0..dim {
                                let diff = q[j] - target_vector[j];
                                dist += diff * diff;
                            }
                            dist
                        }
                        crate::collection::Metric::Dot | crate::collection::Metric::Cosine => {
                            let t = match metric {
                                crate::collection::Metric::Cosine => crate::collection::l2_normalize(target_vector),
                                _ => target_vector.to_vec(),
                            };
                            let mut dot = 0.0;
                            for j in 0..dim {
                                dot += q[j] * t[j];
                            }
                            dot
                        }
                    };
                    scores.push(score);
                }
            }
            Ok(scores)
        }
    }
}
// </HANDWRITE>
