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
impl DistanceCalculator for MockDistanceCalculator {
    async fn compute_batched(
        &self,
        queries: &[f32],
        targets: &[f32],
        dim: usize,
    ) -> anyhow::Result<Vec<f32>> {
        let num_targets = targets.len() / dim;
        let mut scores = Vec::with_capacity(num_targets);
        for i in 0..num_targets {
            let target_vector = &targets[i * dim..(i + 1) * dim];
            let mut score = 0.0;
            for j in 0..dim {
                score += queries[j] * target_vector[j]; // simple dot product mock
            }
            scores.push(score);
        }
        Ok(scores)
    }
}

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
