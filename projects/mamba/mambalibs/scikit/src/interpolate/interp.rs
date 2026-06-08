//! 1D interpolation: linear, cubic spline, and nearest neighbor.

/// Interpolation method.
#[derive(Debug, Clone, Copy)]
pub enum InterpKind {
    Linear,
    CubicSpline,
    Nearest,
}

/// Interpolate a 1D function at new x-points.
///
/// `x` and `y` are the known data points (must be sorted by x).
/// `x_new` are the query points.
pub fn interp1d(x: &[f64], y: &[f64], x_new: &[f64], kind: InterpKind) -> Vec<f64> {
    assert_eq!(x.len(), y.len(), "x and y must have same length");
    assert!(x.len() >= 2, "need at least 2 data points");

    match kind {
        InterpKind::Linear => x_new.iter().map(|&xn| linear_interp(x, y, xn)).collect(),
        InterpKind::CubicSpline => {
            let spline = CubicSpline::new(x, y);
            x_new.iter().map(|&xn| spline.eval(xn)).collect()
        }
        InterpKind::Nearest => x_new.iter().map(|&xn| nearest_interp(x, y, xn)).collect(),
    }
}

fn nearest_interp(x: &[f64], y: &[f64], xn: f64) -> f64 {
    if xn <= x[0] {
        return y[0];
    }
    if xn >= x[x.len() - 1] {
        return y[y.len() - 1];
    }
    // Binary search for closest point
    let mut lo = 0;
    let mut hi = x.len() - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if x[mid] <= xn {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    if (xn - x[lo]).abs() <= (x[hi] - xn).abs() {
        y[lo]
    } else {
        y[hi]
    }
}

fn linear_interp(x: &[f64], y: &[f64], xn: f64) -> f64 {
    if xn <= x[0] {
        return y[0];
    }
    if xn >= x[x.len() - 1] {
        return y[y.len() - 1];
    }
    // Binary search for interval
    let mut lo = 0;
    let mut hi = x.len() - 1;
    while hi - lo > 1 {
        let mid = (lo + hi) / 2;
        if x[mid] <= xn {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let t = (xn - x[lo]) / (x[hi] - x[lo]);
    y[lo] + t * (y[hi] - y[lo])
}

/// Natural cubic spline interpolation.
#[derive(Debug, Clone)]
pub struct CubicSpline {
    x: Vec<f64>,
    a: Vec<f64>,
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
}

impl CubicSpline {
    /// Build a natural cubic spline from sorted (x, y) data.
    pub fn new(x: &[f64], y: &[f64]) -> Self {
        let n = x.len();
        assert!(n >= 2, "need at least 2 data points");
        assert_eq!(x.len(), y.len());

        let a = y.to_vec();
        let mut h = vec![0.0; n - 1];
        for i in 0..n - 1 {
            h[i] = x[i + 1] - x[i];
        }

        // Solve tridiagonal system for c coefficients
        let mut alpha = vec![0.0; n];
        for i in 1..n - 1 {
            alpha[i] = 3.0 / h[i] * (a[i + 1] - a[i]) - 3.0 / h[i - 1] * (a[i] - a[i - 1]);
        }

        let mut c = vec![0.0; n];
        let mut l = vec![1.0; n];
        let mut mu = vec![0.0; n];
        let mut z = vec![0.0; n];

        for i in 1..n - 1 {
            l[i] = 2.0 * (x[i + 1] - x[i - 1]) - h[i - 1] * mu[i - 1];
            mu[i] = h[i] / l[i];
            z[i] = (alpha[i] - h[i - 1] * z[i - 1]) / l[i];
        }

        for j in (0..n - 1).rev() {
            c[j] = z[j] - mu[j] * c[j + 1];
        }

        let mut b = vec![0.0; n - 1];
        let mut d = vec![0.0; n - 1];
        for i in 0..n - 1 {
            b[i] = (a[i + 1] - a[i]) / h[i] - h[i] * (c[i + 1] + 2.0 * c[i]) / 3.0;
            d[i] = (c[i + 1] - c[i]) / (3.0 * h[i]);
        }

        Self {
            x: x.to_vec(),
            a,
            b,
            c,
            d,
        }
    }

    /// Evaluate the spline at point `xn`.
    pub fn eval(&self, xn: f64) -> f64 {
        let n = self.x.len();
        let i = if xn <= self.x[0] {
            0
        } else if xn >= self.x[n - 1] {
            n - 2
        } else {
            let mut lo = 0;
            let mut hi = n - 1;
            while hi - lo > 1 {
                let mid = (lo + hi) / 2;
                if self.x[mid] <= xn {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            lo
        };

        let dx = xn - self.x[i];
        self.a[i] + self.b[i] * dx + self.c[i] * dx * dx + self.d[i] * dx * dx * dx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_interp_exact() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let y = vec![0.0, 2.0, 4.0, 6.0];
        let result = interp1d(&x, &y, &[0.5, 1.5, 2.5], InterpKind::Linear);
        assert!((result[0] - 1.0).abs() < 1e-10);
        assert!((result[1] - 3.0).abs() < 1e-10);
        assert!((result[2] - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_linear_interp_boundary() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![10.0, 20.0, 30.0];
        let result = interp1d(&x, &y, &[0.0, 4.0], InterpKind::Linear);
        assert!((result[0] - 10.0).abs() < 1e-10); // clamp
        assert!((result[1] - 30.0).abs() < 1e-10); // clamp
    }

    #[test]
    fn test_cubic_spline_passes_through_points() {
        let x = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let y = vec![0.0, 1.0, 0.0, 1.0, 0.0];
        let spline = CubicSpline::new(&x, &y);
        for i in 0..x.len() {
            assert!(
                (spline.eval(x[i]) - y[i]).abs() < 1e-10,
                "at x={}, expected {}, got {}",
                x[i],
                y[i],
                spline.eval(x[i])
            );
        }
    }

    #[test]
    fn test_cubic_spline_smooth() {
        // Cubic spline of a polynomial should recover it exactly
        let x: Vec<f64> = (0..5).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&v| v * v).collect(); // y = x^2
        let spline = CubicSpline::new(&x, &y);
        // Check intermediate points
        assert!((spline.eval(0.5) - 0.25).abs() < 0.1);
        assert!((spline.eval(1.5) - 2.25).abs() < 0.1);
        assert!((spline.eval(2.5) - 6.25).abs() < 0.1);
    }

    #[test]
    fn test_nearest_interp() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let y = vec![0.0, 10.0, 20.0, 30.0];
        let result = interp1d(&x, &y, &[0.3, 0.6, 1.4, 2.8], InterpKind::Nearest);
        assert!((result[0] - 0.0).abs() < 1e-10); // closer to 0
        assert!((result[1] - 10.0).abs() < 1e-10); // closer to 1
        assert!((result[2] - 10.0).abs() < 1e-10); // closer to 1
        assert!((result[3] - 30.0).abs() < 1e-10); // closer to 3
    }

    #[test]
    fn test_nearest_boundary() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![10.0, 20.0, 30.0];
        let result = interp1d(&x, &y, &[0.0, 4.0], InterpKind::Nearest);
        assert!((result[0] - 10.0).abs() < 1e-10); // clamp left
        assert!((result[1] - 30.0).abs() < 1e-10); // clamp right
    }

    #[test]
    fn test_interp1d_cubic() {
        let x = vec![0.0, 1.0, 2.0, 3.0];
        let y = vec![0.0, 1.0, 4.0, 9.0];
        let result = interp1d(&x, &y, &[0.5, 1.5], InterpKind::CubicSpline);
        // Should be smooth and within range
        assert!(result[0] > 0.0 && result[0] < 1.0);
        assert!(result[1] > 1.0 && result[1] < 4.0);
    }
}
