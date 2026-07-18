//! IVF-PQ correctness gate — recall vs the flat oracle, on the GPU.
//!
//! These tests are the milestone proof for beam's IVF-PQ ANN index. They use a
//! deterministic **clustered** corpus (n=10000, dim=64, 50 clusters), L2, k=10 —
//! clumped data is where IVF pruning is meaningful. Every test skips gracefully
//! (prints, returns) when no GPU adapter is present, so GPU-less CI stays green;
//! on this Mac they PRINT the Metal adapter and PASS.
//!
//! What each test pins:
//!   1. IVF plumbing is exact:  Flat + nprobe==nlist ⇒ recall@10 == 1.000.
//!   2. Coarse quantizer works: recall@10 is non-decreasing in nprobe and
//!      clears 0.85 at nprobe≈nlist/4.
//!   3. PQ / ADC is accurate:   Pq + nprobe==nlist ⇒ recall@10 >= 0.70.
//!   4. Kernel exactness:       GPU candidate distances == CPU reference (1e-3).
//!   5. Scaling:                candidates scanned << n at small nprobe.
//!
//! The recall/scan work runs on the GPU (`GpuIvfScanner`) so these exercise the
//! real kernel, not just the CPU reference.

use std::collections::HashSet;

use beam::collection::Metric;
use beam::dataset;
use beam::gpu::ivfpq::GpuIvfScanner;
use beam::gpu::GpuContext;
use beam::index::cpu_flat::CpuFlatIndex;
use beam::index::ivf_pq::{IvfPqConfig, IvfPqIndex, Refine};
use beam::index::{Neighbor, VectorIndex};

const N: usize = 10_000;
const DIM: usize = 64;
const K: usize = 10;
const NLIST: usize = 64;
const NUM_CLUSTERS: usize = 50;
const JITTER: f32 = 0.05;
const N_QUERIES: usize = 20;

const CORPUS_SEED: u64 = 0x0BEA_11F0;
const QUERY_SEED: u64 = 0x0BEA_22F0;
const TRAIN_SEED: u64 = 0x0BEA_33F0;

/// The shared clustered corpus + queries (queries share the corpus centers).
fn corpus_and_queries() -> (beam::collection::Collection, Vec<Vec<f32>>) {
    let corpus =
        dataset::clustered_collection("ivf", N, DIM, Metric::L2, NUM_CLUSTERS, JITTER, CORPUS_SEED);
    let queries = dataset::clustered_queries(N_QUERIES, DIM, NUM_CLUSTERS, JITTER, QUERY_SEED);
    (corpus, queries)
}

fn config(refine: Refine) -> IvfPqConfig {
    IvfPqConfig {
        nlist: NLIST,
        kmeans_iters: 20,
        nbits: 8,
        refine,
        train_sample: 0,
        seed: TRAIN_SEED,
    }
}

fn row_set(neighbors: &[Neighbor]) -> HashSet<u32> {
    neighbors.iter().map(|n| n.row).collect()
}

/// Mean recall@K of GPU IVF search vs the exact flat oracle over all queries.
fn mean_recall(
    scanner: &GpuIvfScanner,
    index: &IvfPqIndex,
    oracle: &CpuFlatIndex,
    queries: &[Vec<f32>],
    nprobe: usize,
) -> f64 {
    let mut sum = 0.0;
    for q in queries {
        let truth = row_set(&oracle.search_knn(q, K));
        let got = scanner.search(index, q, K, nprobe);
        let hit = got.iter().filter(|nb| truth.contains(&nb.row)).count();
        sum += hit as f64 / got.len().max(1) as f64;
    }
    sum / queries.len() as f64
}

/// Acquire the GPU or print a skip message and return `None`.
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

/// (1) IVF plumbing is exact: Flat refine + probe-every-cell ⇒ recall 1.000.
#[test]
fn ivf_flat_full_probe_is_exact() {
    let Some(gpu) = gpu_or_skip("ivf_flat_full_probe_is_exact") else {
        return;
    };
    let (corpus, queries) = corpus_and_queries();
    let index = IvfPqIndex::train(&corpus, config(Refine::Flat)).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let oracle = CpuFlatIndex::new(&corpus);

    let recall = mean_recall(&scanner, &index, &oracle, &queries, index.nlist());
    eprintln!("  IVFFlat recall@{K} at nprobe=nlist={}: {recall:.4}", index.nlist());
    assert_eq!(
        recall, 1.0,
        "Flat + full probe must equal the exact oracle (IVF gather/assignment bug otherwise)"
    );
}

