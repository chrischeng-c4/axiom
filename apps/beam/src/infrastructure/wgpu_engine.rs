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
        if queries.len() != dim {
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

        let num_targets = targets.len() / dim;
        let mut scores = Vec::with_capacity(num_targets);

        let q = match metric {
            crate::collection::Metric::Cosine => crate::collection::l2_normalize(queries),
            _ => queries.to_vec(),
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
                    let mut dot = 0.0;
                    for j in 0..dim {
                        dot += q[j] * target_vector[j];
                    }
                    dot
                }
            };
            scores.push(score);
        }

        Ok(scores)
    }
}
// </HANDWRITE>
