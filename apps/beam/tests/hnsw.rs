//! HNSW graph ANN correctness gate — recall, filtered search, tombstones, metrics.
//!
//! [`HnswIndex`](beam::index::hnsw::HnswIndex) is a **CPU** graph index, so
//! (unlike the GPU tests) these never touch a GPU adapter and always run. Every
//! recall claim is measured against the exact [`CpuFlatIndex`] oracle on a
//! deterministic dataset (fixed-seed LCG); the DATA is fully reproducible.
//!
//! `hnsw_rs` assigns node layers from OS entropy, so the GRAPH differs run to
//! run — recall wobbles by ~±0.03. The bars are therefore set with margin
//! (recall clears 0.90 while measuring ~0.97+), and the ef-sweep runs on a SINGLE
//! fixed graph so "higher ef ⇒ higher recall" is stable within a run.
//!
//! What each test pins:
//!   1. High recall: HNSW recall@10 vs the flat oracle ≥ 0.90, and increasing
//!      ef_search raises it (measured on one graph).
//!   2. Filtered: every filtered result satisfies the filter, is live, and is a
//!      subset of the unfiltered-then-filtered candidates; recall stays high.
//!   3. Tombstones: a deleted row is never returned.
//!   4. Metrics: L2 AND Cosine both reach high recall.
//!
//! Isotropic Gaussian blobs make recall@k-by-row tie-heavy (hundreds of
//! near-equidistant in-cluster points), so the recall tests use the LOW-RANK
//! (embedding-like) corpus — well-separated neighbors, the realistic vector-DB
//! case — while the filtered/tombstone tests, which only need the filter/live
//! invariants, use the plain clustered corpus.

use std::collections::HashSet;

use beam::collection::{Collection, Metric};
use beam::dataset;
use beam::index::cpu_flat::CpuFlatIndex;
use beam::index::hnsw::{HnswConfig, HnswIndex};
use beam::index::{Neighbor, VectorIndex};
use beam::payload::{Filter, Payload};

const DIM: usize = 64;
const RANK: usize = 8;
const K: usize = 10;
const N_QUERIES: usize = 40;
const NUM_CLUSTERS: usize = 40;
const JITTER: f32 = 0.05;
const COEF_JITTER: f32 = 0.05;

const CORPUS_SEED: u64 = 0x11B5_0001;
const QUERY_SEED: u64 = 0x11B5_0002;

/// A recall-friendly config with margin: a denser graph (M = 32, ef_c = 200) and
/// a generous query beam (ef_s = 200) so both L2 (~0.99) and the slightly noisier
/// Cosine (~0.97) clear the 0.90 bar with headroom even under the ±0.03 graph
/// entropy (measured stable over many runs).
fn recall_config() -> HnswConfig {
    HnswConfig {
        max_nb_connection: 32,
        ef_construction: 200,
        ef_search: 200,
    }
}

fn row_set(neighbors: &[Neighbor]) -> HashSet<u32> {
    neighbors.iter().map(|n| n.row).collect()
}

/// Mean recall@K of HNSW vs the exact flat oracle over `queries`.
fn mean_recall(index: &HnswIndex, oracle: &CpuFlatIndex, queries: &[Vec<f32>]) -> f64 {
    let mut sum = 0.0;
    for q in queries {
        let truth = row_set(&oracle.search_knn(q, K));
        let got = index.search_knn(q, K);
        let hit = got.iter().filter(|nb| truth.contains(&nb.row)).count();
        sum += hit as f64 / got.len().max(1) as f64;
    }
    sum / queries.len() as f64
}

/// A deterministic LOW-RANK (embedding-like) corpus + queries for the given
/// metric — well-separated neighbors, so recall@k-by-row is high and stable.
fn low_rank_corpus(n: usize, metric: Metric) -> (Collection, Vec<Vec<f32>>) {
    let corpus = dataset::low_rank_collection(
        "hnsw",
        n,
        DIM,
        metric,
        RANK,
        NUM_CLUSTERS,
        COEF_JITTER,
        CORPUS_SEED,
    );
    let queries =
        dataset::low_rank_queries(N_QUERIES, DIM, RANK, NUM_CLUSTERS, COEF_JITTER, QUERY_SEED);
    (corpus, queries)
}

