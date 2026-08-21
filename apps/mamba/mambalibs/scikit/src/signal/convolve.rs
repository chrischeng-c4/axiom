//! 1D convolution and cross-correlation.

/// Convolution output mode.
#[derive(Debug, Clone, Copy)]
pub enum ConvolveMode {
    /// Full output (length = len(a) + len(b) - 1).
    Full,
    /// Output same length as first input.
    Same,
    /// Only parts where signals fully overlap.
    Valid,
}

/// 1D convolution of `a` and `b`.
pub fn convolve(a: &[f64], b: &[f64], mode: ConvolveMode) -> Vec<f64> {
    if a.is_empty() || b.is_empty() {
        return vec![];
    }
    let full_len = a.len() + b.len() - 1;
    let mut result = vec![0.0; full_len];

    for i in 0..a.len() {
        for j in 0..b.len() {
            result[i + j] += a[i] * b[j];
        }
    }

    match mode {
        ConvolveMode::Full => result,
        ConvolveMode::Same => {
            let start = (b.len() - 1) / 2;
            result[start..start + a.len()].to_vec()
        }
        ConvolveMode::Valid => {
            let (longer, shorter) = if a.len() >= b.len() {
                (a.len(), b.len())
            } else {
                (b.len(), a.len())
            };
            let valid_len = longer - shorter + 1;
            let start = shorter - 1;
            result[start..start + valid_len].to_vec()
        }
    }
}

/// 1D cross-correlation of `a` and `b`.
pub fn correlate(a: &[f64], b: &[f64], mode: ConvolveMode) -> Vec<f64> {
    // Correlation is convolution with reversed kernel
    let b_rev: Vec<f64> = b.iter().rev().copied().collect();
    convolve(a, &b_rev, mode)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convolve_full() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![0.0, 1.0, 0.5];
        let result = convolve(&a, &b, ConvolveMode::Full);
        assert_eq!(result.len(), 5);
        assert!((result[0] - 0.0).abs() < 1e-10);
        assert!((result[1] - 1.0).abs() < 1e-10);
        assert!((result[2] - 2.5).abs() < 1e-10);
        assert!((result[3] - 4.0).abs() < 1e-10);
        assert!((result[4] - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_convolve_same() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 1.0, 1.0];
        let result = convolve(&a, &b, ConvolveMode::Same);
        assert_eq!(result.len(), 5);
    }

    #[test]
    fn test_convolve_valid() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 1.0, 1.0];
        let result = convolve(&a, &b, ConvolveMode::Valid);
        assert_eq!(result.len(), 3);
        assert!((result[0] - 6.0).abs() < 1e-10); // 1+2+3
        assert!((result[1] - 9.0).abs() < 1e-10); // 2+3+4
        assert!((result[2] - 12.0).abs() < 1e-10); // 3+4+5
    }

    #[test]
    fn test_correlate() {
        let a = vec![1.0, 2.0, 3.0];
        let result = correlate(&a, &a, ConvolveMode::Full);
        assert_eq!(result.len(), 5);
        // Auto-correlation peak at center
        let max_val = result.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        assert!((max_val - result[2]).abs() < 1e-10);
    }
}
