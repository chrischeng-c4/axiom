//! Batched GPU flat query correctness gate.
//!
//! [`GpuFlatIndex::search_knn_batch`] processes a whole query set in as few GPU
//! dispatches as possible (tiling the batch and scoring each tile's `T × n`
//! distance sub-matrix in one dispatch) — the throughput lever. These tests pin
//! the correctness contract: a batched query returns, per query, **exactly** what
//! the serial [`search_knn`](beam::index::VectorIndex::search_knn) returns.
//!
//!   1. Batched == serial: for a deterministic corpus + a batch of queries, the
//!      per-query batched result equals the serial GPU result (row sets + per-row
//!      scores within 1e-3), for L2 and Dot.
//!   2. Tombstones: after deleting some rows, the batched query excludes them and
//!      still equals the live serial result (the same live-mask/sentinel path).
//!   3. Batch-size / tiling invariance: tile=1, a small tile, and a tile smaller
//!      than the query count (so the batch spans multiple tiles) all produce
//!      identical results — exercising the internal tiling boundary.
//!
//! Every test skips gracefully (prints, returns) when no GPU adapter is present,
//! so GPU-less CI stays green; on this Mac they PRINT the Metal adapter and PASS.

use std::collections::{HashMap, HashSet};

use beam::collection::Metric;
use beam::dataset;
use beam::gpu::{GpuContext, GpuFlatIndex};
use beam::index::{Neighbor, VectorIndex};

const N: usize = 2000;
const DIM: usize = 48;
const K: usize = 10;
const N_QUERIES: usize = 16;
const DATASET_SEED: u64 = 0x0BEA_0B01;
const QUERY_SEED: u64 = 0x0BEA_0B02;

fn row_set(neighbors: &[Neighbor]) -> HashSet<u32> {
    neighbors.iter().map(|n| n.row).collect()
}

/// Assert one batched result equals one serial result: same top-k row set, and
/// per-row scores within 1e-3 (float tie-reordering allowed — compared as sets).
fn assert_same(context: &str, batched: &[Neighbor], serial: &[Neighbor]) {
    assert_eq!(
        row_set(batched),
        row_set(serial),
        "{context}: batched row set != serial\n  batched={batched:?}\n  serial={serial:?}"
    );
    let serial_by_row: HashMap<u32, f32> = serial.iter().map(|n| (n.row, n.score)).collect();
    for nb in batched {
        let s = serial_by_row[&nb.row];
        assert!(
            (s - nb.score).abs() <= 1e-3,
            "{context} row {}: batched score {} vs serial {} exceeds 1e-3",
            nb.row,
            nb.score,
            s
        );
    }
}

/// Batched per-query result == serial `search_knn` per query, for one metric.
fn assert_batched_matches_serial(gpu: &GpuContext, metric: Metric) {
    let collection = dataset::random_collection("batch", N, DIM, metric, DATASET_SEED);
    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
    let index = GpuFlatIndex::new(gpu, &collection);

    let batched = index.search_knn_batch(&queries, K);
    assert_eq!(batched.len(), queries.len());
    for (qi, q) in queries.iter().enumerate() {
        let serial = index.search_knn(q, K);
        assert_eq!(serial.len(), K, "{metric:?} q{qi}: serial should return K");
        assert_eq!(batched[qi].len(), K, "{metric:?} q{qi}: batched should return K");
        assert_same(&format!("{metric:?} q{qi}"), &batched[qi], &serial);
    }
    eprintln!("  batched == serial OK for {metric:?} (n={N}, dim={DIM}, k={K}, {N_QUERIES} queries)");
}

#[test]
fn batched_matches_serial_l2_and_dot() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping batched_matches_serial_l2_and_dot");
        return;
    };
    let (backend, name) = gpu.adapter_info();
    eprintln!("GPU adapter: {name} ({backend})");
    assert_batched_matches_serial(&gpu, Metric::L2);
    assert_batched_matches_serial(&gpu, Metric::Dot);
}

