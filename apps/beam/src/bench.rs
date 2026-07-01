//! `beam bench` — the human-facing proof that GPU vector search runs and matches
//! the CPU oracle.
//!
//! It builds a deterministic in-memory collection, runs the same queries on both
//! [`CpuFlatIndex`](crate::index::cpu_flat::CpuFlatIndex) and
//! [`GpuFlatIndex`](crate::gpu::GpuFlatIndex), and prints the GPU adapter, the
//! GPU-vs-CPU top-k agreement (recall), and the average GPU query time.
//!
//! If no GPU adapter is available it prints `no GPU adapter available` and
//! returns a non-zero [`ExitCode`].

use std::collections::HashSet;
use std::process::ExitCode;
use std::time::Instant;

use crate::collection::Metric;
use crate::dataset;
use crate::gpu::{GpuContext, GpuFlatIndex};
use crate::index::cpu_flat::CpuFlatIndex;
use crate::index::VectorIndex;

/// Fixed seeds so the bench is fully reproducible run-to-run.
const DATASET_SEED: u64 = 0xBEA3_0001;
const QUERY_SEED: u64 = 0xBEA3_0002;

/// Bench parameters (mirrors the `beam bench` flags).
#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub n: usize,
    pub dim: usize,
    pub k: usize,
    pub queries: usize,
    pub metric: Metric,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            n: 100_000,
            dim: 128,
            k: 10,
            queries: 20,
            metric: Metric::L2,
        }
    }
}

/// Run the bench. Returns `ExitCode::SUCCESS` on a completed run,
/// `ExitCode::FAILURE` when no GPU adapter is available (message already printed).
pub fn run(cfg: &BenchConfig) -> anyhow::Result<ExitCode> {
    let Some(gpu) = GpuContext::new() else {
        println!("no GPU adapter available");
        return Ok(ExitCode::FAILURE);
    };

    let (backend, name) = gpu.adapter_info();
    println!("GPU: {name} ({backend})");
    println!(
        "dataset: n={} dim={} metric={:?} k={} queries={} (deterministic seed)",
        cfg.n, cfg.dim, cfg.metric, cfg.k, cfg.queries
    );

    // Deterministic corpus + queries.
    let collection = dataset::random_collection("bench", cfg.n, cfg.dim, cfg.metric, DATASET_SEED);
    let queries = dataset::random_queries(cfg.queries, cfg.dim, QUERY_SEED);

    let cpu = CpuFlatIndex::new(&collection);
    let gpu_index = GpuFlatIndex::new(&gpu, &collection);

    // Warm up the pipeline once so the reported timing excludes one-time
    // shader/first-dispatch compilation cost.
    if let Some(first) = queries.first() {
        let _ = gpu_index.search_knn(first, cfg.k);
    }

    let mut matched = 0usize;
    let mut total = 0usize;
    let mut gpu_elapsed = std::time::Duration::ZERO;

    for q in &queries {
        let cpu_res = cpu.search_knn(q, cfg.k);
        let cpu_rows: HashSet<u32> = cpu_res.iter().map(|nb| nb.row).collect();

        let t0 = Instant::now();
        let gpu_res = gpu_index.search_knn(q, cfg.k);
        gpu_elapsed += t0.elapsed();

        for nb in &gpu_res {
            if cpu_rows.contains(&nb.row) {
                matched += 1;
            }
        }
        total += gpu_res.len();
    }

    let recall = if total == 0 {
        1.0
    } else {
        matched as f64 / total as f64
    };
    let avg_ms = gpu_elapsed.as_secs_f64() * 1000.0 / queries.len().max(1) as f64;

    println!("recall vs CPU oracle: {recall:.3} (exact flat)");
    println!(
        "GPU query timing: avg {avg_ms:.3} ms/query over {} queries",
        queries.len()
    );

    Ok(ExitCode::SUCCESS)
}
