//! Ensemble methods: Random Forest and Gradient Boosting (stub with decision stumps).

use super::error::{MlError, Result};
use super::traits::{Estimator, Predictor};

/// Random Forest classifier (bagging of decision stumps).
#[derive(Debug, Clone)]
pub struct RandomForestClassifier {
    pub n_estimators: usize,
    pub max_depth: usize,
    seed: u64,
    stumps: Option<Vec<Stump>>,
}

/// A simple decision stump (1-level split).
#[derive(Debug, Clone)]
struct Stump {
    feature: usize,
    threshold: f64,
    left_val: f64,
    right_val: f64,
}

impl RandomForestClassifier {
    pub fn new(n_estimators: usize) -> Self {
        Self {
            n_estimators,
            max_depth: 5,
            seed: 42,
            stumps: None,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }
}

impl Estimator for RandomForestClassifier {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples != y.len() {
            return Err(MlError::ShapeMismatch("x rows != y length".into()));
        }

        let mut rng = self.seed;
        let mut stumps = Vec::with_capacity(self.n_estimators);

        for _ in 0..self.n_estimators {
            // Bootstrap sample
            let indices: Vec<usize> = (0..n_samples)
                .map(|_| (lcg_next(&mut rng) % n_samples as u64) as usize)
                .collect();

            // Find best split across random subset of features
            let n_try = (n_features as f64).sqrt().ceil() as usize;
            let mut best_stump: Option<Stump> = None;
            let mut best_gini = f64::INFINITY;

            for _ in 0..n_try {
                let feat = (lcg_next(&mut rng) % n_features as u64) as usize;

                // Try median as threshold
                let mut vals: Vec<f64> =
                    indices.iter().map(|&i| x[i * n_features + feat]).collect();
                vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let threshold = vals[vals.len() / 2];

                // Compute Gini for this split
                let (left_val, right_val, gini) =
                    compute_split_gini(&indices, x, n_features, y, feat, threshold);

                if gini < best_gini {
                    best_gini = gini;
                    best_stump = Some(Stump {
                        feature: feat,
                        threshold,
                        left_val,
                        right_val,
                    });
                }
            }

            if let Some(stump) = best_stump {
                stumps.push(stump);
            }
        }

        self.stumps = Some(stumps);
        Ok(())
    }
}

impl Predictor for RandomForestClassifier {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let stumps = self.stumps.as_ref().ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;
        let mut predictions = Vec::with_capacity(n_samples);

        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let mut votes: Vec<f64> = Vec::new();
            for stump in stumps {
                let val = if row[stump.feature] <= stump.threshold {
                    stump.left_val
                } else {
                    stump.right_val
                };
                votes.push(val);
            }
            // Majority vote
            predictions.push(majority_vote(&votes));
        }
        Ok(predictions)
    }
}

/// Gradient Boosting classifier (additive model of decision stumps).
#[derive(Debug, Clone)]
pub struct GradientBoostingClassifier {
    pub n_estimators: usize,
    pub learning_rate: f64,
    stumps: Option<Vec<(f64, Stump)>>,
    base_prediction: f64,
}

impl GradientBoostingClassifier {
    pub fn new(n_estimators: usize) -> Self {
        Self {
            n_estimators,
            learning_rate: 0.1,
            stumps: None,
            base_prediction: 0.0,
        }
    }

    pub fn with_learning_rate(mut self, lr: f64) -> Self {
        self.learning_rate = lr;
        self
    }
}

impl Estimator for GradientBoostingClassifier {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples != y.len() {
            return Err(MlError::ShapeMismatch("x rows != y length".into()));
        }

        // Convert labels to {-1, 1}
        let y_signed: Vec<f64> = y
            .iter()
            .map(|&v| if v > 0.5 { 1.0 } else { -1.0 })
            .collect();

        // Base prediction = mean
        self.base_prediction = y_signed.iter().sum::<f64>() / n_samples as f64;
        let mut predictions = vec![self.base_prediction; n_samples];
        let mut stumps_lr = Vec::with_capacity(self.n_estimators);

        for _ in 0..self.n_estimators {
            // Compute pseudo-residuals
            let residuals: Vec<f64> = y_signed
                .iter()
                .zip(predictions.iter())
                .map(|(&yi, &pi)| yi - pi.tanh())
                .collect();

            // Fit stump to residuals
            let mut best_stump: Option<Stump> = None;
            let mut best_mse = f64::INFINITY;

            for feat in 0..n_features {
                let mut vals: Vec<(f64, usize)> = (0..n_samples)
                    .map(|i| (x[i * n_features + feat], i))
                    .collect();
                vals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

                for split_idx in 1..n_samples {
                    let threshold = (vals[split_idx - 1].0 + vals[split_idx].0) / 2.0;
                    let left_mean = vals[..split_idx]
                        .iter()
                        .map(|&(_, i)| residuals[i])
                        .sum::<f64>()
                        / split_idx as f64;
                    let right_mean = vals[split_idx..]
                        .iter()
                        .map(|&(_, i)| residuals[i])
                        .sum::<f64>()
                        / (n_samples - split_idx) as f64;

                    let mse: f64 = vals[..split_idx]
                        .iter()
                        .map(|&(_, i)| (residuals[i] - left_mean).powi(2))
                        .sum::<f64>()
                        + vals[split_idx..]
                            .iter()
                            .map(|&(_, i)| (residuals[i] - right_mean).powi(2))
                            .sum::<f64>();

                    if mse < best_mse {
                        best_mse = mse;
                        best_stump = Some(Stump {
                            feature: feat,
                            threshold,
                            left_val: left_mean,
                            right_val: right_mean,
                        });
                    }
                }
            }

            if let Some(stump) = best_stump {
                for i in 0..n_samples {
                    let row = &x[i * n_features..(i + 1) * n_features];
                    let val = if row[stump.feature] <= stump.threshold {
                        stump.left_val
                    } else {
                        stump.right_val
                    };
                    predictions[i] += self.learning_rate * val;
                }
                stumps_lr.push((self.learning_rate, stump));
            }
        }

