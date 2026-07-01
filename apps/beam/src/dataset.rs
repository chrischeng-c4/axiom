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

/// A deterministic **low-rank (low-intrinsic-dimension)** generative model — the
/// realistic-embedding stand-in that makes product quantization actually pay off.
///
/// Real embeddings are nominally `dim`-dimensional but live on a much
/// lower-dimensional manifold: their coordinates are strongly correlated. Pure
/// isotropic Gaussian noise (`random_collection`) is the pathological *worst*
/// case for PQ — every subspace is independent, so 256 centroids can't summarize
/// it and recall collapses. This model instead draws each point as
///
/// ```text
///   x = B · c  +  ε
/// ```
///
/// where `B` is a fixed random `dim × rank` basis (`rank << dim`), `c` is a
/// **clustered** coefficient vector in the small `rank`-dim space, and `ε` is a
/// small full-`dim` Gaussian noise. So every point sits near a `rank`-dim
/// subspace (low intrinsic dim) *and* clumps into clusters — exactly the
/// structure PQ exploits and IVF prunes. Fully LCG-seeded: `(seed, dim, rank,
/// num_clusters)` reproduces the basis and the coefficient clusters byte-for-byte.
#[derive(Debug, Clone)]
pub struct LowRankModel {
    /// `dim × rank` basis, row-major: `basis[d * rank + r]`. Columns are unit-norm
    /// so the projected signal scale is stable across `rank`.
    basis: Vec<f32>,
    /// Cluster centers for the low-dim coefficients, `num_clusters * rank`.
    coef_centers: Vec<f32>,
    rank: usize,
    num_clusters: usize,
    /// Gaussian jitter added to a coefficient cluster center (in `rank`-space).
    coef_jitter: f32,
    /// Std-dev of the small full-`dim` off-manifold noise `ε`.
    noise: f32,
}

impl LowRankModel {
    /// Build the model deterministically from `seed`: a unit-column random
    /// `dim × rank` basis and `num_clusters` coefficient centers in `rank`-space.
    /// `rank` is clamped to `1..=dim`.
    pub fn new(
        dim: usize,
        rank: usize,
        num_clusters: usize,
        coef_jitter: f32,
        noise: f32,
        seed: u64,
    ) -> Self {
        assert!(num_clusters > 0, "num_clusters must be > 0");
        let rank = rank.clamp(1, dim.max(1));

        // Basis: fill each of the `rank` columns with Gaussian entries, then
        // normalize the column to unit L2 so `x = B·c` keeps a stable scale.
        let mut rng = Lcg::new(seed ^ 0x10AD_4A31_B0A5_15C7);
        let mut basis = vec![0.0f32; dim * rank];
        for r in 0..rank {
            let mut norm_sq = 0.0f32;
            for d in 0..dim {
                let v = rng.next_gaussian();
                basis[d * rank + r] = v;
                norm_sq += v * v;
            }
            let inv = if norm_sq > 0.0 { 1.0 / norm_sq.sqrt() } else { 0.0 };
            for d in 0..dim {
                basis[d * rank + r] *= inv;
            }
        }

        // Coefficient cluster centers in `rank`-space, uniform in [-1, 1).
        let mut crng = Lcg::new(seed ^ 0xC0EF_CE27_E12A_B00C);
        let mut coef_centers = vec![0.0f32; num_clusters * rank];
        for c in coef_centers.iter_mut() {
            *c = crng.next_signed();
        }

        Self {
            basis,
            coef_centers,
            rank,
            num_clusters,
            coef_jitter,
            noise,
        }
    }

    /// Draw one `dim`-vector into `out`: pick a coefficient cluster, jitter it in
    /// `rank`-space, project through the basis, and add small full-`dim` noise.
    /// `coef` is a caller-owned scratch of length `rank` (avoids a per-point
    /// allocation across a million draws). `rng` is advanced for reproducibility.
    pub fn draw(&self, rng: &mut Lcg, coef: &mut [f32], out: &mut [f32]) {
        let cl = ((rng.next_unit() * self.num_clusters as f32) as usize)
            .min(self.num_clusters - 1);
        let cbase = cl * self.rank;
        for (r, cr) in coef.iter_mut().enumerate() {
            *cr = self.coef_centers[cbase + r] + self.coef_jitter * rng.next_gaussian();
        }
        for (d, o) in out.iter_mut().enumerate() {
            let brow = &self.basis[d * self.rank..(d + 1) * self.rank];
            let mut acc = 0.0f32;
            for r in 0..self.rank {
                acc += brow[r] * coef[r];
            }
            *o = acc + self.noise * rng.next_gaussian();
        }
    }
}

/// Default off-manifold noise for the low-rank generators (small vs the on-
/// manifold signal, so intrinsic dim stays ≈ `rank`).
pub const LOW_RANK_NOISE: f32 = 0.02;