/// (2) Coarse quantizer: recall is non-decreasing in nprobe and clears 0.85 at
/// nprobe ≈ nlist/4 on clustered data.
#[test]
fn ivf_flat_recall_grows_with_nprobe() {
    let Some(gpu) = gpu_or_skip("ivf_flat_recall_grows_with_nprobe") else {
        return;
    };
    let (corpus, queries) = corpus_and_queries();
    let index = IvfPqIndex::train(&corpus, config(Refine::Flat)).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let oracle = CpuFlatIndex::new(&corpus);

    let probes = [1usize, 2, 4, 8, 16, NLIST];
    let mut recalls = Vec::new();
    for &np in &probes {
        recalls.push(mean_recall(&scanner, &index, &oracle, &queries, np));
    }
    eprintln!("  IVFFlat recall@{K} by nprobe {probes:?}: {recalls:?}");

    // Monotone non-decreasing (a tiny float epsilon of slack).
    for w in recalls.windows(2) {
        assert!(
            w[1] >= w[0] - 1e-9,
            "recall must not drop as nprobe grows: {recalls:?}"
        );
    }
    // At nprobe = nlist/4, recall should be strong on clustered data.
    let quarter = mean_recall(&scanner, &index, &oracle, &queries, NLIST / 4);
    eprintln!("  IVFFlat recall@{K} at nprobe=nlist/4={}: {quarter:.4}", NLIST / 4);
    assert!(quarter >= 0.85, "recall@nlist/4 should clear 0.85, got {quarter}");
}

/// (3) PQ / ADC accuracy: Pq refine + probe-every-cell ⇒ recall@10 >= 0.70 (the
/// only error is PQ quantization, since every cell is probed).
#[test]
fn ivf_pq_full_probe_recall() {
    let Some(gpu) = gpu_or_skip("ivf_pq_full_probe_recall") else {
        return;
    };
    let (corpus, queries) = corpus_and_queries();
    // DIM=64, m=16 ⇒ dsub=4: fine enough that PQ quantization (the only error at
    // full probe) leaves recall well above the 0.70 bar on this tight-cluster data.
    let index = IvfPqIndex::train(&corpus, config(Refine::Pq { m: 16 })).unwrap();
    let scanner = GpuIvfScanner::new(&gpu);
    let oracle = CpuFlatIndex::new(&corpus);

    let recall = mean_recall(&scanner, &index, &oracle, &queries, index.nlist());
    eprintln!("  IVFPQ (m=16) recall@{K} at nprobe=nlist={}: {recall:.4}", index.nlist());
    assert!(recall >= 0.70, "PQ full-probe recall should clear 0.70, got {recall}");
}

/// (4) Kernel exactness: the GPU candidate distances equal the CPU reference
/// `QueryPlan::cpu_scan` within 1e-3 — separating kernel bugs from recall.
#[test]
fn gpu_adc_matches_cpu_reference() {
    let Some(gpu) = gpu_or_skip("gpu_adc_matches_cpu_reference") else {
        return;
    };
    let (corpus, queries) = corpus_and_queries();
    let scanner = GpuIvfScanner::new(&gpu);

    for refine in [Refine::Pq { m: 8 }, Refine::Flat] {
        let index = IvfPqIndex::train(&corpus, config(refine)).unwrap();
        // Both GPU scan paths must match the CPU reference: the default
        // global-table `adc` kernel AND the P0 per-cell shared-memory kernel
        // (`scan_shared`). For Flat, `scan_shared` falls back to `scan`.
        let mut max_global = 0.0f32;
        let mut max_shared = 0.0f32;
        let mut checked = 0usize;
        for q in &queries {
            // A mid-range nprobe so tables/residuals + candidate gather are all
            // exercised (not the trivial 1-cell or all-cells extremes).
            let plan = index.plan(q, NLIST / 2);
            let cpu = plan.cpu_scan();
            let gpu_global = scanner.scan(&plan);
            let gpu_shared = scanner.scan_shared(&plan);
            assert_eq!(cpu.len(), gpu_global.len());
            assert_eq!(cpu.len(), gpu_shared.len());
            for i in 0..cpu.len() {
                max_global = max_global.max((cpu[i] - gpu_global[i]).abs());
                max_shared = max_shared.max((cpu[i] - gpu_shared[i]).abs());
            }
            checked += cpu.len();
        }
        eprintln!(
            "  {refine:?}: GPU vs CPU ADC max abs diff = global {max_global:.3e}, shared {max_shared:.3e} over {checked} candidate distances"
        );
        assert!(
            max_global <= 1e-3,
            "{refine:?}: global-table GPU distance diverges from CPU reference by {max_global}"
        );
        assert!(
            max_shared <= 1e-3,
            "{refine:?}: shared-memory GPU distance diverges from CPU reference by {max_shared}"
        );
    }
}