#[test]
fn batched_excludes_tombstones() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping batched_excludes_tombstones");
        return;
    };

    // Deterministic corpus with external ids id-0..id-(N-1); delete a fixed,
    // spread-out set of rows so tombstones accumulate.
    let mut collection = dataset::random_collection("batch-del", N, DIM, Metric::L2, DATASET_SEED);
    let deleted: HashSet<u32> = (0..N as u32).filter(|r| r % 7 == 0).collect();
    for &r in &deleted {
        assert!(collection.delete(&format!("id-{r}")), "row {r} should be live before delete");
    }
    assert!(collection.tombstoned() > 0, "should have tombstones");

    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
    let index = GpuFlatIndex::new(&gpu, &collection);

    let batched = index.search_knn_batch(&queries, K);
    for (qi, q) in queries.iter().enumerate() {
        // No deleted row may appear.
        for nb in &batched[qi] {
            assert!(
                !deleted.contains(&nb.row),
                "q{qi}: batched result includes tombstoned row {}",
                nb.row
            );
        }
        // And the batched live result equals the serial live result.
        assert_same(&format!("tombstone q{qi}"), &batched[qi], &index.search_knn(q, K));
    }
    eprintln!(
        "  tombstones excluded OK ({} live, {} tombstoned)",
        collection.len(),
        collection.tombstoned()
    );
}

#[test]
fn batched_is_tiling_invariant() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping batched_is_tiling_invariant");
        return;
    };

    let collection = dataset::random_collection("batch-tile", N, DIM, Metric::L2, DATASET_SEED);
    // More queries than the smallest tiles below, so tile=1/small/large all span a
    // different number of dispatches while the batch itself is unchanged.
    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
    let index = GpuFlatIndex::new(&gpu, &collection);

    // The auto-tiled public path is the reference.
    let auto = index.search_knn_batch(&queries, K);

    // tile=1 (one dispatch per query), a small tile, and a tile larger than the
    // query count (one dispatch for the whole batch) must all agree with `auto`
    // AND with the serial path — the internal tiling boundary is exercised at
    // tile=1 and tile=5 (N_QUERIES=16 → 16 and 4 tiles respectively).
    for &tile in &[1usize, 5, N_QUERIES, N_QUERIES + 8] {
        let tiled = index.search_knn_batch_tiled(&queries, K, tile);
        assert_eq!(tiled.len(), queries.len());
        for (qi, q) in queries.iter().enumerate() {
            assert_same(&format!("tile={tile} q{qi} vs serial"), &tiled[qi], &index.search_knn(q, K));
            assert_same(&format!("tile={tile} q{qi} vs auto"), &tiled[qi], &auto[qi]);
        }
    }
    eprintln!("  tiling invariance OK (tiles 1, 5, {N_QUERIES}, {} over {N_QUERIES} queries)", N_QUERIES + 8);
}

/// A single query passed as a one-element batch equals the serial result — the
/// B=1 boundary, and a guard that `search_knn_batch` never over/under-returns.
#[test]
fn batched_single_query_equals_serial() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping batched_single_query_equals_serial");
        return;
    };
    let collection = dataset::random_collection("batch-one", N, DIM, Metric::Dot, DATASET_SEED);
    let queries = dataset::random_queries(4, DIM, QUERY_SEED);
    let index = GpuFlatIndex::new(&gpu, &collection);

    for q in &queries {
        let batched = index.search_knn_batch(std::slice::from_ref(q), K);
        assert_eq!(batched.len(), 1);
        assert_same("single", &batched[0], &index.search_knn(q, K));
    }

    // An empty batch and k=0 both yield all-empty results (the serial contract).
    assert!(index.search_knn_batch::<Vec<f32>>(&[], K).is_empty());
    let zero = index.search_knn_batch(&queries, 0);
    assert_eq!(zero.len(), queries.len());
    assert!(zero.iter().all(|r| r.is_empty()));
    eprintln!("  single-query + empty/k=0 edge cases OK");
}

/// A wrong-dimension query yields an empty result at its position while the
/// well-formed queries in the same batch are unaffected (serial contract).
#[test]
fn batched_wrong_dim_query_is_empty_slot() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping batched_wrong_dim_query_is_empty_slot");
        return;
    };
    let collection = dataset::random_collection("batch-dim", 500, DIM, Metric::L2, DATASET_SEED);
    let index = GpuFlatIndex::new(&gpu, &collection);
    let good = dataset::random_queries(1, DIM, QUERY_SEED).pop().unwrap();
    let bad = vec![0.0f32; DIM + 1];

    let batch: Vec<Vec<f32>> = vec![good.clone(), bad, good.clone()];
    let res = index.search_knn_batch(&batch, K);
    assert_eq!(res.len(), 3);
    assert!(res[1].is_empty(), "wrong-dim query slot should be empty");
    assert_same("good slot 0", &res[0], &index.search_knn(&good, K));
    assert_same("good slot 2", &res[2], &index.search_knn(&good, K));
    eprintln!("  wrong-dim slot handled OK");
}