/// Build a deterministic **low-rank** collection: `n` points drawn from a
/// [`LowRankModel`] (`rank`-dim clustered manifold + small noise) under `metric`.
/// This is the embedding-like corpus where IVF-PQ recall is meaningful — unlike
/// the isotropic [`random_collection`], PQ's per-subspace codebooks capture the
/// correlated structure. The basis + coefficient clusters are seeded by `seed`;
/// the point stream is a separate LCG so [`low_rank_queries`] with the same
/// `seed` shares the manifold but draws distinct points.
pub fn low_rank_collection(
    id: impl Into<String>,
    n: usize,
    dim: usize,
    metric: Metric,
    rank: usize,
    num_clusters: usize,
    coef_jitter: f32,
    seed: u64,
) -> Collection {
    let model = LowRankModel::new(dim, rank, num_clusters, coef_jitter, LOW_RANK_NOISE, seed);
    let mut rng = Lcg::new(seed ^ 0xA5A5_10AD_2222_1111);
    let mut collection = Collection::new(id, dim, metric);
    let mut coef = vec![0.0f32; rank.clamp(1, dim.max(1))];
    let mut v = vec![0.0f32; dim];
    for i in 0..n {
        model.draw(&mut rng, &mut coef, &mut v);
        collection
            .add(format!("id-{i}"), &v)
            .expect("fixed-dim vector always matches collection dim");
    }
    collection
}

/// Build `count` deterministic low-rank query vectors that share the corpus's
/// basis + coefficient clusters (same `seed`) but come from an independent point
/// stream — representative near-neighbor queries on the same manifold.
pub fn low_rank_queries(
    count: usize,
    dim: usize,
    rank: usize,
    num_clusters: usize,
    coef_jitter: f32,
    seed: u64,
) -> Vec<Vec<f32>> {
    let model = LowRankModel::new(dim, rank, num_clusters, coef_jitter, LOW_RANK_NOISE, seed);
    let mut rng = Lcg::new(seed ^ 0x3333_10AD_4444_5555);
    let mut coef = vec![0.0f32; rank.clamp(1, dim.max(1))];
    let mut v = vec![0.0f32; dim];
    (0..count)
        .map(|_| {
            model.draw(&mut rng, &mut coef, &mut v);
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

    #[test]
    fn low_rank_collection_is_reproducible_and_low_dimensional() {
        let (dim, rank) = (48usize, 6usize);
        let a = low_rank_collection("lr", 300, dim, Metric::L2, rank, 5, 0.05, 31);
        let b = low_rank_collection("lr", 300, dim, Metric::L2, rank, 5, 0.05, 31);
        assert_eq!(a.data(), b.data(), "same seed → identical low-rank corpus");
        assert_eq!(a.len(), 300);

        // The corpus should be (near-)confined to a `rank`-dim subspace. Project
        // every centered point onto the top-`rank` directions of the empirical
        // basis and confirm the residual off-subspace energy is tiny — evidence
        // of genuinely low intrinsic dimension. We test the weaker, cheap proxy:
        // the mean coordinate variance is dominated by a handful of directions.
        // Concretely, reconstruct the model's own basis and check each point's
        // off-manifold (noise) energy is small vs its on-manifold energy.
        let model = LowRankModel::new(dim, rank, 5, 0.05, LOW_RANK_NOISE, 31);
        // Gram-Schmidt the basis columns to an orthonormal set Q (dim x rank).
        let mut q = vec![0.0f32; dim * rank];
        for r in 0..rank {
            let mut col: Vec<f32> = (0..dim).map(|d| model.basis[d * rank + r]).collect();
            for p in 0..r {
                let dot: f32 = (0..dim).map(|d| col[d] * q[d * rank + p]).sum();
                for d in 0..dim {
                    col[d] -= dot * q[d * rank + p];
                }
            }
            let norm: f32 = col.iter().map(|x| x * x).sum::<f32>().sqrt();
            let inv = if norm > 1e-6 { 1.0 / norm } else { 0.0 };
            for d in 0..dim {
                q[d * rank + r] = col[d] * inv;
            }
        }
        let mut on = 0.0f64;
        let mut total = 0.0f64;
        for i in 0..a.len() {
            let row = a.row(i);
            let energy: f32 = row.iter().map(|x| x * x).sum();
            // Energy captured by the rank-dim orthonormal subspace.
            let mut proj = 0.0f32;
            for r in 0..rank {
                let c: f32 = (0..dim).map(|d| row[d] * q[d * rank + r]).sum();
                proj += c * c;
            }
            on += proj as f64;
            total += energy as f64;
        }
        let captured = on / total;
        assert!(
            captured > 0.9,
            "rank-{rank} subspace should capture >90% of energy (low intrinsic dim), got {captured:.3}"
        );
    }
}
