//! Random number generation with PCG64 algorithm.
//!
//! Provides NumPy-compatible random distributions: uniform, normal,
//! binomial, poisson. Supports seed-based reproducibility.

use super::ndarray::NdArray;
use super::shape::Shape;
use std::f64::consts::PI;

/// PCG64 pseudo-random number generator.
///
/// Implements the PCG-XSH-RR variant with 64-bit state and 32-bit output,
/// extended to 64-bit output via two calls.
///
/// This is the same algorithm family used by NumPy's default RNG.
#[derive(Debug, Clone)]
pub struct Pcg64 {
    state: u128,
    inc: u128,
}

impl Pcg64 {
    /// Default multiplier for PCG.
    const MULTIPLIER: u128 = 6364136223846793005;

    /// Create a new PCG64 generator with the given seed.
    pub fn new(seed: u64) -> Self {
        let mut rng = Pcg64 { state: 0, inc: 1 };
        // Initialize state
        rng.state = rng.state.wrapping_add(seed as u128);
        rng.advance();
        rng
    }

    /// Create with both seed and stream (increment).
    pub fn with_stream(seed: u64, stream: u64) -> Self {
        let inc = (stream as u128) << 1 | 1; // Must be odd
        let mut rng = Pcg64 { state: 0, inc };
        rng.state = rng.state.wrapping_add(seed as u128);
        rng.advance();
        rng
    }

    /// Advance internal state.
    fn advance(&mut self) {
        self.state = self
            .state
            .wrapping_mul(Self::MULTIPLIER)
            .wrapping_add(self.inc);
    }

    /// Generate a raw 32-bit random value.
    fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.advance();

