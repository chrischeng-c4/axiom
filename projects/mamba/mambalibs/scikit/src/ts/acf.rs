//! Autocorrelation (ACF), partial autocorrelation (PACF), and Ljung-Box test.

use super::error::{Result, TsError};

// ============================================================================
// ACF
// ============================================================================

/// Compute the autocorrelation function for lags 0..=max_lag.
///
/// Returns a vector of length `max_lag + 1` where `result[k]` is the
/// autocorrelation at lag k. `result[0]` is always 1.0.
pub fn acf(data: &[f64], max_lag: usize) -> Result<Vec<f64>> {
    let n = data.len();
    if n < 2 {
        return Err(TsError::InsufficientData { need: 2, got: n });
    }
    let max_lag = max_lag.min(n - 1);

    let mean = data.iter().sum::<f64>() / n as f64;
    let var: f64 = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;

    if var.abs() < 1e-15 {
        // Constant series
        let mut result = vec![0.0; max_lag + 1];
        result[0] = 1.0;
        return Ok(result);
    }

    let mut result = Vec::with_capacity(max_lag + 1);
    for k in 0..=max_lag {
        let cov: f64 = (0..n - k)
            .map(|i| (data[i] - mean) * (data[i + k] - mean))
            .sum::<f64>()
            / n as f64;
        result.push(cov / var);
    }
    Ok(result)
}

// ============================================================================
// PACF (Durbin-Levinson recursion)
// ============================================================================

/// Compute the partial autocorrelation function for lags 1..=max_lag.
///
/// Uses the Durbin-Levinson algorithm.
/// Returns a vector of length `max_lag` where `result[k-1]` is the PACF at lag k.
pub fn pacf(data: &[f64], max_lag: usize) -> Result<Vec<f64>> {
    let acf_vals = acf(data, max_lag)?;
    pacf_from_acf(&acf_vals)
}

/// Compute PACF from pre-computed ACF values using Durbin-Levinson.
pub fn pacf_from_acf(acf_vals: &[f64]) -> Result<Vec<f64>> {
    let max_lag = acf_vals.len() - 1; // acf_vals[0] = 1.0
    if max_lag == 0 {
        return Ok(Vec::new());
    }

    let mut pacf_result = Vec::with_capacity(max_lag);
    let mut phi = vec![vec![0.0; max_lag + 1]; max_lag + 1];

    // Order 1
    phi[1][1] = acf_vals[1];
    pacf_result.push(phi[1][1]);

    for k in 2..=max_lag {
        // Numerator: r(k) - sum_{j=1}^{k-1} phi[k-1][j] * r(k-j)
        let num = acf_vals[k] - (1..k).map(|j| phi[k - 1][j] * acf_vals[k - j]).sum::<f64>();
        // Denominator: 1 - sum_{j=1}^{k-1} phi[k-1][j] * r(j)
        let den = 1.0 - (1..k).map(|j| phi[k - 1][j] * acf_vals[j]).sum::<f64>();

        if den.abs() < 1e-15 {
            // Degenerate — fill remaining with 0
            for _ in pacf_result.len()..max_lag {
                pacf_result.push(0.0);
            }
            return Ok(pacf_result);
        }

        phi[k][k] = num / den;
        pacf_result.push(phi[k][k]);

        // Update lower-order coefficients
        for j in 1..k {
            phi[k][j] = phi[k - 1][j] - phi[k][k] * phi[k - 1][k - j];
        }
    }

    Ok(pacf_result)
}

// ============================================================================
// Ljung-Box test
// ============================================================================

/// Result of the Ljung-Box test.
#[derive(Debug, Clone)]
pub struct LjungBoxResult {
    pub statistic: f64,
    pub p_value: f64,
    pub lags: usize,
}

impl LjungBoxResult {
    pub fn is_significant(&self, alpha: f64) -> bool {
        self.p_value < alpha
    }
}

/// Perform the Ljung-Box test for autocorrelation.
///
/// Tests H0: the data are independently distributed (no autocorrelation).
/// `lags` is the number of lags to include in the test.
///
/// Uses chi-squared CDF approximation for p-value.
pub fn ljung_box(data: &[f64], lags: usize) -> Result<LjungBoxResult> {
    let n = data.len();
    if n < lags + 1 {
        return Err(TsError::InsufficientData {
            need: lags + 1,
            got: n,
        });
    }

    let acf_vals = acf(data, lags)?;
    let nf = n as f64;

    let q: f64 = (1..=lags)
        .map(|k| {
            let r_k = acf_vals[k];
            r_k * r_k / (nf - k as f64)
        })
        .sum::<f64>()
        * nf
        * (nf + 2.0);

    let p_value = chi2_survival(q, lags as f64);

    Ok(LjungBoxResult {
        statistic: q,
        p_value,
        lags,
    })
}

