// <HANDWRITE gap="standardize:claim-code" tracker="projects-lumen-tests-hnsw-ef-recall-rs" reason="Existing code claimed during Score standardization until deterministic generator coverage lands.">
//! CPU HNSW recall@10 + latency sweep vs brute-force ground truth.
//!
//! Validates the kNN bottleneck fix: lowering search-`ef` from 512 and (after
//! the DistDot swap) the cheaper kernel must hold recall@10 at parity while
//! cutting latency. Recall is measured against an EXACT brute-force top-K, so
//! the result is an absolute recall number, not relative to a baseline.
//!
//! ```sh
//! cargo test --release -p lumen --test hnsw_ef_recall -- --ignored --nocapture
//! # tune corpus size:  LUMEN_BENCH_N=100000 cargo test --release ...
//! ```
//!
//! Corpus is intentionally NON-normalized + clustered (the realistic + adverse
//! case): it exercises the engine's internal normalization on the cosine path
//! and the near-duplicate vectors that stress the kernel's float edge cases.

use std::collections::HashSet;
use std::time::Instant;

use lumen::types::{VectorBackend, VectorMetric, VectorSpec};
use lumen::vector_index::{HnswCpuIndex, VectorIndex};

const DIM: usize = 128;
const K: usize = 10;
const CLUSTERS: usize = 200;
const QUERIES: usize = 200;

struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(6_364_136_223_846_793_005) ^ 0x9E37_79B9_7F4A_7C15)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.0
    }
    fn unit(&mut self) -> f32 {
        ((self.next_u64() >> 32) as f64 / u32::MAX as f64 * 2.0 - 1.0) as f32
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let mut d = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        d += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    if na == 0.0 || nb == 0.0 {
        0.0
    } else {
        d / (na.sqrt() * nb.sqrt())
    }
}

/// Exact top-K external ids by true cosine, for one query.
fn brute_topk(query: &[f32], corpus: &[Vec<f32>], k: usize) -> HashSet<usize> {
    let mut scored: Vec<(usize, f32)> = corpus
        .iter()
        .enumerate()
        .map(|(i, v)| (i, cosine(query, v)))
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    scored.into_iter().take(k).map(|(i, _)| i).collect()
}

fn percentile(sorted_us: &[u128], p: f64) -> f64 {
    if sorted_us.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_us.len() as f64 - 1.0) * p).round() as usize;
    sorted_us[idx] as f64 / 1000.0
}

#[test]
#[ignore = "bench/validation harness — run with --release --ignored --nocapture"]
fn hnsw_ef_recall_latency_sweep() {
    let n: usize = std::env::var("LUMEN_BENCH_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);

    let mut rng = Lcg::new(0xB07_71E_u64);
    // Cluster centroids.
    let centroids: Vec<Vec<f32>> = (0..CLUSTERS)
        .map(|_| (0..DIM).map(|_| rng.unit()).collect())
        .collect();
    // Corpus = centroid + noise (NON-normalized, clustered → near-duplicates).
    let corpus: Vec<Vec<f32>> = (0..n)
        .map(|i| {
            let c = &centroids[i % CLUSTERS];
            c.iter().map(|&x| x + 0.15 * rng.unit()).collect()
        })
        .collect();

    println!("\nbuilding HNSW index: N={n} dim={DIM} clusters={CLUSTERS} ...");
    let t_build = Instant::now();
    let spec = VectorSpec {
        dim: DIM as u32,
        metric: VectorMetric::Cosine,
        backend: VectorBackend::HnswCpu,
        quantize: None,
    };
    let idx = HnswCpuIndex::new(spec);
    for (i, v) in corpus.iter().enumerate() {
        idx.add(&format!("d{i}"), v).unwrap();
    }
    println!("  build took {:.1}s", t_build.elapsed().as_secs_f64());

    // Queries near random centroids (have a real nearest-neighbour set).
    let queries: Vec<Vec<f32>> = (0..QUERIES)
        .map(|q| {
            let c = &centroids[q % CLUSTERS];
            c.iter().map(|&x| x + 0.10 * rng.unit()).collect()
        })
        .collect();

    // Ground truth (exact) per query.
    let truth: Vec<HashSet<usize>> = queries.iter().map(|q| brute_topk(q, &corpus, K)).collect();

    println!("\n=== ef sweep (N={n}, dim={DIM}, k={K}, {QUERIES} queries) ===");
    println!("  ef   recall@10    mean(ms)   p50(ms)   p99(ms)");
    for &ef in &[64usize, 96, 128, 192, 256, 512] {
        idx.set_ef_search(ef);
        // warm
        for q in queries.iter().take(20) {
            let _ = idx.search_knn(q, K).unwrap();
        }
        let mut lat_us: Vec<u128> = Vec::with_capacity(QUERIES);
        let mut hit = 0usize;
        for (qi, q) in queries.iter().enumerate() {
            let t = Instant::now();
            let res = idx.search_knn(q, K).unwrap();
            lat_us.push(t.elapsed().as_micros());
            let got: HashSet<usize> = res
                .iter()
                .filter_map(|(eid, _)| eid.trim_start_matches('d').parse::<usize>().ok())
                .collect();
            hit += truth[qi].intersection(&got).count();
        }
        lat_us.sort_unstable();
        let recall = hit as f64 / (QUERIES * K) as f64;
        let mean = lat_us.iter().sum::<u128>() as f64 / QUERIES as f64 / 1000.0;
        println!(
            "  {ef:<4} {recall:>8.4}    {mean:>8.3}  {:>8.3}  {:>8.3}",
            percentile(&lat_us, 0.50),
            percentile(&lat_us, 0.99),
        );
    }
    println!();
}

// </HANDWRITE>
