//! Batched GPU flat query correctness gate.
//!
//! [`GpuFlatIndex::search_knn_batch`] processes a whole query set in as few GPU
//! dispatches as possible — the throughput lever. For `k <= `[`beam::gpu::MAX_TOPK`]
//! it runs the **GPU-side per-query top-k** kernel (one workgroup per query selects
//! the top-k on the GPU; readback is only `num_q * k` (id, score) pairs); for larger
//! `k` it falls back to the `num_q * n` distance-matrix + CPU top-k path. These tests
//! pin the correctness contract: a batched query returns, per query, **exactly** what
//! the serial [`search_knn`](beam::index::VectorIndex::search_knn) — and the exact
//! CPU oracle — returns, on both paths.
//!
//!   1. Batched == serial: for a deterministic corpus + a batch of queries, the
//!      per-query batched result equals the serial GPU result (row sets + per-row
//!      scores within 1e-3), for L2 and Dot.
//!   2. Tombstones: after deleting some rows, the batched query excludes them and
//!      still equals the live serial result (the same live-mask path).
//!   3. Batch-size / tiling invariance: tile=1, a small tile, and a tile smaller
//!      than the query count (so the batch spans multiple tiles) all produce
//!      identical results — exercising the internal tiling boundary.
//!   4. GPU top-k == CPU oracle across k ∈ {1, 10, 32}, L2 and Dot (the exact
//!      selection is on the GPU).
//!   5. GPU top-k == the previous distance-matrix + CPU-top-k path (both batched).
//!   6. `k > MAX_TOPK` falls back to the distance-matrix path and stays exact.
//!
//! Every test skips gracefully (prints, returns) when no GPU adapter is present,
//! so GPU-less CI stays green; on this Mac they PRINT the Metal adapter and PASS.

use std::collections::{HashMap, HashSet};

use beam::collection::Metric;
use beam::dataset;
use beam::gpu::{GpuContext, GpuFlatIndex, MAX_TOPK, TILE_N_GEMM, TILE_Q_GEMM};
use beam::index::cpu_flat::CpuFlatIndex;
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
        assert_eq!(
            batched[qi].len(),
            K,
            "{metric:?} q{qi}: batched should return K"
        );
        assert_same(&format!("{metric:?} q{qi}"), &batched[qi], &serial);
    }
    eprintln!(
        "  batched == serial OK for {metric:?} (n={N}, dim={DIM}, k={K}, {N_QUERIES} queries)"
    );
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
        assert!(
            collection.delete(&format!("id-{r}")),
            "row {r} should be live before delete"
        );
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
        assert_same(
            &format!("tombstone q{qi}"),
            &batched[qi],
            &index.search_knn(q, K),
        );
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
            assert_same(
                &format!("tile={tile} q{qi} vs serial"),
                &tiled[qi],
                &index.search_knn(q, K),
            );
            assert_same(&format!("tile={tile} q{qi} vs auto"), &tiled[qi], &auto[qi]);
        }
    }
    eprintln!(
        "  tiling invariance OK (tiles 1, 5, {N_QUERIES}, {} over {N_QUERIES} queries)",
        N_QUERIES + 8
    );
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

/// GPU-side top-k batched result == the exact CPU oracle
/// ([`CpuFlatIndex::search_knn`]) per query, for k ∈ {1, 10, 32} (all ≤ MAX_TOPK,
/// so the GPU-top-k kernel does the selection), on L2 and Dot. This is the goal:
/// the top-k is computed ON the GPU and equals ground truth (row set + scores).
#[test]
fn gpu_topk_matches_cpu_oracle() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gpu_topk_matches_cpu_oracle");
        return;
    };
    let (backend, name) = gpu.adapter_info();
    eprintln!("GPU adapter: {name} ({backend})");

    for metric in [Metric::L2, Metric::Dot] {
        let collection = dataset::random_collection("topk", N, DIM, metric, DATASET_SEED);
        let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
        let cpu = CpuFlatIndex::new(&collection);
        let index = GpuFlatIndex::new(&gpu, &collection);

        for &k in &[1usize, 10, 32] {
            assert!(k <= MAX_TOPK, "test k must stay on the GPU-top-k path");
            let batched = index.search_knn_batch(&queries, k);
            assert_eq!(batched.len(), queries.len());
            for (qi, q) in queries.iter().enumerate() {
                let oracle = cpu.search_knn(q, k);
                assert_eq!(oracle.len(), k, "{metric:?} k={k} q{qi}: oracle len");
                assert_eq!(batched[qi].len(), k, "{metric:?} k={k} q{qi}: gpu-topk len");
                assert_same(
                    &format!("{metric:?} k={k} q{qi} vs oracle"),
                    &batched[qi],
                    &oracle,
                );
            }
        }
        eprintln!("  GPU top-k == CPU oracle OK for {metric:?} (k ∈ 1,10,32)");
    }
}

