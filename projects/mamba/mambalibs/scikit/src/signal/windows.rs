//! Window functions for spectral analysis.

use std::f64::consts::PI;

/// Hann (Hanning) window.
pub fn hann(n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f64 / (n - 1) as f64).cos()))
        .collect()
}

/// Hamming window.
pub fn hamming(n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    (0..n)
        .map(|i| 0.54 - 0.46 * (2.0 * PI * i as f64 / (n - 1) as f64).cos())
        .collect()
}

/// Blackman window.
pub fn blackman(n: usize) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    (0..n)
        .map(|i| {
            let x = 2.0 * PI * i as f64 / (n - 1) as f64;
            0.42 - 0.5 * x.cos() + 0.08 * (2.0 * x).cos()
        })
        .collect()
}

/// Kaiser window.
///
/// `beta` controls the tradeoff between main-lobe width and side-lobe level.
pub fn kaiser(n: usize, beta: f64) -> Vec<f64> {
    if n == 0 {
        return vec![];
    }
    if n == 1 {
        return vec![1.0];
    }
    let i0_beta = bessel_i0(beta);
    (0..n)
        .map(|i| {
            let x = 2.0 * i as f64 / (n - 1) as f64 - 1.0;
            bessel_i0(beta * (1.0 - x * x).sqrt()) / i0_beta
        })
        .collect()
}

/// Modified Bessel function of the first kind, order 0 (series expansion).
fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let x2 = x * x / 4.0;
    for k in 1..50 {
        term *= x2 / (k as f64 * k as f64);
        sum += term;
        if term < 1e-16 * sum {
            break;
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hann_symmetry() {
        let w = hann(8);
        assert_eq!(w.len(), 8);
        assert!((w[0] - 0.0).abs() < 1e-10);
        assert!((w[7] - 0.0).abs() < 1e-10);
        // Symmetric
        for i in 0..4 {
            assert!((w[i] - w[7 - i]).abs() < 1e-10);
        }
        // Odd-length: exact peak at center
        let w9 = hann(9);
        assert!((w9[4] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_hamming_bounds() {
        let w = hamming(16);
        for &v in &w {
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_blackman_endpoints() {
        let w = blackman(16);
        assert!(w[0].abs() < 1e-10);
        assert!(w[15].abs() < 1e-10);
    }

    #[test]
    fn test_kaiser_unity_center() {
        let w = kaiser(16, 5.0);
        // Kaiser window peaks near center
        let mid = w.len() / 2;
        assert!(w[mid] > 0.9);
    }
}
