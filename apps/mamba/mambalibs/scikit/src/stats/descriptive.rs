//! Descriptive statistics functions.
//!
//! Implements advanced statistical measures: skewness, kurtosis, z-scores, moments.

/// Calculate the skewness of a dataset.
///
/// Skewness measures the asymmetry of the distribution.
/// - Positive skew: tail extends to the right
/// - Negative skew: tail extends to the left
/// - Zero skew: symmetric distribution
///
/// Uses Fisher's definition (adjusted for sample size).
pub fn skew(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 3.0 {
        return f64::NAN;
    }

    let mean = data.iter().sum::<f64>() / n;
    let m2: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let m3: f64 = data.iter().map(|&x| (x - mean).powi(3)).sum::<f64>() / n;

    if m2 == 0.0 {
        return 0.0;
    }

    let g1 = m3 / m2.powf(1.5);

    // Adjust for sample bias (Fisher's adjustment)
    let adjustment = ((n * (n - 1.0)).sqrt()) / (n - 2.0);
    adjustment * g1
}

/// Calculate the kurtosis of a dataset.
///
/// Kurtosis measures the "tailedness" of the distribution.
/// - Excess kurtosis > 0: heavy tails (leptokurtic)
/// - Excess kurtosis < 0: light tails (platykurtic)
/// - Excess kurtosis = 0: normal distribution (mesokurtic)
///
/// Returns excess kurtosis (kurtosis - 3).
pub fn kurtosis(data: &[f64]) -> f64 {
    let n = data.len() as f64;
    if n < 4.0 {
        return f64::NAN;
    }

    let mean = data.iter().sum::<f64>() / n;
    let m2: f64 = data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n;
    let m4: f64 = data.iter().map(|&x| (x - mean).powi(4)).sum::<f64>() / n;

    if m2 == 0.0 {
        return 0.0;
    }

    let g2 = m4 / (m2 * m2) - 3.0;

    // Adjust for sample bias
    let adjustment = (n - 1.0) / ((n - 2.0) * (n - 3.0));
    adjustment * ((n + 1.0) * g2 + 6.0)
}

/// Calculate z-scores (standard scores) for each data point.
///
/// Z-score = (x - mean) / std
pub fn zscore(data: &[f64]) -> Vec<f64> {
    let n = data.len() as f64;
    if n < 2.0 {
        return data.to_vec();
    }

    let mean = data.iter().sum::<f64>() / n;
    let std = (data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

    if std == 0.0 {
        return vec![0.0; data.len()];
    }

    data.iter().map(|&x| (x - mean) / std).collect()
}

/// Calculate the nth central moment.
///
/// Central moment = E[(X - μ)^n]
pub fn moment(data: &[f64], n: i32) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    let mean = data.iter().sum::<f64>() / data.len() as f64;
    data.iter().map(|&x| (x - mean).powi(n)).sum::<f64>() / data.len() as f64
}

/// Calculate the nth raw moment.
///
/// Raw moment = E[X^n]
pub fn raw_moment(data: &[f64], n: i32) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    data.iter().map(|&x| x.powi(n)).sum::<f64>() / data.len() as f64
}

/// Calculate the mode of a dataset.
///
/// Returns the most frequent value. For continuous data, this may not be meaningful.
/// For ties, returns the first mode encountered.
pub fn mode(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    use std::collections::HashMap;

    // Discretize by rounding to avoid floating point issues
    let precision = 1e10;
    let mut counts: HashMap<i64, (usize, f64)> = HashMap::new();

    for &x in data {
        let key = (x * precision).round() as i64;
        let entry = counts.entry(key).or_insert((0, x));
        entry.0 += 1;
    }

    counts
        .into_values()
        .max_by_key(|(count, _)| *count)
        .map(|(_, value)| value)
}

