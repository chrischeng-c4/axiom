//! Decision tree (CART) for classification and regression.

use super::error::{MlError, Result};

/// Split criterion for decision trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Criterion {
    Gini,
    Entropy,
    Mse,
}

/// A node in the decision tree.
#[derive(Debug, Clone)]
enum Node {
    Leaf {
        value: f64,
    },
    Split {
        feature: usize,
        threshold: f64,
        left: Box<Node>,
        right: Box<Node>,
    },
}

/// CART decision tree.
#[derive(Debug, Clone)]
pub struct DecisionTree {
    root: Option<Node>,
    pub max_depth: usize,
    pub min_samples_split: usize,
    pub criterion: Criterion,
}

impl Default for DecisionTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DecisionTree {
    pub fn new() -> Self {
        Self {
            root: None,
            max_depth: 10,
            min_samples_split: 2,
            criterion: Criterion::Gini,
        }
    }

    pub fn with_max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    pub fn with_criterion(mut self, criterion: Criterion) -> Self {
        self.criterion = criterion;
        self
    }

    pub fn with_min_samples_split(mut self, min_samples: usize) -> Self {
        self.min_samples_split = min_samples;
        self
    }

    pub fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n = y.len();
        if x.len() != n * n_features {
            return Err(MlError::ShapeMismatch(format!(
                "x len {} != n*features {}",
                x.len(),
                n * n_features
            )));
        }
        let indices: Vec<usize> = (0..n).collect();
        self.root = Some(self.build_tree(x, n_features, y, &indices, 0));
        Ok(())
    }

    pub fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let root = self.root.as_ref().ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;
        let mut preds = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            preds.push(predict_single(root, row));
        }
        Ok(preds)
    }

    fn build_tree(
        &self,
        x: &[f64],
        n_features: usize,
        y: &[f64],
        indices: &[usize],
        depth: usize,
    ) -> Node {
        // Stopping conditions
        if indices.len() < self.min_samples_split || depth >= self.max_depth {
            return Node::Leaf {
                value: leaf_value(y, indices, self.criterion),
            };
        }

        // Check if all labels are the same
        let first_y = y[indices[0]];
        if indices.iter().all(|&i| (y[i] - first_y).abs() < 1e-15) {
            return Node::Leaf { value: first_y };
        }

        // Find best split
        let (best_feature, best_threshold, best_score) =
            self.find_best_split(x, n_features, y, indices);

        if best_score.is_infinite() {
            return Node::Leaf {
                value: leaf_value(y, indices, self.criterion),
            };
        }

        // Split indices
        let (left_idx, right_idx): (Vec<usize>, Vec<usize>) = indices
            .iter()
            .partition(|&&i| x[i * n_features + best_feature] <= best_threshold);

        if left_idx.is_empty() || right_idx.is_empty() {
            return Node::Leaf {
                value: leaf_value(y, indices, self.criterion),
            };
        }

        let left = self.build_tree(x, n_features, y, &left_idx, depth + 1);
        let right = self.build_tree(x, n_features, y, &right_idx, depth + 1);

        Node::Split {
            feature: best_feature,
            threshold: best_threshold,
            left: Box::new(left),
            right: Box::new(right),
        }
    }

    fn find_best_split(
        &self,
        x: &[f64],
        n_features: usize,
        y: &[f64],
        indices: &[usize],
    ) -> (usize, f64, f64) {
        let mut best_feature = 0;
        let mut best_threshold = 0.0;
        let mut best_score = f64::INFINITY;

        for feat in 0..n_features {
            // Collect unique sorted values for this feature
            let mut vals: Vec<f64> = indices.iter().map(|&i| x[i * n_features + feat]).collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            vals.dedup();

            // Try midpoints as thresholds
            for w in vals.windows(2) {
                let threshold = (w[0] + w[1]) / 2.0;

                let mut left_y = Vec::new();
                let mut right_y = Vec::new();
                for &i in indices {
                    if x[i * n_features + feat] <= threshold {
                        left_y.push(y[i]);
                    } else {
                        right_y.push(y[i]);
                    }
                }

                if left_y.is_empty() || right_y.is_empty() {
                    continue;
                }

                let score = weighted_impurity(&left_y, &right_y, self.criterion);
                if score < best_score {
                    best_score = score;
                    best_feature = feat;
                    best_threshold = threshold;
                }
            }
        }

        (best_feature, best_threshold, best_score)
    }
}

