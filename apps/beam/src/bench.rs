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
use crate::payload::Filter;

/// Fixed seeds so the bench is fully reproducible run-to-run.
const DATASET_SEED: u64 = 0xBEA3_0001;
const QUERY_SEED: u64 = 0xBEA3_0002;
/// Gaussian jitter around each cluster center for the clustered corpus/queries.
const CLUSTER_JITTER: f32 = 0.05;
/// Coefficient-cluster jitter (in the rank-dim space) for the low-rank corpus.
const LOW_RANK_COEF_JITTER: f32 = 0.05;
/// Faiss-style cap on the k-means training sample: fit centroids + codebooks on
/// at most this many vectors, then assign + encode all `n`. Keeps `n = 1M` runs
/// tractable without changing the indexed set.
const TRAIN_SAMPLE_CAP: usize = 100_000;

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
    /// Intrinsic dimension of the synthetic corpus. `0` (default) draws the
    /// isotropic clustered data (PQ's worst case). `rank > 0` (e.g. 16 for
    /// dim=128) draws embedding-like **low-rank** data where PQ recall is
    /// meaningful — the P1 realism knob.
    pub rank: usize,
    /// Optional attribute filter for filtered k-NN. When `Some(cat)`, every row
    /// `i` is tagged `category = i % FILTER_CATEGORIES` and the query keeps only
    /// rows with `category == cat` (~1/8 selectivity), reporting filtered recall
    /// vs the filtered CPU oracle + timing. `None` (default) is the original
    /// unfiltered bench.
    pub filter_category: Option<i64>,
    /// CRUD churn fraction in `[0, 1]`. When `> 0`, a deterministic ~`churn`
    /// fraction of rows is deleted and reinserted (delete + re-add the same id and
    /// vector, LSM-style, so tombstones accumulate) BEFORE querying, then recall is
    /// reported vs the live oracle. `0.0` (default) leaves the corpus untouched —
    /// the original bench. Proves search still equals the live oracle after
    /// mutation.
    pub churn: f64,
}

/// Number of distinct `category` buckets the `--filter` bench assigns
/// (`category = i % FILTER_CATEGORIES`), so a single category is ~1/8 of rows.
pub const FILTER_CATEGORIES: i64 = 8;

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
            rank: 0,
            filter_category: None,
            churn: 0.0,
        }
    }
}

/// Deterministically churn ~`frac` of the collection's live rows: delete each
/// picked id then re-add it under the same id + vector + payload (LSM-style, so
/// the old physical row is tombstoned and a fresh live row appended). The live
/// set is unchanged, but tombstones now accumulate — so a subsequent search must
/// still equal the live oracle. Returns the number of rows churned.
fn apply_churn(collection: &mut crate::collection::Collection, frac: f64) -> usize {
    if frac <= 0.0 {
        return 0;
    }
    let cap = collection.capacity();
    let count = (((cap as f64) * frac).round() as usize).clamp(1, cap.max(1));
    let stride = (cap / count).max(1);
    let mut churned = 0usize;
    let mut i = 0usize;
    while i < cap {
        if collection.is_live(i as u32) {
            // Capture the row's id/vector/payload (owned) before mutating.
            let id = collection.external_ids()[i].clone();
            let vector = collection.row(i).to_vec();
            let payload = collection.payload(i).clone();
            collection.delete(&id);
            collection
                .add_with_payload(id, &vector, payload)
                .expect("re-add of a captured row always matches dim");
            churned += 1;
        }
        i += stride;
    }
    churned
}