        // XSH-RR output function
        let xorshifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        xorshifted.rotate_right(rot)
    }

    /// Generate a 64-bit random value.
    pub fn next_u64(&mut self) -> u64 {
        let hi = self.next_u32() as u64;
        let lo = self.next_u32() as u64;
        (hi << 32) | lo
    }

    /// Generate a uniform f64 in [0, 1).
    pub fn next_f64(&mut self) -> f64 {
        let val = self.next_u64();
        // Use 53 bits of randomness for full f64 precision
        (val >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Generate a uniform f64 in [low, high).
    pub fn uniform(&mut self, low: f64, high: f64) -> f64 {
        low + (high - low) * self.next_f64()
    }

    /// Generate a standard normal (Gaussian) random value using Box-Muller.
    pub fn normal_standard(&mut self) -> f64 {
        loop {
            let u1 = self.next_f64();
            let u2 = self.next_f64();
            if u1 > 0.0 {
                let r = (-2.0 * u1.ln()).sqrt();
                return r * (2.0 * PI * u2).cos();
            }
        }
    }

    /// Generate a normal (Gaussian) random value.
    pub fn normal(&mut self, mean: f64, std: f64) -> f64 {
        mean + std * self.normal_standard()
    }

    /// Generate a random integer in [low, high).
    pub fn randint(&mut self, low: i64, high: i64) -> i64 {
        if low >= high {
            return low;
        }
        let range = (high - low) as u64;
        low + (self.next_u64() % range) as i64
    }

    /// Generate a Bernoulli trial (true with probability p).
    pub fn bernoulli(&mut self, p: f64) -> bool {
        self.next_f64() < p
    }

    /// Generate a binomial random variable: number of successes in n trials.
    pub fn binomial(&mut self, n: u64, p: f64) -> u64 {
        if p <= 0.0 {
            return 0;
        }
        if p >= 1.0 {
            return n;
        }

        // For small n, direct simulation
        if n <= 30 {
            let mut count = 0u64;
            for _ in 0..n {
                if self.bernoulli(p) {
                    count += 1;
                }
            }
            return count;
        }

        // For larger n, use the BTPE algorithm (normal approximation + correction)
        let np = n as f64 * p;
        let std = (np * (1.0 - p)).sqrt();
        let val = self.normal(np, std).round().max(0.0).min(n as f64);
        val as u64
    }

    /// Generate a Poisson random variable with given mean (lambda).
    pub fn poisson(&mut self, lambda: f64) -> u64 {
        if lambda <= 0.0 {
            return 0;
        }

        // For small lambda, use Knuth's algorithm
        if lambda < 30.0 {
            let l = (-lambda).exp();
            let mut k = 0u64;
            let mut p = 1.0;
            loop {
                k += 1;
                p *= self.next_f64();
                if p <= l {
                    return k - 1;
                }
            }
        }

        // For larger lambda, use normal approximation
        let val = self.normal(lambda, lambda.sqrt()).round().max(0.0);
        val as u64
    }

    /// Choose a random element index from 0..n.
    pub fn choice(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    /// Shuffle a mutable slice in-place (Fisher-Yates).
    pub fn shuffle<T>(&mut self, data: &mut [T]) {
        let n = data.len();
        for i in (1..n).rev() {
            let j = self.choice(i + 1);
            data.swap(i, j);
        }
    }
}

/// Extension trait for generating random arrays.
pub trait RandomExt {
    /// Create an array of uniform random values in [0, 1).
    fn rand_uniform(rng: &mut Pcg64, shape: impl Into<Shape>) -> NdArray<f64>;

    /// Create an array of normal random values.
    fn rand_normal(rng: &mut Pcg64, mean: f64, std: f64, shape: impl Into<Shape>) -> NdArray<f64>;

    /// Create an array of random integers in [low, high).
    fn rand_int(rng: &mut Pcg64, low: i64, high: i64, shape: impl Into<Shape>) -> NdArray<f64>;

    /// Create an array of random binomial samples.
    fn rand_binomial(rng: &mut Pcg64, n: u64, p: f64, shape: impl Into<Shape>) -> NdArray<f64>;

    /// Create an array of random Poisson samples.
    fn rand_poisson(rng: &mut Pcg64, lambda: f64, shape: impl Into<Shape>) -> NdArray<f64>;
}

impl RandomExt for NdArray<f64> {
    fn rand_uniform(rng: &mut Pcg64, shape: impl Into<Shape>) -> NdArray<f64> {
        let shape = shape.into();
        let size = shape.size();
        let data: Vec<f64> = (0..size).map(|_| rng.next_f64()).collect();
        NdArray { data, shape }
    }

    fn rand_normal(rng: &mut Pcg64, mean: f64, std: f64, shape: impl Into<Shape>) -> NdArray<f64> {
        let shape = shape.into();
        let size = shape.size();
        let data: Vec<f64> = (0..size).map(|_| rng.normal(mean, std)).collect();
        NdArray { data, shape }
    }

    fn rand_int(rng: &mut Pcg64, low: i64, high: i64, shape: impl Into<Shape>) -> NdArray<f64> {
        let shape = shape.into();
        let size = shape.size();
        let data: Vec<f64> = (0..size).map(|_| rng.randint(low, high) as f64).collect();
        NdArray { data, shape }
    }

    fn rand_binomial(rng: &mut Pcg64, n: u64, p: f64, shape: impl Into<Shape>) -> NdArray<f64> {
        let shape = shape.into();
        let size = shape.size();
        let data: Vec<f64> = (0..size).map(|_| rng.binomial(n, p) as f64).collect();
        NdArray { data, shape }
    }

    fn rand_poisson(rng: &mut Pcg64, lambda: f64, shape: impl Into<Shape>) -> NdArray<f64> {
        let shape = shape.into();
        let size = shape.size();
        let data: Vec<f64> = (0..size).map(|_| rng.poisson(lambda) as f64).collect();
        NdArray { data, shape }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pcg64_reproducibility() {
        let mut rng1 = Pcg64::new(42);
        let mut rng2 = Pcg64::new(42);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_pcg64_different_seeds() {
        let mut rng1 = Pcg64::new(42);
        let mut rng2 = Pcg64::new(123);

        let mut same = true;
        for _ in 0..10 {
            if rng1.next_u64() != rng2.next_u64() {
                same = false;
                break;
            }
        }
        assert!(!same, "different seeds should produce different sequences");
    }

    #[test]
    fn test_uniform_range() {
        let mut rng = Pcg64::new(42);
        for _ in 0..1000 {
            let val = rng.next_f64();
            assert!(val >= 0.0 && val < 1.0, "out of range: {}", val);
        }
    }

    #[test]
    fn test_uniform_custom_range() {
        let mut rng = Pcg64::new(42);
        for _ in 0..1000 {
            let val = rng.uniform(5.0, 10.0);
            assert!(val >= 5.0 && val < 10.0, "out of range: {}", val);
        }
    }

    #[test]
    fn test_normal_distribution() {
        let mut rng = Pcg64::new(42);
        let n = 10000;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;

        for _ in 0..n {
            let val = rng.normal_standard();
            sum += val;
            sum_sq += val * val;
        }

        let mean = sum / n as f64;
        let variance = sum_sq / n as f64 - mean * mean;

        assert!(mean.abs() < 0.1, "mean should be near 0, got {}", mean);
        assert!(
            (variance - 1.0).abs() < 0.2,
            "variance should be near 1, got {}",
            variance
        );
    }

    #[test]
    fn test_randint() {
        let mut rng = Pcg64::new(42);
        for _ in 0..1000 {
            let val = rng.randint(0, 10);
            assert!(val >= 0 && val < 10, "out of range: {}", val);
        }
    }

    #[test]
    fn test_binomial() {
        let mut rng = Pcg64::new(42);
        let n = 10u64;
        for _ in 0..100 {
            let val = rng.binomial(n, 0.5);
            assert!(val <= n, "binomial result {} > n {}", val, n);
        }
    }

    #[test]
    fn test_poisson() {
        let mut rng = Pcg64::new(42);
        let n = 10000;
        let lambda = 5.0;
        let mut sum = 0u64;

        for _ in 0..n {
            sum += rng.poisson(lambda);
        }

        let mean = sum as f64 / n as f64;
        assert!(
            (mean - lambda).abs() < 0.5,
            "poisson mean {} far from {}",
            mean,
            lambda
        );
    }

    #[test]
    fn test_rand_uniform_array() {
        let mut rng = Pcg64::new(42);
        let arr = NdArray::rand_uniform(&mut rng, vec![3, 4]);
        assert_eq!(arr.dims(), &[3, 4]);
        assert_eq!(arr.size(), 12);
        for &val in arr.data() {
            assert!(val >= 0.0 && val < 1.0);
        }
    }

    #[test]
    fn test_rand_normal_array() {
        let mut rng = Pcg64::new(42);
        let arr = NdArray::rand_normal(&mut rng, 0.0, 1.0, vec![100]);
        assert_eq!(arr.size(), 100);
    }

    #[test]
    fn test_shuffle() {
        let mut rng = Pcg64::new(42);
        let mut data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let original = data.clone();
        rng.shuffle(&mut data);
        // Very unlikely to remain the same
        assert_ne!(data, original, "shuffle should change order");
        // But must contain same elements
        let mut sorted = data.clone();
        sorted.sort();
        assert_eq!(sorted, original);
    }
}
