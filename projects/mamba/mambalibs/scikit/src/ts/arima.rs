//! ARIMA(p, d, q) modeling.
//!
//! - AR coefficients estimated via Yule-Walker equations (uses `NdArray::solve`)
//! - MA coefficients estimated via conditional sum of squares (CSS) optimization
//! - Differencing for the I(d) part

use super::error::{Result, TsError};

/// An ARIMA(p, d, q) model.
#[derive(Debug, Clone)]
pub struct ArimaModel {
    pub p: usize,
    pub d: usize,
    pub q: usize,
    pub ar_coeffs: Vec<f64>,
    pub ma_coeffs: Vec<f64>,
    pub intercept: f64,
    diff_mean: f64,
}

/// ARIMA order specification.
#[derive(Debug, Clone, Copy)]
pub struct ArimaOrder {
    pub p: usize,
    pub d: usize,
    pub q: usize,
}

impl ArimaOrder {
    pub fn new(p: usize, d: usize, q: usize) -> Self {
        Self { p, d, q }
    }
}

impl ArimaModel {
    /// Fit an ARIMA(p,d,q) model to the data.
    pub fn fit(data: &[f64], order: ArimaOrder) -> Result<Self> {
        let n = data.len();
        let min_needed = order.p + order.d + order.q + 2;
        if n < min_needed {
            return Err(TsError::InsufficientData {
                need: min_needed,
                got: n,
            });
        }

        // Step 1: Difference the series
        let diffed = difference(data, order.d);
        let mean = diffed.iter().sum::<f64>() / diffed.len() as f64;

        // Center the data
        let centered: Vec<f64> = diffed.iter().map(|x| x - mean).collect();

        // Step 2: Estimate AR coefficients via Yule-Walker
        let ar_coeffs = if order.p > 0 {
            yule_walker(&centered, order.p)?
        } else {
            Vec::new()
        };

        // Step 3: Estimate MA coefficients via CSS
        let ma_coeffs = if order.q > 0 {
            estimate_ma_css(&centered, &ar_coeffs, order.q)?
        } else {
            Vec::new()
        };

        Ok(ArimaModel {
            p: order.p,
            d: order.d,
            q: order.q,
            ar_coeffs,
            ma_coeffs,
            intercept: mean,
            diff_mean: mean,
        })
    }

    /// One-step-ahead predictions on the training data (in-sample).
    pub fn fitted_values(&self, data: &[f64]) -> Vec<f64> {
        let diffed = difference(data, self.d);
        let n = diffed.len();
        let mut preds = vec![self.diff_mean; n];
        let mut residuals = vec![0.0; n];

        let start = self.p.max(self.q);
        for t in start..n {
            let mut val = self.intercept;
            // AR part
            for j in 0..self.p {
                val += self.ar_coeffs[j] * (diffed[t - 1 - j] - self.diff_mean);
            }
            // MA part
            for j in 0..self.q {
                val += self.ma_coeffs[j] * residuals[t - 1 - j];
            }
            preds[t] = val;
            residuals[t] = diffed[t] - val;
        }

        // Undifference
        undifference(&preds, data, self.d)
    }

    /// Forecast `h` steps ahead.
    pub fn forecast(&self, data: &[f64], h: usize) -> Vec<f64> {
        let diffed = difference(data, self.d);
        let n = diffed.len();

        // Build residuals from in-sample
        let mut residuals = vec![0.0; n];
        let start = self.p.max(self.q);
        for t in start..n {
            let mut val = self.intercept;
            for j in 0..self.p {
                val += self.ar_coeffs[j] * (diffed[t - 1 - j] - self.diff_mean);
            }
            for j in 0..self.q {
                val += self.ma_coeffs[j] * residuals[t - 1 - j];
            }
            residuals[t] = diffed[t] - val;
        }

        // Extend series for forecasting
        let mut extended = diffed.clone();
        let mut ext_resid = residuals;

        for _ in 0..h {
            let t = extended.len();
            let mut val = self.intercept;
            for j in 0..self.p {
                val += self.ar_coeffs[j] * (extended[t - 1 - j] - self.diff_mean);
            }
            for j in 0..self.q {
                if t - 1 - j < ext_resid.len() {
                    val += self.ma_coeffs[j] * ext_resid[t - 1 - j];
                }
                // Future residuals are 0 (expected value)
            }
            extended.push(val);
            ext_resid.push(0.0);
        }

        // Extract forecast portion and undifference
        let forecast_diff: Vec<f64> = extended[n..].to_vec();
        undifference_forecast(&forecast_diff, data, self.d)
    }

    /// Compute AIC for model comparison.
    ///
    /// AIC = n * ln(RSS/n) + 2 * k, where k = p + q + 1
    pub fn aic(&self, data: &[f64]) -> f64 {
        let diffed = difference(data, self.d);
        let fitted = self.fitted_values(data);
        let fitted_diff = difference(&fitted, self.d);

        let n = diffed.len();
        let start = self.p.max(self.q);
        let effective_n = n - start;
        if effective_n == 0 {
            return f64::INFINITY;
        }

        let rss: f64 = (start..n)
            .map(|i| (diffed[i] - fitted_diff[i]).powi(2))
            .sum();

        let k = (self.p + self.q + 1) as f64;
        effective_n as f64 * (rss / effective_n as f64).ln() + 2.0 * k
    }
}