/// Tag every row `i` with `category = i % FILTER_CATEGORIES` (see
/// [`FILTER_CATEGORIES`]), the deterministic attribute the `--filter` bench
/// filters on.
fn assign_category_payloads(collection: &mut crate::collection::Collection) {
    use crate::payload::Payload;
    // Freshly-built corpus (no tombstones yet), so iterate physical rows.
    for i in 0..collection.capacity() {
        collection.set_payload(i, Payload::new().with("category", i as i64 % FILTER_CATEGORIES));
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

/// Original exact GPU flat scan vs the CPU oracle. With `--filter <cat>` set,
/// every row is tagged `category = i % 8` and the search keeps only
/// `category == cat` rows, reporting filtered recall vs the filtered CPU oracle.
fn run_flat(gpu: &GpuContext, cfg: &BenchConfig) -> anyhow::Result<ExitCode> {
    println!(
        "index=flat dataset: n={} dim={} metric={:?} k={} queries={} (uniform, deterministic seed)",
        cfg.n, cfg.dim, cfg.metric, cfg.k, cfg.queries
    );

    let mut collection =
        dataset::random_collection("bench", cfg.n, cfg.dim, cfg.metric, DATASET_SEED);
    if cfg.filter_category.is_some() {
        assign_category_payloads(&mut collection);
    }
    if cfg.churn > 0.0 {
        let churned = apply_churn(&mut collection, cfg.churn);
        println!(
            "churn: deleted + reinserted {churned} rows ({:.1}% of n); {} live, {} tombstoned (recall is vs the live oracle)",
            cfg.churn * 100.0,
            collection.len(),
            collection.tombstoned(),
        );
    }
    let queries = dataset::random_queries(cfg.queries, cfg.dim, QUERY_SEED);

    let cpu = CpuFlatIndex::new(&collection);
    let gpu_index = GpuFlatIndex::new(gpu, &collection);

    // The optional attribute filter (keep only `category == cat`).
    let filter = cfg
        .filter_category
        .map(|cat| Filter::new().eq("category", cat));
    if let Some(cat) = cfg.filter_category {
        println!(
            "filter: category == {cat} (~1/{} selectivity, deterministic category = row % {})",
            FILTER_CATEGORIES, FILTER_CATEGORIES
        );
    }

    if let Some(first) = queries.first() {
        let _ = match &filter {
            Some(f) => gpu_index.search_knn_filtered(first, cfg.k, f),
            None => gpu_index.search_knn(first, cfg.k),
        };
    }

    let mut matched = 0usize;
    let mut total = 0usize;
    let mut gpu_elapsed = std::time::Duration::ZERO;

    for q in &queries {
        let cpu_rows: HashSet<u32> = match &filter {
            Some(f) => cpu.search_knn_filtered(q, cfg.k, f),
            None => cpu.search_knn(q, cfg.k),
        }
        .iter()
        .map(|nb| nb.row)
        .collect();
        let t0 = Instant::now();
        let gpu_res = match &filter {
            Some(f) => gpu_index.search_knn_filtered(q, cfg.k, f),
            None => gpu_index.search_knn(q, cfg.k),
        };
        gpu_elapsed += t0.elapsed();
        matched += gpu_res.iter().filter(|nb| cpu_rows.contains(&nb.row)).count();
        total += gpu_res.len();
    }

    let recall = if total == 0 { 1.0 } else { matched as f64 / total as f64 };
    let avg_ms = gpu_elapsed.as_secs_f64() * 1000.0 / cfg.queries.max(1) as f64;
    let label = if filter.is_some() {
        "filtered CPU oracle"
    } else {
        "CPU oracle"
    };
    println!("recall vs {label}: {recall:.3} (exact flat)");
    println!("candidates scanned / n: 1.000 (brute force)");
    println!("GPU query timing: avg {avg_ms:.3} ms/query over {} queries", cfg.queries);
    Ok(ExitCode::SUCCESS)
}

/// Print the built index's memory footprint plus the analytic flat-vs-PQ
/// contrast — the P2 headline: PQ codes (`n·m` bytes) are ~`dim·4/m`× smaller
/// than exact residuals (`n·dim·4` bytes), so PQ scales where flat does not.
fn report_footprint(index: &IvfPqIndex, cfg: &BenchConfig) {
    let mb = |bytes: usize| bytes as f64 / (1024.0 * 1024.0);
    println!(
        "index memory footprint: payload {:.1} MB + fixed overhead {:.2} MB (coarse centroids + PQ codebooks)",
        mb(index.payload_bytes()),
        mb(index.overhead_bytes()),
    );
    let flat_bytes = cfg.n * cfg.dim * std::mem::size_of::<f32>();
    if cfg.m > 0 && cfg.dim.is_multiple_of(cfg.m) {
        let pq_bytes = cfg.n * cfg.m;
        println!(
            "  contrast: flat residuals n·dim·4 = {:.1} MB  vs  pq codes n·m = {:.1} MB  → {:.1}× smaller",
            mb(flat_bytes),
            mb(pq_bytes),
            flat_bytes as f64 / pq_bytes as f64,
        );
    }
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

    // Corpus + queries share the same generative structure (representative near
    // neighbors) — this is where IVF pruning actually helps. Matching the cluster
    // count to nlist lets the coarse quantizer learn one centroid per cluster, so
    // pruning at a modest nprobe is (near-)lossless and the ivfpq recall gap is
    // cleanly attributable to PQ compression rather than coarse misses.
    //
    // `--rank 0` (default) draws ISOTROPIC clustered data — the pathological worst
    // case for PQ (independent subspaces, nothing to quantize). `--rank r > 0`
    // draws embedding-like LOW-RANK data (points near an r-dim manifold), where
    // PQ's per-subspace codebooks capture the correlated structure and recall is
    // high. Same clustering/pruning either way, so the recall delta isolates the
    // effect of intrinsic dimension on PQ.
    let num_clusters = cfg.nlist.clamp(1, cfg.n.max(1));
    let (mut collection, queries, data_desc) = if cfg.rank > 0 {
        let c = dataset::low_rank_collection(
            "bench",
            cfg.n,
            cfg.dim,
            Metric::L2,
            cfg.rank,
            num_clusters,
            LOW_RANK_COEF_JITTER,
            DATASET_SEED,
        );
        let q = dataset::low_rank_queries(
            cfg.queries,
            cfg.dim,
            cfg.rank,
            num_clusters,
            LOW_RANK_COEF_JITTER,
            QUERY_SEED,
        );
        (c, q, format!("low-rank rank={}", cfg.rank))
    } else {
        let c = dataset::clustered_collection(
            "bench",
            cfg.n,
            cfg.dim,
            Metric::L2,
            num_clusters,
            CLUSTER_JITTER,
            DATASET_SEED,
        );
        let q = dataset::clustered_queries(
            cfg.queries,
            cfg.dim,
            num_clusters,
            CLUSTER_JITTER,
            QUERY_SEED,
        );
        (c, q, "isotropic clustered".to_string())
    };

    // Deterministic per-row attribute for filtered search (must be assigned
    // before training, which snapshots the payloads into the index).
    if cfg.filter_category.is_some() {
        assign_category_payloads(&mut collection);
    }
    // Optional CRUD churn before training, so the index materializes over the
    // churned (tombstone-carrying) collection and must still equal the live oracle.
    if cfg.churn > 0.0 {
        let churned = apply_churn(&mut collection, cfg.churn);
        println!(
            "churn: deleted + reinserted {churned} rows ({:.1}% of n); {} live, {} tombstoned (recall is vs the live oracle)",
            cfg.churn * 100.0,
            collection.len(),
            collection.tombstoned(),
        );
    }
    let filter = cfg
        .filter_category
        .map(|cat| Filter::new().eq("category", cat));

    let refine = match cfg.index {
        IndexKind::IvfFlat => Refine::Flat,
        IndexKind::IvfPq => Refine::Pq { m: cfg.m },
        IndexKind::Flat => unreachable!(),
    };
    // Standard Faiss practice: fit centroids + codebooks on a bounded sample
    // (min(n, 100k)) so training stays tractable at n = 1M; every vector is still
    // assigned + encoded. Fewer Lloyd iters at very large n keeps build time sane.
    let train_sample = cfg.n.min(TRAIN_SAMPLE_CAP);
    let kmeans_iters = if cfg.n > 200_000 { 12 } else { 20 };
    let config = IvfPqConfig {
        nlist: cfg.nlist,
        kmeans_iters,
        nbits: 8,
        refine,
        train_sample,
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
    println!(" ({data_desc}, deterministic seed)");
    if let Some(cat) = cfg.filter_category {
        println!(
            "filter: category == {cat} (~1/{} selectivity, deterministic category = row % {}); candidates filtered within probed cells",
            FILTER_CATEGORIES, FILTER_CATEGORIES
        );
    }
    if train_sample < cfg.n {
        println!(
            "training: coarse + PQ k-means fit on a {train_sample}-vector sample ({} iters); all {} vectors assigned + encoded",
            kmeans_iters, cfg.n
        );
    }

    let index = IvfPqIndex::train(&collection, config)?;
    report_footprint(&index, cfg);
    let scanner = GpuIvfScanner::new(gpu);
    let oracle = CpuFlatIndex::new(&collection);

    // Warm both scan pipelines so timing excludes one-time shader compilation.
    if let Some(first) = queries.first() {
        let plan = index.plan(first, cfg.nprobe);
        let _ = scanner.scan(&plan);
        let _ = scanner.scan_shared(&plan);
    }

    let mut matched = 0usize;
    let mut total = 0usize;
    let mut cand_sum = 0usize;
    let mut total_elapsed = std::time::Duration::ZERO;
    // Break the per-query cost into its three phases so the bottleneck is visible:
    // host `plan` (coarse probe + ADC-table/residual build + candidate gather),
    // GPU `scan` (upload + kernel + blocking readback), and host `top-k`.
    let mut plan_elapsed = std::time::Duration::ZERO;
    let mut scan_elapsed = std::time::Duration::ZERO;
    let mut topk_elapsed = std::time::Duration::ZERO;

    for q in &queries {
        let truth: HashSet<u32> = match &filter {
            Some(f) => oracle.search_knn_filtered(q, cfg.k, f),
            None => oracle.search_knn(q, cfg.k),
        }
        .iter()
        .map(|nb| nb.row)
        .collect();

        let t0 = Instant::now();
        let plan = index.plan(q, cfg.nprobe);
        let t1 = Instant::now();
        let dist = scanner.scan(&plan);
        let t2 = Instant::now();
        let res = match &filter {
            Some(f) => index.topk_candidates_filtered(&plan.rows, &dist, cfg.k, f),
            None => index.topk_candidates(&plan.rows, &dist, cfg.k),
        };
        let t3 = Instant::now();
        total_elapsed += t3 - t0;
        plan_elapsed += t1 - t0;
        scan_elapsed += t2 - t1;
        topk_elapsed += t3 - t2;

        cand_sum += plan.num_candidates();
        matched += res.iter().filter(|nb| truth.contains(&nb.row)).count();
        total += res.len();
    }

    let recall = if total == 0 { 1.0 } else { matched as f64 / total as f64 };
    let cand_ratio = cand_sum as f64 / (cfg.queries.max(1) as f64 * cfg.n.max(1) as f64);
    let per = |d: std::time::Duration| d.as_secs_f64() * 1000.0 / cfg.queries.max(1) as f64;

    let oracle_label = if filter.is_some() {
        "filtered flat oracle"
    } else {
        "flat oracle"
    };
    println!("recall@{} vs {oracle_label}: {recall:.3}", cfg.k);
    println!(
        "candidates scanned / n: {cand_ratio:.4} (avg {} of {} vectors per query)",
        cand_sum / cfg.queries.max(1),
        cfg.n
    );
    println!(
        "GPU query timing: avg {:.3} ms/query over {} queries (plan {:.3} + scan {:.3} + topk {:.3})",
        per(total_elapsed),
        cfg.queries,
        per(plan_elapsed),
        per(scan_elapsed),
        per(topk_elapsed),
    );

    // For IVF-PQ, also time the P0 per-cell SHARED-MEMORY ADC kernel so the
    // global-vs-shared tradeoff on THIS GPU is explicit. On Apple Silicon the
    // small ADC table stays L2-resident, so the cached global-table scan wins;
    // the shared-memory design is the right one for discrete GPUs / large cells.
    if cfg.index == IndexKind::IvfPq && cfg.m <= 16 {
        let mut shared_scan = std::time::Duration::ZERO;
        for q in &queries {
            let plan = index.plan(q, cfg.nprobe);
            let t = Instant::now();
            let _ = scanner.scan_shared(&plan);
            shared_scan += t.elapsed();
        }
        println!(
            "  ADC kernel scan-only: global-table {:.3} ms  vs  shared-memory (P0) {:.3} ms/query",
            per(scan_elapsed),
            per(shared_scan),
        );
    }
    Ok(ExitCode::SUCCESS)
}
