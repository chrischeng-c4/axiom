//! Logistic regression (binary and one-vs-rest multiclass).
//!
//! Uses gradient descent optimization.

use super::error::{MlError, Result};

/// Binary logistic regression.
#[derive(Debug, Clone)]
pub struct LogisticRegression {
    pub weights: Option<Vec<f64>>,
    pub intercept: Option<f64>,
    pub learning_rate: f64,
    pub max_iter: usize,
    pub tol: f64,
}

impl Default for LogisticRegression {
    fn default() -> Self {
        Self::new()
    }
}

impl LogisticRegression {
    pub fn new() -> Self {
        Self {
            weights: None,
            intercept: None,
            learning_rate: 0.01,
            max_iter: 1000,
            tol: 1e-6,
        }
    }

    pub fn with_lr(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }

    pub fn with_max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Fit binary logistic regression. y values should be 0.0 or 1.0.
    pub fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n = y.len();
        if x.len() != n * n_features {
            return Err(MlError::ShapeMismatch(format!(
                "x has {} elements, expected {}",
                x.len(),
                n * n_features
            )));
        }

        let mut w = vec![0.0; n_features];
        let mut b = 0.0;

        for _iter in 0..self.max_iter {
            let mut grad_w = vec![0.0; n_features];
            let mut grad_b = 0.0;

            for i in 0..n {
                let row = &x[i * n_features..(i + 1) * n_features];
                let z: f64 = row
                    .iter()
                    .zip(w.iter())
                    .map(|(xi, wi)| xi * wi)
                    .sum::<f64>()
                    + b;
                let p = sigmoid(z);
                let err = p - y[i];

                for j in 0..n_features {
                    grad_w[j] += err * row[j];
                }
                grad_b += err;
            }

            let mut max_grad = 0.0_f64;
            for j in 0..n_features {
                grad_w[j] /= n as f64;
                w[j] -= self.learning_rate * grad_w[j];
                max_grad = max_grad.max(grad_w[j].abs());
            }
            grad_b /= n as f64;
            b -= self.learning_rate * grad_b;
            max_grad = max_grad.max(grad_b.abs());

            if max_grad < self.tol {
                break;
            }
        }

        self.weights = Some(w);
        self.intercept = Some(b);
        Ok(())
    }

    /// Predict probabilities for class 1.
    pub fn predict_proba(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let w = self.weights.as_ref().ok_or(MlError::NotFitted)?;
        let b = self.intercept.unwrap_or(0.0);
        let n_samples = x.len() / n_features;

        let mut proba = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let z: f64 = row
                .iter()
                .zip(w.iter())
                .map(|(xi, wi)| xi * wi)
                .sum::<f64>()
                + b;
            proba.push(sigmoid(z));
        }
        Ok(proba)
    }

    /// Predict class labels (0.0 or 1.0) using threshold 0.5.
    pub fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let proba = self.predict_proba(x, n_features)?;
        Ok(proba
            .iter()
            .map(|&p| if p >= 0.5 { 1.0 } else { 0.0 })
            .collect())
    }

    /// Compute accuracy on labeled data.
    pub fn score(&self, x: &[f64], n_features: usize, y: &[f64]) -> Result<f64> {
        let preds = self.predict(x, n_features)?;
        let correct = preds
            .iter()
            .zip(y.iter())
            .filter(|(p, t)| (**p - **t).abs() < 0.5)
            .count();
        Ok(correct as f64 / y.len() as f64)
    }
}

fn sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let ex = x.exp();
        ex / (1.0 + ex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_classification() {
        // Simple linearly separable data
        let x = vec![
            0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 2.0, 3.0, 3.0,
        ];
        let y = vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0];

        let mut model = LogisticRegression::new().with_lr(0.1).with_max_iter(500);
        model.fit(&x, 2, &y).unwrap();

        let preds = model.predict(&x, 2).unwrap();
        let accuracy = preds
            .iter()
            .zip(y.iter())
            .filter(|(p, t)| (**p - **t).abs() < 0.5)
            .count() as f64
            / y.len() as f64;
        assert!(accuracy >= 0.75, "accuracy={}", accuracy);
    }

    #[test]
    fn test_predict_proba() {
        let x = vec![0.0, 10.0];
        let y = vec![0.0, 1.0];
        let mut model = LogisticRegression::new().with_lr(0.1).with_max_iter(200);
        model.fit(&x, 1, &y).unwrap();
        let proba = model.predict_proba(&x, 1).unwrap();
        assert!(proba[0] < 0.5);
        assert!(proba[1] > 0.5);
    }

    #[test]
    fn test_not_fitted() {
        let model = LogisticRegression::new();
        assert!(model.predict(&[1.0], 1).is_err());
    }

    #[test]
    fn test_sigmoid_edge_cases() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-10);
        assert!(sigmoid(100.0) > 0.99);
        assert!(sigmoid(-100.0) < 0.01);
    }
}
