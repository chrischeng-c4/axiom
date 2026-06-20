//! Numerical quadrature (integration of functions and data).

/// Result of numerical quadrature.
#[derive(Debug, Clone)]
pub struct QuadResult {
    /// Estimated integral value.
    pub value: f64,
    /// Estimated absolute error.
    pub error: f64,
    /// Number of function evaluations.
    pub neval: usize,
}

/// Adaptive quadrature using recursive Simpson's rule.
///
/// Integrates `f` over `[a, b]` with absolute tolerance `tol`.
///
/// # Example
///
/// ```
/// use scikit::integrate::quad;
///
/// let result = quad(|x| x * x, 0.0, 1.0, 1e-10);
/// assert!((result.value - 1.0 / 3.0).abs() < 1e-10);
/// ```
pub fn quad<F: Fn(f64) -> f64>(f: F, a: f64, b: f64, tol: f64) -> QuadResult {
    let mut neval = 0;
    let value = adaptive_simpson(&f, a, b, tol, 50, &mut neval);
    let error = tol; // conservative estimate
    QuadResult {
        value,
        error,
        neval,
    }
}

fn adaptive_simpson<F: Fn(f64) -> f64>(
    f: &F,
    a: f64,
    b: f64,
    tol: f64,
    max_depth: usize,
    neval: &mut usize,
) -> f64 {
    let mid = (a + b) / 2.0;
    let fa = f(a);
    let fb = f(b);
    let fm = f(mid);
    *neval += 3;

    let s_whole = (b - a) / 6.0 * (fa + 4.0 * fm + fb);

    adaptive_simpson_recursive(f, a, b, tol, s_whole, fa, fb, fm, max_depth, neval)
}

fn adaptive_simpson_recursive<F: Fn(f64) -> f64>(
    f: &F,
    a: f64,
    b: f64,
    tol: f64,
    s_prev: f64,
    fa: f64,
    fb: f64,
    fm: f64,
    depth: usize,
    neval: &mut usize,
) -> f64 {
    let mid = (a + b) / 2.0;
    let m1 = (a + mid) / 2.0;
    let m2 = (mid + b) / 2.0;
    let f1 = f(m1);
    let f2 = f(m2);
    *neval += 2;

    let h = b - a;
    let s_left = h / 12.0 * (fa + 4.0 * f1 + fm);
    let s_right = h / 12.0 * (fm + 4.0 * f2 + fb);
    let s_combined = s_left + s_right;

    if depth <= 0 || (s_combined - s_prev).abs() < 15.0 * tol {
        return s_combined + (s_combined - s_prev) / 15.0;
    }

    adaptive_simpson_recursive(f, a, mid, tol / 2.0, s_left, fa, fm, f1, depth - 1, neval)
        + adaptive_simpson_recursive(f, mid, b, tol / 2.0, s_right, fm, fb, f2, depth - 1, neval)
}

/// Trapezoid rule for evenly spaced data.
///
/// `y` = function values, `dx` = spacing between x values.
pub fn trapezoid(y: &[f64], dx: f64) -> f64 {
    if y.len() < 2 {
        return 0.0;
    }
    let mut sum = 0.0;
    for i in 1..y.len() {
        sum += (y[i - 1] + y[i]) / 2.0;
    }
    sum * dx
}

/// Cumulative trapezoid integration.
///
/// Returns a vector of length `y.len() - 1` with cumulative integrals.
pub fn cumulative_trapezoid(y: &[f64], dx: f64) -> Vec<f64> {
    if y.len() < 2 {
        return vec![];
    }
    let mut result = Vec::with_capacity(y.len() - 1);
    let mut cumsum = 0.0;
    for i in 1..y.len() {
        cumsum += (y[i - 1] + y[i]) / 2.0 * dx;
        result.push(cumsum);
    }
    result
}

/// Simpson's rule for evenly spaced data.
///
/// Uses composite Simpson's rule. If the number of intervals is odd,
/// falls back to the trapezoid rule for the last interval.
pub fn simps(y: &[f64], dx: f64) -> f64 {
    if y.len() < 2 {
        return 0.0;
    }
    if y.len() < 3 {
        return trapezoid(y, dx);
    }

    let n = y.len() - 1; // number of intervals
    let mut result = 0.0;

    // Apply Simpson's 1/3 rule for pairs of intervals
    let n_simps = if n % 2 == 0 { n } else { n - 1 };
    let mut i = 0;
    while i < n_simps {
        result += dx / 3.0 * (y[i] + 4.0 * y[i + 1] + y[i + 2]);
        i += 2;
    }

    // Handle the last interval with trapezoid if n is odd
    if n % 2 != 0 {
        result += dx / 2.0 * (y[n - 1] + y[n]);
    }

    result
}

/// Fixed-order Gaussian quadrature on [a, b].
///
/// Uses `n`-point Gauss-Legendre quadrature (n = 1..5 supported).
pub fn fixed_quad<F: Fn(f64) -> f64>(f: F, a: f64, b: f64, n: usize) -> f64 {
    let (nodes, weights) = gauss_legendre_nodes(n);
    let half_width = (b - a) / 2.0;
    let mid = (a + b) / 2.0;

    let mut sum = 0.0;
    for i in 0..nodes.len() {
        let x = mid + half_width * nodes[i];
        sum += weights[i] * f(x);
    }
    sum * half_width
}

