//! Minimization algorithms: Nelder-Mead and BFGS.

use super::error::{OptimizeError, Result};

/// Result of a minimization procedure.
#[derive(Debug, Clone)]
pub struct MinimizeResult {
    /// Solution vector.
    pub x: Vec<f64>,
    /// Function value at solution.
    pub fun: f64,
    /// Number of iterations.
    pub nit: usize,
    /// Whether the optimization converged.
    pub success: bool,
}

/// Nelder-Mead (downhill simplex) minimization.
///
/// Minimizes `f(x)` starting from `x0` without requiring derivatives.
pub fn nelder_mead<F>(f: F, x0: &[f64], max_iter: usize, tol: f64) -> Result<MinimizeResult>
where
    F: Fn(&[f64]) -> f64,
{
    let n = x0.len();
    if n == 0 {
        return Err(OptimizeError::InvalidInput("empty x0".into()));
    }

    let alpha = 1.0; // reflection
    let gamma = 2.0; // expansion
    let rho = 0.5; // contraction
    let sigma = 0.5; // shrink

    // Initialize simplex: x0 + unit vectors * step
    let mut simplex: Vec<Vec<f64>> = Vec::with_capacity(n + 1);
    simplex.push(x0.to_vec());
    for i in 0..n {
        let mut point = x0.to_vec();
        let delta = if point[i].abs() > 1e-10 {
            point[i] * 0.05
        } else {
            1.0
        };
        point[i] += delta;
        simplex.push(point);
    }

    let mut values: Vec<f64> = simplex.iter().map(|p| f(p)).collect();

    for iter in 0..max_iter {
        // Sort by function value
        let mut indices: Vec<usize> = (0..=n).collect();
        indices.sort_by(|&a, &b| values[a].partial_cmp(&values[b]).unwrap());

        let sorted_simplex: Vec<Vec<f64>> = indices.iter().map(|&i| simplex[i].clone()).collect();
        let sorted_values: Vec<f64> = indices.iter().map(|&i| values[i]).collect();
        simplex = sorted_simplex;
        values = sorted_values;

        // Check convergence
        let range = values[n] - values[0];
        if range < tol {
            return Ok(MinimizeResult {
                x: simplex[0].clone(),
                fun: values[0],
                nit: iter,
                success: true,
            });
        }

        // Centroid of all points except worst
        let centroid: Vec<f64> = (0..n)
            .map(|j| simplex[..n].iter().map(|p| p[j]).sum::<f64>() / n as f64)
            .collect();

        // Reflection
        let reflected: Vec<f64> = (0..n)
            .map(|j| centroid[j] + alpha * (centroid[j] - simplex[n][j]))
            .collect();
        let f_r = f(&reflected);

        if f_r < values[0] {
            // Try expansion
            let expanded: Vec<f64> = (0..n)
                .map(|j| centroid[j] + gamma * (reflected[j] - centroid[j]))
                .collect();
            let f_e = f(&expanded);
            if f_e < f_r {
                simplex[n] = expanded;
                values[n] = f_e;
            } else {
                simplex[n] = reflected;
                values[n] = f_r;
            }
        } else if f_r < values[n - 1] {
            simplex[n] = reflected;
            values[n] = f_r;
        } else {
            // Contraction
            let contracted: Vec<f64> = (0..n)
                .map(|j| centroid[j] + rho * (simplex[n][j] - centroid[j]))
                .collect();
            let f_c = f(&contracted);
            if f_c < values[n] {
                simplex[n] = contracted;
                values[n] = f_c;
            } else {
                // Shrink
                for i in 1..=n {
                    for j in 0..n {
                        simplex[i][j] = simplex[0][j] + sigma * (simplex[i][j] - simplex[0][j]);
                    }
                    values[i] = f(&simplex[i]);
                }
            }
        }
    }

    Ok(MinimizeResult {
        x: simplex[0].clone(),
        fun: values[0],
        nit: max_iter,
        success: false,
    })
}

