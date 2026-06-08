//! Linear regression models: OLS, Ridge, Lasso.

use super::error::{MlError, Result};
use super::traits::{Estimator, Predictor};

// ============================================================================
// OLS (Ordinary Least Squares)
// ============================================================================

/// Ordinary Least Squares linear regression.
///
/// Fits y = X * w + b using the normal equations: w = (X^T X)^{-1} X^T y.
#[derive(Debug, Clone)]
pub struct LinearRegression {
    pub weights: Option<Vec<f64>>,
    pub intercept: Option<f64>,
    pub fit_intercept: bool,
}

impl Default for LinearRegression {
    fn default() -> Self {
        Self::new()
    }
}

impl LinearRegression {
    pub fn new() -> Self {
        Self {
            weights: None,
            intercept: None,
            fit_intercept: true,
        }
    }

    pub fn with_intercept(mut self, fit_intercept: bool) -> Self {
        self.fit_intercept = fit_intercept;
        self
    }
}

impl Estimator for LinearRegression {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = y.len();
        if x.len() != n_samples * n_features {
            return Err(MlError::ShapeMismatch(format!(
                "x has {} elements, expected {}×{}={}",
                x.len(),
                n_samples,
                n_features,
                n_samples * n_features
            )));
        }

        // Build augmented matrix if fit_intercept
        let p = if self.fit_intercept {
            n_features + 1
        } else {
            n_features
        };

        // Compute X^T X (p×p) and X^T y (p×1)
        let mut xtx = vec![0.0; p * p];
        let mut xty = vec![0.0; p];

        for i in 0..n_samples {
            let row_start = i * n_features;
            for j in 0..n_features {
                let xj = x[row_start + j];
                xty[j] += xj * y[i];
                for k in 0..n_features {
                    xtx[j * p + k] += xj * x[row_start + k];
                }
                if self.fit_intercept {
                    xtx[j * p + (p - 1)] += xj;
                    xtx[(p - 1) * p + j] += xj;
                }
            }
            if self.fit_intercept {
                xty[p - 1] += y[i];
                xtx[(p - 1) * p + (p - 1)] += 1.0;
            }
        }

        // Solve via Gaussian elimination
        let params = solve_linear_system(&xtx, &xty, p)?;

        if self.fit_intercept {
            self.weights = Some(params[..n_features].to_vec());
            self.intercept = Some(params[n_features]);
        } else {
            self.weights = Some(params);
            self.intercept = Some(0.0);
        }

        Ok(())
    }
}

impl Predictor for LinearRegression {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let w = self.weights.as_ref().ok_or(MlError::NotFitted)?;
        let b = self.intercept.unwrap_or(0.0);
        let n_samples = x.len() / n_features;

        let mut preds = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let val: f64 = row
                .iter()
                .zip(w.iter())
                .map(|(xi, wi)| xi * wi)
                .sum::<f64>()
                + b;
            preds.push(val);
        }
        Ok(preds)
    }
}

/// R² score (coefficient of determination).
impl LinearRegression {
    pub fn score(&self, x: &[f64], n_features: usize, y: &[f64]) -> Result<f64> {
        let preds = self.predict(x, n_features)?;
        Ok(r2_score(y, &preds))
    }
}

// ============================================================================
// Ridge Regression
// ============================================================================

/// Ridge regression: OLS with L2 regularization.
///
/// Solves: w = (X^T X + alpha * I)^{-1} X^T y
#[derive(Debug, Clone)]
pub struct RidgeRegression {
    pub weights: Option<Vec<f64>>,
    pub intercept: Option<f64>,
    pub alpha: f64,
    pub fit_intercept: bool,
}

impl RidgeRegression {
    pub fn new(alpha: f64) -> Self {
        Self {
            weights: None,
            intercept: None,
            alpha,
            fit_intercept: true,
        }
    }
}

impl Estimator for RidgeRegression {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n_samples = y.len();
        let p = if self.fit_intercept {
            n_features + 1
        } else {
            n_features
        };

        let mut xtx = vec![0.0; p * p];
        let mut xty = vec![0.0; p];

        for i in 0..n_samples {
            let row_start = i * n_features;
            for j in 0..n_features {
                let xj = x[row_start + j];
                xty[j] += xj * y[i];
                for k in 0..n_features {
                    xtx[j * p + k] += xj * x[row_start + k];
                }
                if self.fit_intercept {
                    xtx[j * p + (p - 1)] += xj;
                    xtx[(p - 1) * p + j] += xj;
                }
            }
            if self.fit_intercept {
                xty[p - 1] += y[i];
                xtx[(p - 1) * p + (p - 1)] += 1.0;
            }
        }

