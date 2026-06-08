//! Train-test split utility.

use super::error::{MlError, Result};

/// Result of a train-test split.
#[derive(Debug)]
pub struct SplitResult {
    pub x_train: Vec<f64>,
    pub x_test: Vec<f64>,
    pub y_train: Vec<f64>,
    pub y_test: Vec<f64>,
}

/// Split data into training and test sets.
///
/// `test_size` is the fraction of data to use for testing (0.0 to 1.0).
/// `seed` controls random shuffling. If `None`, data is split without shuffling.
pub fn train_test_split(
    x: &[f64],
    n_features: usize,
    y: &[f64],
    test_size: f64,
    seed: Option<u64>,
) -> Result<SplitResult> {
    if !(0.0..=1.0).contains(&test_size) {
        return Err(MlError::InvalidParameter(format!(
            "test_size must be in [0, 1], got {}",
            test_size
        )));
    }

    let n_samples = y.len();
    if x.len() != n_samples * n_features {
        return Err(MlError::ShapeMismatch("x and y size mismatch".into()));
    }

    let n_test = (n_samples as f64 * test_size).round() as usize;
    let n_train = n_samples - n_test;

    // Create index permutation
    let mut indices: Vec<usize> = (0..n_samples).collect();
    if let Some(seed) = seed {
        // Fisher-Yates shuffle with LCG
        let mut rng = seed;
        for i in (1..n_samples).rev() {
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let j = (rng % (i as u64 + 1)) as usize;
            indices.swap(i, j);
        }
    }

    let mut x_train = Vec::with_capacity(n_train * n_features);
    let mut y_train = Vec::with_capacity(n_train);
    let mut x_test = Vec::with_capacity(n_test * n_features);
    let mut y_test = Vec::with_capacity(n_test);

    for &idx in &indices[..n_train] {
        x_train.extend_from_slice(&x[idx * n_features..(idx + 1) * n_features]);
        y_train.push(y[idx]);
    }
    for &idx in &indices[n_train..] {
        x_test.extend_from_slice(&x[idx * n_features..(idx + 1) * n_features]);
        y_test.push(y[idx]);
    }

    Ok(SplitResult {
        x_train,
        x_test,
        y_train,
        y_test,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Happy cases ────────────────────────────────────────────────────

    #[test]
    fn test_basic_split() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let result = train_test_split(&x, 2, &y, 0.3, Some(42)).unwrap();
        assert_eq!(result.y_train.len(), 7);
        assert_eq!(result.y_test.len(), 3);
        assert_eq!(result.x_train.len(), 14);
        assert_eq!(result.x_test.len(), 6);
    }

    #[test]
    fn test_no_shuffle() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = train_test_split(&x, 1, &y, 0.4, None).unwrap();
        assert_eq!(result.y_train, vec![1.0, 2.0, 3.0]);
        assert_eq!(result.y_test, vec![4.0, 5.0]);
    }

    #[test]
    fn test_deterministic() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let r1 = train_test_split(&x, 1, &y, 0.3, Some(123)).unwrap();
        let r2 = train_test_split(&x, 1, &y, 0.3, Some(123)).unwrap();
        assert_eq!(r1.y_train, r2.y_train);
        assert_eq!(r1.y_test, r2.y_test);
    }

    #[test]
    fn test_multi_feature_split() {
        // 4 samples × 3 features = 12 elements
        let x: Vec<f64> = (0..12).map(|i| i as f64).collect();
        let y = vec![10.0, 20.0, 30.0, 40.0];
        let result = train_test_split(&x, 3, &y, 0.25, Some(99)).unwrap();
        assert_eq!(result.y_train.len(), 3);
        assert_eq!(result.y_test.len(), 1);
        assert_eq!(result.x_train.len(), 9); // 3 samples × 3 features
        assert_eq!(result.x_test.len(), 3); // 1 sample × 3 features
    }

    #[test]
    fn test_all_data_preserved() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = train_test_split(&x, 1, &y, 0.4, Some(7)).unwrap();
        // All y values should appear in either train or test
        let mut all_y: Vec<f64> = result
            .y_train
            .iter()
            .chain(result.y_test.iter())
            .copied()
            .collect();
        all_y.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(all_y, vec![10.0, 20.0, 30.0, 40.0, 50.0]);
    }

    #[test]
    fn test_different_seeds_differ() {
        let x: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let y: Vec<f64> = (0..20).map(|i| i as f64).collect();
        let r1 = train_test_split(&x, 1, &y, 0.3, Some(1)).unwrap();
        let r2 = train_test_split(&x, 1, &y, 0.3, Some(999)).unwrap();
        // With 20 samples and different seeds, extremely unlikely to get same order
        assert_ne!(r1.y_train, r2.y_train);
    }

    // ── Error cases ────────────────────────────────────────────────────

    #[test]
    fn test_invalid_test_size_above_one() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let err = train_test_split(&x, 1, &y, 1.5, None);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), MlError::InvalidParameter(_)));
    }

    #[test]
    fn test_invalid_test_size_negative() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let err = train_test_split(&x, 1, &y, -0.1, None);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), MlError::InvalidParameter(_)));
    }

    #[test]
    fn test_shape_mismatch_x_too_short() {
        let x = vec![1.0, 2.0, 3.0]; // 3 elements but need 4 (2 samples × 2 features)
        let y = vec![1.0, 2.0];
        let err = train_test_split(&x, 2, &y, 0.5, None);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), MlError::ShapeMismatch(_)));
    }

    #[test]
    fn test_shape_mismatch_x_too_long() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0]; // 5 elements but need 4
        let y = vec![1.0, 2.0];
        let err = train_test_split(&x, 2, &y, 0.5, None);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), MlError::ShapeMismatch(_)));
    }

    // ── Edge cases ─────────────────────────────────────────────────────

    #[test]
    fn test_test_size_zero() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = train_test_split(&x, 1, &y, 0.0, None).unwrap();
        assert_eq!(result.y_train.len(), 5);
        assert_eq!(result.y_test.len(), 0);
        assert_eq!(result.x_train.len(), 5);
        assert_eq!(result.x_test.len(), 0);
    }

    #[test]
    fn test_test_size_one() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = train_test_split(&x, 1, &y, 1.0, None).unwrap();
        assert_eq!(result.y_train.len(), 0);
        assert_eq!(result.y_test.len(), 5);
        assert_eq!(result.x_train.len(), 0);
        assert_eq!(result.x_test.len(), 5);
    }

    #[test]
    fn test_single_sample_to_test() {
        let x = vec![42.0];
        let y = vec![99.0];
        let result = train_test_split(&x, 1, &y, 1.0, None).unwrap();
        assert_eq!(result.y_test, vec![99.0]);
        assert_eq!(result.x_test, vec![42.0]);
        assert!(result.y_train.is_empty());
    }

    #[test]
    fn test_single_sample_to_train() {
        let x = vec![42.0];
        let y = vec![99.0];
        let result = train_test_split(&x, 1, &y, 0.0, None).unwrap();
        assert_eq!(result.y_train, vec![99.0]);
        assert_eq!(result.x_train, vec![42.0]);
        assert!(result.y_test.is_empty());
    }

    #[test]
    fn test_two_samples_half_split() {
        let x = vec![1.0, 2.0];
        let y = vec![10.0, 20.0];
        let result = train_test_split(&x, 1, &y, 0.5, None).unwrap();
        assert_eq!(result.y_train.len(), 1);
        assert_eq!(result.y_test.len(), 1);
    }

    #[test]
    fn test_multi_feature_preserves_rows() {
        // 3 samples × 2 features, no shuffle → deterministic order
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let y = vec![10.0, 20.0, 30.0];
        let result = train_test_split(&x, 2, &y, 0.34, None).unwrap();
        // n_test = round(3 * 0.34) = 1
        assert_eq!(result.y_train.len(), 2);
        assert_eq!(result.y_test.len(), 1);
        // Without shuffle, first 2 samples go to train
        assert_eq!(result.x_train, vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(result.x_test, vec![5.0, 6.0]);
    }
}
