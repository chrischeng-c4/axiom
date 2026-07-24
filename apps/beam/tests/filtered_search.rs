//! Filtered k-NN correctness gate — payloads + filtered search.
//!
//! The metadata-payload feature is only real if the GPU's filtered top-k equals
//! the exact CPU oracle's filtered top-k. These tests build a deterministic
//! clustered corpus (via the `dataset.rs` LCG) with deterministic per-row
//! payloads (`category = i % 8`, `bucket = i % 100`, `row = i`) and assert, for
//! both the flat and the IVF path:
//!
//!   1. Filtered GPU == filtered CPU oracle (row set + scores within 1e-3), for
//!      an `Eq` filter and an `IntRange` filter.
//!   2. The filter actually restricts: every returned neighbor satisfies the
//!      filter, and a selective filter changes the result vs unfiltered search.
//!   3. Selectivity edges: a filter matching < k rows returns exactly the
//!      matching rows (length = #matches, not k); a filter matching 0 rows
//!      returns empty.
//!
//! For the IVF path we use `Refine::Flat` + `nprobe == nlist`, which is exact
//! over the whole corpus, so filtered IVF must reproduce the filtered flat
//! oracle. Every test skips gracefully (prints, returns) when no GPU adapter is
//! present, so GPU-less CI stays green; on this Mac they PRINT the Metal adapter
//! and PASS.

use std::collections::{HashMap, HashSet};

use beam::collection::{Collection, Metric};
use beam::dataset;
use beam::gpu::ivfpq::GpuIvfScanner;
use beam::gpu::{GpuContext, GpuFlatIndex};
use beam::index::cpu_flat::CpuFlatIndex;
use beam::index::ivf_pq::{IvfPqConfig, IvfPqIndex, Refine};
use beam::index::{Neighbor, VectorIndex};
use beam::payload::{Filter, Payload};

const N: usize = 4000;
const DIM: usize = 64;
const K: usize = 10;
const NLIST: usize = 32;
const NUM_CLUSTERS: usize = 32;
const JITTER: f32 = 0.05;
const N_QUERIES: usize = 8;

const CORPUS_SEED: u64 = 0xF117_E000;
const QUERY_SEED: u64 = 0xF117_E011;
const TRAIN_SEED: u64 = 0xF117_E022;

/// The shared clustered L2 corpus with deterministic per-row payloads.
fn corpus_with_payloads() -> Collection {
    let mut c = dataset::clustered_collection(
        "filt",
        N,
        DIM,
        Metric::L2,
        NUM_CLUSTERS,
        JITTER,
        CORPUS_SEED,
    );
    for i in 0..c.len() {
        c.set_payload(
            i,
            Payload::new()
                .with("category", (i % 8) as i64)
                .with("bucket", (i % 100) as i64)
                .with("row", i as i64),
        );
    }
    c
}

fn queries() -> Vec<Vec<f32>> {
    dataset::clustered_queries(N_QUERIES, DIM, NUM_CLUSTERS, JITTER, QUERY_SEED)
}

fn ivf_config() -> IvfPqConfig {
    IvfPqConfig {
        nlist: NLIST,
        kmeans_iters: 20,
        nbits: 8,
        refine: Refine::Flat,
        train_sample: 0,
        seed: TRAIN_SEED,
    }
}

/// The two filters both paths are checked against: an equality clause (~1/8 of
/// rows) and an inclusive integer range (~1/5 of rows).
fn test_filters() -> Vec<(&'static str, Filter)> {
    vec![
        ("category == 3", Filter::new().eq("category", 3i64)),
        (
            "20 <= bucket <= 40",
            Filter::new().int_range("bucket", 20, 40),
        ),
    ]
}

fn row_set(neighbors: &[Neighbor]) -> HashSet<u32> {
    neighbors.iter().map(|n| n.row).collect()
}

