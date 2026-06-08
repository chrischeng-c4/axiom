//! DBSCAN density-based clustering.

use super::error::{MlError, Result};

/// DBSCAN clustering algorithm.
#[derive(Debug, Clone)]
pub struct DBSCAN {
    pub eps: f64,
    pub min_samples: usize,
    /// Cluster labels (-1 = noise).
    pub labels: Option<Vec<i32>>,
}

impl DBSCAN {
    pub fn new(eps: f64, min_samples: usize) -> Self {
        Self {
            eps,
            min_samples,
            labels: None,
        }
    }

    /// Fit the model to data. `x` is row-major (n_samples × n_features).
    pub fn fit(&mut self, x: &[f64], n_features: usize) -> Result<()> {
        if self.eps <= 0.0 {
            return Err(MlError::InvalidParameter("eps must be > 0".into()));
        }
        let n_samples = x.len() / n_features;
        let mut labels = vec![-1i32; n_samples];
        let mut cluster_id = 0i32;

        for i in 0..n_samples {
            if labels[i] != -1 {
                continue;
            }
            let neighbors = region_query(x, n_features, i, self.eps);
            if neighbors.len() < self.min_samples {
                continue; // noise (will remain -1)
            }

            // Start new cluster
            labels[i] = cluster_id;
            let mut seed_set: Vec<usize> = neighbors.iter().filter(|&&j| j != i).copied().collect();

            let mut idx = 0;
            while idx < seed_set.len() {
                let q = seed_set[idx];
                if labels[q] == -1 {
                    labels[q] = cluster_id; // was noise, now border
                }
                if labels[q] != -1 && labels[q] != cluster_id {
                    idx += 1;
                    continue;
                }
                labels[q] = cluster_id;

                let q_neighbors = region_query(x, n_features, q, self.eps);
                if q_neighbors.len() >= self.min_samples {
                    for &j in &q_neighbors {
                        if labels[j] == -1 || labels[j] == -1 {
                            if !seed_set.contains(&j) {
                                seed_set.push(j);
                            }
                        }
                    }
                }
                idx += 1;
            }
            cluster_id += 1;
        }

        self.labels = Some(labels);
        Ok(())
    }

    /// Number of clusters found (excluding noise).
    pub fn n_clusters(&self) -> usize {
        self.labels
            .as_ref()
            .map(|l| {
                let max = l.iter().max().copied().unwrap_or(-1);
                if max < 0 {
                    0
                } else {
                    max as usize + 1
                }
            })
            .unwrap_or(0)
    }
}

fn region_query(x: &[f64], n_features: usize, idx: usize, eps: f64) -> Vec<usize> {
    let n_samples = x.len() / n_features;
    let point = &x[idx * n_features..(idx + 1) * n_features];
    let eps2 = eps * eps;

    (0..n_samples)
        .filter(|&j| {
            let other = &x[j * n_features..(j + 1) * n_features];
            let d2: f64 = point
                .iter()
                .zip(other.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum();
            d2 <= eps2
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dbscan_two_clusters() {
        let mut x = Vec::new();
        // Cluster 1: near origin
        for i in 0..10 {
            x.push(i as f64 * 0.1);
            x.push(i as f64 * 0.1);
        }
        // Cluster 2: near (10, 10)
        for i in 0..10 {
            x.push(10.0 + i as f64 * 0.1);
            x.push(10.0 + i as f64 * 0.1);
        }

        let mut db = DBSCAN::new(1.0, 3);
        db.fit(&x, 2).unwrap();
        assert_eq!(db.n_clusters(), 2);
    }

    #[test]
    fn test_dbscan_noise() {
        // Points too far apart
        let x = vec![0.0, 0.0, 100.0, 100.0, 200.0, 200.0];
        let mut db = DBSCAN::new(1.0, 2);
        db.fit(&x, 2).unwrap();
        // All should be noise
        let labels = db.labels.as_ref().unwrap();
        assert!(labels.iter().all(|&l| l == -1));
    }

    #[test]
    fn test_dbscan_single_cluster() {
        let x = vec![0.0, 0.0, 0.1, 0.0, 0.0, 0.1, 0.1, 0.1];
        let mut db = DBSCAN::new(0.5, 2);
        db.fit(&x, 2).unwrap();
        assert_eq!(db.n_clusters(), 1);
        let labels = db.labels.as_ref().unwrap();
        assert!(labels.iter().all(|&l| l == 0));
    }
}
