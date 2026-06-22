//! Exponential smoothing methods.
//!
//! - Simple Exponential Smoothing (SES)
//! - Double Exponential Smoothing (Holt's linear trend)
//! - Triple Exponential Smoothing (Holt-Winters, additive & multiplicative)

use super::error::{Result, TsError};

// ============================================================================
// Simple Exponential Smoothing
// ============================================================================

/// Simple exponential smoothing result.
#[derive(Debug, Clone)]
pub struct SesResult {
    pub fitted: Vec<f64>,
    pub alpha: f64,
    pub level: f64,
}

/// Simple exponential smoothing.
///
/// `alpha` ∈ (0, 1] controls the smoothing. Higher = more reactive.
pub fn ses(data: &[f64], alpha: f64) -> Result<SesResult> {
    validate_alpha(alpha, "alpha")?;
    if data.is_empty() {
        return Err(TsError::InsufficientData { need: 1, got: 0 });
    }

    let mut fitted = Vec::with_capacity(data.len());
    let mut level = data[0];
    fitted.push(level);

    for &x in &data[1..] {
        level = alpha * x + (1.0 - alpha) * level;
        fitted.push(level);
    }

    Ok(SesResult {
        fitted,
        alpha,
        level,
    })
}

impl SesResult {
    /// Forecast `h` steps ahead (flat forecast for SES).
    pub fn forecast(&self, h: usize) -> Vec<f64> {
        vec![self.level; h]
    }
}

// ============================================================================
// Holt's Linear Trend (Double Exponential Smoothing)
// ============================================================================

/// Result of Holt's linear trend method.
#[derive(Debug, Clone)]
pub struct HoltResult {
    pub fitted: Vec<f64>,
    pub alpha: f64,
    pub beta: f64,
    pub level: f64,
    pub trend: f64,
}

/// Holt's linear trend method (double exponential smoothing).
///
/// - `alpha` controls level smoothing
/// - `beta` controls trend smoothing
pub fn holt(data: &[f64], alpha: f64, beta: f64) -> Result<HoltResult> {
    validate_alpha(alpha, "alpha")?;
    validate_alpha(beta, "beta")?;
    if data.len() < 2 {
        return Err(TsError::InsufficientData {
            need: 2,
            got: data.len(),
        });
    }

    let mut level = data[0];
    let mut trend = data[1] - data[0];
    let mut fitted = Vec::with_capacity(data.len());
    fitted.push(level);

    for &x in &data[1..] {
        let prev_level = level;
        level = alpha * x + (1.0 - alpha) * (prev_level + trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * trend;
        fitted.push(level + trend);
    }

    Ok(HoltResult {
        fitted,
        alpha,
        beta,
        level,
        trend,
    })
}

impl HoltResult {
    /// Forecast `h` steps ahead using linear extrapolation.
    pub fn forecast(&self, h: usize) -> Vec<f64> {
        (1..=h)
            .map(|i| self.level + self.trend * i as f64)
            .collect()
    }
}

// ============================================================================
// Holt-Winters (Triple Exponential Smoothing)
// ============================================================================

/// Holt-Winters seasonal model type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HwSeasonal {
    Additive,
    Multiplicative,
}

/// Holt-Winters parameters.
#[derive(Debug, Clone)]
pub struct HoltWintersParams {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
    pub period: usize,
    pub seasonal: HwSeasonal,
}

/// Result of Holt-Winters smoothing.
#[derive(Debug, Clone)]
pub struct HoltWintersResult {
    pub fitted: Vec<f64>,
    pub params: HoltWintersParams,
    pub level: f64,
    pub trend: f64,
    pub seasonal_components: Vec<f64>,
}

/// Holt-Winters triple exponential smoothing.
pub fn holt_winters(data: &[f64], params: &HoltWintersParams) -> Result<HoltWintersResult> {
    validate_alpha(params.alpha, "alpha")?;
    validate_alpha(params.beta, "beta")?;
    validate_alpha(params.gamma, "gamma")?;

    let period = params.period;
    if period < 2 {
        return Err(TsError::InvalidParameter("period must be >= 2".into()));
    }
    if data.len() < 2 * period {
        return Err(TsError::InsufficientData {
            need: 2 * period,
            got: data.len(),
        });
    }

    match params.seasonal {
        HwSeasonal::Additive => hw_additive(data, params),
        HwSeasonal::Multiplicative => hw_multiplicative(data, params),
    }
}