/// Assert `gpu_res` and `cpu_res` are the same filtered top-k: identical row
/// set (float tie-reordering allowed) and per-row scores within 1e-3.
fn assert_same_topk(cpu_res: &[Neighbor], gpu_res: &[Neighbor], ctx: &str) {
    assert_eq!(
        row_set(cpu_res),
        row_set(gpu_res),
        "{ctx}: GPU filtered top-k row set != CPU oracle\n  cpu={cpu_res:?}\n  gpu={gpu_res:?}"
    );
    let cpu_by_row: HashMap<u32, f32> = cpu_res.iter().map(|n| (n.row, n.score)).collect();
    for nb in gpu_res {
        let cpu_score = cpu_by_row[&nb.row];
        assert!(
            (cpu_score - nb.score).abs() <= 1e-3,
            "{ctx} row {}: GPU score {} vs CPU {} exceeds 1e-3",
            nb.row,
            nb.score,
            cpu_score
        );
    }
}

/// Every returned neighbor's payload must satisfy the filter.
fn assert_all_match(collection: &Collection, res: &[Neighbor], filter: &Filter, ctx: &str) {
    for nb in res {
        assert!(
            filter.matches(collection.payload(nb.row as usize)),
            "{ctx}: returned row {} does not satisfy the filter (payload {:?})",
            nb.row,
            collection.payload(nb.row as usize)
        );
    }
}

fn gpu_or_skip(test: &str) -> Option<GpuContext> {
    match GpuContext::new() {
        Some(gpu) => {
            let (backend, name) = gpu.adapter_info();
            eprintln!("[{test}] GPU adapter: {name} ({backend})");
            Some(gpu)
        }
        None => {
            eprintln!("[{test}] no GPU adapter; skipping");
            None
        }
    }
}

/// (1a) Flat path: filtered GPU top-k == filtered CPU oracle, for an Eq filter
/// and an IntRange filter, over several queries.
#[test]
fn filtered_flat_gpu_matches_cpu_oracle() {
    let Some(gpu) = gpu_or_skip("filtered_flat_gpu_matches_cpu_oracle") else {
        return;
    };
    let corpus = corpus_with_payloads();
    let cpu = CpuFlatIndex::new(&corpus);
    let gpu_index = GpuFlatIndex::new(&gpu, &corpus);

    for q in &queries() {
        for (name, filter) in test_filters() {
            let cpu_res = cpu.search_knn_filtered(q, K, &filter);
            let gpu_res = gpu_index.search_knn_filtered(q, K, &filter);
            let ctx = format!("flat [{name}]");
            assert_eq!(
                cpu_res.len(),
                K,
                "{ctx}: oracle should fill k (filter matches >> k)"
            );
            assert_same_topk(&cpu_res, &gpu_res, &ctx);
            assert_all_match(&corpus, &gpu_res, &filter, &ctx);
        }
    }
    eprintln!("  filtered flat GPU == filtered CPU oracle for all queries + filters");
}

/// (1b) IVF path: filtered GPU top-k == filtered CPU oracle. With Refine::Flat +
/// nprobe == nlist the IVF scan is exact over the whole corpus, so filtered IVF
/// must reproduce the filtered flat oracle.
#[test]
fn filtered_ivf_gpu_matches_cpu_oracle() {
    let Some(gpu) = gpu_or_skip("filtered_ivf_gpu_matches_cpu_oracle") else {
        return;
    };
    let corpus = corpus_with_payloads();
    let index = IvfPqIndex::train(&corpus, ivf_config()).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let oracle = CpuFlatIndex::new(&corpus);
    let nprobe = index.nlist();

    for q in &queries() {
        for (name, filter) in test_filters() {
            let cpu_res = oracle.search_knn_filtered(q, K, &filter);
            let gpu_res = scanner.search_filtered(&index, q, K, nprobe, &filter);
            let ctx = format!("ivf [{name}]");
            assert_eq!(
                cpu_res.len(),
                K,
                "{ctx}: oracle should fill k (filter matches >> k)"
            );
            assert_same_topk(&cpu_res, &gpu_res, &ctx);
            assert_all_match(&corpus, &gpu_res, &filter, &ctx);
        }
    }
    eprintln!(
        "  filtered IVF (Flat, full probe) GPU == filtered flat oracle for all queries + filters"
    );
}

