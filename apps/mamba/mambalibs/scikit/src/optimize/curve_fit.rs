//! Non-linear curve fitting (Gauss-Newton method).

use super::error::{OptimizeError, Result};

/// Result of curve fitting.
#[derive(Debug, Clone)]
pub struct CurveFitResult {
    /// Fitted parameters.
    pub params: Vec<f64>,
    /// Residual values at solution.
    pub residuals: Vec<f64>,
    /// Sum of squared residuals.
    pub cost: f64,
    /// Number of iterations.
    pub nit: usize,
    /// Whether the fit converged.
    pub success: bool,
}

/// Fit a model function to data using Gauss-Newton method.
///
/// The model function `f(x, params) -> y` is fit to `(xdata, ydata)`.
///
/// # Arguments
/// * `model` - Model function: `(x_value, &params) -> predicted_y`
/// * `xdata` - Independent variable values
/// * `ydata` - Observed dependent variable values
/// * `p0` - Initial parameter guess
/// * `max_iter` - Maximum number of iterations
/// * `tol` - Convergence tolerance on parameter change
pub fn curve_fit<F>(
    model: F,
    xdata: &[f64],
    ydata: &[f64],
    p0: &[f64],
    max_iter: usize,
    tol: f64,
) -> Result<CurveFitResult>
where
    F: Fn(f64, &[f64]) -> f64,
{
    if xdata.len() != ydata.len() {
        return Err(OptimizeError::InvalidInput(
            "xdata and ydata must have same length".into(),
        ));
    }
    if xdata.is_empty() || p0.is_empty() {
        return Err(OptimizeError::InvalidInput("empty input".into()));
    }

    let n = xdata.len(); // number of data points
    let p = p0.len(); // number of parameters
    let mut params = p0.to_vec();
    let eps = 1e-8; // finite difference step

    for iter in 0..max_iter {
        // Compute residuals: r_i = ydata_i - model(xdata_i, params)
        let residuals: Vec<f64> = (0..n)
            .map(|i| ydata[i] - model(xdata[i], &params))
            .collect();

        // Compute Jacobian columns: J[i][j] = d(residual_i)/d(params_j)
        // residual = ydata - model, so J[i][j] = -d(model)/d(params_j)
        let mut jacobian_cols: Vec<Vec<f64>> = Vec::with_capacity(p);
        for j in 0..p {
            let mut params_h = params.clone();
            params_h[j] += eps;
            let col: Vec<f64> = (0..n)
                .map(|i| -(model(xdata[i], &params_h) - model(xdata[i], &params)) / eps)
                .collect();
            jacobian_cols.push(col);
        }

        // Form J^T * J and J^T * r
        let mut jt_j = vec![0.0; p * p];
        let mut jt_r = vec![0.0; p];
        for j in 0..p {
            for k in 0..p {
                jt_j[j * p + k] = jacobian_cols[j]
                    .iter()
                    .zip(jacobian_cols[k].iter())
                    .map(|(&a, &b)| a * b)
                    .sum();
            }
            jt_r[j] = jacobian_cols[j]
                .iter()
                .zip(residuals.iter())
                .map(|(&ji, &r)| ji * r)
                .sum();
        }

        // Add damping (Levenberg-Marquardt style)
        let lambda = 1e-3;
        for i in 0..p {
            jt_j[i * p + i] += lambda;
        }

        // Gauss-Newton step: (J^T J + λI) * delta = -J^T r
        let neg_jt_r: Vec<f64> = jt_r.iter().map(|v| -v).collect();
        let delta = solve_linear(p, &jt_j, &neg_jt_r);

        // Update parameters
        let mut max_change = 0.0_f64;
        for i in 0..p {
            params[i] += delta[i];
            max_change = max_change.max(delta[i].abs());
        }

        if max_change < tol {
            let residuals: Vec<f64> = (0..n)
                .map(|i| ydata[i] - model(xdata[i], &params))
                .collect();
            let cost = residuals.iter().map(|r| r * r).sum();
            return Ok(CurveFitResult {
                params,
                residuals,
                cost,
                nit: iter,
                success: true,
            });
        }
    }

    let residuals: Vec<f64> = (0..n)
        .map(|i| ydata[i] - model(xdata[i], &params))
        .collect();
    let cost = residuals.iter().map(|r| r * r).sum();

    Ok(CurveFitResult {
        params,
        residuals,
        cost,
        nit: max_iter,
        success: false,
    })
}

/// Simple Gaussian elimination for small systems.
fn solve_linear(n: usize, a: &[f64], b: &[f64]) -> Vec<f64> {
    let mut aug = vec![0.0; n * (n + 1)];
    for i in 0..n {
        for j in 0..n {
            aug[i * (n + 1) + j] = a[i * n + j];
        }
        aug[i * (n + 1) + n] = b[i];
    }

    // Forward elimination
    for k in 0..n {
        let mut max_idx = k;
        let mut max_val = aug[k * (n + 1) + k].abs();
        for i in k + 1..n {
            let v = aug[i * (n + 1) + k].abs();
            if v > max_val {
                max_val = v;
                max_idx = i;
            }
        }
        if max_idx != k {
            for j in 0..=n {
                aug.swap(k * (n + 1) + j, max_idx * (n + 1) + j);
            }
        }
        let pivot = aug[k * (n + 1) + k];
        if pivot.abs() < 1e-14 {
            continue;
        }
        for i in k + 1..n {
            let factor = aug[i * (n + 1) + k] / pivot;
            for j in k..=n {
                aug[i * (n + 1) + j] -= factor * aug[k * (n + 1) + j];
            }
        }
    }

    // Back substitution
    let mut x = vec![0.0; n];
    for i in (0..n).rev() {
        let pivot = aug[i * (n + 1) + i];
        if pivot.abs() < 1e-14 {
            continue;
        }
        x[i] = aug[i * (n + 1) + n];
        for j in i + 1..n {
            x[i] -= aug[i * (n + 1) + j] * x[j];
        }
        x[i] /= pivot;
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_curve_fit_linear() {
        // Fit y = a*x + b to data
        let xdata: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let ydata: Vec<f64> = xdata.iter().map(|&x| 2.0 * x + 3.0).collect();
        let model = |x: f64, p: &[f64]| p[0] * x + p[1];

        let result = curve_fit(model, &xdata, &ydata, &[0.0, 0.0], 100, 1e-8).unwrap();
        assert!(result.success);
        assert!((result.params[0] - 2.0).abs() < 0.1);
        assert!((result.params[1] - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_curve_fit_exponential() {
        // Fit y = a * exp(b * x)
        let xdata: Vec<f64> = (0..10).map(|i| i as f64 * 0.5).collect();
        let ydata: Vec<f64> = xdata.iter().map(|&x| 2.0 * (0.5 * x).exp()).collect();
        let model = |x: f64, p: &[f64]| p[0] * (p[1] * x).exp();

        let result = curve_fit(model, &xdata, &ydata, &[1.0, 0.3], 200, 1e-8).unwrap();
        assert!((result.params[0] - 2.0).abs() < 0.5);
        assert!((result.params[1] - 0.5).abs() < 0.3);
    }

    #[test]
    fn test_curve_fit_empty() {
        let result = curve_fit(|_, _| 0.0, &[], &[], &[1.0], 10, 1e-8);
        assert!(result.is_err());
    }
}
