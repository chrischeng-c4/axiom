//! K-means clustering with k-means++ initialization.

use super::error::{MlError, Result};

/// K-means clustering.
#[derive(Debug, Clone)]
pub struct KMeans {
    pub k: usize,
    pub max_iter: usize,
    pub tol: f64,
    pub centroids: Option<Vec<f64>>,
    pub labels: Option<Vec<usize>>,
    pub inertia: Option<f64>,
    n_features: usize,
    seed: u64,
}

impl KMeans {
    pub fn new(k: usize) -> Self {
        Self {
            k,
            max_iter: 300,
            tol: 1e-4,
            centroids: None,
            labels: None,
            inertia: None,
            n_features: 0,
            seed: 42,
        }
    }

    pub fn with_max_iter(mut self, max_iter: usize) -> Self {
        self.max_iter = max_iter;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn fit(&mut self, x: &[f64], n_features: usize) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples < self.k {
            return Err(MlError::InvalidParameter(format!(
                "n_samples ({}) < k ({})",
                n_samples, self.k
            )));
        }

        self.n_features = n_features;

        // K-means++ initialization
        let mut centroids = kmeans_plus_plus(x, n_features, self.k, self.seed);
        let mut labels = vec![0usize; n_samples];

        for _iter in 0..self.max_iter {
            // Assign labels
            for i in 0..n_samples {
                let row = &x[i * n_features..(i + 1) * n_features];
                labels[i] = nearest_centroid(row, &centroids, n_features);
            }

            // Update centroids
            let mut new_centroids = vec![0.0; self.k * n_features];
            let mut counts = vec![0usize; self.k];
            for i in 0..n_samples {
                let label = labels[i];
                counts[label] += 1;
                for j in 0..n_features {
                    new_centroids[label * n_features + j] += x[i * n_features + j];
                }
            }
            for c in 0..self.k {
                if counts[c] > 0 {
                    for j in 0..n_features {
                        new_centroids[c * n_features + j] /= counts[c] as f64;
                    }
                }
            }

            // Check convergence
            let shift: f64 = centroids
                .iter()
                .zip(new_centroids.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();

            centroids = new_centroids;

            if shift < self.tol {
                break;
            }
        }

        // Compute inertia
        let mut inertia = 0.0;
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let c = labels[i];
            let centroid = &centroids[c * n_features..(c + 1) * n_features];
            inertia += row
                .iter()
                .zip(centroid.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>();
        }

        self.centroids = Some(centroids);
        self.labels = Some(labels);
        self.inertia = Some(inertia);
        Ok(())
    }

    /// Predict cluster labels for new data.
    pub fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<usize>> {
        let centroids = self.centroids.as_ref().ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;
        let mut labels = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            labels.push(nearest_centroid(row, centroids, n_features));
        }
        Ok(labels)
    }
}

/// K-means++ initialization.
fn kmeans_plus_plus(x: &[f64], n_features: usize, k: usize, seed: u64) -> Vec<f64> {
    let n_samples = x.len() / n_features;
    let mut rng = seed;
    let mut centroids = Vec::with_capacity(k * n_features);

    // Pick first centroid randomly
    let idx = lcg_next(&mut rng) % n_samples as u64;
    centroids.extend_from_slice(&x[idx as usize * n_features..(idx as usize + 1) * n_features]);

    for _ in 1..k {
        // Compute distances to nearest centroid
        let n_centroids = centroids.len() / n_features;
        let mut dists = Vec::with_capacity(n_samples);
        let mut total_dist = 0.0;

        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let min_dist = (0..n_centroids)
                .map(|c| {
                    let centroid = &centroids[c * n_features..(c + 1) * n_features];
                    row.iter()
                        .zip(centroid.iter())
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f64>()
                })
                .fold(f64::INFINITY, f64::min);
            dists.push(min_dist);
            total_dist += min_dist;
        }

        // Weighted random selection
        if total_dist < 1e-15 {
            // All points are the same
            centroids.extend_from_slice(&x[0..n_features]);
            continue;
        }

        let threshold = (lcg_next(&mut rng) as f64 / u64::MAX as f64) * total_dist;
        let mut cumsum = 0.0;
        let mut chosen = 0;
        for (i, &d) in dists.iter().enumerate() {
            cumsum += d;
            if cumsum >= threshold {
                chosen = i;
                break;
            }
        }
        centroids.extend_from_slice(&x[chosen * n_features..(chosen + 1) * n_features]);
    }

    centroids
}

fn nearest_centroid(point: &[f64], centroids: &[f64], n_features: usize) -> usize {
    let k = centroids.len() / n_features;
    let mut best = 0;
    let mut best_dist = f64::INFINITY;
    for c in 0..k {
        let centroid = &centroids[c * n_features..(c + 1) * n_features];
        let dist: f64 = point
            .iter()
            .zip(centroid.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum();
        if dist < best_dist {
            best_dist = dist;
            best = c;
        }
    }
    best
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
    fn test_two_clusters() {
        // Two well-separated clusters
        let mut x = Vec::new();
        for _ in 0..20 {
            x.push(0.0);
            x.push(0.0);
        }
        for _ in 0..20 {
            x.push(10.0);
            x.push(10.0);
        }

        let mut km = KMeans::new(2);
        km.fit(&x, 2).unwrap();

        let labels = km.labels.as_ref().unwrap();
        // All points in the first group should have the same label
        let label_a = labels[0];
        assert!(labels[..20].iter().all(|&l| l == label_a));
        // All points in the second group should have a different label
        let label_b = labels[20];
        assert!(labels[20..].iter().all(|&l| l == label_b));
        assert_ne!(label_a, label_b);
    }

    #[test]
    fn test_predict() {
        let x = vec![0.0, 0.0, 10.0, 10.0];
        let mut km = KMeans::new(2);
        km.fit(&x, 2).unwrap();

        let new_x = vec![1.0, 1.0, 9.0, 9.0];
        let labels = km.predict(&new_x, 2).unwrap();
        assert_ne!(labels[0], labels[1]);
    }

    #[test]
    fn test_inertia() {
        let x = vec![0.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0, 10.0];
        let mut km = KMeans::new(2);
        km.fit(&x, 2).unwrap();
        assert!(km.inertia.unwrap() < 1.0);
    }

    #[test]
    fn test_not_fitted() {
        let km = KMeans::new(2);
        assert!(km.predict(&[1.0], 1).is_err());
    }

    #[test]
    fn test_k_greater_than_n() {
        let x = vec![1.0, 2.0];
        let mut km = KMeans::new(3);
        assert!(km.fit(&x, 1).is_err());
    }
}
