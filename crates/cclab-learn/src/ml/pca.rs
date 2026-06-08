//! Principal Component Analysis (PCA) via power iteration.

use super::error::{MlError, Result};

/// PCA model.
#[derive(Debug, Clone)]
pub struct PCA {
    pub n_components: usize,
    pub components: Option<Vec<f64>>,
    pub explained_variance: Option<Vec<f64>>,
    pub mean: Option<Vec<f64>>,
    n_features: usize,
}

impl PCA {
    pub fn new(n_components: usize) -> Self {
        Self {
            n_components,
            components: None,
            explained_variance: None,
            mean: None,
            n_features: 0,
        }
    }

    pub fn fit(&mut self, x: &[f64], n_features: usize) -> Result<()> {
        let n_samples = x.len() / n_features;
        if n_samples < 2 {
            return Err(MlError::InvalidParameter("need at least 2 samples".into()));
        }
        if self.n_components > n_features {
            return Err(MlError::InvalidParameter(format!(
                "n_components ({}) > n_features ({})",
                self.n_components, n_features
            )));
        }

        self.n_features = n_features;

        // Compute mean
        let mut mean = vec![0.0; n_features];
        for i in 0..n_samples {
            for j in 0..n_features {
                mean[j] += x[i * n_features + j];
            }
        }
        for m in &mut mean {
            *m /= n_samples as f64;
        }

        // Center data
        let mut centered = x.to_vec();
        for i in 0..n_samples {
            for j in 0..n_features {
                centered[i * n_features + j] -= mean[j];
            }
        }

        // Compute covariance matrix (n_features × n_features)
        let mut cov = vec![0.0; n_features * n_features];
        for i in 0..n_samples {
            let row = &centered[i * n_features..(i + 1) * n_features];
            for j in 0..n_features {
                for k in j..n_features {
                    let val = row[j] * row[k];
                    cov[j * n_features + k] += val;
                    if j != k {
                        cov[k * n_features + j] += val;
                    }
                }
            }
        }
        let denom = (n_samples - 1) as f64;
        for v in &mut cov {
            *v /= denom;
        }

        // Extract top-k eigenvectors via power iteration with deflation
        let mut components = Vec::with_capacity(self.n_components * n_features);
        let mut variances = Vec::with_capacity(self.n_components);

        let mut work_cov = cov;

        for _ in 0..self.n_components {
            let (eigval, eigvec) = power_iteration(&work_cov, n_features, 200);
            variances.push(eigval);
            components.extend_from_slice(&eigvec);

            // Deflate: C = C - lambda * v * v^T
            for j in 0..n_features {
                for k in 0..n_features {
                    work_cov[j * n_features + k] -= eigval * eigvec[j] * eigvec[k];
                }
            }
        }

        self.components = Some(components);
        self.explained_variance = Some(variances);
        self.mean = Some(mean);
        Ok(())
    }

    /// Transform data into the PCA space.
    pub fn transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let components = self.components.as_ref().ok_or(MlError::NotFitted)?;
        let mean = self.mean.as_ref().ok_or(MlError::NotFitted)?;
        let n_samples = x.len() / n_features;

        let mut result = Vec::with_capacity(n_samples * self.n_components);
        for i in 0..n_samples {
            for c in 0..self.n_components {
                let component = &components[c * n_features..(c + 1) * n_features];
                let val: f64 = (0..n_features)
                    .map(|j| (x[i * n_features + j] - mean[j]) * component[j])
                    .sum();
                result.push(val);
            }
        }
        Ok(result)
    }

    /// Fit and transform in one step.
    pub fn fit_transform(&mut self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        self.fit(x, n_features)?;
        self.transform(x, n_features)
    }

    /// Inverse transform: project back to original space.
    pub fn inverse_transform(&self, x_transformed: &[f64]) -> Result<Vec<f64>> {
        let components = self.components.as_ref().ok_or(MlError::NotFitted)?;
        let mean = self.mean.as_ref().ok_or(MlError::NotFitted)?;
        let n_features = self.n_features;
        let n_samples = x_transformed.len() / self.n_components;

        let mut result = Vec::with_capacity(n_samples * n_features);
        for i in 0..n_samples {
            for j in 0..n_features {
                let mut val = mean[j];
                for c in 0..self.n_components {
                    val +=
                        x_transformed[i * self.n_components + c] * components[c * n_features + j];
                }
                result.push(val);
            }
        }
        Ok(result)
    }

    /// Explained variance ratio.
    pub fn explained_variance_ratio(&self) -> Option<Vec<f64>> {
        let variances = self.explained_variance.as_ref()?;
        let total: f64 = variances.iter().sum();
        if total < 1e-15 {
            return Some(vec![0.0; variances.len()]);
        }
        Some(variances.iter().map(|v| v / total).collect())
    }
}

