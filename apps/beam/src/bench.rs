//! `beam bench` — the human-facing proof that GPU vector search runs and matches
//! the CPU oracle, across the flat and IVF-PQ index backends.
//!
//! `--index flat` (default) builds a deterministic uniform collection and runs
//! the exact GPU flat scan vs the CPU oracle, printing recall (always 1.000) and
//! GPU query timing — the original behavior, unchanged.
//!
//! `--index ivfflat` / `--index ivfpq` build a deterministic **clustered**
//! collection (where ANN pruning is meaningful), train an
//! [`IvfPqIndex`](crate::index::ivf_pq::IvfPqIndex), and run the GPU candidate
//! scan ([`GpuIvfScanner`](crate::gpu::ivfpq::GpuIvfScanner)). They print recall
//! vs the flat oracle, the candidates-scanned/`n` ratio (the pruning proof), and
//! average GPU query time — so the flat-vs-ANN tradeoff is visible.
//!
//! If no GPU adapter is available it prints `no GPU adapter available` and
//! returns a non-zero [`ExitCode`].

use std::collections::HashSet;
use std::process::ExitCode;
use std::time::Instant;

use crate::collection::Metric;
use crate::dataset;
use crate::gpu::ivfpq::GpuIvfScanner;
use crate::gpu::{GpuContext, GpuFlatIndex};
use crate::index::cpu_flat::CpuFlatIndex;
use crate::index::ivf_pq::{IvfPqConfig, IvfPqIndex, Refine};
use crate::index::VectorIndex;

/// Fixed seeds so the bench is fully reproducible run-to-run.
const DATASET_SEED: u64 = 0xBEA3_0001;
const QUERY_SEED: u64 = 0xBEA3_0002;
/// Gaussian jitter around each cluster center for the clustered corpus/queries.
const CLUSTER_JITTER: f32 = 0.05;

/// Which index backend `beam bench` exercises.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndexKind {
    /// Exact GPU brute-force scan (the original bench).
    Flat,
    /// IVF with exact per-cell residual refinement (no PQ compression).
    IvfFlat,
    /// IVF with product-quantized residuals (compressed, approximate).
    IvfPq,
}

impl IndexKind {
    /// Parse the CLI spelling (`flat` / `ivfflat` / `ivfpq`).
    pub fn parse(s: &str) -> Option<IndexKind> {
        match s.trim().to_ascii_lowercase().as_str() {
            "flat" => Some(IndexKind::Flat),
            "ivfflat" | "ivf-flat" => Some(IndexKind::IvfFlat),
            "ivfpq" | "ivf-pq" => Some(IndexKind::IvfPq),
            _ => None,
        }
    }
}

/// Bench parameters (mirrors the `beam bench` flags).
#[derive(Debug, Clone)]
pub struct BenchConfig {
    pub n: usize,
    pub dim: usize,
    pub k: usize,
    pub queries: usize,
    pub metric: Metric,
    pub index: IndexKind,
    /// IVF coarse-cell count (ivfflat / ivfpq).
    pub nlist: usize,
    /// Cells probed per query (ivfflat / ivfpq).
    pub nprobe: usize,
    /// PQ subvector count (ivfpq); `dim` must be divisible by `m`.
    pub m: usize,
}