fn hw_additive(data: &[f64], params: &HoltWintersParams) -> Result<HoltWintersResult> {
    let n = data.len();
    let m = params.period;
    let (alpha, beta, gamma) = (params.alpha, params.beta, params.gamma);

    // Initialize level and trend from first two periods
    let first_season_avg: f64 = data[..m].iter().sum::<f64>() / m as f64;
    let second_season_avg: f64 = data[m..2 * m].iter().sum::<f64>() / m as f64;
    let mut level = first_season_avg;
    let mut trend = (second_season_avg - first_season_avg) / m as f64;

    // Initialize seasonal components from first period
    let mut seasonal: Vec<f64> = (0..m).map(|i| data[i] - first_season_avg).collect();

    let mut fitted = Vec::with_capacity(n);
    // Fitted value for first point
    fitted.push(level + trend + seasonal[0]);

    for i in 1..n {
        let prev_level = level;
        let s_idx = i % m;

        level = alpha * (data[i] - seasonal[s_idx]) + (1.0 - alpha) * (prev_level + trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * trend;
        seasonal[s_idx] = gamma * (data[i] - level) + (1.0 - gamma) * seasonal[s_idx];

        fitted.push(level + trend + seasonal[(i + 1) % m]);
    }

    Ok(HoltWintersResult {
        fitted,
        params: params.clone(),
        level,
        trend,
        seasonal_components: seasonal,
    })
}

fn hw_multiplicative(data: &[f64], params: &HoltWintersParams) -> Result<HoltWintersResult> {
    let n = data.len();
    let m = params.period;
    let (alpha, beta, gamma) = (params.alpha, params.beta, params.gamma);

    // Check for zeros/negatives (multiplicative needs positive data)
    if data.iter().any(|&x| x <= 0.0) {
        return Err(TsError::InvalidParameter(
            "multiplicative seasonality requires positive data".into(),
        ));
    }

    let first_season_avg: f64 = data[..m].iter().sum::<f64>() / m as f64;
    let second_season_avg: f64 = data[m..2 * m].iter().sum::<f64>() / m as f64;
    let mut level = first_season_avg;
    let mut trend = (second_season_avg - first_season_avg) / m as f64;

    // Initialize seasonal as ratios
    let mut seasonal: Vec<f64> = (0..m).map(|i| data[i] / first_season_avg).collect();

    let mut fitted = Vec::with_capacity(n);
    fitted.push((level + trend) * seasonal[0]);

    for i in 1..n {
        let prev_level = level;
        let s_idx = i % m;

        level = alpha * (data[i] / seasonal[s_idx]) + (1.0 - alpha) * (prev_level + trend);
        trend = beta * (level - prev_level) + (1.0 - beta) * trend;
        seasonal[s_idx] = gamma * (data[i] / level) + (1.0 - gamma) * seasonal[s_idx];

        fitted.push((level + trend) * seasonal[(i + 1) % m]);
    }

    Ok(HoltWintersResult {
        fitted,
        params: params.clone(),
        level,
        trend,
        seasonal_components: seasonal,
    })
}

impl HoltWintersResult {
    /// Forecast `h` steps ahead.
    pub fn forecast(&self, h: usize) -> Vec<f64> {
        let m = self.params.period;
        (1..=h)
            .map(|i| {
                let s = self.seasonal_components[(self.fitted.len() + i - 1) % m];
                let base = self.level + self.trend * i as f64;
                match self.params.seasonal {
                    HwSeasonal::Additive => base + s,
                    HwSeasonal::Multiplicative => base * s,
                }
            })
            .collect()
    }
}

fn validate_alpha(val: f64, name: &str) -> Result<()> {
    if !(0.0..=1.0).contains(&val) {
        Err(TsError::InvalidParameter(format!(
            "{} must be in [0, 1], got {}",
            name, val
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ses_basic() {
        let data = vec![10.0, 12.0, 13.0, 14.0, 15.0];
        let result = ses(&data, 0.3).unwrap();
        assert_eq!(result.fitted.len(), 5);
        assert!((result.fitted[0] - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_ses_forecast() {
        let data = vec![10.0, 12.0, 14.0, 16.0];
        let result = ses(&data, 0.5).unwrap();
        let fc = result.forecast(3);
        assert_eq!(fc.len(), 3);
        // All forecasts should be the same (flat)
        assert!((fc[0] - fc[1]).abs() < 1e-10);
    }

    #[test]
    fn test_holt_trend() {
        let data: Vec<f64> = (0..20).map(|i| 10.0 + 2.0 * i as f64).collect();
        let result = holt(&data, 0.8, 0.2).unwrap();
        let fc = result.forecast(3);
        // Forecast should continue the upward trend
        assert!(fc[0] > data[19]);
        assert!(fc[1] > fc[0]);
    }

    #[test]
    fn test_holt_winters_additive() {
        let data: Vec<f64> = (0..48)
            .map(|i| {
                100.0 + i as f64 * 0.5 + 10.0 * (2.0 * std::f64::consts::PI * i as f64 / 12.0).sin()
            })
            .collect();
        let params = HoltWintersParams {
            alpha: 0.3,
            beta: 0.1,
            gamma: 0.3,
            period: 12,
            seasonal: HwSeasonal::Additive,
        };
        let result = holt_winters(&data, &params).unwrap();
        assert_eq!(result.fitted.len(), 48);
        let fc = result.forecast(12);
        assert_eq!(fc.len(), 12);
    }

    #[test]
    fn test_holt_winters_multiplicative() {
        let data: Vec<f64> = (0..48)
            .map(|i| {
                let trend = 100.0 + i as f64;
                let seasonal = 1.0 + 0.2 * (2.0 * std::f64::consts::PI * i as f64 / 12.0).sin();
                trend * seasonal
            })
            .collect();
        let params = HoltWintersParams {
            alpha: 0.3,
            beta: 0.1,
            gamma: 0.3,
            period: 12,
            seasonal: HwSeasonal::Multiplicative,
        };
        let result = holt_winters(&data, &params).unwrap();
        assert_eq!(result.fitted.len(), 48);
    }

    #[test]
    fn test_invalid_alpha() {
        assert!(ses(&[1.0], 1.5).is_err());
        assert!(ses(&[1.0], -0.1).is_err());
    }

    #[test]
    fn test_insufficient_data_ses() {
        assert!(ses(&[], 0.5).is_err());
    }

    #[test]
    fn test_insufficient_data_holt() {
        assert!(holt(&[1.0], 0.5, 0.5).is_err());
    }

    #[test]
    fn test_insufficient_data_hw() {
        let params = HoltWintersParams {
            alpha: 0.3,
            beta: 0.1,
            gamma: 0.3,
            period: 12,
            seasonal: HwSeasonal::Additive,
        };
        assert!(holt_winters(&[1.0; 10], &params).is_err());
    }
}