        // Add L2 penalty (don't regularize intercept)
        for j in 0..n_features {
            xtx[j * p + j] += self.alpha;
        }

        let params = solve_linear_system(&xtx, &xty, p)?;

        if self.fit_intercept {
            self.weights = Some(params[..n_features].to_vec());
            self.intercept = Some(params[n_features]);
        } else {
            self.weights = Some(params);
            self.intercept = Some(0.0);
        }

        Ok(())
    }
}

impl Predictor for RidgeRegression {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let w = self.weights.as_ref().ok_or(MlError::NotFitted)?;
        let b = self.intercept.unwrap_or(0.0);
        let n_samples = x.len() / n_features;

        let mut preds = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let val: f64 = row
                .iter()
                .zip(w.iter())
                .map(|(xi, wi)| xi * wi)
                .sum::<f64>()
                + b;
            preds.push(val);
        }
        Ok(preds)
    }
}

// ============================================================================
// Lasso Regression (coordinate descent)
// ============================================================================

/// Lasso regression: OLS with L1 regularization.
///
/// Uses coordinate descent optimization.
#[derive(Debug, Clone)]
pub struct LassoRegression {
    pub weights: Option<Vec<f64>>,
    pub intercept: Option<f64>,
    pub alpha: f64,
    pub max_iter: usize,
    pub tol: f64,
}

impl LassoRegression {
    pub fn new(alpha: f64) -> Self {
        Self {
            weights: None,
            intercept: None,
            alpha,
            max_iter: 1000,
            tol: 1e-4,
        }
    }
}

impl Estimator for LassoRegression {
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        let n = y.len();
        let mut w = vec![0.0; n_features];
        let y_mean = y.iter().sum::<f64>() / n as f64;
        let mut intercept = y_mean;

        // Precompute column norms
        let mut col_norms = vec![0.0; n_features];
        for j in 0..n_features {
            for i in 0..n {
                col_norms[j] += x[i * n_features + j].powi(2);
            }
        }

        for _iter in 0..self.max_iter {
            let mut max_change = 0.0_f64;

            for j in 0..n_features {
                if col_norms[j] < 1e-15 {
                    continue;
                }

                // Compute residual correlation
                let mut rho = 0.0;
                for i in 0..n {
                    let pred: f64 = (0..n_features)
                        .filter(|&k| k != j)
                        .map(|k| x[i * n_features + k] * w[k])
                        .sum::<f64>()
                        + intercept;
                    rho += x[i * n_features + j] * (y[i] - pred);
                }

                // Soft thresholding
                let new_w = soft_threshold(rho, self.alpha * n as f64 / 2.0) / col_norms[j];
                max_change = max_change.max((new_w - w[j]).abs());
                w[j] = new_w;
            }

            // Update intercept
            let mut sum_resid = 0.0;
            for i in 0..n {
                let pred: f64 = (0..n_features).map(|j| x[i * n_features + j] * w[j]).sum();
                sum_resid += y[i] - pred;
            }
            intercept = sum_resid / n as f64;

            if max_change < self.tol {
                break;
            }
        }

        self.weights = Some(w);
        self.intercept = Some(intercept);
        Ok(())
    }
}

impl Predictor for LassoRegression {
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let w = self.weights.as_ref().ok_or(MlError::NotFitted)?;
        let b = self.intercept.unwrap_or(0.0);
        let n_samples = x.len() / n_features;