/// Power iteration to find the dominant eigenvector.
fn power_iteration(matrix: &[f64], n: usize, max_iter: usize) -> (f64, Vec<f64>) {
    // Initialize with [1, 0, 0, ...] then normalize
    let mut v = vec![0.0; n];
    v[0] = 1.0;
    // Use a more robust initialization
    for i in 0..n {
        v[i] = 1.0 / (n as f64).sqrt();
    }

    let mut eigenvalue = 0.0;

    for _ in 0..max_iter {
        // w = M * v
        let mut w = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                w[i] += matrix[i * n + j] * v[j];
            }
        }

        // Compute eigenvalue (Rayleigh quotient)
        let new_eigenvalue: f64 = w.iter().zip(v.iter()).map(|(wi, vi)| wi * vi).sum();

        // Normalize w
        let norm: f64 = w.iter().map(|x| x * x).sum::<f64>().sqrt();
        if norm < 1e-15 {
            break;
        }
        for val in &mut w {
            *val /= norm;
        }

        // Check convergence
        if (new_eigenvalue - eigenvalue).abs() < 1e-12 {
            eigenvalue = new_eigenvalue;
            v = w;
            break;
        }

        eigenvalue = new_eigenvalue;
        v = w;
    }

    (eigenvalue, v)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pca_basic() {
        // Data strongly correlated along first axis
        let x = vec![1.0, 2.0, 2.0, 4.0, 3.0, 6.0, 4.0, 8.0, 5.0, 10.0];
        let mut pca = PCA::new(1);
        pca.fit(&x, 2).unwrap();

        assert_eq!(pca.components.as_ref().unwrap().len(), 2); // 1 component × 2 features
        assert!(pca.explained_variance.as_ref().unwrap()[0] > 0.0);
    }

    #[test]
    fn test_transform() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut pca = PCA::new(1);
        let transformed = pca.fit_transform(&x, 2).unwrap();
        assert_eq!(transformed.len(), 4); // 4 samples × 1 component
    }

    #[test]
    fn test_inverse_transform() {
        // Data with two real principal components (not perfectly correlated)
        let x = vec![
            1.0, 5.0, 2.0, 3.0, 3.0, 6.0, 4.0, 2.0, 5.0, 7.0, 6.0, 1.0, 7.0, 8.0, 8.0, 4.0, 9.0,
            9.0, 10.0, 3.0,
        ];
        let mut pca = PCA::new(2);
        let transformed = pca.fit_transform(&x, 2).unwrap();
        let recovered = pca.inverse_transform(&transformed).unwrap();

        // With 2 components and 2 features, should recover well
        for (orig, rec) in x.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 0.5, "orig={}, rec={}", orig, rec);
        }
    }

    #[test]
    fn test_explained_variance_ratio() {
        let x = vec![1.0, 0.0, 2.0, 0.1, 3.0, -0.1, 4.0, 0.0, 5.0, 0.1];
        let mut pca = PCA::new(2);
        pca.fit(&x, 2).unwrap();
        let ratio = pca.explained_variance_ratio().unwrap();
        // First component should capture most variance
        assert!(ratio[0] > 0.8);
        assert!((ratio.iter().sum::<f64>() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_too_many_components() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let mut pca = PCA::new(3);
        assert!(pca.fit(&x, 2).is_err());
    }
}
