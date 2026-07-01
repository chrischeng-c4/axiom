//! Deterministic synthetic datasets for the bench + GPU/CPU parity tests.
//!
//! Everything here is seeded by a plain LCG — no wall-clock, no thread RNG — so
//! a given `(seed, n, dim)` always produces byte-identical vectors. That is what
//! makes the bench's recall claim and the parity test reproducible.

use crate::collection::{Collection, Metric};

/// A 64-bit linear congruential generator (constants from Knuth's MMIX). Pure,
/// deterministic, and dependency-free — the only randomness source Beam uses.
#[derive(Debug, Clone)]
pub struct Lcg {
    state: u64,
}

impl Lcg {
    /// Seed the generator. The same seed always yields the same stream.
    pub fn new(seed: u64) -> Self {
        // Avoid the fixed point at 0 for the multiply-only warmup.
        Self {
            state: seed ^ 0x9E37_79B9_7F4A_7C15,
        }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        // Use the high bits (better distributed than the low bits of an LCG).
        (self.state >> 32) as u32
    }

    /// Next f32 in `[0, 1)`.
    pub fn next_unit(&mut self) -> f32 {
        // 24-bit mantissa's worth of precision, mapped into [0, 1).
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Next f32 in `[-1, 1)`.
    pub fn next_signed(&mut self) -> f32 {
        self.next_unit() * 2.0 - 1.0
    }

    /// Next standard-normal f32 (mean 0, variance 1) via the Box-Muller
    /// transform over two of this generator's uniforms. Deterministic like the
    /// rest of the LCG; used to jitter clustered corpus points so IVF recall is
    /// measured on realistic (clumped) data rather than pure uniform noise.
    pub fn next_gaussian(&mut self) -> f32 {
        // Clamp u1 away from 0 so ln() is finite; u2 selects the angle. We keep
        // only the cosine leg (one gaussian per call) for a simple, stable stream.
        let u1 = self.next_unit().max(1.0e-7);
        let u2 = self.next_unit();
        (-2.0 * u1.ln()).sqrt() * (std::f32::consts::TAU * u2).cos()
    }
}

/// A deterministic mixture-of-clusters model: `num_clusters` random centers in
/// `[-1, 1)^dim`. Both the clustered corpus and the clustered query set draw
/// from the SAME centers (given the same `seed`/`num_clusters`) but from
/// independent point streams, so queries land near — but are never identical to
/// — corpus points. This clumped distribution is what makes IVF pruning (and its
/// recall) meaningful, unlike uniform noise where every cell looks alike.
#[derive(Debug, Clone)]
pub struct ClusterModel {
    centers: Vec<f32>,
    dim: usize,
    num_clusters: usize,
}

impl ClusterModel {
    /// Generate the cluster centers deterministically from `seed`. Center
    /// components are uniform in `[-1, 1)`.
    pub fn new(dim: usize, num_clusters: usize, seed: u64) -> Self {
        assert!(num_clusters > 0, "num_clusters must be > 0");
        let mut rng = Lcg::new(seed);
        let mut centers = vec![0.0f32; num_clusters * dim];
        for c in centers.iter_mut() {
            *c = rng.next_signed();
        }
        Self {
            centers,
            dim,
            num_clusters,
        }
    }

