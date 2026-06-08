//! Gaussian Naive Bayes classifier.

use super::error::{MlError, Result};
use super::traits::{Classifier, Estimator, Predictor};
use std::f64::consts::PI;

/// Gaussian Naive Bayes classifier.
#[derive(Debug, Clone)]
pub struct GaussianNB {
    /// Per-class mean for each feature: [class][feature].
    means: Option<Vec<Vec<f64>>>,
    /// Per-class variance for each feature.
    variances: Option<Vec<Vec<f64>>>,
    /// Prior probabilities for each class.
    priors: Option<Vec<f64>>,
    /// Sorted unique class labels.
    classes: Option<Vec<f64>>,
    n_features: usize,
}

impl GaussianNB {
    pub fn new() -> Self {
        Self {
            means: None,
            variances: None,
            priors: None,
            classes: None,
            n_features: 0,
        }
    }
}

impl Default for GaussianNB {
    fn default() -> Self {
        Self::new()
    }
}

impl Estimator for GaussianNB {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples != y.len() {
            return Err(MlError::ShapeMismatch("x rows != y length".into()));
        }

        // Find unique classes
        let mut classes: Vec<f64> = Vec::new();
        for &v in y {
            if !classes.iter().any(|&c| (c - v).abs() < 1e-10) {
                classes.push(v);
            }
        }
        classes.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n_classes = classes.len();
        let mut means = vec![vec![0.0; n_features]; n_classes];
        let mut variances = vec![vec![0.0; n_features]; n_classes];
        let mut counts = vec![0usize; n_classes];

        // Compute means
        for i in 0..n_samples {
            let ci = classes
                .iter()
                .position(|&c| (c - y[i]).abs() < 1e-10)
                .unwrap();
            counts[ci] += 1;
            for j in 0..n_features {
                means[ci][j] += x[i * n_features + j];
            }
        }
        for ci in 0..n_classes {
            for j in 0..n_features {
                means[ci][j] /= counts[ci] as f64;
            }
        }

        // Compute variances
        for i in 0..n_samples {
            let ci = classes
                .iter()
                .position(|&c| (c - y[i]).abs() < 1e-10)
                .unwrap();
            for j in 0..n_features {
                let diff = x[i * n_features + j] - means[ci][j];
                variances[ci][j] += diff * diff;
            }
        }
        for ci in 0..n_classes {
            for j in 0..n_features {
                variances[ci][j] = variances[ci][j] / counts[ci] as f64 + 1e-9; // add epsilon
            }
        }

        let priors: Vec<f64> = counts
            .iter()
            .map(|&c| c as f64 / n_samples as f64)
            .collect();

        self.means = Some(means);
        self.variances = Some(variances);
        self.priors = Some(priors);
        self.classes = Some(classes);
        self.n_features = n_features;
        Ok(())
    }
}

impl Predictor for GaussianNB {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let proba = self.predict_proba(x, n_features)?;
        let classes = self.classes.as_ref().ok_or(MlError::NotFitted)?;
        let n_classes = classes.len();
        let n_samples = x.len() / n_features;
        let mut predictions = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &proba[i * n_classes..(i + 1) * n_classes];
            let best = row
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
                .unwrap()
                .0;
            predictions.push(classes[best]);
        }
        Ok(predictions)
    }
}

impl Classifier for GaussianNB {
    fn predict_proba(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let means = self.means.as_ref().ok_or(MlError::NotFitted)?;
        let variances = self.variances.as_ref().ok_or(MlError::NotFitted)?;
        let priors = self.priors.as_ref().ok_or(MlError::NotFitted)?;
        let n_classes = means.len();
        let n_samples = x.len() / n_features;

        let mut result = Vec::with_capacity(n_samples * n_classes);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let mut log_probs = Vec::with_capacity(n_classes);
            for ci in 0..n_classes {
                let mut log_p = priors[ci].ln();
                for j in 0..n_features {
                    let mu = means[ci][j];
                    let var = variances[ci][j];
                    let diff = row[j] - mu;
                    log_p += -0.5 * (2.0 * PI * var).ln() - diff * diff / (2.0 * var);
                }
                log_probs.push(log_p);
            }
            // Softmax for probabilities
            let max_lp = log_probs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            let sum_exp: f64 = log_probs.iter().map(|&lp| (lp - max_lp).exp()).sum();
            for lp in &log_probs {
                result.push(((lp - max_lp).exp()) / sum_exp);
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_nb_basic() {
        // Simple 2D, 2 class problem
        let x = vec![
            0.0, 0.0, 0.1, 0.1, 0.2, 0.0, // class 0
            5.0, 5.0, 5.1, 5.1, 5.2, 5.0, // class 1
        ];
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

        let mut nb = GaussianNB::new();
        nb.fit(&x, 2, &y).unwrap();

        let pred = nb.predict(&[0.0, 0.0, 5.0, 5.0], 2).unwrap();
        assert!((pred[0] - 0.0).abs() < 1e-10);
        assert!((pred[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gaussian_nb_proba() {
        let x = vec![0.0, 0.0, 10.0, 10.0];
        let y = vec![0.0, 1.0];

        let mut nb = GaussianNB::new();
        nb.fit(&x, 2, &y).unwrap();

        let proba = nb.predict_proba(&[0.0, 0.0], 2).unwrap();
        assert_eq!(proba.len(), 2);
        assert!(proba[0] > proba[1]); // should be more likely class 0
    }

    #[test]
    fn test_gaussian_nb_not_fitted() {
        let nb = GaussianNB::new();
        assert!(nb.predict(&[1.0], 1).is_err());
    }
}
