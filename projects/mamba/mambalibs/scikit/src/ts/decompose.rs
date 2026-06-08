//! Seasonal decomposition of time series.
//!
//! Supports additive and multiplicative models using moving-average
//! trend extraction followed by seasonal averaging.

use super::error::{Result, TsError};

/// Decomposition model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecomposeModel {
    Additive,
    Multiplicative,
}

/// Result of seasonal decomposition.
#[derive(Debug, Clone)]
pub struct DecomposeResult {
    pub observed: Vec<f64>,
    pub trend: Vec<f64>,
    pub seasonal: Vec<f64>,
    pub residual: Vec<f64>,
    pub model: DecomposeModel,
    pub period: usize,
}

/// Perform classical seasonal decomposition.
///
/// `period` is the seasonal period (e.g. 12 for monthly data with yearly cycle).
/// Uses centered moving average for trend extraction.
pub fn seasonal_decompose(
    data: &[f64],
    period: usize,
    model: DecomposeModel,
) -> Result<DecomposeResult> {
    let n = data.len();
    if period < 2 {
        return Err(TsError::InvalidParameter("period must be >= 2".into()));
    }
    if n < 2 * period {
        return Err(TsError::InsufficientData {
            need: 2 * period,
            got: n,
        });
    }

    // Step 1: Extract trend via centered moving average
    let trend = centered_moving_average(data, period);

    // Step 2: Detrend
    let detrended = detrend(data, &trend, model);

    // Step 3: Compute seasonal component by averaging each position within period
    let seasonal_pattern = compute_seasonal_pattern(&detrended, period, model);

    // Tile the seasonal pattern across the full length
    let seasonal: Vec<f64> = (0..n).map(|i| seasonal_pattern[i % period]).collect();

    // Step 4: Compute residual
    let residual = compute_residual(data, &trend, &seasonal, model);

    Ok(DecomposeResult {
        observed: data.to_vec(),
        trend,
        seasonal,
        residual,
        model,
        period,
    })
}

/// Centered moving average for trend extraction.
///
/// For even period, uses a 2×period moving average (convolution).
fn centered_moving_average(data: &[f64], period: usize) -> Vec<f64> {
    let n = data.len();
    let mut trend = vec![f64::NAN; n];

    if period % 2 == 1 {
        // Odd period: simple centered MA
        let half = period / 2;
        for i in half..n - half {
            let sum: f64 = data[i - half..=i + half].iter().sum();
            trend[i] = sum / period as f64;
        }
    } else {
        // Even period: 2-step MA (first period-MA, then 2-MA to center)
        let half = period / 2;
        // First pass: period-MA
        let mut ma1 = vec![f64::NAN; n];
        for i in 0..n - period + 1 {
            let sum: f64 = data[i..i + period].iter().sum();
            ma1[i + half] = sum / period as f64;
        }
        // Second pass: 2-MA to center
        for i in 1..n {
            if !ma1[i].is_nan() && !ma1[i - 1].is_nan() {
                trend[i] = (ma1[i] + ma1[i - 1]) / 2.0;
            }
        }
    }

    trend
}

/// Remove trend from data.
fn detrend(data: &[f64], trend: &[f64], model: DecomposeModel) -> Vec<f64> {
    data.iter()
        .zip(trend.iter())
        .map(|(&d, &t)| {
            if t.is_nan() {
                f64::NAN
            } else {
                match model {
                    DecomposeModel::Additive => d - t,
                    DecomposeModel::Multiplicative => {
                        if t.abs() < 1e-15 {
                            f64::NAN
                        } else {
                            d / t
                        }
                    }
                }
            }
        })
        .collect()
}

/// Average the detrended values at each position within the period.
fn compute_seasonal_pattern(detrended: &[f64], period: usize, model: DecomposeModel) -> Vec<f64> {
    let mut sums = vec![0.0; period];
    let mut counts = vec![0usize; period];

    for (i, &val) in detrended.iter().enumerate() {
        if !val.is_nan() {
            sums[i % period] += val;
            counts[i % period] += 1;
        }
    }

    let mut pattern: Vec<f64> = sums
        .iter()
        .zip(counts.iter())
        .map(|(&s, &c)| if c > 0 { s / c as f64 } else { 0.0 })
        .collect();

    // Normalize: additive → zero-mean, multiplicative → mean = 1
    match model {
        DecomposeModel::Additive => {
            let mean = pattern.iter().sum::<f64>() / period as f64;
            for v in &mut pattern {
                *v -= mean;
            }
        }
        DecomposeModel::Multiplicative => {
            let mean = pattern.iter().sum::<f64>() / period as f64;
            if mean.abs() > 1e-15 {
                for v in &mut pattern {
                    *v /= mean;
                }
            }
        }
    }

    pattern
}