    /// Draw one point: pick a uniform cluster, then add Gaussian jitter (scaled
    /// by `jitter`) around its center. `rng` is advanced, so successive calls
    /// yield distinct points.
    pub fn draw(&self, rng: &mut Lcg, jitter: f32, out: &mut [f32]) {
        let cl = ((rng.next_unit() * self.num_clusters as f32) as usize).min(self.num_clusters - 1);
        let base = cl * self.dim;
        for (j, x) in out.iter_mut().enumerate() {
            *x = self.centers[base + j] + jitter * rng.next_gaussian();
        }
    }
}

/// Build a deterministic **clustered** collection: `n` points drawn from a
/// `num_clusters`-mode mixture (each point = a random center + `jitter` Gaussian
/// noise) under `metric`. External ids are `id-0 .. id-(n-1)`. The center stream
/// is seeded by `seed`; the point stream is a separate LCG so
/// [`clustered_queries`] with the same `seed` shares centers but not points.
pub fn clustered_collection(
    id: impl Into<String>,
    n: usize,
    dim: usize,
    metric: Metric,
    num_clusters: usize,
    jitter: f32,
    seed: u64,
) -> Collection {
    let model = ClusterModel::new(dim, num_clusters, seed);
    let mut rng = Lcg::new(seed ^ 0xC0FF_EE00_D15E_A5E5);
    let mut collection = Collection::new(id, dim, metric);
    let mut v = vec![0.0f32; dim];
    for i in 0..n {
        model.draw(&mut rng, jitter, &mut v);
        collection
            .add(format!("id-{i}"), &v)
            .expect("fixed-dim vector always matches collection dim");
    }
    collection
}

/// Build `count` deterministic clustered query vectors that share the corpus's
/// cluster centers (same `seed`/`num_clusters`) but come from an independent
/// point stream — representative near-neighbor queries, never exact corpus rows.
pub fn clustered_queries(
    count: usize,
    dim: usize,
    num_clusters: usize,
    jitter: f32,
    seed: u64,
) -> Vec<Vec<f32>> {
    let model = ClusterModel::new(dim, num_clusters, seed);
    let mut rng = Lcg::new(seed ^ 0x9111_D0E5_A11C_E123);
    let mut v = vec![0.0f32; dim];
    (0..count)
        .map(|_| {
            model.draw(&mut rng, jitter, &mut v);
            v.clone()
        })
        .collect()
}

/// Build a deterministic collection of `n` random `dim`-vectors (components in
/// `[-1, 1)`) under `metric`. External ids are `id-0 .. id-(n-1)`.
pub fn random_collection(
    id: impl Into<String>,
    n: usize,
    dim: usize,
    metric: Metric,
    seed: u64,
) -> Collection {
    let mut rng = Lcg::new(seed);
    let mut collection = Collection::new(id, dim, metric);
    let mut v = vec![0.0f32; dim];
    for i in 0..n {
        for x in v.iter_mut() {
            *x = rng.next_signed();
        }
        collection
            .add(format!("id-{i}"), &v)
            .expect("fixed-dim vector always matches collection dim");
    }
    collection
}

/// Build `count` deterministic random query vectors of length `dim` (components
/// in `[-1, 1)`).
pub fn random_queries(count: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = Lcg::new(seed);
    (0..count)
        .map(|_| (0..dim).map(|_| rng.next_signed()).collect())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcg_is_deterministic() {
        let a: Vec<f32> = {
            let mut r = Lcg::new(42);
            (0..8).map(|_| r.next_signed()).collect()
        };
        let b: Vec<f32> = {
            let mut r = Lcg::new(42);
            (0..8).map(|_| r.next_signed()).collect()
        };
        assert_eq!(a, b);
        assert!(a.iter().all(|x| (-1.0..1.0).contains(x)));
        // A different seed diverges.
        let c: Vec<f32> = {
            let mut r = Lcg::new(43);
            (0..8).map(|_| r.next_signed()).collect()
        };
        assert_ne!(a, c);
    }

    #[test]
    fn collection_is_reproducible() {
        let c1 = random_collection("x", 10, 4, Metric::L2, 7);
        let c2 = random_collection("x", 10, 4, Metric::L2, 7);
        assert_eq!(c1.data(), c2.data());
        assert_eq!(c1.len(), 10);
    }

    #[test]
    fn clustered_collection_is_reproducible_and_clumped() {
        let a = clustered_collection("c", 200, 8, Metric::L2, 5, 0.02, 11);
        let b = clustered_collection("c", 200, 8, Metric::L2, 5, 0.02, 11);
        assert_eq!(a.data(), b.data(), "same seed → identical corpus");
        assert_eq!(a.len(), 200);

        // Tight jitter ⇒ each point sits within a small radius of one of the 5
        // centers, so its nearest-center distance is far smaller than a random
        // uniform point's would be — evidence the data is genuinely clustered.
        let model = ClusterModel::new(8, 5, 11);
        for i in 0..a.len() {
            let row = a.row(i);
            let nearest = (0..5)
                .map(|c| {
                    (0..8)
                        .map(|j| {
                            let d = row[j] - model.centers[c * 8 + j];
                            d * d
                        })
                        .sum::<f32>()
                })
                .fold(f32::INFINITY, f32::min);
            assert!(nearest < 0.25, "point {i} should hug a center, got {nearest}");
        }
    }

    #[test]
    fn clustered_queries_share_centers_but_differ_from_corpus() {
        let corpus = clustered_collection("c", 50, 8, Metric::L2, 6, 0.02, 21);
        let queries = clustered_queries(50, 8, 6, 0.02, 21);
        // Same seed ⇒ same centers, but independent point streams ⇒ query 0 is
        // NOT corpus row 0 (would be a trivial exact-match otherwise).
        assert_ne!(queries[0].as_slice(), corpus.row(0));
    }
}
