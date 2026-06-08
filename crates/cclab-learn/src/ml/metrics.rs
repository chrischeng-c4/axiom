//! Evaluation metrics for classification and regression.

// ============================================================================
// Classification metrics
// ============================================================================

/// Compute classification accuracy.
pub fn accuracy(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.is_empty() {
        return 0.0;
    }
    let correct = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (**t - **p).abs() < 0.5)
        .count();
    correct as f64 / y_true.len() as f64
}

/// Compute precision for a given positive class.
pub fn precision(y_true: &[f64], y_pred: &[f64], pos_class: f64) -> f64 {
    let tp = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (**p - pos_class).abs() < 0.5 && (**t - pos_class).abs() < 0.5)
        .count();
    let fp = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (**p - pos_class).abs() < 0.5 && (**t - pos_class).abs() >= 0.5)
        .count();
    if tp + fp == 0 {
        0.0
    } else {
        tp as f64 / (tp + fp) as f64
    }
}

/// Compute recall for a given positive class.
pub fn recall(y_true: &[f64], y_pred: &[f64], pos_class: f64) -> f64 {
    let tp = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (**t - pos_class).abs() < 0.5 && (**p - pos_class).abs() < 0.5)
        .count();
    let fn_ = y_true
        .iter()
        .zip(y_pred.iter())
        .filter(|(t, p)| (**t - pos_class).abs() < 0.5 && (**p - pos_class).abs() >= 0.5)
        .count();
    if tp + fn_ == 0 {
        0.0
    } else {
        tp as f64 / (tp + fn_) as f64
    }
}

/// Compute F1 score for a given positive class.
pub fn f1_score(y_true: &[f64], y_pred: &[f64], pos_class: f64) -> f64 {
    let p = precision(y_true, y_pred, pos_class);
    let r = recall(y_true, y_pred, pos_class);
    if p + r < 1e-15 {
        0.0
    } else {
        2.0 * p * r / (p + r)
    }
}

/// Compute confusion matrix for binary classification.
///
/// Returns `(tp, fp, fn, tn)`.
pub fn confusion_matrix(
    y_true: &[f64],
    y_pred: &[f64],
    pos_class: f64,
) -> (usize, usize, usize, usize) {
    let mut tp = 0;
    let mut fp = 0;
    let mut fn_ = 0;
    let mut tn = 0;

    for (t, p) in y_true.iter().zip(y_pred.iter()) {
        let is_pos_true = (*t - pos_class).abs() < 0.5;
        let is_pos_pred = (*p - pos_class).abs() < 0.5;
        match (is_pos_true, is_pos_pred) {
            (true, true) => tp += 1,
            (false, true) => fp += 1,
            (true, false) => fn_ += 1,
            (false, false) => tn += 1,
        }
    }

    (tp, fp, fn_, tn)
}

// ============================================================================
// Regression metrics
// ============================================================================

/// Mean Squared Error.
pub fn mse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.is_empty() {
        return 0.0;
    }
    y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| (t - p).powi(2))
        .sum::<f64>()
        / y_true.len() as f64
}

/// Root Mean Squared Error.
pub fn rmse(y_true: &[f64], y_pred: &[f64]) -> f64 {
    mse(y_true, y_pred).sqrt()
}

/// Mean Absolute Error.
pub fn mae(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.is_empty() {
        return 0.0;
    }
    y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| (t - p).abs())
        .sum::<f64>()
        / y_true.len() as f64
}

/// R² score (coefficient of determination).
pub fn r2_score(y_true: &[f64], y_pred: &[f64]) -> f64 {
    crate::ml::linear::r2_score(y_true, y_pred)
}

/// Mean Absolute Percentage Error.
pub fn mape(y_true: &[f64], y_pred: &[f64]) -> f64 {
    if y_true.is_empty() {
        return 0.0;
    }
    let n = y_true.len() as f64;
    y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(t, p)| {
            if t.abs() < 1e-15 {
                0.0
            } else {
                ((t - p) / t).abs()
            }
        })
        .sum::<f64>()
        / n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accuracy() {
        let y_true = vec![1.0, 0.0, 1.0, 1.0, 0.0];
        let y_pred = vec![1.0, 0.0, 0.0, 1.0, 0.0];
        assert!((accuracy(&y_true, &y_pred) - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_precision_recall() {
        let y_true = vec![1.0, 1.0, 0.0, 0.0, 1.0];
        let y_pred = vec![1.0, 0.0, 0.0, 1.0, 1.0];
        // TP=2, FP=1, FN=1, TN=1
        let p = precision(&y_true, &y_pred, 1.0);
        let r = recall(&y_true, &y_pred, 1.0);
        assert!((p - 2.0 / 3.0).abs() < 1e-10);
        assert!((r - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_f1() {
        let y_true = vec![1.0, 1.0, 0.0, 0.0];
        let y_pred = vec![1.0, 0.0, 0.0, 0.0];
        let f1 = f1_score(&y_true, &y_pred, 1.0);
        // precision = 1/1 = 1.0, recall = 1/2 = 0.5
        assert!((f1 - 2.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_confusion_matrix() {
        let y_true = vec![1.0, 1.0, 0.0, 0.0];
        let y_pred = vec![1.0, 0.0, 0.0, 1.0];
        let (tp, fp, fn_, tn) = confusion_matrix(&y_true, &y_pred, 1.0);
        assert_eq!((tp, fp, fn_, tn), (1, 1, 1, 1));
    }

    #[test]
    fn test_mse() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.0, 2.0, 4.0];
        assert!((mse(&y_true, &y_pred) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_rmse() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.0, 2.0, 3.0];
        assert!((rmse(&y_true, &y_pred) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_mae() {
        let y_true = vec![1.0, 2.0, 3.0];
        let y_pred = vec![1.5, 2.5, 3.5];
        assert!((mae(&y_true, &y_pred) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_r2() {
        let y_true = vec![1.0, 2.0, 3.0, 4.0];
        let y_pred = vec![1.0, 2.0, 3.0, 4.0];
        assert!((r2_score(&y_true, &y_pred) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_mape() {
        let y_true = vec![100.0, 200.0];
        let y_pred = vec![110.0, 180.0];
        // MAPE = (10/100 + 20/200) / 2 = 0.1
        assert!((mape(&y_true, &y_pred) - 0.1).abs() < 1e-10);
    }
}