        self.stumps = Some(stumps_lr);
        Ok(())
    }
}

impl Predictor for GradientBoostingClassifier {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let stumps = self.stumps.as_ref().ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;
        let mut predictions = Vec::with_capacity(n_samples);

        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let mut score = self.base_prediction;
            for (lr, stump) in stumps {
                let val = if row[stump.feature] <= stump.threshold {
                    stump.left_val
                } else {
                    stump.right_val
                };
                score += lr * val;
            }
            predictions.push(if score >= 0.0 { 1.0 } else { 0.0 });
        }
        Ok(predictions)
    }
}

fn compute_split_gini(
    indices: &[usize],
    x: &[f64],
    n_features: usize,
    y: &[f64],
    feature: usize,
    threshold: f64,
) -> (f64, f64, f64) {
    let mut left_counts: Vec<(f64, usize)> = Vec::new();
    let mut right_counts: Vec<(f64, usize)> = Vec::new();
    let mut n_left = 0usize;
    let mut n_right = 0usize;

    for &i in indices {
        let val = x[i * n_features + feature];
        let label = y[i];
        if val <= threshold {
            n_left += 1;
            if let Some(e) = left_counts
                .iter_mut()
                .find(|(l, _)| (*l - label).abs() < 1e-10)
            {
                e.1 += 1;
            } else {
                left_counts.push((label, 1));
            }
        } else {
            n_right += 1;
            if let Some(e) = right_counts
                .iter_mut()
                .find(|(l, _)| (*l - label).abs() < 1e-10)
            {
                e.1 += 1;
            } else {
                right_counts.push((label, 1));
            }
        }
    }

    let gini_left = gini_impurity(&left_counts, n_left);
    let gini_right = gini_impurity(&right_counts, n_right);
    let total = n_left + n_right;
    let gini = (n_left as f64 * gini_left + n_right as f64 * gini_right) / total as f64;

    let left_val = left_counts
        .iter()
        .max_by_key(|c| c.1)
        .map(|c| c.0)
        .unwrap_or(0.0);
    let right_val = right_counts
        .iter()
        .max_by_key(|c| c.1)
        .map(|c| c.0)
        .unwrap_or(0.0);

    (left_val, right_val, gini)
}

fn gini_impurity(counts: &[(f64, usize)], total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    }
    1.0 - counts
        .iter()
        .map(|&(_, c)| (c as f64 / total as f64).powi(2))
        .sum::<f64>()
}

fn majority_vote(votes: &[f64]) -> f64 {
    let mut counts: Vec<(f64, usize)> = Vec::new();
    for &v in votes {
        if let Some(e) = counts.iter_mut().find(|(c, _)| (*c - v).abs() < 1e-10) {
            e.1 += 1;
        } else {
            counts.push((v, 1));
        }
    }
    counts.sort_by(|a, b| b.1.cmp(&a.1));
    counts[0].0
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_forest_basic() {
        let x = vec![
            0.0, 0.0, 0.1, 0.1, 0.2, 0.0, // class 0
            5.0, 5.0, 5.1, 5.1, 5.2, 5.0, // class 1
        ];
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

        let mut rf = RandomForestClassifier::new(100);
        rf.fit(&x, 2, &y).unwrap();

        let pred = rf.predict(&[0.0, 0.0, 5.0, 5.0], 2).unwrap();
        assert!((pred[0] - 0.0).abs() < 1e-10);
        assert!((pred[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_gradient_boosting_basic() {
        let x = vec![
            0.0, 0.0, 0.1, 0.1, 0.2, 0.0, // class 0
            5.0, 5.0, 5.1, 5.1, 5.2, 5.0, // class 1
        ];
        let y = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];

        let mut gb = GradientBoostingClassifier::new(50).with_learning_rate(0.1);
        gb.fit(&x, 2, &y).unwrap();

        let pred = gb.predict(&[0.0, 0.0, 5.0, 5.0], 2).unwrap();
        assert!((pred[0] - 0.0).abs() < 1e-10);
        assert!((pred[1] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_rf_not_fitted() {
        let rf = RandomForestClassifier::new(10);
        assert!(rf.predict(&[1.0], 1).is_err());
    }
}