// ============================================================================
// Differencing
// ============================================================================

/// Apply d-th order differencing.
pub fn difference(data: &[f64], d: usize) -> Vec<f64> {
    let mut result = data.to_vec();
    for _ in 0..d {
        let prev = result.clone();
        result = prev.windows(2).map(|w| w[1] - w[0]).collect();
    }
    result
}

/// Undifference in-sample predictions.
fn undifference(preds_diff: &[f64], original: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return preds_diff.to_vec();
    }

    // For d=1: pred_original[t] = pred_diff[t-1] + original[t-1]
    // We simply return the original-scale fitted values using cumulative sum
    let mut result = preds_diff.to_vec();
    for _ in 0..d {
        let mut undiffed = Vec::with_capacity(result.len() + 1);
        undiffed.push(original[0]); // Use first original value as anchor
        for &r in &result {
            let prev = *undiffed.last().unwrap();
            undiffed.push(prev + r);
        }
        result = undiffed;
    }
    // Trim to match original length
    result.truncate(original.len());
    result
}

/// Undifference forecast values.
fn undifference_forecast(forecast_diff: &[f64], original: &[f64], d: usize) -> Vec<f64> {
    if d == 0 {
        return forecast_diff.to_vec();
    }

    let mut result = forecast_diff.to_vec();
    for dd in 0..d {
        let mut undiffed = Vec::with_capacity(result.len());
        // Use the last value(s) of the original data as anchor
        let anchor = if dd == 0 {
            *original.last().unwrap()
        } else {
            // For higher-order differencing, use the cumsum approach
            *original.last().unwrap()
        };
        let mut prev = anchor;
        for &r in &result {
            prev += r;
            undiffed.push(prev);
        }
        result = undiffed;
    }
    result
}

// ============================================================================
// Yule-Walker AR estimation
// ============================================================================

/// Estimate AR coefficients via Yule-Walker using Durbin-Levinson recursion.
///
/// More numerically stable than solving the full Toeplitz system directly.
fn yule_walker(data: &[f64], p: usize) -> Result<Vec<f64>> {
    let n = data.len();
    if n < p + 1 {
        return Err(TsError::InsufficientData {
            need: p + 1,
            got: n,
        });
    }

    // Compute autocorrelations r(0)..r(p)
    let mean = data.iter().sum::<f64>() / n as f64;
    let centered: Vec<f64> = data.iter().map(|x| x - mean).collect();

    let mut r = vec![0.0; p + 1];
    for k in 0..=p {
        for i in 0..n - k {
            r[k] += centered[i] * centered[i + k];
        }
        r[k] /= n as f64;
    }

    if r[0].abs() < 1e-15 {
        return Ok(vec![0.0; p]);
    }

    // Normalize to autocorrelation (r[k] / r[0])
    let acf: Vec<f64> = r.iter().map(|&x| x / r[0]).collect();

    // Durbin-Levinson recursion
    let mut phi = vec![vec![0.0; p + 1]; p + 1];
    phi[1][1] = acf[1];

    for k in 2..=p {
        let num = acf[k]
            - (1..k).map(|j| phi[k - 1][j] * acf[k - j]).sum::<f64>();
        let den = 1.0
            - (1..k).map(|j| phi[k - 1][j] * acf[j]).sum::<f64>();

        if den.abs() < 1e-15 {
            // Near-singular: return what we have so far
            let mut coeffs = vec![0.0; p];
            for j in 1..k {
                coeffs[j - 1] = phi[k - 1][j];
            }
            return Ok(coeffs);
        }

        phi[k][k] = num / den;
        for j in 1..k {
            phi[k][j] = phi[k - 1][j] - phi[k][k] * phi[k - 1][k - j];
        }
    }

    let coeffs: Vec<f64> = (1..=p).map(|j| phi[p][j]).collect();
    Ok(coeffs)
}

// ============================================================================
// MA estimation via Conditional Sum of Squares
// ============================================================================

