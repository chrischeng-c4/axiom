//! ODE solvers (Euler, RK4, adaptive RK45).

/// Result of an ODE solver.
#[derive(Debug, Clone)]
pub struct OdeResult {
    /// Time points.
    pub t: Vec<f64>,
    /// Solution values at each time point.
    /// For a system of n equations, `y[i]` is a vector of length n.
    pub y: Vec<Vec<f64>>,
    /// Whether the solver succeeded.
    pub success: bool,
    /// Number of function evaluations.
    pub nfev: usize,
}

/// ODE solver method.
#[derive(Debug, Clone, Copy)]
pub enum OdeSolver {
    /// Forward Euler (first order).
    Euler,
    /// Classic Runge-Kutta (fourth order).
    RK4,
    /// Adaptive Runge-Kutta-Fehlberg (fourth/fifth order).
    RK45,
}

/// Solve an initial value problem: dy/dt = f(t, y), y(t0) = y0.
///
/// # Arguments
///
/// * `f` - Right-hand side function f(t, y) -> dy/dt
/// * `t_span` - (t0, t_final)
/// * `y0` - Initial condition
/// * `solver` - Which solver to use
/// * `max_step` - Maximum step size (for adaptive solvers, this is the initial step)
/// * `rtol` - Relative tolerance (for adaptive solvers)
///
/// # Example
///
/// ```
/// use scikit::integrate::{solve_ivp, OdeSolver};
///
/// // dy/dt = -y, y(0) = 1 => y(t) = e^(-t)
/// let result = solve_ivp(
///     |_t, y| vec![-y[0]],
///     (0.0, 1.0),
///     &[1.0],
///     OdeSolver::RK4,
///     0.01,
///     1e-6,
/// );
/// assert!(result.success);
/// let y_final = result.y.last().unwrap()[0];
/// assert!((y_final - (-1.0_f64).exp()).abs() < 1e-4);
/// ```
pub fn solve_ivp<F>(
    f: F,
    t_span: (f64, f64),
    y0: &[f64],
    solver: OdeSolver,
    max_step: f64,
    rtol: f64,
) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    match solver {
        OdeSolver::Euler => euler_solve(&f, t_span, y0, max_step),
        OdeSolver::RK4 => rk4_solve(&f, t_span, y0, max_step),
        OdeSolver::RK45 => rk45_solve(&f, t_span, y0, max_step, rtol),
    }
}

/// Forward Euler method.
pub fn euler<F>(f: &F, t: f64, y: &[f64], dt: f64) -> Vec<f64>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let dy = f(t, y);
    y.iter().zip(dy.iter()).map(|(&yi, &dyi)| yi + dt * dyi).collect()
}

/// Classic Runge-Kutta (RK4) step.
pub fn rk4<F>(f: &F, t: f64, y: &[f64], dt: f64) -> Vec<f64>
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let n = y.len();
    let k1 = f(t, y);

    let y2: Vec<f64> = (0..n).map(|i| y[i] + 0.5 * dt * k1[i]).collect();
    let k2 = f(t + 0.5 * dt, &y2);

    let y3: Vec<f64> = (0..n).map(|i| y[i] + 0.5 * dt * k2[i]).collect();
    let k3 = f(t + 0.5 * dt, &y3);

    let y4: Vec<f64> = (0..n).map(|i| y[i] + dt * k3[i]).collect();
    let k4 = f(t + dt, &y4);

    (0..n)
        .map(|i| y[i] + dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]))
        .collect()
}

fn euler_solve<F>(f: &F, t_span: (f64, f64), y0: &[f64], dt: f64) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let (t0, tf) = t_span;
    let mut t_vals = vec![t0];
    let mut y_vals = vec![y0.to_vec()];
    let mut t = t0;
    let mut y = y0.to_vec();
    let mut nfev = 0;

    while t < tf {
        let step = dt.min(tf - t);
        y = euler(f, t, &y, step);
        nfev += 1;
        t += step;
        t_vals.push(t);
        y_vals.push(y.clone());
    }

    OdeResult {
        t: t_vals,
        y: y_vals,
        success: true,
        nfev,
    }
}