/// (2) The filter actually restricts: every filtered neighbor satisfies the
/// filter, and a selective filter changes the result vs unfiltered search
/// (some unfiltered neighbor is filtered out).
#[test]
fn filter_restricts_results() {
    let Some(gpu) = gpu_or_skip("filter_restricts_results") else {
        return;
    };
    let corpus = corpus_with_payloads();
    let gpu_index = GpuFlatIndex::new(&gpu, &corpus);
    let filter = Filter::new().eq("category", 3i64);

    let mut any_changed = false;
    for q in &queries() {
        let unfiltered = gpu_index.search_knn(q, K);
        let filtered = gpu_index.search_knn_filtered(q, K, &filter);

        // Every filtered neighbor satisfies the filter.
        assert_all_match(&corpus, &filtered, &filter, "restrict");

        // The unfiltered top-k contained at least one row the filter drops,
        // and the filtered result differs — proof the filter changed the answer.
        let unfiltered_all_match = unfiltered
            .iter()
            .all(|nb| filter.matches(corpus.payload(nb.row as usize)));
        if !unfiltered_all_match {
            any_changed = true;
            assert_ne!(
                row_set(&unfiltered),
                row_set(&filtered),
                "filter dropped rows yet result set is unchanged"
            );
        }
    }
    assert!(
        any_changed,
        "a 1/8-selectivity filter should change the top-k for at least one query"
    );
    eprintln!("  filtered results all satisfy the filter and differ from unfiltered");
}

/// (3) Selectivity edges, on the flat GPU, flat CPU, AND IVF paths:
///   - a filter matching < k rows returns exactly the matching rows;
///   - a filter matching 0 rows returns empty.
#[test]
fn selectivity_edge_cases() {
    let Some(gpu) = gpu_or_skip("selectivity_edge_cases") else {
        return;
    };
    let corpus = corpus_with_payloads();
    let cpu = CpuFlatIndex::new(&corpus);
    let gpu_index = GpuFlatIndex::new(&gpu, &corpus);
    let index = IvfPqIndex::train(&corpus, ivf_config()).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let nprobe = index.nlist();
    let q = &queries()[0];

    // --- Fewer than k matches: `row` in [0, 5] matches exactly rows 0..=5. ---
    let few = Filter::new().int_range("row", 0, 5);
    let expected: HashSet<u32> = (0..=5).collect();

    let cpu_few = cpu.search_knn_filtered(q, K, &few);
    let gpu_few = gpu_index.search_knn_filtered(q, K, &few);
    let ivf_few = scanner.search_filtered(&index, q, K, nprobe, &few);
    for (name, res) in [("cpu", &cpu_few), ("gpu-flat", &gpu_few), ("ivf", &ivf_few)] {
        assert_eq!(
            res.len(),
            6,
            "{name}: a filter matching 6 rows must return exactly 6 (not k={K}), got {}",
            res.len()
        );
        assert_eq!(
            row_set(res),
            expected,
            "{name}: must return exactly rows 0..=5"
        );
        assert_all_match(&corpus, res, &few, name);
    }

    // --- Zero matches: no row has category 42. ---
    let none = Filter::new().eq("category", 42i64);
    assert!(
        cpu.search_knn_filtered(q, K, &none).is_empty(),
        "cpu: 0-match filter must be empty"
    );
    assert!(
        gpu_index.search_knn_filtered(q, K, &none).is_empty(),
        "gpu-flat: 0-match filter must be empty"
    );
    assert!(
        scanner
            .search_filtered(&index, q, K, nprobe, &none)
            .is_empty(),
        "ivf: 0-match filter must be empty"
    );
    eprintln!("  selectivity edges: <k returns exact matches (6), 0-match returns empty");
}
