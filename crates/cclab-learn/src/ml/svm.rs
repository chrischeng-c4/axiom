//! Support Vector Machine (linear SVM using SGD).

use super::error::{MlError, Result};
use super::traits::{Estimator, Predictor};

/// Linear SVM classifier (binary, trained via SGD on hinge loss).
#[derive(Debug, Clone)]
pub struct LinearSVC {
    pub c: f64,
    pub max_iter: usize,
    pub learning_rate: f64,
    weights: Option<Vec<f64>>,
    bias: Option<f64>,
}

impl LinearSVC {
    pub fn new() -> Self {
        Self {
            c: 1.0,
            max_iter: 1000,
            learning_rate: 0.01,
            weights: None,
            bias: None,
        }
    }

    pub fn with_c(mut self, c: f64) -> Self {
        self.c = c;
        self
    }

    pub fn with_max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    /// Decision function: w·x + b
    pub fn decision_function(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let w = self.weights.as_ref().ok_or(MlError::NotFitted)?;
        let b = self.bias.ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;
        let mut scores = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let score: f64 = row
                .iter()
                .zip(w.iter())
                .map(|(xi, wi)| xi * wi)
                .sum::<f64>()
                + b;
            scores.push(score);
        }
        Ok(scores)
    }
}

impl Default for LinearSVC {
    fn default() -> Self {
        Self::new()
    }
}

impl Estimator for LinearSVC {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples != y.len() {
            return Err(MlError::ShapeMismatch("x rows != y length".into()));
        }

        let mut w = vec![0.0; n_features];
        let mut b = 0.0;

        for epoch in 0..self.max_iter {
            let lr = self.learning_rate / (1.0 + 0.001 * epoch as f64);
            for i in 0..n_samples {
                let row = &x[i * n_features..(i + 1) * n_features];
                let yi = if y[i] > 0.5 { 1.0 } else { -1.0 };
                let score: f64 = row
                    .iter()
                    .zip(w.iter())
                    .map(|(xi, wi)| xi * wi)
                    .sum::<f64>()
                    + b;

                if yi * score < 1.0 {
                    // Misclassified or within margin
                    for j in 0..n_features {
                        w[j] -= lr * (w[j] / (self.c * n_samples as f64) - yi * row[j]);
                    }
                    b += lr * yi;
                } else {
                    for j in 0..n_features {
                        w[j] -= lr * w[j] / (self.c * n_samples as f64);
                    }
                }
            }
        }

        self.weights = Some(w);
        self.bias = Some(b);
        Ok(())
    }
}

impl Predictor for LinearSVC {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let scores = self.decision_function(x, n_features)?;
        Ok(scores
            .iter()
            .map(|&s| if s >= 0.0 { 1.0 } else { 0.0 })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_svc_separable() {
        // Linearly separable: class 0 = x < 5, class 1 = x >= 5
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&v| if v >= 5.0 { 1.0 } else { 0.0 })
            .collect();

        let mut svm = LinearSVC::new().with_c(1.0).with_max_iter(500);
        svm.fit(&x, 1, &y).unwrap();

        let pred = svm.predict(&[1.0, 8.0], 1).unwrap();
        assert!((pred[0] - 0.0).abs() < 1e-10);
        assert!((pred[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_svc_decision_function() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&v| if v >= 5.0 { 1.0 } else { 0.0 })
            .collect();

        let mut svm = LinearSVC::new();
        svm.fit(&x, 1, &y).unwrap();
        let scores = svm.decision_function(&[1.0, 8.0], 1).unwrap();
        assert!(scores[0] < scores[1]);
    }
}