/// Estimate MA coefficients using CSS (grid search + refinement).
///
/// This is a simplified approach: iterate over the residuals and
/// use gradient-free optimization.
fn estimate_ma_css(data: &[f64], ar_coeffs: &[f64], q: usize) -> Result<Vec<f64>> {
    let p = ar_coeffs.len();
    let n = data.len();
    let start = p.max(q);

    if n <= start {
        return Ok(vec![0.0; q]);
    }

    // Initial MA coefficients = 0
    let mut ma = vec![0.0; q];
    let step_sizes = [0.1, 0.01, 0.001];
    let max_iters = 100;

    for &step in &step_sizes {
        for _ in 0..max_iters {
            let base_css = compute_css(data, ar_coeffs, &ma, start);
            let mut improved = false;

            for j in 0..q {
                // Try +step
                ma[j] += step;
                let css_plus = compute_css(data, ar_coeffs, &ma, start);
                ma[j] -= step;

                // Try -step
                ma[j] -= step;
                let css_minus = compute_css(data, ar_coeffs, &ma, start);
                ma[j] += step;

                if css_plus < base_css && css_plus <= css_minus {
                    ma[j] += step;
                    improved = true;
                } else if css_minus < base_css {
                    ma[j] -= step;
                    improved = true;
                }
            }

            if !improved {
                break;
            }
        }
    }

    // Clamp MA coefficients to invertibility region
    for coeff in &mut ma {
        *coeff = coeff.clamp(-0.99, 0.99);
    }

    Ok(ma)
}

/// Compute conditional sum of squares for given AR and MA coefficients.
fn compute_css(data: &[f64], ar: &[f64], ma: &[f64], start: usize) -> f64 {
    let n = data.len();
    let p = ar.len();
    let q = ma.len();
    let mut residuals = vec![0.0; n];
    let mut css = 0.0;

    for t in start..n {
        let mut pred = 0.0;
        for j in 0..p {
            pred += ar[j] * data[t - 1 - j];
        }
        for j in 0..q {
            pred += ma[j] * residuals[t - 1 - j];
        }
        residuals[t] = data[t] - pred;
        css += residuals[t] * residuals[t];
    }
    css
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difference_d1() {
        let data = vec![1.0, 3.0, 6.0, 10.0];
        let d = difference(&data, 1);
        assert_eq!(d.len(), 3);
        assert!((d[0] - 2.0).abs() < 1e-10);
        assert!((d[1] - 3.0).abs() < 1e-10);
        assert!((d[2] - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_difference_d2() {
        let data = vec![1.0, 3.0, 6.0, 10.0, 15.0];
        let d = difference(&data, 2);
        assert_eq!(d.len(), 3);
        // First diff: [2, 3, 4, 5], second diff: [1, 1, 1]
        assert!((d[0] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_ar1_fit() {
        // Generate AR(1) process: x[t] = 0.7 * x[t-1] + noise
        let mut data = vec![0.0; 500];
        let mut rng = 42u64;
        for i in 1..500 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let noise = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 0.5;
            data[i] = 0.7 * data[i - 1] + noise;
        }

        let model = ArimaModel::fit(&data, ArimaOrder::new(1, 0, 0)).unwrap();
        assert_eq!(model.ar_coeffs.len(), 1);
        // AR(1) coefficient should be close to 0.7
        assert!(
            (model.ar_coeffs[0] - 0.7).abs() < 0.15,
            "AR(1) coeff = {}, expected ~0.7",
            model.ar_coeffs[0]
        );
    }

    #[test]
    fn test_arima_forecast() {
        let data: Vec<f64> = (0..100).map(|i| 50.0 + 2.0 * i as f64).collect();
        let model = ArimaModel::fit(&data, ArimaOrder::new(1, 1, 0)).unwrap();
        let forecast = model.forecast(&data, 5);
        assert_eq!(forecast.len(), 5);
        // Forecast should continue the upward trend
        assert!(forecast[0] > data[99]);
    }

    #[test]
    fn test_arima_fitted() {
        let data: Vec<f64> = (0..50).map(|i| 10.0 + i as f64).collect();
        let model = ArimaModel::fit(&data, ArimaOrder::new(1, 1, 0)).unwrap();
        let fitted = model.fitted_values(&data);
        assert_eq!(fitted.len(), 50);
    }

    #[test]
    fn test_insufficient_data() {
        assert!(ArimaModel::fit(&[1.0, 2.0], ArimaOrder::new(3, 1, 1)).is_err());
    }

    #[test]
    fn test_arma_with_ma() {
        // Simple test that ARMA(1,1) doesn't panic
        let mut data = vec![0.0; 200];
        let mut rng = 99u64;
        for i in 1..200 {
            rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
            let noise = ((rng >> 33) as f64 / u32::MAX as f64 - 0.5) * 1.0;
            data[i] = 0.5 * data[i - 1] + noise;
        }
        let model = ArimaModel::fit(&data, ArimaOrder::new(1, 0, 1)).unwrap();
        assert_eq!(model.ma_coeffs.len(), 1);
        let fc = model.forecast(&data, 3);
        assert_eq!(fc.len(), 3);
    }

    #[test]
    fn test_aic() {
        let data: Vec<f64> = (0..100).map(|i| (i as f64 * 0.1).sin() * 10.0).collect();
        let m1 = ArimaModel::fit(&data, ArimaOrder::new(1, 0, 0)).unwrap();
        let m2 = ArimaModel::fit(&data, ArimaOrder::new(2, 0, 0)).unwrap();
        let aic1 = m1.aic(&data);
        let aic2 = m2.aic(&data);
        // Both should be finite
        assert!(aic1.is_finite());
        assert!(aic2.is_finite());
    }
}
