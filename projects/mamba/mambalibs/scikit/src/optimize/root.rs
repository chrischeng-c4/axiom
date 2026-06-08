//! Root-finding algorithms: Brent's method and Newton-Raphson.

use super::error::{OptimizeError, Result};

/// Find a root of `f` in the interval `[a, b]` using Brent's method.
///
/// Requires `f(a)` and `f(b)` to have opposite signs.
pub fn brentq<F>(f: F, a: f64, b: f64, tol: f64, max_iter: usize) -> Result<f64>
where
    F: Fn(f64) -> f64,
{
    let mut a = a;
    let mut b = b;
    let mut fa = f(a);
    let mut fb = f(b);

    if fa * fb > 0.0 {
        return Err(OptimizeError::InvalidBounds(
            "f(a) and f(b) must have opposite signs".into(),
        ));
    }

    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut mflag = true;
    let mut d = 0.0;

    for _ in 0..max_iter {
        if fb.abs() < tol {
            return Ok(b);
        }
        if fa.abs() < tol {
            return Ok(a);
        }
        if (b - a).abs() < tol {
            return Ok(b);
        }

        let s = if (fa - fc).abs() > 1e-14 && (fb - fc).abs() > 1e-14 {
            // Inverse quadratic interpolation
            a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb))
        } else {
            // Secant method
            b - fb * (b - a) / (fb - fa)
        };

        // Conditions for bisection
        let cond1 = if a < b {
            !(s > (3.0 * a + b) / 4.0 && s < b)
        } else {
            !(s > b && s < (3.0 * a + b) / 4.0)
        };
        let cond2 = mflag && (s - b).abs() >= (b - c).abs() / 2.0;
        let cond3 = !mflag && (s - b).abs() >= (c - d).abs() / 2.0;
        let cond4 = mflag && (b - c).abs() < tol;
        let cond5 = !mflag && (c - d).abs() < tol;

        let s = if cond1 || cond2 || cond3 || cond4 || cond5 {
            mflag = true;
            (a + b) / 2.0
        } else {
            mflag = false;
            s
        };

        let fs = f(s);
        d = c;
        c = b;
        fc = fb;

        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }

        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }

    Err(OptimizeError::ConvergenceFailed(max_iter))
}

/// Newton-Raphson root finding.
///
/// Finds root of `f` using its derivative `df`, starting from `x0`.
pub fn newton<F, D>(f: F, df: D, x0: f64, tol: f64, max_iter: usize) -> Result<f64>
where
    F: Fn(f64) -> f64,
    D: Fn(f64) -> f64,
{
    let mut x = x0;

    for iter in 0..max_iter {
        let fx = f(x);
        if fx.abs() < tol {
            return Ok(x);
        }

        let dfx = df(x);
        if dfx.abs() < 1e-14 {
            return Err(OptimizeError::InvalidInput(format!(
                "derivative near zero at x={x}, iteration {iter}"
            )));
        }

        x -= fx / dfx;
    }

    Err(OptimizeError::ConvergenceFailed(max_iter))
}

/// Bisection method for root finding.
///
/// Simpler but slower alternative to Brent's method.
pub fn bisect<F>(f: F, a: f64, b: f64, tol: f64, max_iter: usize) -> Result<f64>
where
    F: Fn(f64) -> f64,
{
    let mut a = a;
    let mut b = b;
    let mut fa = f(a);

    if fa * f(b) > 0.0 {
        return Err(OptimizeError::InvalidBounds(
            "f(a) and f(b) must have opposite signs".into(),
        ));
    }

    for _ in 0..max_iter {
        let mid = (a + b) / 2.0;
        let fmid = f(mid);

        if fmid.abs() < tol || (b - a) / 2.0 < tol {
            return Ok(mid);
        }

        if fa * fmid < 0.0 {
            b = mid;
        } else {
            a = mid;
            fa = fmid;
        }
    }

    Ok((a + b) / 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brentq_polynomial() {
        // x^2 - 4 = 0, root at x = 2
        let root = brentq(|x| x * x - 4.0, 0.0, 5.0, 1e-10, 100).unwrap();
        assert!((root - 2.0).abs() < 1e-8);
    }

    #[test]
    fn test_brentq_trig() {
        // sin(x) = 0, root at x = π
        let root = brentq(|x| x.sin(), 3.0, 4.0, 1e-10, 100).unwrap();
        assert!((root - std::f64::consts::PI).abs() < 1e-8);
    }

    #[test]
    fn test_brentq_no_sign_change() {
        let result = brentq(|x| x * x + 1.0, 0.0, 5.0, 1e-10, 100);
        assert!(result.is_err());
    }

    #[test]
    fn test_newton_sqrt2() {
        // x^2 - 2 = 0, root at √2
        let root = newton(|x| x * x - 2.0, |x| 2.0 * x, 1.0, 1e-10, 100).unwrap();
        assert!((root - std::f64::consts::SQRT_2).abs() < 1e-8);
    }

    #[test]
    fn test_bisect() {
        let root = bisect(|x| x * x * x - 8.0, 0.0, 5.0, 1e-10, 100).unwrap();
        assert!((root - 2.0).abs() < 1e-8);
    }
}
