//! K-fold cross-validation scoring.

use super::error::{MlError, Result};
use super::metrics;
use super::traits::{Estimator, Predictor};

/// Perform k-fold cross-validation and return per-fold scores.
///
/// # Arguments
/// * `estimator` - A clonable model implementing `Estimator + Predictor`.
/// * `x` - Flat feature matrix (n_samples * n_features), row-major.
/// * `n_features` - Number of features per sample.
/// * `y` - Target vector (n_samples).
/// * `cv` - Number of folds (must be >= 2).
/// * `scoring` - Scoring function name: `"accuracy"` or `"mse"`.
///
/// # Returns
/// A vector of scores, one per fold.
pub fn cross_val_score<E>(
    estimator: &E,
    x: &[f64],
    n_features: usize,
    y: &[f64],
    cv: usize,
    scoring: &str,
) -> Result<Vec<f64>>
where
    E: Clone + Estimator + Predictor,
{
    let n_samples = y.len();
    if x.len() != n_samples * n_features {
        return Err(MlError::ShapeMismatch(format!(
            "x has {} elements, expected {}x{}={}",
            x.len(),
            n_samples,
            n_features,
            n_samples * n_features
        )));
    }
    if cv < 2 {
        return Err(MlError::InvalidParameter("cv must be >= 2".into()));
    }
    if n_samples < cv {
        return Err(MlError::InvalidParameter(format!(
            "n_samples ({}) < cv ({})",
            n_samples, cv
        )));
    }

    let scorer = resolve_scorer(scoring)?;
    let fold_size = n_samples / cv;
    let mut scores = Vec::with_capacity(cv);

    for fold in 0..cv {
        let test_start = fold * fold_size;
        let test_end = if fold == cv - 1 {
            n_samples // last fold absorbs remainder
        } else {
            test_start + fold_size
        };

        // Build train/test splits
        let (x_train, y_train, x_test, y_test) = split_fold(x, n_features, y, test_start, test_end);

        // Clone the estimator, fit on training data, predict on test data
        let mut model = estimator.clone();
        model.fit(&x_train, n_features, &y_train)?;
        let preds = model.predict(&x_test, n_features)?;

        let score = scorer(&y_test, &preds);
        scores.push(score);
    }

    Ok(scores)
}

/// Split data into train and test for a single fold.
fn split_fold(
    x: &[f64],
    n_features: usize,
    y: &[f64],
    test_start: usize,
    test_end: usize,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n_samples = y.len();
    let n_test = test_end - test_start;
    let n_train = n_samples - n_test;

    let mut x_train = Vec::with_capacity(n_train * n_features);
    let mut y_train = Vec::with_capacity(n_train);
    let mut x_test = Vec::with_capacity(n_test * n_features);
    let mut y_test = Vec::with_capacity(n_test);

    for i in 0..n_samples {
        let row = &x[i * n_features..(i + 1) * n_features];
        if i >= test_start && i < test_end {
            x_test.extend_from_slice(row);
            y_test.push(y[i]);
        } else {
            x_train.extend_from_slice(row);
            y_train.push(y[i]);
        }
    }

    (x_train, y_train, x_test, y_test)
}

type ScorerFn = fn(&[f64], &[f64]) -> f64;

fn resolve_scorer(scoring: &str) -> Result<ScorerFn> {
    match scoring {
        "accuracy" => Ok(metrics::accuracy),
        "mse" => Ok(|y_true: &[f64], y_pred: &[f64]| {
            // Return negative MSE so that higher is better (sklearn convention)
            -metrics::mse(y_true, y_pred)
        }),
        "neg_mse" => Ok(|y_true: &[f64], y_pred: &[f64]| -metrics::mse(y_true, y_pred)),
        "raw_mse" => Ok(metrics::mse),
        _ => Err(MlError::InvalidParameter(format!(
            "unknown scoring '{}', expected 'accuracy', 'mse', 'neg_mse', or 'raw_mse'",
            scoring
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::knn::KNeighborsClassifier;
    use crate::ml::linear::LinearRegression;

    #[test]
    fn test_cross_val_score_regression() {
        // y = 2*x, perfect linear relationship
        // 10 samples, 5-fold CV
        let x: Vec<f64> = (1..=10).map(|i| i as f64).collect();
        let y: Vec<f64> = (1..=10).map(|i| (i * 2) as f64).collect();

        let model = LinearRegression::new();
        let scores = cross_val_score(&model, &x, 1, &y, 5, "mse").unwrap();

        assert_eq!(scores.len(), 5);
        // Perfect linear data should give near-zero MSE (neg_mse near 0)
        for &s in &scores {
            assert!(s > -1.0, "expected neg MSE close to 0, got {}", s);
        }
    }

    #[test]
    fn test_cross_val_score_classification() {
        // Two clusters: class 0 near x=0, class 1 near x=10
        // Interleave classes so each fold gets both classes in training set.
        // 8 samples, 2 features each
        let x = vec![
            0.0, 0.0, //  class 0
            10.0, 10.0, // class 1
            0.1, 0.1, //  class 0
            10.1, 10.1, // class 1
            0.2, 0.2, //  class 0
            10.2, 10.2, // class 1
            0.3, 0.3, //  class 0
            10.3, 10.3, // class 1
        ];
        let y = vec![0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0];

        let model = KNeighborsClassifier::new(1);
        let scores = cross_val_score(&model, &x, 2, &y, 2, "accuracy").unwrap();

        assert_eq!(scores.len(), 2);
        // Well-separated clusters: accuracy should be 1.0 for each fold
        for &s in &scores {
            assert!(s >= 0.75, "expected high accuracy, got {}", s);
        }
    }

    #[test]
    fn test_cross_val_score_invalid_cv() {
        let x = vec![1.0, 2.0];
        let y = vec![1.0, 2.0];
        let model = LinearRegression::new();
        let result = cross_val_score(&model, &x, 1, &y, 1, "mse");
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_val_score_invalid_scoring() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let model = LinearRegression::new();
        let result = cross_val_score(&model, &x, 1, &y, 2, "unknown");
        assert!(result.is_err());
    }

    #[test]
    fn test_cross_val_score_shape_mismatch() {
        let x = vec![1.0, 2.0, 3.0]; // 3 elements but 2 samples x 2 features = 4
        let y = vec![1.0, 2.0];
        let model = LinearRegression::new();
        let result = cross_val_score(&model, &x, 2, &y, 2, "mse");
        assert!(result.is_err());
    }
}
