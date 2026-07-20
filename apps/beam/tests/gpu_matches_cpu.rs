//! GPU-vs-CPU parity: the real milestone proof.
//!
//! For each metric (L2, Dot, Cosine) we build a deterministic dataset, index it
//! with both the exact CPU oracle and the GPU (Metal via wgpu) flat index, and
//! assert their top-k **row sets** agree (float tie-reordering allowed — compared
//! as sets) with scores within 1e-3.
//!
//! Skips gracefully (prints a message, returns) when no GPU adapter is present,
//! so CI without a GPU stays green. On this Mac it PRINTS the Metal adapter and
//! PASSES (run with `-- --nocapture` to see the adapter line).

use std::collections::HashSet;

use beam::collection::Metric;
use beam::dataset;
use beam::gpu::{GpuContext, GpuFlatIndex};
use beam::index::cpu_flat::CpuFlatIndex;
use beam::index::{Neighbor, VectorIndex};

const N: usize = 2000;
const DIM: usize = 64;
const K: usize = 10;
const N_QUERIES: usize = 8;
const DATASET_SEED: u64 = 0x1234_5678;
const QUERY_SEED: u64 = 0x8765_4321;

fn row_set(neighbors: &[Neighbor]) -> HashSet<u32> {
    neighbors.iter().map(|n| n.row).collect()
}

/// Assert CPU and GPU agree for one metric: same top-k row set, per-row scores
/// within 1e-3.
fn assert_parity(gpu: &GpuContext, metric: Metric) {
    let collection = dataset::random_collection("parity", N, DIM, metric, DATASET_SEED);
    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);

    let cpu = CpuFlatIndex::new(&collection);
    let gpu_index = GpuFlatIndex::new(gpu, &collection);

    for (qi, q) in queries.iter().enumerate() {
        let cpu_res = cpu.search_knn(q, K);
        let gpu_res = gpu_index.search_knn(q, K);

        assert_eq!(
            cpu_res.len(),
            K,
            "{metric:?} q{qi}: CPU should return exactly K neighbors"
        );
        assert_eq!(
            gpu_res.len(),
            K,
            "{metric:?} q{qi}: GPU should return exactly K neighbors"
        );

        // Compare as SETS (float tie-reordering is allowed).
        let cpu_rows = row_set(&cpu_res);
        let gpu_rows = row_set(&gpu_res);
        assert_eq!(
            cpu_rows, gpu_rows,
            "{metric:?} q{qi}: GPU top-k row set != CPU oracle\n  cpu={cpu_res:?}\n  gpu={gpu_res:?}"
        );

        // Scores must agree within tolerance per row.
        let cpu_by_row: std::collections::HashMap<u32, f32> =
            cpu_res.iter().map(|n| (n.row, n.score)).collect();
        for nb in &gpu_res {
            let cpu_score = cpu_by_row[&nb.row];
            assert!(
                (cpu_score - nb.score).abs() <= 1e-3,
                "{metric:?} q{qi} row {}: GPU score {} vs CPU {} exceeds 1e-3",
                nb.row,
                nb.score,
                cpu_score
            );
        }
    }
    eprintln!("  parity OK for metric {metric:?} (n={N}, dim={DIM}, k={K}, {N_QUERIES} queries)");
}

// <HANDWRITE gap="missing-generator:unit-test" tracker="pending-tracker" reason="unit-test section in gpu_matches_cpu.rs is hand-written pending codegen support">
#[test]
fn gpu_matches_cpu_oracle() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gpu_matches_cpu_oracle");
        return;
    };
    let (backend, name) = gpu.adapter_info();
    eprintln!("GPU adapter: {name} ({backend})");

    for metric in [Metric::L2, Metric::Dot, Metric::Cosine] {
        assert_parity(&gpu, metric);
    }
}
// </HANDWRITE>