fn predict_single(node: &Node, row: &[f64]) -> f64 {
    match node {
        Node::Leaf { value } => *value,
        Node::Split {
            feature,
            threshold,
            left,
            right,
        } => {
            if row[*feature] <= *threshold {
                predict_single(left, row)
            } else {
                predict_single(right, row)
            }
        }
    }
}

fn leaf_value(y: &[f64], indices: &[usize], criterion: Criterion) -> f64 {
    match criterion {
        Criterion::Mse => {
            // Regression: mean
            let sum: f64 = indices.iter().map(|&i| y[i]).sum();
            sum / indices.len() as f64
        }
        Criterion::Gini | Criterion::Entropy => {
            // Classification: majority class
            let mut counts = std::collections::HashMap::new();
            for &i in indices {
                let class = y[i] as i64;
                *counts.entry(class).or_insert(0usize) += 1;
            }
            counts
                .into_iter()
                .max_by_key(|&(_, c)| c)
                .map(|(class, _)| class as f64)
                .unwrap_or(0.0)
        }
    }
}

fn impurity(y: &[f64], criterion: Criterion) -> f64 {
    let n = y.len() as f64;
    if n < 1.0 {
        return 0.0;
    }
    match criterion {
        Criterion::Gini => {
            let mut counts = std::collections::HashMap::new();
            for &val in y {
                *counts.entry(val as i64).or_insert(0usize) += 1;
            }
            1.0 - counts
                .values()
                .map(|&c| (c as f64 / n).powi(2))
                .sum::<f64>()
        }
        Criterion::Entropy => {
            let mut counts = std::collections::HashMap::new();
            for &val in y {
                *counts.entry(val as i64).or_insert(0usize) += 1;
            }
            -counts
                .values()
                .map(|&c| {
                    let p = c as f64 / n;
                    if p > 0.0 {
                        p * p.ln()
                    } else {
                        0.0
                    }
                })
                .sum::<f64>()
        }
        Criterion::Mse => {
            let mean = y.iter().sum::<f64>() / n;
            y.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n
        }
    }
}

fn weighted_impurity(left: &[f64], right: &[f64], criterion: Criterion) -> f64 {
    let total = (left.len() + right.len()) as f64;
    let w_left = left.len() as f64 / total;
    let w_right = right.len() as f64 / total;
    w_left * impurity(left, criterion) + w_right * impurity(right, criterion)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification() {
        // XOR-like pattern with depth
        let x = vec![0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0];
        let y = vec![0.0, 1.0, 1.0, 0.0];

        let mut tree = DecisionTree::new()
            .with_max_depth(5)
            .with_criterion(Criterion::Gini);
        tree.fit(&x, 2, &y).unwrap();
        let preds = tree.predict(&x, 2).unwrap();
        // Should perfectly fit XOR with enough depth
        for (p, t) in preds.iter().zip(y.iter()) {
            assert!((p - t).abs() < 1e-10, "pred={}, true={}", p, t);
        }
    }

    #[test]
    fn test_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0];

        let mut tree = DecisionTree::new()
            .with_max_depth(5)
            .with_criterion(Criterion::Mse);
        tree.fit(&x, 1, &y).unwrap();
        let preds = tree.predict(&x, 1).unwrap();

        // Should approximate the linear relationship
        for (p, t) in preds.iter().zip(y.iter()) {
            assert!((p - t).abs() < 3.0, "pred={}, true={}", p, t);
        }
    }

    #[test]
    fn test_entropy_criterion() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 0.0, 1.0, 1.0];
        let mut tree = DecisionTree::new().with_criterion(Criterion::Entropy);
        tree.fit(&x, 1, &y).unwrap();
        let preds = tree.predict(&x, 1).unwrap();
        assert!((preds[0] - 0.0).abs() < 1e-10);
        assert!((preds[3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_not_fitted() {
        let tree = DecisionTree::new();
        assert!(tree.predict(&[1.0], 1).is_err());
    }
}