/// GPU-side top-k == the previous distance-matrix + CPU-top-k batched path, per
/// query (row set + scores). Guards that swapping the readback from `T × n`
/// distances to `T × k` (id, score) pairs did not change the result.
#[test]
fn gpu_topk_matches_distmatrix_path() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gpu_topk_matches_distmatrix_path");
        return;
    };
    let collection = dataset::random_collection("topk-dm", N, DIM, Metric::L2, DATASET_SEED);
    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
    let index = GpuFlatIndex::new(&gpu, &collection);

    for &k in &[1usize, 10, 32] {
        let topk = index.search_knn_batch(&queries, k); // GPU top-k (k ≤ MAX_TOPK)
        let distmatrix = index.search_knn_batch_distmatrix(&queries, k); // forced T×n path
        assert_eq!(topk.len(), distmatrix.len());
        for qi in 0..queries.len() {
            assert_same(
                &format!("k={k} q{qi} topk vs distmatrix"),
                &topk[qi],
                &distmatrix[qi],
            );
        }
    }
    eprintln!("  GPU top-k == distance-matrix path OK (k ∈ 1,10,32)");
}

/// `k > MAX_TOPK` falls back to the distance-matrix + CPU-top-k path (the GPU
/// register/shared top-k is capped at MAX_TOPK) and stays exact vs the CPU oracle,
/// so large-k batched queries still work.
#[test]
fn gpu_topk_falls_back_above_max_k() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gpu_topk_falls_back_above_max_k");
        return;
    };
    let k = MAX_TOPK + 8; // above the GPU-top-k cap → distance-matrix fallback
    let collection = dataset::random_collection("topk-fb", N, DIM, Metric::L2, DATASET_SEED);
    let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
    let cpu = CpuFlatIndex::new(&collection);
    let index = GpuFlatIndex::new(&gpu, &collection);

    let batched = index.search_knn_batch(&queries, k);
    // The fallback and the explicit distance-matrix path must agree, and both must
    // equal the exact CPU oracle.
    let distmatrix = index.search_knn_batch_distmatrix(&queries, k);
    for (qi, q) in queries.iter().enumerate() {
        let oracle = cpu.search_knn(q, k);
        assert_eq!(batched[qi].len(), k, "q{qi}: fallback should return k={k}");
        assert_same(
            &format!("fallback k={k} q{qi} vs oracle"),
            &batched[qi],
            &oracle,
        );
        assert_same(
            &format!("fallback k={k} q{qi} vs distmatrix"),
            &batched[qi],
            &distmatrix[qi],
        );
    }
    eprintln!("  k > MAX_TOPK ({k}) falls back to distance-matrix path + stays exact OK");
}

// ---- GEMM-tiled kernel (`main_batch_tiled`, `search_knn_batch_gemm`) ----------
//
// The shared-memory tiled kernel: a tile of TILE_Q_GEMM queries reuses each staged
// DB block (DB-row reuse across the query tile), split-k for occupancy, vec4 inner
// loop, GPU-side per-query top-k merged across splits on the host. These tests pin
// its exactness directly (the auto path already routes L2/Dot dim-48 batches here,
// but these force the `search_knn_batch_gemm` entry and cross its tile boundaries).

/// GEMM-tiled result == the exact CPU oracle per query for k ∈ {1, 10, 32}, on L2
/// and Dot — the direct-arithmetic (vec4) distances select the same rows as ground
/// truth (row set + scores within 1e-3).
#[test]
fn gemm_tiled_matches_cpu_oracle() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gemm_tiled_matches_cpu_oracle");
        return;
    };
    let (backend, name) = gpu.adapter_info();
    eprintln!("GPU adapter: {name} ({backend})");

    for metric in [Metric::L2, Metric::Dot] {
        let collection = dataset::random_collection("gemm", N, DIM, metric, DATASET_SEED);
        let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
        let cpu = CpuFlatIndex::new(&collection);
        let index = GpuFlatIndex::new(&gpu, &collection);

        for &k in &[1usize, 10, 32] {
            assert!(k <= MAX_TOPK, "test k must stay on the GEMM-tiled path");
            let tiled = index.search_knn_batch_gemm(&queries, k);
            assert_eq!(tiled.len(), queries.len());
            for (qi, q) in queries.iter().enumerate() {
                let oracle = cpu.search_knn(q, k);
                assert_eq!(oracle.len(), k, "{metric:?} k={k} q{qi}: oracle len");
                assert_eq!(tiled[qi].len(), k, "{metric:?} k={k} q{qi}: gemm len");
                assert_same(
                    &format!("gemm {metric:?} k={k} q{qi} vs oracle"),
                    &tiled[qi],
                    &oracle,
                );
            }
        }
        eprintln!("  GEMM-tiled == CPU oracle OK for {metric:?} (k ∈ 1,10,32)");
    }
}

