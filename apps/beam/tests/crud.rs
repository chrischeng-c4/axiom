//! CRUD correctness gate — delete / update / upsert reflected in search.
//!
//! A real vector database mutates. These tests build a deterministic clustered
//! corpus (via the `dataset.rs` LCG) with external ids `v0..v(N-1)` and assert,
//! for both the flat GPU path AND the IVF path (Refine::Flat + full probe, so it
//! is exact vs the live oracle):
//!
//!   1. Delete excludes: a deleted id never appears in a result, and the result
//!      equals a freshly-built `CpuFlatIndex` over ONLY the live rows (GPU == live
//!      CPU oracle). Deleting a query's own nearest neighbor makes the next result
//!      take its place — reflected mask-only via `refresh_mask` (no re-upload /
//!      no re-train).
//!   2. Update replaces: after `update`, a query near the NEW location returns the
//!      id and a query near the OLD location does not; the external id resolves to
//!      the new vector.
//!   3. Upsert: a new id is added, an existing id is replaced (live count
//!      unchanged), and the replacement is found at its new location.
//!   4. Counts: `len()` tracks the live count across a mix of add/delete/update/
//!      upsert; `capacity()`/`tombstoned()`/`compact()` behave.
//!
//! Every GPU test skips gracefully (prints, returns) when no GPU adapter is
//! present, so GPU-less CI stays green; on this Mac they PRINT the Metal adapter
//! and PASS. The counts test needs no GPU and always runs.

use std::collections::{HashMap, HashSet};

use beam::collection::{Collection, Metric};
use beam::dataset;
use beam::gpu::ivfpq::GpuIvfScanner;
use beam::gpu::{GpuContext, GpuFlatIndex};
use beam::index::cpu_flat::CpuFlatIndex;
use beam::index::ivf_pq::{IvfPqConfig, IvfPqIndex, Refine};
use beam::index::{Neighbor, VectorIndex};
use beam::payload::Payload;

const N: usize = 3000;
const DIM: usize = 32;
const K: usize = 10;
const NLIST: usize = 32;
const NUM_CLUSTERS: usize = 32;
const JITTER: f32 = 0.05;
const N_QUERIES: usize = 8;

const CORPUS_SEED: u64 = 0x0C0D_1000;
const QUERY_SEED: u64 = 0x0C0D_2000;
const TRAIN_SEED: u64 = 0x0C0D_3000;

/// A deterministic clustered L2 corpus with external ids `v0..v(N-1)`.
fn build_corpus() -> Collection {
    let model = dataset::ClusterModel::new(DIM, NUM_CLUSTERS, CORPUS_SEED);
    let mut rng = dataset::Lcg::new(CORPUS_SEED ^ 0xC0FF_EE00_D15E_A5E5);
    let mut c = Collection::new("crud", DIM, Metric::L2);
    let mut v = vec![0.0f32; DIM];
    for i in 0..N {
        model.draw(&mut rng, JITTER, &mut v);
        c.add(format!("v{i}"), &v)
            .expect("fixed-dim vector always matches collection dim");
    }
    c
}

/// Clustered near-neighbor queries sharing the corpus centers.
fn queries() -> Vec<Vec<f32>> {
    dataset::clustered_queries(N_QUERIES, DIM, NUM_CLUSTERS, JITTER, QUERY_SEED)
}

/// Exact IVF config: Flat refine + (with full probe) an exact scan over the live
/// set, so IVF must reproduce the live flat oracle.
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

/// A compacted copy of `c` holding ONLY its live rows — the "freshly-built index
/// over the live rows" the mutated GPU/IVF index is checked against.
fn live_only(c: &Collection) -> Collection {
    let mut lc = c.clone();
    lc.compact();
    lc
}

fn id_set(ns: &[Neighbor]) -> HashSet<String> {
    ns.iter().map(|n| n.external_id.clone()).collect()
}