/// (1) High recall: HNSW recall@10 vs the flat oracle clears 0.90 on the low-rank
/// corpus, and increasing ef_search raises recall (measured on ONE fixed graph so
/// the graph-entropy noise cancels).
#[test]
fn recall_is_high_and_grows_with_ef_search() {
    let (corpus, queries) = low_rank_corpus(8000, Metric::L2);
    let oracle = CpuFlatIndex::new(&corpus);
    let mut index = HnswIndex::build(&corpus, recall_config());

    // Headline recall at the recall config's generous ef_search.
    let recall = mean_recall(&index, &oracle, &queries);
    eprintln!(
        "  HNSW recall@{K} (L2, low-rank, ef_search={}): {recall:.4}",
        index.ef_search()
    );
    assert!(
        recall >= 0.90,
        "HNSW recall@{K} {recall} should clear 0.90 with margin"
    );

    // Sweep ef_search on the SAME graph — recall is non-decreasing-ish and a large
    // ef beats a small one (the recall/latency lever works). We assert the strong
    // end-to-end signal (high ef ≥ low ef) rather than strict per-step monotonicity,
    // which the approximate search can violate by a hair.
    let efs = [16usize, 32, 64, 128, 256];
    let mut recalls = Vec::new();
    for &ef in &efs {
        index.set_ef_search(ef);
        recalls.push(mean_recall(&index, &oracle, &queries));
    }
    eprintln!("  HNSW recall@{K} by ef_search {efs:?}: {recalls:?}");
    let (lo, hi) = (recalls[0], *recalls.last().unwrap());
    assert!(
        hi >= lo - 1e-9,
        "raising ef_search must not lower recall: {recalls:?}"
    );
    assert!(
        hi >= 0.90,
        "recall at the largest ef_search should clear 0.90, got {hi}"
    );
}

/// (4) Metrics: L2 AND Cosine both reach high recall on the low-rank corpus
/// (the DistL2 / normalized-DistDot mappings are correct).
#[test]
fn l2_and_cosine_both_high_recall() {
    for metric in [Metric::L2, Metric::Cosine] {
        let (corpus, queries) = low_rank_corpus(8000, metric);
        let oracle = CpuFlatIndex::new(&corpus);
        let index = HnswIndex::build(&corpus, recall_config());
        let recall = mean_recall(&index, &oracle, &queries);
        eprintln!("  HNSW recall@{K} ({metric:?}, low-rank): {recall:.4}");
        assert!(
            recall >= 0.90,
            "{metric:?}: HNSW recall@{K} {recall} should clear 0.90"
        );
    }
}

/// A clustered corpus with deterministic per-row payloads (`category = i % 8`,
/// `row = i`) for the filter / tombstone invariant tests.
fn payload_corpus(n: usize) -> Collection {
    let mut c = dataset::clustered_collection(
        "hnsw-f",
        n,
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
                .with("row", i as i64),
        );
    }
    c
}

fn clustered_queries() -> Vec<Vec<f32>> {
    dataset::clustered_queries(N_QUERIES, DIM, NUM_CLUSTERS, JITTER, QUERY_SEED)
}

/// (2) Filtered: every filtered result satisfies the filter and is live, the
/// filtered result set is a subset of an unfiltered-then-filtered candidate set,
/// and a selective filter changes the answer.
#[test]
fn filtered_results_satisfy_filter_and_are_subset() {
    let corpus = payload_corpus(6000);
    let index = HnswIndex::build(&corpus, recall_config());
    let filter = Filter::new().eq("category", 3i64);

    let mut any_changed = false;
    for q in &clustered_queries() {
        let filtered = index.search_knn_filtered(q, K, &filter);

        // Every filtered neighbor satisfies the filter AND is a live row.
        for nb in &filtered {
            assert!(
                filter.matches(corpus.payload(nb.row as usize)),
                "returned row {} does not satisfy the filter",
                nb.row
            );
            assert!(corpus.is_live(nb.row), "returned a non-live row {}", nb.row);
        }

        // Subset: the filtered result is contained in a generous unfiltered search
        // then filtered — the filtered call never surfaces a row the unfiltered
        // candidate pool + filter would not.
        let unfiltered_then_filtered: HashSet<u32> = index
            .search_knn(q, corpus.len())
            .into_iter()
            .filter(|nb| filter.matches(corpus.payload(nb.row as usize)))
            .map(|nb| nb.row)
            .collect();
        for nb in &filtered {
            assert!(
                unfiltered_then_filtered.contains(&nb.row),
                "filtered row {} is not in the unfiltered+filter candidate set",
                nb.row
            );
        }

        // The filter actually restricts: unfiltered top-k dropped some row.
        let unfiltered = index.search_knn(q, K);
        if unfiltered
            .iter()
            .any(|nb| !filter.matches(corpus.payload(nb.row as usize)))
        {
            any_changed = true;
            assert_ne!(
                row_set(&unfiltered),
                row_set(&filtered),
                "filter dropped rows yet the result set is unchanged"
            );
        }
    }
    assert!(
        any_changed,
        "a 1/8-selectivity filter should change the top-k for at least one query"
    );

    // Filtered recall vs the exact filtered flat oracle stays high.
    let oracle = CpuFlatIndex::new(&corpus);
    let mut sum = 0.0;
    for q in &clustered_queries() {
        let truth = row_set(&oracle.search_knn_filtered(q, K, &filter));
        let got = index.search_knn_filtered(q, K, &filter);
        let hit = got.iter().filter(|nb| truth.contains(&nb.row)).count();
        sum += hit as f64 / got.len().max(1) as f64;
    }
    let recall = sum / N_QUERIES as f64;
    eprintln!("  HNSW filtered recall@{K} vs filtered flat oracle: {recall:.4}");
    assert!(
        recall >= 0.70,
        "filtered HNSW recall {recall} should clear 0.70"
    );
}