impl Default for BenchConfig {
    fn default() -> Self {
        Self {
            n: 100_000,
            dim: 128,
            k: 10,
            queries: 20,
            metric: Metric::L2,
            index: IndexKind::Flat,
            nlist: 256,
            nprobe: 16,
            m: 8,
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

    match cfg.index {
        IndexKind::Flat => run_flat(&gpu, cfg),
        IndexKind::IvfFlat | IndexKind::IvfPq => run_ivf(&gpu, cfg),
    }
}

/// Original exact GPU flat scan vs the CPU oracle.
fn run_flat(gpu: &GpuContext, cfg: &BenchConfig) -> anyhow::Result<ExitCode> {
    println!(
        "index=flat dataset: n={} dim={} metric={:?} k={} queries={} (uniform, deterministic seed)",
        cfg.n, cfg.dim, cfg.metric, cfg.k, cfg.queries
    );

    let collection = dataset::random_collection("bench", cfg.n, cfg.dim, cfg.metric, DATASET_SEED);
    let queries = dataset::random_queries(cfg.queries, cfg.dim, QUERY_SEED);

    let cpu = CpuFlatIndex::new(&collection);
    let gpu_index = GpuFlatIndex::new(gpu, &collection);

    if let Some(first) = queries.first() {
        let _ = gpu_index.search_knn(first, cfg.k);
    }

    let mut matched = 0usize;
    let mut total = 0usize;
    let mut gpu_elapsed = std::time::Duration::ZERO;

    for q in &queries {
        let cpu_rows: HashSet<u32> = cpu.search_knn(q, cfg.k).iter().map(|nb| nb.row).collect();
        let t0 = Instant::now();
        let gpu_res = gpu_index.search_knn(q, cfg.k);
        gpu_elapsed += t0.elapsed();
        matched += gpu_res.iter().filter(|nb| cpu_rows.contains(&nb.row)).count();
        total += gpu_res.len();
    }

    let recall = if total == 0 { 1.0 } else { matched as f64 / total as f64 };
    let avg_ms = gpu_elapsed.as_secs_f64() * 1000.0 / cfg.queries.max(1) as f64;
    println!("recall vs CPU oracle: {recall:.3} (exact flat)");
    println!("candidates scanned / n: 1.000 (brute force)");
    println!("GPU query timing: avg {avg_ms:.3} ms/query over {} queries", cfg.queries);
    Ok(ExitCode::SUCCESS)
}

/// IVF (flat-refine or PQ) GPU candidate scan vs the flat oracle.
fn run_ivf(gpu: &GpuContext, cfg: &BenchConfig) -> anyhow::Result<ExitCode> {
    if cfg.metric != Metric::L2 {
        anyhow::bail!(
            "index={:?} supports Metric::L2 only (got {:?}); IVF-PQ is L2-only for now",
            cfg.index,
            cfg.metric
        );
    }
    if cfg.index == IndexKind::IvfPq && (cfg.m == 0 || !cfg.dim.is_multiple_of(cfg.m)) {
        anyhow::bail!("ivfpq needs dim ({}) divisible by m ({})", cfg.dim, cfg.m);
    }

    // Clustered corpus + queries share cluster centers (representative near
    // neighbors) — this is where IVF pruning actually helps. Matching the cluster
    // count to nlist lets the coarse quantizer learn one centroid per cluster, so
    // pruning at a modest nprobe is (near-)lossless and the ivfpq recall gap is
    // cleanly attributable to PQ compression rather than coarse misses.
    let num_clusters = cfg.nlist.clamp(1, cfg.n.max(1));
    let collection = dataset::clustered_collection(
        "bench",
        cfg.n,
        cfg.dim,
        Metric::L2,
        num_clusters,
        CLUSTER_JITTER,
        DATASET_SEED,
    );
    let queries = dataset::clustered_queries(cfg.queries, cfg.dim, num_clusters, CLUSTER_JITTER, QUERY_SEED);

    let refine = match cfg.index {
        IndexKind::IvfFlat => Refine::Flat,
        IndexKind::IvfPq => Refine::Pq { m: cfg.m },
        IndexKind::Flat => unreachable!(),
    };
    let config = IvfPqConfig {
        nlist: cfg.nlist,
        kmeans_iters: 20,
        nbits: 8,
        refine,
        seed: DATASET_SEED,
    };

    print!(
        "index={} dataset: n={} dim={} clusters={} k={} nlist={} nprobe={}",
        match cfg.index {
            IndexKind::IvfFlat => "ivfflat",
            IndexKind::IvfPq => "ivfpq",
            IndexKind::Flat => unreachable!(),
        },
        cfg.n,
        cfg.dim,
        num_clusters,
        cfg.k,
        cfg.nlist,
        cfg.nprobe
    );
    if cfg.index == IndexKind::IvfPq {
        print!(" m={} (dsub={})", cfg.m, cfg.dim / cfg.m);
    }
    println!(" (clustered, deterministic seed)");

    let index = IvfPqIndex::train(&collection, config)?;
    let scanner = GpuIvfScanner::new(gpu);
    let oracle = CpuFlatIndex::new(&collection);

    // Warm the pipeline so timing excludes one-time shader compilation.
    if let Some(first) = queries.first() {
        let plan = index.plan(first, cfg.nprobe);
        let _ = scanner.scan(&plan);
    }

    let mut matched = 0usize;
    let mut total = 0usize;
    let mut cand_sum = 0usize;
    let mut gpu_elapsed = std::time::Duration::ZERO;

    for q in &queries {
        let truth: HashSet<u32> = oracle.search_knn(q, cfg.k).iter().map(|nb| nb.row).collect();

        let t0 = Instant::now();
        let plan = index.plan(q, cfg.nprobe);
        let dist = scanner.scan(&plan);
        let res = index.topk_candidates(&plan.rows, &dist, cfg.k);
        gpu_elapsed += t0.elapsed();

        cand_sum += plan.num_candidates();
        matched += res.iter().filter(|nb| truth.contains(&nb.row)).count();
        total += res.len();
    }

    let recall = if total == 0 { 1.0 } else { matched as f64 / total as f64 };
    let cand_ratio = cand_sum as f64 / (cfg.queries.max(1) as f64 * cfg.n.max(1) as f64);
    let avg_ms = gpu_elapsed.as_secs_f64() * 1000.0 / cfg.queries.max(1) as f64;

    println!("recall@{} vs flat oracle: {recall:.3}", cfg.k);
    println!(
        "candidates scanned / n: {cand_ratio:.4} (avg {} of {} vectors per query)",
        cand_sum / cfg.queries.max(1),
        cfg.n
    );
    println!("GPU query timing: avg {avg_ms:.3} ms/query over {} queries", cfg.queries);
    Ok(ExitCode::SUCCESS)
}
