//! K-Nearest Neighbors classifier and regressor.

use super::error::{MlError, Result};
use super::traits::{Estimator, Predictor};

/// K-Nearest Neighbors classifier.
#[derive(Debug, Clone)]
pub struct KNeighborsClassifier {
    pub k: usize,
    x_train: Option<Vec<f64>>,
    y_train: Option<Vec<f64>>,
    n_features: usize,
}

impl KNeighborsClassifier {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            x_train: None,
            y_train: None,
            n_features: 0,
        }
    }
}

impl Estimator for KNeighborsClassifier {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples != y.len() {
            return Err(MlError::ShapeMismatch("x rows != y length".into()));
        }
        if n_samples < self.k {
            return Err(MlError::InvalidParameter(format!(
                "n_samples ({n_samples}) < k ({})",
                self.k
            )));
        }
        self.x_train = Some(x.to_vec());
        self.y_train = Some(y.to_vec());
        self.n_features = n_features;
        Ok(())
    }
}

impl Predictor for KNeighborsClassifier {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let x_train = self.x_train.as_ref().ok_or(MlError::NotFitted)?;
        let y_train = self.y_train.as_ref().ok_or(MlError::NotFitted)?;
        let n_train = x_train.len() / n_features;
        let n_test = x.len() / n_features;

        let mut predictions = Vec::with_capacity(n_test);
        for i in 0..n_test {
            let test_row = &x[i * n_features..(i + 1) * n_features];
            // Compute distances to all training points
            let mut dists: Vec<(f64, f64)> = (0..n_train)
                .map(|j| {
                    let train_row = &x_train[j * n_features..(j + 1) * n_features];
                    let d: f64 = test_row
                        .iter()
                        .zip(train_row.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    (d, y_train[j])
                })
                .collect();
            dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

            // Majority vote among k nearest
            let neighbors = &dists[..self.k];
            let pred = majority_vote(neighbors.iter().map(|&(_, y)| y));
            predictions.push(pred);
        }
        Ok(predictions)
    }
}

/// K-Nearest Neighbors regressor.
#[derive(Debug, Clone)]
pub struct KNeighborsRegressor {
    pub k: usize,
    x_train: Option<Vec<f64>>,
    y_train: Option<Vec<f64>>,
    n_features: usize,
}

impl KNeighborsRegressor {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            x_train: None,
            y_train: None,
            n_features: 0,
        }
    }
}

impl Estimator for KNeighborsRegressor {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        self.x_train = Some(x.to_vec());
        self.y_train = Some(y.to_vec());
        self.n_features = n_features;
        Ok(())
    }
}

impl Predictor for KNeighborsRegressor {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let x_train = self.x_train.as_ref().ok_or(MlError::NotFitted)?;
        let y_train = self.y_train.as_ref().ok_or(MlError::NotFitted)?;
        let n_train = x_train.len() / n_features;
        let n_test = x.len() / n_features;

        let mut predictions = Vec::with_capacity(n_test);
        for i in 0..n_test {
            let test_row = &x[i * n_features..(i + 1) * n_features];
            let mut dists: Vec<(f64, f64)> = (0..n_train)
                .map(|j| {
                    let train_row = &x_train[j * n_features..(j + 1) * n_features];
                    let d: f64 = test_row
                        .iter()
                        .zip(train_row.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum();
                    (d, y_train[j])
                })
                .collect();
            dists.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            let mean: f64 = dists[..self.k].iter().map(|&(_, y)| y).sum::<f64>() / self.k as f64;
            predictions.push(mean);
        }
        Ok(predictions)
    }
}

fn majority_vote(values: impl Iterator<Item = f64>) -> f64 {
    let mut counts: Vec<(f64, usize)> = Vec::new();
    for v in values {
        if let Some(entry) = counts.iter_mut().find(|(c, _)| (*c - v).abs() < 1e-10) {
            entry.1 += 1;
        } else {
            counts.push((v, 1));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts[0].0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_knn_classifier() {
        // Two classes: class 0 near origin, class 1 near (10,10)
        let x = vec![0.0, 0.0, 0.1, 0.1, 10.0, 10.0, 10.1, 10.1];
        let y = vec![0.0, 0.0, 1.0, 1.0];

        let mut knn = KNeighborsClassifier::new(2);
        knn.fit(&x, 2, &y).unwrap();

        let pred = knn.predict(&[0.2, 0.2, 9.8, 9.8], 2).unwrap();
        assert!((pred[0] - 0.0).abs() < 1e-10);
        assert!((pred[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_knn_regressor() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0]; // y = 2x

        let mut knn = KNeighborsRegressor::new(2);
        knn.fit(&x, 1, &y).unwrap();

        let pred = knn.predict(&[2.5], 1).unwrap();
        // Average of y[1]=4.0 and y[2]=6.0
        assert!((pred[0] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_knn_not_fitted() {
        let knn = KNeighborsClassifier::new(3);
        assert!(knn.predict(&[1.0], 1).is_err());
    }
}
