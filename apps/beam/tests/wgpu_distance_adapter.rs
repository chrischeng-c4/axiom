// HANDWRITE-BEGIN gap="missing-generator:adapter-tests:025e6842" tracker="pending-tracker" reason="scaffold for apps/beam/tests/wgpu_distance_adapter.rs — fill in by hand and update tracker when codegen is ready"
use std::sync::Arc;
use beam::collection::Metric;
use beam::domain::ports::DistanceCalculator;
use beam::gpu::GpuContext;
use beam::infrastructure::wgpu_engine::WgpuDistanceEngine;

async fn test_metric_batching(metric: Metric) {
    let ctx = GpuContext::new().map(Arc::new);
    let engine = WgpuDistanceEngine::new(ctx.clone());
    let cpu_engine = WgpuDistanceEngine::new(None);

    let dim = 128;
    let num_queries = 4;
    let num_targets = 16;

    // Generate deterministic vectors
    let mut queries = Vec::with_capacity(num_queries * dim);
    for i in 0..num_queries {
        for j in 0..dim {
            queries.push((i + j) as f32 / 1000.0);
        }
    }

    let mut targets = Vec::with_capacity(num_targets * dim);
    for i in 0..num_targets {
        for j in 0..dim {
            targets.push((i * j) as f32 / 1000.0);
        }
    }

    let gpu_scores = engine
        .compute_batched(&queries, &targets, dim, metric)
        .await
        .expect("GPU compute");

    let cpu_scores = cpu_engine
        .compute_batched(&queries, &targets, dim, metric)
        .await
        .expect("CPU compute");

    assert_eq!(gpu_scores.len(), num_queries * num_targets);
    assert_eq!(cpu_scores.len(), num_queries * num_targets);

    for (gpu_s, cpu_s) in gpu_scores.iter().zip(cpu_scores.iter()) {
        assert!((gpu_s - cpu_s).abs() < 1e-4, "Scores mismatch: GPU={}, CPU={}", gpu_s, cpu_s);
    }
}

#[tokio::test]
async fn test_wgpu_distance_adapter_batching_l2() {
    test_metric_batching(Metric::L2).await;
}

#[tokio::test]
async fn test_wgpu_distance_adapter_batching_dot() {
    test_metric_batching(Metric::Dot).await;
}

#[tokio::test]
async fn test_wgpu_distance_adapter_batching_cosine() {
    test_metric_batching(Metric::Cosine).await;
}

#[tokio::test]
async fn test_wgpu_distance_adapter_fallback() {
    let cpu_engine = WgpuDistanceEngine::new(None);
    let dim = 128;
    let num_queries = 2;
    let num_targets = 4;

    let queries = vec![1.0; num_queries * dim];
    let targets = vec![1.0; num_targets * dim];

    let cpu_scores = cpu_engine
        .compute_batched(&queries, &targets, dim, Metric::L2)
        .await
        .expect("CPU fallback compute");

    assert_eq!(cpu_scores.len(), num_queries * num_targets);
    for score in cpu_scores {
        assert_eq!(score, 0.0);
    }
}
// HANDWRITE-END
