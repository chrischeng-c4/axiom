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
}