/// Calculate the median of a dataset.
pub fn median(data: &[f64]) -> Option<f64> {
    if data.is_empty() {
        return None;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

/// Calculate the interquartile range (IQR).
pub fn iqr(data: &[f64]) -> Option<f64> {
    let q1 = percentile(data, 25.0)?;
    let q3 = percentile(data, 75.0)?;
    Some(q3 - q1)
}

/// Calculate the percentile of a dataset.
///
/// Uses linear interpolation between closest ranks.
pub fn percentile(data: &[f64], p: f64) -> Option<f64> {
    if data.is_empty() || p < 0.0 || p > 100.0 {
        return None;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let idx = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower = idx.floor() as usize;
    let upper = idx.ceil() as usize;

    if lower == upper {
        Some(sorted[lower])
    } else {
        let frac = idx - lower as f64;
        Some(sorted[lower] * (1.0 - frac) + sorted[upper] * frac)
    }
}

/// Calculate the geometric mean.
pub fn geometric_mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    // Check for non-positive values
    if data.iter().any(|&x| x <= 0.0) {
        return f64::NAN;
    }

    let log_sum: f64 = data.iter().map(|&x| x.ln()).sum();
    (log_sum / data.len() as f64).exp()
}

/// Calculate the harmonic mean.
pub fn harmonic_mean(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    // Check for zero or negative values
    if data.iter().any(|&x| x <= 0.0) {
        return f64::NAN;
    }

    let inv_sum: f64 = data.iter().map(|&x| 1.0 / x).sum();
    data.len() as f64 / inv_sum
}

/// Calculate the trimmed mean (removing a fraction of extreme values).
pub fn trim_mean(data: &[f64], proportiontocut: f64) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    let n = sorted.len();
    let k = (n as f64 * proportiontocut) as usize;

    if k * 2 >= n {
        return median(data).unwrap_or(f64::NAN);
    }

    let trimmed = &sorted[k..n - k];
    trimmed.iter().sum::<f64>() / trimmed.len() as f64
}

/// Calculate the standard error of the mean.
pub fn sem(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return f64::NAN;
    }

    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;
    let std = (data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    std / n.sqrt()
}

/// Calculate the coefficient of variation.
pub fn variation(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    let n = data.len() as f64;
    let mean = data.iter().sum::<f64>() / n;

    if mean == 0.0 {
        return f64::NAN;
    }

    let std = (data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();
    std / mean.abs()
}

/// Calculate the median absolute deviation (MAD).
pub fn median_abs_deviation(data: &[f64]) -> f64 {
    if data.is_empty() {
        return f64::NAN;
    }

    let med = median(data).unwrap();
    let deviations: Vec<f64> = data.iter().map(|&x| (x - med).abs()).collect();
    median(&deviations).unwrap_or(f64::NAN)
}

/// Describe basic statistics of a dataset.
#[derive(Debug, Clone)]
pub struct DescribeResult {
    pub count: usize,
    pub mean: f64,
    pub std: f64,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub skewness: f64,
    pub kurtosis: f64,
}

/// Calculate descriptive statistics for a dataset.
pub fn describe(data: &[f64]) -> DescribeResult {
    let n = data.len() as f64;

    let mean = if n > 0.0 {
        data.iter().sum::<f64>() / n
    } else {
        f64::NAN
    };

    let std = if n > 1.0 {
        (data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt()
    } else {
        f64::NAN
    };

    let mut sorted = data.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    DescribeResult {
        count: data.len(),
        mean,
        std,
        min: sorted.first().copied().unwrap_or(f64::NAN),
        q1: percentile(data, 25.0).unwrap_or(f64::NAN),
        median: median(data).unwrap_or(f64::NAN),
        q3: percentile(data, 75.0).unwrap_or(f64::NAN),
        max: sorted.last().copied().unwrap_or(f64::NAN),
        skewness: skew(data),
        kurtosis: kurtosis(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skew_symmetric() {
        // Symmetric distribution should have skewness near 0
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(skew(&data).abs() < 0.1);
    }

    #[test]
    fn test_skew_positive() {
        // Right-skewed distribution
        let data = vec![1.0, 1.0, 1.0, 1.0, 10.0];
        assert!(skew(&data) > 0.0);
    }

    #[test]
    fn test_kurtosis_normal() {
        // Normal-like distribution should have excess kurtosis near 0
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert!(kurtosis(&data).abs() < 2.0);
    }

    #[test]
    fn test_zscore() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let z = zscore(&data);
        assert_eq!(z.len(), 5);
        // Mean of z-scores should be 0
        let mean_z: f64 = z.iter().sum::<f64>() / z.len() as f64;
        assert!(mean_z.abs() < 1e-10);
    }

    #[test]
    fn test_mode() {
        let data = vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0];
        assert_eq!(mode(&data), Some(3.0));
    }

    #[test]
    fn test_median() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(median(&data), Some(3.0));

        let data2 = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(median(&data2), Some(2.5));
    }

    #[test]
    fn test_iqr() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let result = iqr(&data).unwrap();
        assert!((result - 4.0).abs() < 0.5);
    }

    #[test]
    fn test_geometric_mean() {
        let data = vec![2.0, 8.0];
        assert!((geometric_mean(&data) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_harmonic_mean() {
        let data = vec![1.0, 2.0, 4.0];
        let hm = harmonic_mean(&data);
        assert!((hm - 12.0 / 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_trim_mean() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let tm = trim_mean(&data, 0.1);
        // Should remove 1 element from each end
        assert!((tm - 5.5).abs() < 0.5);
    }

    #[test]
    fn test_sem() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let s = sem(&data);
        // std = sqrt(2.5), sem = sqrt(2.5) / sqrt(5)
        assert!((s - (2.5_f64.sqrt() / 5.0_f64.sqrt())).abs() < 1e-10);
    }

    #[test]
    fn test_variation() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cv = variation(&data);
        // mean = 3, std = sqrt(2.5), cv = sqrt(2.5) / 3
        assert!((cv - 2.5_f64.sqrt() / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_describe() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = describe(&data);
        assert_eq!(result.count, 5);
        assert!((result.mean - 3.0).abs() < 1e-10);
        assert!((result.min - 1.0).abs() < 1e-10);
        assert!((result.max - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_moment() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        // Second central moment = variance (population)
        let m2 = moment(&data, 2);
        // Variance of [1,2,3,4,5] = 2.0 (population)
        assert!((m2 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_raw_moment() {
        let data = vec![1.0, 2.0, 3.0];
        // First raw moment = mean
        let m1 = raw_moment(&data, 1);
        assert!((m1 - 2.0).abs() < 1e-10);
    }

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(percentile(&data, 50.0), Some(5.5)); // Median
        assert_eq!(percentile(&data, 0.0), Some(1.0));
        assert_eq!(percentile(&data, 100.0), Some(10.0));
    }

    #[test]
    fn test_median_abs_deviation() {
        let data = vec![1.0, 1.0, 2.0, 2.0, 4.0, 6.0, 9.0];
        let mad = median_abs_deviation(&data);
        // Median = 2, deviations = [1, 1, 0, 0, 2, 4, 7], median of deviations = 1
        assert!((mad - 1.0).abs() < 1e-10);
    }
}