/// Compute residual = observed - trend - seasonal (additive)
///                  or observed / (trend * seasonal) (multiplicative).
fn compute_residual(
    data: &[f64],
    trend: &[f64],
    seasonal: &[f64],
    model: DecomposeModel,
) -> Vec<f64> {
    data.iter()
        .zip(trend.iter())
        .zip(seasonal.iter())
        .map(|((&d, &t), &s)| {
            if t.is_nan() {
                f64::NAN
            } else {
                match model {
                    DecomposeModel::Additive => d - t - s,
                    DecomposeModel::Multiplicative => {
                        let ts = t * s;
                        if ts.abs() < 1e-15 {
                            f64::NAN
                        } else {
                            d / ts
                        }
                    }
                }
            }
        })
        .collect()
}

/// Compute the strength of trend: `1 - Var(residual) / Var(detrended)`.
impl DecomposeResult {
    pub fn trend_strength(&self) -> f64 {
        let detrended: Vec<f64> = self
            .observed
            .iter()
            .zip(self.trend.iter())
            .filter(|(_, t)| !t.is_nan())
            .map(|(&o, &t)| match self.model {
                DecomposeModel::Additive => o - t,
                DecomposeModel::Multiplicative => {
                    if t.abs() < 1e-15 {
                        0.0
                    } else {
                        o / t
                    }
                }
            })
            .collect();

        let residual_clean: Vec<f64> = self
            .residual
            .iter()
            .filter(|r| !r.is_nan())
            .copied()
            .collect();

        let var_r = variance(&residual_clean);
        let var_d = variance(&detrended);

        if var_d < 1e-15 {
            0.0
        } else {
            (1.0 - var_r / var_d).max(0.0)
        }
    }

    pub fn seasonal_strength(&self) -> f64 {
        let resid_plus_seasonal: Vec<f64> = self
            .residual
            .iter()
            .zip(self.seasonal.iter())
            .filter(|(r, _)| !r.is_nan())
            .map(|(&r, &s)| match self.model {
                DecomposeModel::Additive => r + s,
                DecomposeModel::Multiplicative => r * s,
            })
            .collect();

        let residual_clean: Vec<f64> = self
            .residual
            .iter()
            .filter(|r| !r.is_nan())
            .copied()
            .collect();

        let var_r = variance(&residual_clean);
        let var_rs = variance(&resid_plus_seasonal);

        if var_rs < 1e-15 {
            0.0
        } else {
            (1.0 - var_r / var_rs).max(0.0)
        }
    }
}

fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / data.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_seasonal_data(n: usize, period: usize) -> Vec<f64> {
        (0..n)
            .map(|i| {
                let trend = i as f64 * 0.5;
                let seasonal = (2.0 * std::f64::consts::PI * i as f64 / period as f64).sin() * 10.0;
                trend + seasonal + 100.0
            })
            .collect()
    }

    #[test]
    fn test_additive_decompose() {
        let data = make_seasonal_data(120, 12);
        let result = seasonal_decompose(&data, 12, DecomposeModel::Additive).unwrap();
        assert_eq!(result.observed.len(), 120);
        assert_eq!(result.trend.len(), 120);
        assert_eq!(result.seasonal.len(), 120);
        assert_eq!(result.residual.len(), 120);
        // Seasonal component should repeat with period 12
        for i in 12..108 {
            let diff = (result.seasonal[i] - result.seasonal[i % 12]).abs();
            assert!(
                diff < 1e-10,
                "seasonal not periodic at i={}: diff={}",
                i,
                diff
            );
        }
    }

    #[test]
    fn test_multiplicative_decompose() {
        let data: Vec<f64> = (0..60)
            .map(|i| {
                let trend = 100.0 + i as f64;
                let seasonal = 1.0 + 0.1 * (2.0 * std::f64::consts::PI * i as f64 / 12.0).sin();
                trend * seasonal
            })
            .collect();
        let result = seasonal_decompose(&data, 12, DecomposeModel::Multiplicative).unwrap();
        assert_eq!(result.model, DecomposeModel::Multiplicative);
        // Seasonal should be close to multiplicative factors (around 1.0)
        let mid = result.seasonal[30];
        assert!(mid > 0.5 && mid < 1.5);
    }

    #[test]
    fn test_trend_strength() {
        let data = make_seasonal_data(120, 12);
        let result = seasonal_decompose(&data, 12, DecomposeModel::Additive).unwrap();
        let ts = result.trend_strength();
        assert!(ts > 0.0 && ts <= 1.0);
    }

    #[test]
    fn test_seasonal_strength() {
        let data = make_seasonal_data(120, 12);
        let result = seasonal_decompose(&data, 12, DecomposeModel::Additive).unwrap();
        let ss = result.seasonal_strength();
        assert!(
            ss > 0.5,
            "seasonal_strength={} should be > 0.5 for strong seasonal data",
            ss
        );
    }

    #[test]
    fn test_insufficient_data() {
        assert!(seasonal_decompose(&[1.0, 2.0, 3.0], 4, DecomposeModel::Additive).is_err());
    }

    #[test]
    fn test_invalid_period() {
        assert!(seasonal_decompose(&[1.0; 20], 1, DecomposeModel::Additive).is_err());
    }
}