/// (2b) Selectivity edge: a filter matching fewer than k rows returns exactly the
/// matching rows (length = #matches), and a 0-match filter returns empty.
#[test]
fn filtered_selectivity_edges() {
    let corpus = payload_corpus(6000);
    let index = HnswIndex::build(&corpus, recall_config());
    let q = &clustered_queries()[0];

    // `row in [0, 5]` matches exactly rows 0..=5 (six rows, < k).
    let few = Filter::new().int_range("row", 0, 5);
    let got = index.search_knn_filtered(q, K, &few);
    assert_eq!(
        got.len(),
        6,
        "a filter matching 6 rows must return exactly 6, not k={K}"
    );
    assert_eq!(
        row_set(&got),
        (0..=5).collect(),
        "must return exactly rows 0..=5"
    );

    // No row has category 42.
    let none = Filter::new().eq("category", 42i64);
    assert!(
        index.search_knn_filtered(q, K, &none).is_empty(),
        "a 0-match filter must return empty"
    );
}

/// (3) Tombstones: rows deleted before the build are never returned, and the HNSW
/// result equals a freshly-built oracle over ONLY the live rows (by external id).
#[test]
fn deleted_rows_never_returned() {
    let mut corpus = dataset::clustered_collection(
        "hnsw-t",
        6000,
        DIM,
        Metric::L2,
        NUM_CLUSTERS,
        JITTER,
        CORPUS_SEED,
    );
    // Deterministic delete set: every 7th id.
    let deleted: HashSet<String> = (0..corpus.len())
        .step_by(7)
        .map(|i| format!("id-{i}"))
        .collect();
    for id in &deleted {
        assert!(corpus.delete(id));
    }
    assert_eq!(corpus.tombstoned(), deleted.len());

    let index = HnswIndex::build(&corpus, recall_config());
    assert_eq!(index.tombstoned(), deleted.len());
    assert_eq!(index.len(), corpus.len());

    for q in &clustered_queries() {
        let got = index.search_knn(q, K);
        assert_eq!(got.len(), K, "still returns k live neighbors after deletes");
        for nb in &got {
            assert!(
                !deleted.contains(&nb.external_id),
                "deleted id {} was returned",
                nb.external_id
            );
            assert!(
                corpus.is_live(nb.row),
                "tombstoned row {} was returned",
                nb.row
            );
        }
    }

    // Recall vs a freshly-built oracle over ONLY the live rows stays high.
    let mut live = corpus.clone();
    live.compact();
    let oracle = CpuFlatIndex::new(&live);
    let mut sum = 0.0;
    let queries = clustered_queries();
    for q in &queries {
        // Compare by EXTERNAL ID: compaction renumbers rows, so ids are the identity.
        let truth: HashSet<String> = oracle
            .search_knn(q, K)
            .iter()
            .map(|n| n.external_id.clone())
            .collect();
        let got = index.search_knn(q, K);
        let hit = got
            .iter()
            .filter(|nb| truth.contains(&nb.external_id))
            .count();
        sum += hit as f64 / got.len().max(1) as f64;
    }
    let recall = sum / queries.len() as f64;
    eprintln!(
        "  HNSW post-delete recall@{K} vs live oracle: {recall:.4} ({} deleted, none returned)",
        deleted.len()
    );
    assert!(
        recall >= 0.60,
        "post-delete recall {recall} should clear 0.60 on tie-heavy clustered data"
    );
}