/// BFGS quasi-Newton minimization.
///
/// Requires both the objective function and its gradient.
pub fn bfgs<F, G>(f: F, grad: G, x0: &[f64], max_iter: usize, tol: f64) -> Result<MinimizeResult>
where
    F: Fn(&[f64]) -> f64,
    G: Fn(&[f64]) -> Vec<f64>,
{
    let n = x0.len();
    if n == 0 {
        return Err(OptimizeError::InvalidInput("empty x0".into()));
    }

    let mut x = x0.to_vec();
    let mut g = grad(&x);

    // Initialize inverse Hessian as identity
    let mut h_inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        h_inv[i][i] = 1.0;
    }

    for iter in 0..max_iter {
        let grad_norm: f64 = g.iter().map(|&v| v * v).sum::<f64>().sqrt();
        if grad_norm < tol {
            return Ok(MinimizeResult {
                x: x.clone(),
                fun: f(&x),
                nit: iter,
                success: true,
            });
        }

        // Search direction: p = -H_inv * g
        let p: Vec<f64> = (0..n)
            .map(|i| {
                -h_inv[i]
                    .iter()
                    .zip(g.iter())
                    .map(|(&h, &gi)| h * gi)
                    .sum::<f64>()
            })
            .collect();

        // Line search (backtracking Armijo)
        let mut alpha = 1.0;
        let c1 = 1e-4;
        let f_x = f(&x);
        let slope: f64 = g.iter().zip(p.iter()).map(|(&gi, &pi)| gi * pi).sum();

        for _ in 0..50 {
            let x_new: Vec<f64> = x
                .iter()
                .zip(p.iter())
                .map(|(&xi, &pi)| xi + alpha * pi)
                .collect();
            if f(&x_new) <= f_x + c1 * alpha * slope {
                break;
            }
            alpha *= 0.5;
        }

        // Update x
        let s: Vec<f64> = p.iter().map(|&pi| alpha * pi).collect();
        for i in 0..n {
            x[i] += s[i];
        }

        // Update gradient
        let g_new = grad(&x);
        let y: Vec<f64> = g_new
            .iter()
            .zip(g.iter())
            .map(|(&gn, &go)| gn - go)
            .collect();

        // BFGS update of inverse Hessian
        let sy: f64 = s.iter().zip(y.iter()).map(|(&si, &yi)| si * yi).sum();
        if sy > 1e-14 {
            // H_inv update via Sherman-Morrison
            let hy: Vec<f64> = (0..n)
                .map(|i| {
                    h_inv[i]
                        .iter()
                        .zip(y.iter())
                        .map(|(&h, &yi)| h * yi)
                        .sum::<f64>()
                })
                .collect();
            let yhy: f64 = y.iter().zip(hy.iter()).map(|(&yi, &hyi)| yi * hyi).sum();

            for i in 0..n {
                for j in 0..n {
                    h_inv[i][j] +=
                        (sy + yhy) / (sy * sy) * s[i] * s[j] - (hy[i] * s[j] + s[i] * hy[j]) / sy;
                }
            }
        }

        g = g_new;
    }

    Ok(MinimizeResult {
        x: x.clone(),
        fun: f(&x),
        nit: max_iter,
        success: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nelder_mead_quadratic() {
        // Minimize f(x) = (x-3)^2
        let result = nelder_mead(|x| (x[0] - 3.0).powi(2), &[0.0], 1000, 1e-10).unwrap();
        assert!(result.success);
        assert!((result.x[0] - 3.0).abs() < 1e-4);
    }

    #[test]
    fn test_nelder_mead_rosenbrock() {
        // Rosenbrock: f(x,y) = (1-x)^2 + 100*(y-x^2)^2, min at (1,1)
        let f = |x: &[f64]| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0].powi(2)).powi(2);
        let result = nelder_mead(f, &[0.0, 0.0], 10000, 1e-10).unwrap();
        assert!((result.x[0] - 1.0).abs() < 0.1);
        assert!((result.x[1] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_bfgs_quadratic() {
        let f = |x: &[f64]| x[0] * x[0] + x[1] * x[1];
        let grad = |x: &[f64]| vec![2.0 * x[0], 2.0 * x[1]];
        let result = bfgs(f, grad, &[5.0, 3.0], 100, 1e-8).unwrap();
        assert!(result.success);
        assert!(result.x[0].abs() < 1e-4);
        assert!(result.x[1].abs() < 1e-4);
    }

    #[test]
    fn test_bfgs_rosenbrock() {
        let f = |x: &[f64]| (1.0 - x[0]).powi(2) + 100.0 * (x[1] - x[0].powi(2)).powi(2);
        let grad = |x: &[f64]| {
            vec![
                -2.0 * (1.0 - x[0]) - 400.0 * x[0] * (x[1] - x[0].powi(2)),
                200.0 * (x[1] - x[0].powi(2)),
            ]
        };
        let result = bfgs(f, grad, &[0.0, 0.0], 500, 1e-8).unwrap();
        assert!((result.x[0] - 1.0).abs() < 0.05);
        assert!((result.x[1] - 1.0).abs() < 0.05);
    }
}