// ============================================================================
// Chi-squared survival function (1 - CDF) approximation
// ============================================================================

/// Approximate chi-squared survival function P(X > x) for X ~ chi2(df).
///
/// Uses the regularized incomplete gamma function.
fn chi2_survival(x: f64, df: f64) -> f64 {
    if x <= 0.0 {
        return 1.0;
    }
    1.0 - regularized_lower_gamma(df / 2.0, x / 2.0)
}

/// Regularized lower incomplete gamma function P(a, x) = gamma(a, x) / Gamma(a).
///
/// Uses series expansion for small x, continued fraction for large x.
fn regularized_lower_gamma(a: f64, x: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    if x == 0.0 {
        return 0.0;
    }
    if x < a + 1.0 {
        gamma_series(a, x)
    } else {
        1.0 - gamma_cf(a, x)
    }
}

/// Series representation for P(a, x).
fn gamma_series(a: f64, x: f64) -> f64 {
    let ln_gamma_a = ln_gamma(a);
    let mut sum = 1.0 / a;
    let mut term = 1.0 / a;
    for n in 1..200 {
        term *= x / (a + n as f64);
        sum += term;
        if term.abs() < sum.abs() * 1e-14 {
            break;
        }
    }
    sum * (-x + a * x.ln() - ln_gamma_a).exp()
}

/// Continued fraction representation for Q(a, x) = 1 - P(a, x).
fn gamma_cf(a: f64, x: f64) -> f64 {
    let ln_gamma_a = ln_gamma(a);
    let mut c = 1e-30_f64;

    // Modified Lentz's method
    let b0 = x + 1.0 - a;
    let mut d = 1.0 / b0;
    let mut f = d;

    for i in 1..200 {
        let an = -(i as f64) * (i as f64 - a);
        let bn = x + (2 * i + 1) as f64 - a;
        d = bn + an * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = bn + an / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let delta = c * d;
        f *= delta;
        if (delta - 1.0).abs() < 1e-14 {
            break;
        }
    }

    f * (-x + a * x.ln() - ln_gamma_a).exp()
}

/// Lanczos approximation for ln(Gamma(x)).
fn ln_gamma(x: f64) -> f64 {
    let coeffs = [
        76.18009172947146,
        -86.50532032941677,
        24.01409824083091,
        -1.231739572450155,
        0.1208650973866179e-2,
        -0.5395239384953e-5,
    ];
    let y = x;
    let tmp = x + 5.5;
    let tmp = tmp - (x + 0.5) * tmp.ln();
    let mut ser = 1.000000000190015;
    for (i, &c) in coeffs.iter().enumerate() {
        ser += c / (y + 1.0 + i as f64);
    }
    -tmp + (2.5066282746310005 * ser / x).ln()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acf_constant() {
        let data = vec![5.0; 10];
        let result = acf(&data, 3).unwrap();
        assert!((result[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_acf_sine() {
        // Sine wave should have periodic ACF
        let data: Vec<f64> = (0..100)
            .map(|i| (i as f64 * std::f64::consts::PI / 5.0).sin())
            .collect();
        let result = acf(&data, 20).unwrap();
        assert!((result[0] - 1.0).abs() < 1e-10);
        // ACF at lag 10 should be close to 1 (period = 10)
        assert!(result[10] > 0.8);
    }

    #[test]
    fn test_pacf_ar1() {
        // AR(1) process: x[t] = 0.7 * x[t-1] + noise
        let mut data = vec![0.0; 200];
        let mut rng = 42u64;
        for i in 1..200 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let noise = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 0.3;
            data[i] = 0.7 * data[i - 1] + noise;
        }
        let result = pacf(&data, 5).unwrap();
        // PACF at lag 1 should be close to 0.7
        assert!((result[0] - 0.7).abs() < 0.15);
        // PACF at lag 2+ should be close to 0
        assert!(result[1].abs() < 0.2);
    }

    #[test]
    fn test_ljung_box_white_noise() {
        // White noise should not be significant
        let mut data = Vec::with_capacity(500);
        let mut rng = 123u64;
        for _ in 0..500 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            data.push((rng >> 33) as f64 / u32::MAX as f64 - 0.5);
        }
        let result = ljung_box(&data, 10).unwrap();
        // p-value should be > 0.05 for white noise (usually)
        assert!(result.p_value > 0.01);
    }

    #[test]
    fn test_ljung_box_autocorrelated() {
        // Highly autocorrelated: cumulative sum
        let mut data = vec![0.0; 100];
        for i in 1..100 {
            data[i] = data[i - 1] + 1.0;
        }
        let result = ljung_box(&data, 5).unwrap();
        assert!(result.is_significant(0.05));
    }

    #[test]
    fn test_insufficient_data() {
        assert!(acf(&[1.0], 1).is_err());
        assert!(ljung_box(&[1.0, 2.0], 5).is_err());
    }
}