/// GEMM-tiled result == the one-workgroup-per-query `main_batch_topk` path, per
/// query (row set + scores). Guards that the two GPU batched top-k kernels agree.
#[test]
fn gemm_tiled_matches_topk_path() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gemm_tiled_matches_topk_path");
        return;
    };
    for metric in [Metric::L2, Metric::Dot] {
        let collection = dataset::random_collection("gemm-tk", N, DIM, metric, DATASET_SEED);
        let queries = dataset::random_queries(N_QUERIES, DIM, QUERY_SEED);
        let index = GpuFlatIndex::new(&gpu, &collection);
        for &k in &[1usize, 10, 32] {
            let tiled = index.search_knn_batch_gemm(&queries, k);
            let topk = index.search_knn_batch_topk_tiled(&queries, k, queries.len());
            assert_eq!(tiled.len(), topk.len());
            for qi in 0..queries.len() {
                assert_same(
                    &format!("{metric:?} k={k} q{qi} gemm vs topk"),
                    &tiled[qi],
                    &topk[qi],
                );
            }
        }
    }
    eprintln!("  GEMM-tiled == main_batch_topk path OK (k ∈ 1,10,32, L2+Dot)");
}

/// GEMM-tiled stays exact across the tile boundaries: n NOT a multiple of
/// TILE_N_GEMM (ragged DB tiles), a query count NOT a multiple of TILE_Q_GEMM
/// (ragged query tiles / idle threads + multiple query-tiles), AND tombstones
/// (skipped rows) — all must still equal the live CPU oracle.
#[test]
fn gemm_tiled_tile_boundaries_and_tombstones() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gemm_tiled_tile_boundaries_and_tombstones");
        return;
    };
    // n chosen off a TILE_N_GEMM multiple; query count off a TILE_Q_GEMM multiple
    // (spanning >1 query-tile with a ragged last tile).
    let n_boundary = 130 * TILE_N_GEMM + 7;
    let nq_boundary = 2 * TILE_Q_GEMM + 5;
    assert!(
        !n_boundary.is_multiple_of(TILE_N_GEMM),
        "n must be off a TILE_N boundary"
    );
    assert!(
        !nq_boundary.is_multiple_of(TILE_Q_GEMM),
        "nq must be off a TILE_Q boundary"
    );

    let mut collection =
        dataset::random_collection("gemm-b", n_boundary, DIM, Metric::L2, DATASET_SEED);
    // Delete a spread-out set so tombstones fall inside DB tiles.
    let deleted: HashSet<u32> = (0..n_boundary as u32).filter(|r| r % 5 == 0).collect();
    for &r in &deleted {
        assert!(collection.delete(&format!("id-{r}")));
    }
    assert!(collection.tombstoned() > 0);

    let queries = dataset::random_queries(nq_boundary, DIM, QUERY_SEED);
    let cpu = CpuFlatIndex::new(&collection);
    let index = GpuFlatIndex::new(&gpu, &collection);

    for &k in &[1usize, 10, 32] {
        let tiled = index.search_knn_batch_gemm(&queries, k);
        assert_eq!(tiled.len(), queries.len());
        for (qi, q) in queries.iter().enumerate() {
            for nb in &tiled[qi] {
                assert!(
                    !deleted.contains(&nb.row),
                    "k={k} q{qi}: tombstoned row {} selected",
                    nb.row
                );
            }
            assert_same(
                &format!("boundary k={k} q{qi}"),
                &tiled[qi],
                &cpu.search_knn(q, k),
            );
        }
    }
    eprintln!(
        "  GEMM-tiled boundaries OK (n={n_boundary} off TILE_N={TILE_N_GEMM}, nq={nq_boundary} off TILE_Q={TILE_Q_GEMM}, {} tombstoned)",
        collection.tombstoned()
    );
}

/// A query dimension not supported by the tiled kernel (not a multiple of 4, or
/// wider than the shared-tile width) makes `search_knn_batch` fall back to the
/// one-workgroup-per-query top-k path and stay exact vs the CPU oracle — so any
/// dim still works, just off the tiled fast path.
#[test]
fn gemm_tiled_falls_back_for_unsupported_dim() {
    let Some(gpu) = GpuContext::new() else {
        eprintln!("no GPU adapter; skipping gemm_tiled_falls_back_for_unsupported_dim");
        return;
    };
    // dim = 50 is not a multiple of 4 → tiled kernel ineligible → topk fallback.
    let dim = 50usize;
    assert!(
        !dim.is_multiple_of(4),
        "test dim must be vec4-unaligned to force the fallback"
    );
    let collection = dataset::random_collection("gemm-fb", 1500, dim, Metric::L2, DATASET_SEED);
    let queries = dataset::random_queries(N_QUERIES, dim, QUERY_SEED);
    let cpu = CpuFlatIndex::new(&collection);
    let index = GpuFlatIndex::new(&gpu, &collection);

    let batched = index.search_knn_batch(&queries, K);
    for (qi, q) in queries.iter().enumerate() {
        assert_eq!(batched[qi].len(), K, "q{qi}: fallback should return K");
        assert_same(
            &format!("dim-fallback q{qi}"),
            &batched[qi],
            &cpu.search_knn(q, K),
        );
    }
    eprintln!("  vec4-unaligned dim={dim} falls back to top-k path + stays exact OK");
}