/// Gauss-Legendre nodes and weights for n-point quadrature on [-1, 1].
fn gauss_legendre_nodes(n: usize) -> (Vec<f64>, Vec<f64>) {
    match n {
        1 => (vec![0.0], vec![2.0]),
        2 => (
            vec![-1.0 / 3.0_f64.sqrt(), 1.0 / 3.0_f64.sqrt()],
            vec![1.0, 1.0],
        ),
        3 => (
            vec![-(3.0 / 5.0_f64).sqrt(), 0.0, (3.0 / 5.0_f64).sqrt()],
            vec![5.0 / 9.0, 8.0 / 9.0, 5.0 / 9.0],
        ),
        4 => {
            let x1 = ((3.0 - 2.0 * (6.0 / 5.0_f64).sqrt()) / 7.0).sqrt();
            let x2 = ((3.0 + 2.0 * (6.0 / 5.0_f64).sqrt()) / 7.0).sqrt();
            let w1 = (18.0 + 30.0_f64.sqrt()) / 36.0;
            let w2 = (18.0 - 30.0_f64.sqrt()) / 36.0;
            (vec![-x2, -x1, x1, x2], vec![w2, w1, w1, w2])
        }
        5 => {
            let x1 = (5.0 - 2.0 * (10.0 / 7.0_f64).sqrt()).sqrt() / 3.0;
            let x2 = (5.0 + 2.0 * (10.0 / 7.0_f64).sqrt()).sqrt() / 3.0;
            let w0 = 128.0 / 225.0;
            let w1 = (322.0 + 13.0 * 70.0_f64.sqrt()) / 900.0;
            let w2 = (322.0 - 13.0 * 70.0_f64.sqrt()) / 900.0;
            (vec![-x2, -x1, 0.0, x1, x2], vec![w2, w1, w0, w1, w2])
        }
        _ => panic!("Gauss-Legendre supports n=1..5, got {n}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_quad_polynomial() {
        // integral of x^2 from 0 to 1 = 1/3
        let result = quad(|x| x * x, 0.0, 1.0, 1e-12);
        assert!(
            (result.value - 1.0 / 3.0).abs() < 1e-10,
            "got {}",
            result.value
        );
    }

    #[test]
    fn test_quad_sin() {
        // integral of sin(x) from 0 to pi = 2
        let result = quad(|x| x.sin(), 0.0, PI, 1e-10);
        assert!((result.value - 2.0).abs() < 1e-8, "got {}", result.value);
    }

    #[test]
    fn test_quad_exp() {
        // integral of e^x from 0 to 1 = e - 1
        let result = quad(|x| x.exp(), 0.0, 1.0, 1e-10);
        let expected = std::f64::consts::E - 1.0;
        assert!(
            (result.value - expected).abs() < 1e-8,
            "got {}",
            result.value
        );
    }

    #[test]
    fn test_trapezoid() {
        // Integrate y = x from 0 to 1 with 101 points
        let n = 101;
        let dx = 1.0 / (n - 1) as f64;
        let y: Vec<f64> = (0..n).map(|i| i as f64 * dx).collect();
        let result = trapezoid(&y, dx);
        assert!((result - 0.5).abs() < 1e-4, "got {}", result);
    }

    #[test]
    fn test_cumulative_trapezoid() {
        let y = vec![0.0, 1.0, 2.0, 3.0];
        let ct = cumulative_trapezoid(&y, 1.0);
        assert_eq!(ct.len(), 3);
        assert!((ct[0] - 0.5).abs() < 1e-10);
        assert!((ct[1] - 2.0).abs() < 1e-10);
        assert!((ct[2] - 4.5).abs() < 1e-10);
    }

    #[test]
    fn test_simps() {
        // Integrate y = x^2 from 0 to 2 with 5 points
        let n = 5;
        let dx = 2.0 / (n - 1) as f64;
        let y: Vec<f64> = (0..n)
            .map(|i| {
                let x = i as f64 * dx;
                x * x
            })
            .collect();
        let result = simps(&y, dx);
        // exact = 8/3 = 2.6667
        assert!((result - 8.0 / 3.0).abs() < 1e-10, "got {}", result);
    }

    #[test]
    fn test_fixed_quad_polynomial() {
        // 3-point Gauss-Legendre is exact for degree <= 5
        let result = fixed_quad(|x| x * x * x, -1.0, 1.0, 3);
        assert!((result - 0.0).abs() < 1e-10, "got {}", result);

        let result2 = fixed_quad(|x| x * x, 0.0, 1.0, 3);
        assert!((result2 - 1.0 / 3.0).abs() < 1e-10, "got {}", result2);
    }

    #[test]
    fn test_fixed_quad_5point() {
        // 5-point exact for degree <= 9
        let result = fixed_quad(|x| x.powi(4), 0.0, 1.0, 5);
        assert!((result - 0.2).abs() < 1e-10, "got {}", result);
    }

    #[test]
    fn test_trapezoid_empty() {
        assert!((trapezoid(&[], 1.0) - 0.0).abs() < 1e-10);
        assert!((trapezoid(&[5.0], 1.0) - 0.0).abs() < 1e-10);
    }
}
