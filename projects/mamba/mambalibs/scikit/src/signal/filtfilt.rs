//! Zero-phase forward-backward digital filtering.

use super::filter::lfilter;

/// Apply zero-phase forward-backward filtering.
///
/// Applies `lfilter` in the forward direction, then reverses the output and
/// applies `lfilter` again. The result is reversed a final time. This eliminates
/// the phase distortion introduced by a single-pass IIR filter, at the cost of
/// doubling the filter order.
///
/// # Arguments
///
/// * `b` - Numerator (feedforward) coefficients
/// * `a` - Denominator (feedback) coefficients
/// * `x` - Input signal
///
/// # Example
///
/// ```
/// use scikit::signal::filtfilt;
///
/// let b = vec![0.5, 0.5]; // simple 2-tap FIR
/// let a = vec![1.0];
/// let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let y = filtfilt(&b, &a, &x);
/// assert_eq!(y.len(), x.len());
/// ```
pub fn filtfilt(b: &[f64], a: &[f64], x: &[f64]) -> Vec<f64> {
    if x.is_empty() {
        return vec![];
    }

    // Forward pass
    let forward = lfilter(b, a, x);

    // Reverse
    let reversed: Vec<f64> = forward.into_iter().rev().collect();

    // Backward pass
    let backward = lfilter(b, a, &reversed);

    // Reverse again to restore original time direction
    backward.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_filtfilt_preserves_length() {
        let b = vec![0.25, 0.25, 0.25, 0.25];
        let a = vec![1.0];
        let x: Vec<f64> = (0..100).map(|i| (2.0 * PI * 5.0 * i as f64 / 100.0).sin()).collect();
        let y = filtfilt(&b, &a, &x);
        assert_eq!(y.len(), x.len());
    }

    #[test]
    fn test_filtfilt_zero_phase() {
        // For a symmetric signal, filtfilt should preserve symmetry better
        // than a single-pass filter (which introduces phase shift).
        let n = 64;
        let x: Vec<f64> = (0..n)
            .map(|i| (2.0 * PI * 2.0 * i as f64 / n as f64).sin())
            .collect();

        let b = vec![0.5, 0.5];
        let a = vec![1.0];

        let y = filtfilt(&b, &a, &x);

        // Zero-phase: peak of output should align with peak of input.
        // Find the index of maximum in mid-section to avoid edge effects.
        let start = n / 4;
        let end = 3 * n / 4;
        let x_peak = (start..end)
            .max_by(|&i, &j| x[i].partial_cmp(&x[j]).unwrap())
            .unwrap();
        let y_peak = (start..end)
            .max_by(|&i, &j| y[i].partial_cmp(&y[j]).unwrap())
            .unwrap();

        // Peaks should be at the same index (zero phase shift)
        assert_eq!(x_peak, y_peak, "filtfilt should not shift peaks");
    }

    #[test]
    fn test_filtfilt_identity() {
        // Identity filter b=[1], a=[1] should return input unchanged
        let b = vec![1.0];
        let a = vec![1.0];
        let x = vec![1.0, 3.0, 5.0, 7.0, 9.0];
        let y = filtfilt(&b, &a, &x);
        for (xi, yi) in x.iter().zip(y.iter()) {
            assert!(
                (xi - yi).abs() < 1e-10,
                "identity filter: expected {xi}, got {yi}"
            );
        }
    }

    #[test]
    fn test_filtfilt_empty() {
        let b = vec![1.0, 0.5];
        let a = vec![1.0];
        let y = filtfilt(&b, &a, &[]);
        assert!(y.is_empty());
    }
}