fn rk4_solve<F>(f: &F, t_span: (f64, f64), y0: &[f64], dt: f64) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let (t0, tf) = t_span;
    let mut t_vals = vec![t0];
    let mut y_vals = vec![y0.to_vec()];
    let mut t = t0;
    let mut y = y0.to_vec();
    let mut nfev = 0;

    while t < tf {
        let step = dt.min(tf - t);
        y = rk4(f, t, &y, step);
        nfev += 4;
        t += step;
        t_vals.push(t);
        y_vals.push(y.clone());
    }

    OdeResult {
        t: t_vals,
        y: y_vals,
        success: true,
        nfev,
    }
}

fn rk45_solve<F>(f: &F, t_span: (f64, f64), y0: &[f64], max_step: f64, rtol: f64) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let (t0, tf) = t_span;
    let n = y0.len();
    let mut t_vals = vec![t0];
    let mut y_vals = vec![y0.to_vec()];
    let mut t = t0;
    let mut y = y0.to_vec();
    let mut dt = max_step;
    let mut nfev = 0;

    // Dormand-Prince coefficients (RK45)
    let a2 = 1.0 / 5.0;
    let a3 = 3.0 / 10.0;
    let a4 = 4.0 / 5.0;
    let a5 = 8.0 / 9.0;

    let max_iter = 1_000_000;
    let mut iter = 0;

    while t < tf - 1e-14 && iter < max_iter {
        iter += 1;
        dt = dt.min(tf - t);
        if dt < 1e-15 {
            break;
        }

        let k1 = f(t, &y);
        nfev += 1;

        let y2: Vec<f64> = (0..n).map(|i| y[i] + dt * a2 * k1[i]).collect();
        let k2 = f(t + a2 * dt, &y2);
        nfev += 1;

        let y3: Vec<f64> = (0..n)
            .map(|i| y[i] + dt * (3.0 / 40.0 * k1[i] + 9.0 / 40.0 * k2[i]))
            .collect();
        let k3 = f(t + a3 * dt, &y3);
        nfev += 1;

        let y4: Vec<f64> = (0..n)
            .map(|i| {
                y[i] + dt * (44.0 / 45.0 * k1[i] - 56.0 / 15.0 * k2[i] + 32.0 / 9.0 * k3[i])
            })
            .collect();
        let k4 = f(t + a4 * dt, &y4);
        nfev += 1;

        let y5: Vec<f64> = (0..n)
            .map(|i| {
                y[i] + dt
                    * (19372.0 / 6561.0 * k1[i] - 25360.0 / 2187.0 * k2[i]
                        + 64448.0 / 6561.0 * k3[i]
                        - 212.0 / 729.0 * k4[i])
            })
            .collect();
        let k5 = f(t + a5 * dt, &y5);
        nfev += 1;

        let y6: Vec<f64> = (0..n)
            .map(|i| {
                y[i] + dt
                    * (9017.0 / 3168.0 * k1[i] - 355.0 / 33.0 * k2[i]
                        + 46732.0 / 5247.0 * k3[i]
                        + 49.0 / 176.0 * k4[i]
                        - 5103.0 / 18656.0 * k5[i])
            })
            .collect();
        let k6 = f(t + dt, &y6);
        nfev += 1;

        // 5th order solution
        let y_new: Vec<f64> = (0..n)
            .map(|i| {
                y[i] + dt
                    * (35.0 / 384.0 * k1[i] + 500.0 / 1113.0 * k3[i]
                        + 125.0 / 192.0 * k4[i]
                        - 2187.0 / 6784.0 * k5[i]
                        + 11.0 / 84.0 * k6[i])
            })
            .collect();

        // 4th order solution for error estimation
        let y_err: Vec<f64> = (0..n)
            .map(|i| {
                y[i] + dt
                    * (5179.0 / 57600.0 * k1[i] + 7571.0 / 16695.0 * k3[i]
                        + 393.0 / 640.0 * k4[i]
                        - 92097.0 / 339200.0 * k5[i]
                        + 187.0 / 2100.0 * k6[i]
                        + 1.0 / 40.0 * k6[i]) // simplified
            })
            .collect();

        // Error estimate
        let err: f64 = y_new
            .iter()
            .zip(y_err.iter())
            .map(|(a, b)| {
                let scale = rtol * a.abs().max(1.0);
                ((a - b) / scale).powi(2)
            })
            .sum::<f64>()
            .sqrt()
            / (n as f64).sqrt();

        if err <= 1.0 || dt <= 1e-14 {
            // Accept step
            t += dt;
            y = y_new;
            t_vals.push(t);
            y_vals.push(y.clone());
        }

        // Adjust step size
        let safety = 0.9;
        let factor = if err > 1e-15 {
            safety * (1.0 / err).powf(0.2)
        } else {
            2.0
        };
        dt *= factor.clamp(0.2, 5.0);
        dt = dt.min(max_step);
    }

    OdeResult {
        t: t_vals,
        y: y_vals,
        success: iter < max_iter,
        nfev,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euler_exponential_decay() {
        // dy/dt = -y, y(0) = 1 => y(t) = e^(-t)
        let result = solve_ivp(
            |_t, y| vec![-y[0]],
            (0.0, 1.0),
            &[1.0],
            OdeSolver::Euler,
            0.001,
            1e-6,
        );
        assert!(result.success);
        let y_final = result.y.last().unwrap()[0];
        let expected = (-1.0_f64).exp();
        assert!(
            (y_final - expected).abs() < 0.01,
            "got {y_final}, expected {expected}"
        );
    }

    #[test]
    fn test_rk4_exponential_decay() {
        let result = solve_ivp(
            |_t, y| vec![-y[0]],
            (0.0, 1.0),
            &[1.0],
            OdeSolver::RK4,
            0.01,
            1e-6,
        );
        assert!(result.success);
        let y_final = result.y.last().unwrap()[0];
        let expected = (-1.0_f64).exp();
        assert!(
            (y_final - expected).abs() < 1e-6,
            "got {y_final}, expected {expected}"
        );
    }

    #[test]
    fn test_rk45_exponential_decay() {
        let result = solve_ivp(
            |_t, y| vec![-y[0]],
            (0.0, 1.0),
            &[1.0],
            OdeSolver::RK45,
            0.1,
            1e-8,
        );
        assert!(result.success);
        let y_final = result.y.last().unwrap()[0];
        let expected = (-1.0_f64).exp();
        assert!(
            (y_final - expected).abs() < 1e-4,
            "got {y_final}, expected {expected}"
        );
    }

    #[test]
    fn test_rk4_harmonic_oscillator() {
        // y'' + y = 0 => system: y1' = y2, y2' = -y1
        // y1(0) = 1, y2(0) = 0 => y1(t) = cos(t), y2(t) = -sin(t)
        let result = solve_ivp(
            |_t, y| vec![y[1], -y[0]],
            (0.0, std::f64::consts::PI),
            &[1.0, 0.0],
            OdeSolver::RK4,
            0.01,
            1e-6,
        );
        assert!(result.success);
        let y_final = &result.y[result.y.len() - 1];
        // At t=pi: cos(pi) = -1, -sin(pi) = 0
        assert!(
            (y_final[0] - (-1.0)).abs() < 1e-4,
            "y1 got {}, expected -1",
            y_final[0]
        );
        assert!(
            y_final[1].abs() < 1e-4,
            "y2 got {}, expected 0",
            y_final[1]
        );
    }

    #[test]
    fn test_euler_step() {
        let y = euler(&|_t, y: &[f64]| vec![-y[0]], 0.0, &[1.0], 0.1);
        assert!((y[0] - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_rk4_step() {
        let y = rk4(&|_t, y: &[f64]| vec![-y[0]], 0.0, &[1.0], 0.1);
        let expected = (-0.1_f64).exp();
        assert!(
            (y[0] - expected).abs() < 1e-6,
            "got {}, expected {expected}",
            y[0]
        );
    }
}