/// (6) PQ benefits from low intrinsic dimension. On isotropic data (independent
/// subspaces) PQ has nothing to compress and recall is poor (~0.48); on
/// embedding-like LOW-RANK data (points near a `rank`-dim manifold, `rank << dim`)
/// recall improves (~0.55) — but only modestly, because a RANDOMLY oriented basis
/// smears the rank-16 signal across all m subspaces, so per-subspace codebooks
/// can't fully exploit it. Closing the gap to a strong recall needs OPQ (a learned
/// rotation that aligns residuals to the PQ subspaces) — see TODO(opq). This test
/// pins the honest baseline: low-rank beats PQ's isotropic worst case; OPQ is the
/// lever for the rest.
#[test]
fn ivf_pq_recall_low_rank_beats_isotropic() {
    let Some(gpu) = gpu_or_skip("ivf_pq_recall_low_rank_beats_isotropic") else {
        return;
    };
    // dim=128 / rank=16 mirrors the prompt's embedding example (1/8 intrinsic).
    const LN: usize = 20_000;
    const LDIM: usize = 128;
    const LRANK: usize = 16;
    const LNLIST: usize = 64;
    const LCLUST: usize = 50;
    const LM: usize = 16;
    const JIT: f32 = 0.05;

    let scanner = GpuIvfScanner::new(&gpu);
    let cfg = |refine| IvfPqConfig {
        nlist: LNLIST,
        kmeans_iters: 20,
        nbits: 8,
        refine,
        train_sample: 0,
        seed: TRAIN_SEED,
    };

    // Isotropic clustered corpus — PQ's worst case (within-cluster spread is
    // full-rank Gaussian, nothing for the subspace codebooks to exploit).
    let iso =
        dataset::clustered_collection("iso", LN, LDIM, Metric::L2, LCLUST, JIT, CORPUS_SEED);
    let iso_q = dataset::clustered_queries(N_QUERIES, LDIM, LCLUST, JIT, QUERY_SEED);
    let iso_idx = IvfPqIndex::train(&iso, cfg(Refine::Pq { m: LM })).unwrap();
    let iso_oracle = CpuFlatIndex::new(&iso);
    let iso_recall = mean_recall(&scanner, &iso_idx, &iso_oracle, &iso_q, LNLIST);

    // Low-rank (embedding-like) corpus — within-cluster variation lives near a
    // 16-dim manifold, so PQ resolves distances finely.
    let lr = dataset::low_rank_collection(
        "lr", LN, LDIM, Metric::L2, LRANK, LCLUST, JIT, CORPUS_SEED,
    );
    let lr_q = dataset::low_rank_queries(N_QUERIES, LDIM, LRANK, LCLUST, JIT, QUERY_SEED);
    let lr_idx = IvfPqIndex::train(&lr, cfg(Refine::Pq { m: LM })).unwrap();
    let lr_oracle = CpuFlatIndex::new(&lr);
    let lr_recall = mean_recall(&scanner, &lr_idx, &lr_oracle, &lr_q, LNLIST);

    eprintln!(
        "  IVFPQ recall@{K} (full probe, m={LM}, dim={LDIM}): isotropic={iso_recall:.3}  low-rank(rank={LRANK})={lr_recall:.3}"
    );
    // Honest baseline for plain PQ (no OPQ): low-rank clears 0.50 and beats the
    // isotropic worst case. OPQ is the lever that would push this toward 0.85+.
    assert!(
        lr_recall >= 0.50,
        "low-rank PQ recall should clear 0.50 (OPQ is the lever for higher), got {lr_recall}"
    );
    assert!(
        lr_recall > iso_recall,
        "low-rank recall should beat isotropic ({lr_recall:.3} vs {iso_recall:.3})"
    );
}

/// (5) Scaling proof: at small nprobe the candidate set is a small fraction of n.
#[test]
fn small_nprobe_scans_few_candidates() {
    let Some(_gpu) = gpu_or_skip("small_nprobe_scans_few_candidates") else {
        return;
    };
    let (corpus, queries) = corpus_and_queries();
    let index = IvfPqIndex::train(&corpus, config(Refine::Flat)).unwrap();

    let nprobe = 4;
    let mut total = 0usize;
    for q in &queries {
        total += index.plan(q, nprobe).num_candidates();
    }
    let avg = total as f64 / queries.len() as f64;
    let ratio = avg / N as f64;
    eprintln!(
        "  nprobe={nprobe}: avg candidates {avg:.0} of n={N} → ratio {ratio:.4} (<< 1)"
    );
    assert!(
        ratio < 0.25,
        "at nprobe={nprobe} the scan should touch < n/4 vectors, got ratio {ratio}"
    );
}