/// Assert two results are the same top-k by EXTERNAL ID (the meaningful identity —
/// the live oracle renumbers rows on compaction) with per-id scores within 1e-3.
fn assert_same_ids_scores(oracle: &[Neighbor], got: &[Neighbor], ctx: &str) {
    assert_eq!(
        id_set(oracle),
        id_set(got),
        "{ctx}: id set mismatch\n  oracle={oracle:?}\n  got={got:?}"
    );
    let by_id: HashMap<&str, f32> = oracle
        .iter()
        .map(|n| (n.external_id.as_str(), n.score))
        .collect();
    for nb in got {
        let want = by_id[nb.external_id.as_str()];
        assert!(
            (want - nb.score).abs() <= 1e-3,
            "{ctx} id {}: got score {} vs oracle {} exceeds 1e-3",
            nb.external_id,
            nb.score,
            want
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

/// (1) Delete excludes — flat GPU: a batch of deleted ids never appears, and the
/// result equals the live CPU oracle.
#[test]
fn delete_excludes_flat() {
    let Some(gpu) = gpu_or_skip("delete_excludes_flat") else {
        return;
    };
    let mut c = build_corpus();
    let q = &queries()[0];

    // Deterministic delete set: every 7th id (~430 of 3000).
    let del: HashSet<String> = (0..N).step_by(7).map(|i| format!("v{i}")).collect();
    for id in &del {
        assert!(c.delete(id));
    }
    assert_eq!(
        c.len(),
        N - del.len(),
        "live count drops by the delete count"
    );
    assert_eq!(c.tombstoned(), del.len());

    // GPU flat over the mutated collection folds the live-mask into the keep-set.
    let gpu_idx = GpuFlatIndex::new(&gpu, &c);
    assert_eq!(gpu_idx.tombstoned(), del.len());
    let got = gpu_idx.search_knn(q, K);
    assert_eq!(got.len(), K);
    for nb in &got {
        assert!(
            !del.contains(&nb.external_id),
            "deleted id {} was returned",
            nb.external_id
        );
    }

    // GPU == freshly-built CpuFlatIndex over ONLY the live rows.
    let live = live_only(&c);
    let oracle = CpuFlatIndex::new(&live);
    assert_same_ids_scores(&oracle.search_knn(q, K), &got, "delete flat");
    eprintln!(
        "  delete flat: {} deleted, GPU == live CPU oracle, no deleted id returned",
        del.len()
    );
}

/// (1) Delete excludes — IVF (Flat, full probe): same, on the IVF path.
#[test]
fn delete_excludes_ivf() {
    let Some(gpu) = gpu_or_skip("delete_excludes_ivf") else {
        return;
    };
    let mut c = build_corpus();
    let q = &queries()[0];
    let del: HashSet<String> = (0..N).step_by(7).map(|i| format!("v{i}")).collect();
    for id in &del {
        c.delete(id);
    }

    let index = IvfPqIndex::train(&c, ivf_config()).unwrap();
    assert_eq!(index.tombstoned(), del.len());
    let scanner = GpuIvfScanner::new(&gpu);
    let nprobe = index.nlist(); // full probe → exact over the live set
    let got = scanner.search(&index, q, K, nprobe);
    assert_eq!(got.len(), K);
    for nb in &got {
        assert!(
            !del.contains(&nb.external_id),
            "deleted id {} was returned",
            nb.external_id
        );
    }

    let live = live_only(&c);
    let oracle = CpuFlatIndex::new(&live);
    assert_same_ids_scores(&oracle.search_knn(q, K), &got, "delete ivf");
    eprintln!("  delete ivf (Flat, full probe): GPU == live oracle, no deleted id returned");
}

/// (1) Delete a query's own nearest neighbor → the next result takes its place,
/// reflected MASK-ONLY (`refresh_mask`, no db re-upload) on the flat GPU path.
#[test]
fn delete_nearest_neighbor_next_takes_place_flat() {
    let Some(gpu) = gpu_or_skip("delete_nearest_neighbor_next_takes_place_flat") else {
        return;
    };
    let mut c = build_corpus();
    let q = &queries()[0];
    let mut gpu_idx = GpuFlatIndex::new(&gpu, &c);
    let pre = gpu_idx.search_knn(q, K);
    assert_eq!(pre.len(), K);

    // Delete the query's own nearest neighbor.
    assert!(c.delete(&pre[0].external_id));
    // Mask-only: reflect the delete WITHOUT re-uploading the vector buffer.
    assert!(
        gpu_idx.refresh_mask(&c),
        "a delete-only change is a mask-only refresh"
    );
    assert_eq!(gpu_idx.len(), c.len());

    let post = gpu_idx.search_knn(q, K);
    assert!(
        post.iter().all(|nb| nb.external_id != pre[0].external_id),
        "deleted nearest neighbor still returned"
    );
    assert_eq!(
        post[0].external_id, pre[1].external_id,
        "the previous 2nd-nearest takes the top spot"
    );

    let live = live_only(&c);
    assert_same_ids_scores(
        &CpuFlatIndex::new(&live).search_knn(q, K),
        &post,
        "delete-nn flat",
    );
    eprintln!(
        "  delete NN flat (mask-only refresh): next result took its place, GPU == live oracle"
    );
}

/// (1) Same on the IVF path, mask-only via `IvfPqIndex::refresh_mask` (no retrain).
#[test]
fn delete_nearest_neighbor_next_takes_place_ivf() {
    let Some(gpu) = gpu_or_skip("delete_nearest_neighbor_next_takes_place_ivf") else {
        return;
    };
    let mut c = build_corpus();
    let q = &queries()[0];
    let mut index = IvfPqIndex::train(&c, ivf_config()).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let nprobe = index.nlist();
    let pre = scanner.search(&index, q, K, nprobe);
    assert_eq!(pre.len(), K);

    assert!(c.delete(&pre[0].external_id));
    assert!(
        index.refresh_mask(&c),
        "a delete-only change is a mask-only refresh (no retrain)"
    );

    let post = scanner.search(&index, q, K, nprobe);
    assert!(
        post.iter().all(|nb| nb.external_id != pre[0].external_id),
        "deleted nearest neighbor still returned"
    );
    assert_eq!(
        post[0].external_id, pre[1].external_id,
        "the previous 2nd-nearest takes the top spot"
    );

    let live = live_only(&c);
    assert_same_ids_scores(
        &CpuFlatIndex::new(&live).search_knn(q, K),
        &post,
        "delete-nn ivf",
    );
    eprintln!(
        "  delete NN ivf (mask-only refresh): next result took its place, GPU == live oracle"
    );
}

/// (2) Update replaces — flat GPU: found near the NEW location, absent near the
/// OLD; the external id resolves to the new vector.
#[test]
fn update_replaces_flat() {
    let Some(gpu) = gpu_or_skip("update_replaces_flat") else {
        return;
    };
    let mut c = build_corpus();
    let target = "v137";
    let old_row = c.row_of(target).unwrap();
    let old_vec = c.row(old_row as usize).to_vec();
    // A location far outside the [-1, 1) clusters, so it is unambiguously nearest
    // to a query at the same spot and far from anything at the old location.
    let new_vec = vec![3.0f32; DIM];

    assert!(c.update(target, &new_vec, Payload::new()));
    let new_row = c.row_of(target).unwrap();
    assert_ne!(
        new_row, old_row,
        "update re-points the id to a fresh appended row"
    );
    assert_eq!(
        c.row(new_row as usize),
        new_vec.as_slice(),
        "id resolves to the new vector"
    );
    assert_eq!(c.len(), N, "update keeps the live count");
    assert_eq!(c.capacity(), N + 1, "old row retained, new row appended");

    let gpu_idx = GpuFlatIndex::new(&gpu, &c);
    let near_new = gpu_idx.search_knn(&new_vec, K);
    assert_eq!(
        near_new[0].external_id, target,
        "updated id found at the new location"
    );
    let near_old = gpu_idx.search_knn(&old_vec, K);
    assert!(
        near_old.iter().all(|nb| nb.external_id != target),
        "the stale (tombstoned) row must not be returned at the old location"
    );

    let live = live_only(&c);
    assert_same_ids_scores(
        &CpuFlatIndex::new(&live).search_knn(&new_vec, K),
        &near_new,
        "update flat",
    );
    eprintln!("  update flat: id re-points to the new vector; found near new, absent near old; GPU == live oracle");
}

/// (2) Update replaces — IVF (Flat, full probe): same, on the IVF path.
#[test]
fn update_replaces_ivf() {
    let Some(gpu) = gpu_or_skip("update_replaces_ivf") else {
        return;
    };
    let mut c = build_corpus();
    let target = "v137";
    let old_vec = c.row(c.row_of(target).unwrap() as usize).to_vec();
    let new_vec = vec![3.0f32; DIM];
    assert!(c.update(target, &new_vec, Payload::new()));

    let index = IvfPqIndex::train(&c, ivf_config()).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let nprobe = index.nlist();

    let near_new = scanner.search(&index, &new_vec, K, nprobe);
    assert_eq!(
        near_new[0].external_id, target,
        "updated id found at the new location"
    );
    let near_old = scanner.search(&index, &old_vec, K, nprobe);
    assert!(
        near_old.iter().all(|nb| nb.external_id != target),
        "the stale (tombstoned) row must not be returned at the old location"
    );

    let live = live_only(&c);
    assert_same_ids_scores(
        &CpuFlatIndex::new(&live).search_knn(&new_vec, K),
        &near_new,
        "update ivf",
    );
    eprintln!(
        "  update ivf (Flat, full probe): found near new, absent near old; GPU == live oracle"
    );
}

/// (3) Upsert — flat GPU: a new id adds, an existing id replaces (live count
/// unchanged), and the replacement is found at its new location.
#[test]
fn upsert_adds_then_replaces_flat() {
    let Some(gpu) = gpu_or_skip("upsert_adds_then_replaces_flat") else {
        return;
    };
    let mut c = build_corpus();
    let before = c.len();

    // Upsert a NEW id → adds.
    let replaced = c.upsert("vNEW", &[2.0f32; DIM], Payload::new()).unwrap();
    assert!(
        !replaced,
        "upsert of a new id reports 'added' (not replaced)"
    );
    assert_eq!(c.len(), before + 1);
    assert!(c.contains("vNEW"));

    // Upsert the SAME id → replaces, live count unchanged.
    let replaced = c.upsert("vNEW", &[-2.0f32; DIM], Payload::new()).unwrap();
    assert!(replaced, "upsert of an existing id reports 'replaced'");
    assert_eq!(c.len(), before + 1, "replace keeps the live count");
    assert_eq!(
        c.row(c.row_of("vNEW").unwrap() as usize),
        vec![-2.0f32; DIM].as_slice()
    );

    let gpu_idx = GpuFlatIndex::new(&gpu, &c);
    let near_new = gpu_idx.search_knn(&[-2.0f32; DIM], K);
    assert_eq!(
        near_new[0].external_id, "vNEW",
        "replacement found at its new location"
    );
    let near_old = gpu_idx.search_knn(&[2.0f32; DIM], K);
    assert!(
        near_old.iter().all(|nb| nb.external_id != "vNEW"),
        "the superseded upsert location must not return the id"
    );

    let live = live_only(&c);
    assert_same_ids_scores(
        &CpuFlatIndex::new(&live).search_knn(&[-2.0f32; DIM], K),
        &near_new,
        "upsert flat",
    );
    eprintln!("  upsert flat: new id added, existing id replaced (live count stable), found at new location");
}

/// (4) `len()` tracks the live count across a mix of add / delete / update /
/// upsert; capacity, tombstone, and compaction accounting are correct. No GPU.
#[test]
fn len_tracks_live_across_mixed_crud() {
    let mut c = build_corpus();
    assert_eq!(c.len(), N);
    assert_eq!(c.capacity(), N);
    assert_eq!(c.tombstoned(), 0);

    // Delete 5.
    for i in 0..5 {
        assert!(c.delete(&format!("v{i}")));
    }
    assert_eq!(c.len(), N - 5);
    assert_eq!(c.tombstoned(), 5);
    assert!(!c.delete("v0"), "re-deleting is a no-op");
    assert_eq!(c.len(), N - 5);

    // Update 3 live ids → live count unchanged, capacity grows by 3.
    for i in 10..13 {
        assert!(c.update(&format!("v{i}"), &[0.25f32; DIM], Payload::new()));
    }
    assert_eq!(c.len(), N - 5, "update leaves the live count unchanged");
    assert_eq!(c.capacity(), N + 3);
    assert!(
        !c.update("nope", &[0.25f32; DIM], Payload::new()),
        "update of an unknown id fails"
    );

    // Upsert 2 new ids → +2.
    assert!(!c.upsert("a", &[0.1f32; DIM], Payload::new()).unwrap());
    assert!(!c.upsert("b", &[0.1f32; DIM], Payload::new()).unwrap());
    assert_eq!(c.len(), N - 3);
    // Upsert an existing id → live count unchanged.
    assert!(c.upsert("a", &[0.2f32; DIM], Payload::new()).unwrap());
    assert_eq!(c.len(), N - 3);

    // Compaction reclaims tombstones without changing the live set.
    let live = c.len();
    assert!(c.tombstoned() > 0);
    c.compact();
    assert_eq!(c.len(), live, "live rows survive compaction");
    assert_eq!(c.capacity(), live, "tombstones reclaimed");
    assert_eq!(c.tombstoned(), 0);
    assert!(c.contains("a") && c.contains("b"));
    for i in 0..5 {
        assert!(
            !c.contains(&format!("v{i}")),
            "deleted ids stay gone after compaction"
        );
    }
    eprintln!("  counts: len() tracked the live set across delete/update/upsert; compaction reclaimed tombstones");
}
