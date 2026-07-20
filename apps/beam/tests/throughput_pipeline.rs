//! Integration and unit tests for Beam's DDD High-Throughput Pipeline.

use std::sync::Arc;

use beam::domain::collection::Collection;
use beam::domain::ports::{DistanceCalculator, VectorRepository};
use beam::domain::scheduler::{PipelineScheduler, QueryBatch};
use beam::application::search_service::SearchApplicationService;
use beam::infrastructure::io_uring_repo::IoUringVectorRepository;
use beam::infrastructure::wgpu_engine::WgpuDistanceEngine;

// =========================================================================
// Mocks for R1: Domain Isolation testing
// =========================================================================

struct MockVectorRepository;
impl VectorRepository for MockVectorRepository {
    async fn fetch_async(&self, offsets: &[u64], vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
        // Return dummy bytes corresponding to float 1.0 for the length of vector
        let floats_count = vector_bytes / 4;
        let mut result = Vec::with_capacity(offsets.len() * vector_bytes);
        for _ in 0..offsets.len() {
            for _ in 0..floats_count {
                result.extend_from_slice(&1.0f32.to_le_bytes());
            }
        }
        Ok(result)
    }
}

struct MockDistanceCalculator;
// <HANDWRITE gap="missing-generator:logic" tracker="#2145" reason="logic section in throughput_pipeline.rs is hand-written pending codegen support">
impl DistanceCalculator for MockDistanceCalculator {
    async fn compute_batched(
        &self,
        queries: &[f32],
        targets: &[f32],
        dim: usize,
        metric: beam::collection::Metric,
    ) -> anyhow::Result<Vec<f32>> {
        if dim == 0 {
            return Err(beam::domain::ports::PipelineError::DimensionMismatch {
                expected: dim,
                got: queries.len(),
            }
            .into());
        }
        if queries.len() != dim {
            return Err(beam::domain::ports::PipelineError::DimensionMismatch {
                expected: dim,
                got: queries.len(),
            }
            .into());
        }
        if !targets.len().is_multiple_of(dim) {
            return Err(beam::domain::ports::PipelineError::VectorCountMismatch {
                expected: targets.len() / dim,
                got: targets.len(),
            }
            .into());
        }

        let num_targets = targets.len() / dim;
        let mut scores = Vec::with_capacity(num_targets);

        let q = match metric {
            beam::collection::Metric::Cosine => beam::collection::l2_normalize(queries),
            _ => queries.to_vec(),
        };

        for i in 0..num_targets {
            let target_vector = &targets[i * dim..(i + 1) * dim];
            let score = match metric {
                beam::collection::Metric::L2 => {
                    let mut dist = 0.0;
                    for j in 0..dim {
                        let diff = q[j] - target_vector[j];
                        dist += diff * diff;
                    }
                    dist
                }
                beam::collection::Metric::Dot | beam::collection::Metric::Cosine => {
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

// =========================================================================
// Test cases
// =========================================================================

#[tokio::test]
async fn test_r1_domain_isolation() {
    // R1: Validate that the Domain Layer (Collection, PipelineScheduler) runs and
    // is fully testable in absolute isolation using mocks.
    let mut collection = Collection::new("test_coll".to_string(), 3, beam::collection::Metric::Dot);
    
    // Set up mock candidates and offsets mapping
    collection.payload.offsets.insert("vector_0".to_string(), 0);
    collection.payload.offsets.insert("vector_1".to_string(), 12);
    collection.payload.offsets.insert("vector_2".to_string(), 24);

    let scheduler = PipelineScheduler::new(MockVectorRepository, MockDistanceCalculator);
    let batch = QueryBatch {
        id: "batch_mock_1".to_string(),
        queries: vec![vec![0.5, 0.5, 0.5]],
        k: 3,
    };

    let results = scheduler.execute_batch(&collection, &batch).await.unwrap();
    assert_eq!(results.len(), 1);
    
    // Each target is filled with 1.0 floats. Query is [0.5, 0.5, 0.5].
    // Dot product is 0.5 * 1.0 + 0.5 * 1.0 + 0.5 * 1.0 = 1.5.
    let query_results = &results[0];
    assert_eq!(query_results.len(), 3);
    for (id, score) in query_results {
        assert!(id.starts_with("vector_"));
        assert_eq!(*score, 1.5);
    }
}

// <HANDWRITE gap="missing-generator:logic" tracker="#2153" reason="logic section in throughput_pipeline.rs is hand-written pending codegen support">
#[tokio::test]
async fn test_r2_r3_infrastructure_and_e2e_pipeline() {
    // Set up a temporary path for the direct I/O mock file
    let tmp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let file_path = tmp_dir.join(format!("beam_test_{}.db", timestamp));

    // Write raw float arrays representing 3 target vectors (dim 3):
    // v0: [1.0, 0.0, 0.0]
    // v1: [0.0, 1.0, 0.0]
    // v2: [0.0, 0.0, 1.0]
    let mut file_content = Vec::new();
    file_content.extend_from_slice(&1.0f32.to_le_bytes());
    file_content.extend_from_slice(&0.0f32.to_le_bytes());
    file_content.extend_from_slice(&0.0f32.to_le_bytes());

    file_content.extend_from_slice(&0.0f32.to_le_bytes());
    file_content.extend_from_slice(&1.0f32.to_le_bytes());
    file_content.extend_from_slice(&0.0f32.to_le_bytes());

    file_content.extend_from_slice(&0.0f32.to_le_bytes());
    file_content.extend_from_slice(&0.0f32.to_le_bytes());
    file_content.extend_from_slice(&1.0f32.to_le_bytes());

    std::fs::write(&file_path, file_content).unwrap();

    // Setup Collection Aggregate Root
    let mut collection = Collection::new("e2e_coll".to_string(), 3, beam::collection::Metric::L2);
    
    // Map offsets: v0 starts at 0, v1 starts at 12 bytes, v2 starts at 24 bytes
    collection.payload.offsets.insert("vector_0".to_string(), 0);
    collection.payload.offsets.insert("vector_1".to_string(), 12);
    collection.payload.offsets.insert("vector_2".to_string(), 24);

    // Initialize actual infrastructure adapters
    let repo = IoUringVectorRepository::new(file_path.clone()).unwrap();
    
    // Initialize GPU context if available, else standard fallback
    let gpu = beam::gpu::GpuContext::new().map(Arc::new);
    let calc = WgpuDistanceEngine::new(gpu);

    // Create Application Service wrapper
    let app_service = SearchApplicationService::new(repo, calc);

    // Query close to v0: [0.9, 0.1, 0.0]. K = 3
    let queries = vec![vec![0.9, 0.1, 0.0]];
    let search_results = app_service.search(&collection, queries, 3).await.unwrap();

    assert_eq!(search_results.len(), 1);
    let topk = &search_results[0];
    assert_eq!(topk.len(), 3);

    // First neighbor must be vector_0 (it's the closest)
    assert_eq!(topk[0].0, "vector_0");
    
    // Verify L2 distance for vector_0: (0.9 - 1.0)^2 + (0.1 - 0.0)^2 + (0.0 - 0.0)^2 = 0.01 + 0.01 = 0.02
    assert!((topk[0].1 - 0.02).abs() < 1e-5);

    // Cleanup the temporary database file
    let _ = std::fs::remove_file(file_path);
}
// </HANDWRITE>

#[tokio::test]
async fn test_metric_aware_port() {
    // Verify L2, Dot, and Cosine calculations and top-k sorting
    let mut collection_l2 = Collection::new("l2_coll".to_string(), 3, beam::collection::Metric::L2);
    collection_l2.payload.offsets.insert("vector_0".to_string(), 0);
    collection_l2.payload.offsets.insert("vector_1".to_string(), 12);
    
    struct SpecificVectorRepository;
    impl VectorRepository for SpecificVectorRepository {
        async fn fetch_async(&self, offsets: &[u64], _vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
            let mut result = Vec::new();
            for &offset in offsets {
                if offset == 0 {
                    result.extend_from_slice(&1.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                } else if offset == 12 {
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                    result.extend_from_slice(&1.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                }
            }
            Ok(result)
        }
    }

    let scheduler = PipelineScheduler::new(SpecificVectorRepository, MockDistanceCalculator);

    let batch = QueryBatch {
        id: "b_l2".to_string(),
        queries: vec![vec![0.8, 0.6, 0.0]],
        k: 2,
    };
    let res_l2 = scheduler.execute_batch(&collection_l2, &batch).await.unwrap();
    assert_eq!(res_l2[0][0].0, "vector_0");
    assert!((res_l2[0][0].1 - 0.40).abs() < 1e-5);
    assert_eq!(res_l2[0][1].0, "vector_1");
    assert!((res_l2[0][1].1 - 0.80).abs() < 1e-5);

    let mut collection_dot = Collection::new("dot_coll".to_string(), 3, beam::collection::Metric::Dot);
    collection_dot.payload.offsets.insert("vector_0".to_string(), 0);
    collection_dot.payload.offsets.insert("vector_1".to_string(), 12);
    let res_dot = scheduler.execute_batch(&collection_dot, &batch).await.unwrap();
    assert_eq!(res_dot[0][0].0, "vector_0");
    assert!((res_dot[0][0].1 - 0.8).abs() < 1e-5);
    assert_eq!(res_dot[0][1].0, "vector_1");
    assert!((res_dot[0][1].1 - 0.6).abs() < 1e-5);
}

#[tokio::test]
async fn test_missing_candidate_offset() {
    let mut collection = Collection::new("coll".to_string(), 3, beam::collection::Metric::Dot);
    collection.payload.offsets.insert("vector_0".to_string(), 0);
    collection.payload.offsets.insert("vector_2".to_string(), 24);

    struct SpecificVectorRepository;
    impl VectorRepository for SpecificVectorRepository {
        async fn fetch_async(&self, offsets: &[u64], _vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
            let mut result = Vec::new();
            for &offset in offsets {
                if offset == 0 {
                    result.extend_from_slice(&1.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                } else if offset == 24 {
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                    result.extend_from_slice(&0.0f32.to_le_bytes());
                    result.extend_from_slice(&2.0f32.to_le_bytes());
                }
            }
            Ok(result)
        }
    }

    let scheduler = PipelineScheduler::new(SpecificVectorRepository, MockDistanceCalculator);

    let batch = QueryBatch {
        id: "b".to_string(),
        queries: vec![vec![1.0, 1.0, 1.0]],
        k: 3,
    };

    let res = scheduler.execute_batch(&collection, &batch).await.unwrap();
    let query_results = &res[0];
    assert_eq!(query_results.len(), 2);
    assert_eq!(query_results[0].0, "vector_2");
    assert_eq!(query_results[0].1, 2.0);
    assert_eq!(query_results[1].0, "vector_0");
    assert_eq!(query_results[1].1, 1.0);
}

#[tokio::test]
async fn test_boundary_validation() {
    let mut collection = Collection::new("coll".to_string(), 3, beam::collection::Metric::Dot);
    collection.payload.offsets.insert("vector_0".to_string(), 0);

    let scheduler = PipelineScheduler::new(MockVectorRepository, MockDistanceCalculator);

    // 1. Dimension mismatch
    let batch_wrong_dim = QueryBatch {
        id: "b1".to_string(),
        queries: vec![vec![1.0, 1.0]],
        k: 1,
    };
    let err = scheduler.execute_batch(&collection, &batch_wrong_dim).await.unwrap_err();
    assert!(err.to_string().contains("Dimension mismatch"));

    // 2. Alignment mismatch
    struct MisalignedVectorRepository;
    impl VectorRepository for MisalignedVectorRepository {
        async fn fetch_async(&self, _offsets: &[u64], _vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
            Ok(vec![0; 5])
        }
    }
    let scheduler_misaligned = PipelineScheduler::new(MisalignedVectorRepository, MockDistanceCalculator);
    let batch = QueryBatch {
        id: "b2".to_string(),
        queries: vec![vec![1.0, 1.0, 1.0]],
        k: 1,
    };
    let err = scheduler_misaligned.execute_batch(&collection, &batch).await.unwrap_err();
    assert!(err.to_string().contains("Byte alignment mismatch"));

    // 3. Vector count mismatch
    struct ShortVectorRepository;
    impl VectorRepository for ShortVectorRepository {
        async fn fetch_async(&self, _offsets: &[u64], _vector_bytes: usize) -> anyhow::Result<Vec<u8>> {
            Ok(vec![0; 0])
        }
    }
    let scheduler_short = PipelineScheduler::new(ShortVectorRepository, MockDistanceCalculator);
    let err = scheduler_short.execute_batch(&collection, &batch).await.unwrap_err();
    assert!(err.to_string().contains("Vector count mismatch"));
}