        let mut preds = Vec::with_capacity(n_samples);
        for i in 0..n_samples {
            let row = &x[i * n_features..(i + 1) * n_features];
            let val: f64 = row
                .iter()
                .zip(w.iter())
                .map(|(xi, wi)| xi * wi)
                .sum::<f64>()
                + b;
            preds.push(val);
        }
        Ok(preds)
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn soft_threshold(x: f64, lambda: f64) -> f64 {
    if x > lambda {
        x - lambda
    } else if x < -lambda {
        x + lambda
    } else {
        0.0
    }
}

/// Solve Ax = b via Gaussian elimination with partial pivoting.
fn solve_linear_system(a: &[f64], b: &[f64], n: usize) -> Result<Vec<f64>> {
    // Build augmented matrix [A|b]
    let mut aug = vec![0.0; n * (n + 1)];
    for i in 0..n {
        for j in 0..n {
            aug[i * (n + 1) + j] = a[i * n + j];
        }
        aug[i * (n + 1) + n] = b[i];
    }

    // Forward elimination with partial pivoting
    for col in 0..n {
        // Find pivot
        let mut max_val = aug[col * (n + 1) + col].abs();
        let mut max_row = col;
        for row in col + 1..n {
            let val = aug[row * (n + 1) + col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-15 {
            return Err(MlError::ShapeMismatch("singular matrix".into()));
        }

        // Swap rows
        if max_row != col {
            for j in 0..=n {
                let tmp = aug[col * (n + 1) + j];
                aug[col * (n + 1) + j] = aug[max_row * (n + 1) + j];
                aug[max_row * (n + 1) + j] = tmp;
            }
        }

        // Eliminate below
        let pivot = aug[col * (n + 1) + col];
        for row in col + 1..n {
            let factor = aug[row * (n + 1) + col] / pivot;
            for j in col..=n {
                aug[row * (n + 1) + j] -= factor * aug[col * (n + 1) + j];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let mut sum = aug[i * (n + 1) + n];
        for j in i + 1..n {
            sum -= aug[i * (n + 1) + j] * x[j];
        }
        x[i] = sum / aug[i * (n + 1) + i];
    }

    Ok(x)
}

pub fn r2_score(y_true: &[f64], y_pred: &[f64]) -> f64 {
    let mean = y_true.iter().sum::<f64>() / y_true.len() as f64;
    let ss_tot: f64 = y_true.iter().map(|y| (y - mean).powi(2)).sum();
    let ss_res: f64 = y_true
        .iter()
        .zip(y_pred.iter())
        .map(|(yt, yp)| (yt - yp).powi(2))
        .sum();
    if ss_tot < 1e-15 {
        0.0
    } else {
        1.0 - ss_res / ss_tot
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ols_simple() {
        // y = 2*x + 1
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![3.0, 5.0, 7.0, 9.0, 11.0];
        let mut model = LinearRegression::new();
        model.fit(&x, 1, &y).unwrap();
        let w = model.weights.as_ref().unwrap();
        assert!((w[0] - 2.0).abs() < 1e-6);
        assert!((model.intercept.unwrap() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_ols_predict() {
        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![2.0, 4.0, 6.0, 8.0];
        let mut model = LinearRegression::new();
        model.fit(&x, 1, &y).unwrap();
        let preds = model.predict(&[5.0, 6.0], 1).unwrap();
        assert!((preds[0] - 10.0).abs() < 1e-6);
        assert!((preds[1] - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_ols_multivariate() {
        // y = 1*x1 + 2*x2 + 3
        let x = vec![
            1.0, 0.0, // sample 1
            0.0, 1.0, // sample 2
            1.0, 1.0, // sample 3
            2.0, 3.0, // sample 4
        ];
        let y = vec![4.0, 5.0, 6.0, 11.0];
        let mut model = LinearRegression::new();
        model.fit(&x, 2, &y).unwrap();
        let w = model.weights.as_ref().unwrap();
        assert!((w[0] - 1.0).abs() < 1e-4);
        assert!((w[1] - 2.0).abs() < 1e-4);
        assert!((model.intercept.unwrap() - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_ridge() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![3.0, 5.0, 7.0, 9.0, 11.0];
        let mut model = RidgeRegression::new(0.1);
        model.fit(&x, 1, &y).unwrap();
        // Ridge shrinks coefficients slightly
        assert!(model.weights.as_ref().unwrap()[0] < 2.0);
        assert!(model.weights.as_ref().unwrap()[0] > 1.5);
    }

    #[test]
    fn test_lasso() {
        // y = 3*x1 + 0*x2 (x2 is irrelevant)
        let x = vec![1.0, 0.5, 2.0, 1.0, 3.0, 0.3, 4.0, 0.8, 5.0, 0.2, 6.0, 0.9];
        let y = vec![3.0, 6.0, 9.0, 12.0, 15.0, 18.0];
        let mut model = LassoRegression::new(0.1);
        model.fit(&x, 2, &y).unwrap();
        let w = model.weights.as_ref().unwrap();
        // x2 coefficient should be close to 0 (sparsity)
        assert!(
            w[1].abs() < 1.0,
            "lasso should shrink irrelevant feature, got {}",
            w[1]
        );
    }

    #[test]
    fn test_r2_score() {
        let y_true = vec![3.0, 5.0, 7.0, 9.0];
        let y_pred = vec![3.0, 5.0, 7.0, 9.0];
        assert!((r2_score(&y_true, &y_pred) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_not_fitted() {
        let model = LinearRegression::new();
        assert!(model.predict(&[1.0], 1).is_err());
    }
}
