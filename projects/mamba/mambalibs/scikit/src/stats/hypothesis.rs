//! Hypothesis testing functions (scipy.stats equivalent).
//!
//! Implements t-tests, ANOVA, chi-square tests, and correlation tests.

use super::distributions::{ChiSquared, ContinuousDistribution, FDistribution, Normal, StudentT};

/// Result of a statistical test.
#[derive(Debug, Clone, Copy)]
pub struct TestResult {
    /// Test statistic value.
    pub statistic: f64,
    /// P-value (two-tailed unless otherwise specified).
    pub pvalue: f64,
}

impl TestResult {
    /// Create a new test result.
    pub fn new(statistic: f64, pvalue: f64) -> Self {
        Self { statistic, pvalue }
    }

    /// Check if result is significant at given alpha level.
    pub fn is_significant(&self, alpha: f64) -> bool {
        self.pvalue < alpha
    }
}

// ============================================================================
// T-Tests
// ============================================================================

/// One-sample t-test.
///
/// Tests whether the mean of a sample differs from a specified value.
///
/// # Arguments
/// * `sample` - The sample data (must have at least 2 elements)
/// * `popmean` - The population mean to test against
///
/// # Returns
/// Test statistic and p-value. Returns NaN for both if sample size < 2.
pub fn ttest_1samp(sample: &[f64], popmean: f64) -> TestResult {
    let n = sample.len() as f64;

    // Validate minimum sample size
    if sample.len() < 2 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let mean: f64 = sample.iter().sum::<f64>() / n;
    let var: f64 = sample.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let se = (var / n).sqrt();

    // Handle zero variance case
    if se == 0.0 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let t = (mean - popmean) / se;
    let df = n - 1.0;

    let dist = StudentT::new(df);
    let pvalue = 2.0 * (1.0 - dist.cdf(t.abs()));

    TestResult::new(t, pvalue)
}

/// Independent two-sample t-test.
///
/// Tests whether the means of two independent samples differ.
///
/// # Arguments
/// * `a` - First sample (must have at least 2 elements)
/// * `b` - Second sample (must have at least 2 elements)
/// * `equal_var` - If true, assume equal variances (pooled t-test)
///
/// # Returns
/// Test statistic and p-value. Returns NaN for both if sample sizes < 2.
pub fn ttest_ind(a: &[f64], b: &[f64], equal_var: bool) -> TestResult {
    // Validate minimum sample sizes
    if a.len() < 2 || b.len() < 2 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let n1 = a.len() as f64;
    let n2 = b.len() as f64;

    let mean1: f64 = a.iter().sum::<f64>() / n1;
    let mean2: f64 = b.iter().sum::<f64>() / n2;

    let var1: f64 = a.iter().map(|&x| (x - mean1).powi(2)).sum::<f64>() / (n1 - 1.0);
    let var2: f64 = b.iter().map(|&x| (x - mean2).powi(2)).sum::<f64>() / (n2 - 1.0);

    let (t, df) = if equal_var {
        // Pooled variance
        let sp = ((n1 - 1.0) * var1 + (n2 - 1.0) * var2) / (n1 + n2 - 2.0);
        let se = (sp * (1.0 / n1 + 1.0 / n2)).sqrt();
        let t = (mean1 - mean2) / se;
        let df = n1 + n2 - 2.0;
        (t, df)
    } else {
        // Welch's t-test
        let se = (var1 / n1 + var2 / n2).sqrt();
        let t = (mean1 - mean2) / se;

        // Welch-Satterthwaite degrees of freedom
        let num = (var1 / n1 + var2 / n2).powi(2);
        let den = (var1 / n1).powi(2) / (n1 - 1.0) + (var2 / n2).powi(2) / (n2 - 1.0);
        let df = num / den;
        (t, df)
    };

    let dist = StudentT::new(df);
    let pvalue = 2.0 * (1.0 - dist.cdf(t.abs()));

    TestResult::new(t, pvalue)
}

/// Paired t-test.
///
/// Tests whether the mean difference between paired observations differs from zero.
///
/// # Arguments
/// * `a` - First set of observations
/// * `b` - Second set of observations (paired with a)
///
/// # Returns
/// Test statistic and p-value
pub fn ttest_rel(a: &[f64], b: &[f64]) -> TestResult {
    assert_eq!(a.len(), b.len(), "samples must have same length");

    let diffs: Vec<f64> = a.iter().zip(b.iter()).map(|(&x, &y)| x - y).collect();
    ttest_1samp(&diffs, 0.0)
}

// ============================================================================
// ANOVA
// ============================================================================

/// One-way ANOVA.
///
/// Tests whether the means of multiple groups differ.
///
/// # Arguments
/// * `groups` - Slice of group data arrays (need at least 2 groups, each with at least 2 elements)
///
/// # Returns
/// F-statistic and p-value. Returns NaN if invalid input.
pub fn f_oneway(groups: &[&[f64]]) -> TestResult {
    // Validate: need at least 2 groups
    if groups.len() < 2 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    // Validate: each group must have at least 2 elements for within-group variance
    if groups.iter().any(|g| g.len() < 2) {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let k = groups.len() as f64;

    // Total sample size
    let n: f64 = groups.iter().map(|g| g.len() as f64).sum();

    // Grand mean
    let grand_sum: f64 = groups.iter().flat_map(|g| g.iter()).sum();
    let grand_mean = grand_sum / n;

    // Group means
    let group_means: Vec<f64> = groups
        .iter()
        .map(|g| g.iter().sum::<f64>() / g.len() as f64)
        .collect();

    // Between-group sum of squares
    let ss_between: f64 = groups
        .iter()
        .zip(group_means.iter())
        .map(|(g, &m)| g.len() as f64 * (m - grand_mean).powi(2))
        .sum();

    // Within-group sum of squares
    let ss_within: f64 = groups
        .iter()
        .zip(group_means.iter())
        .flat_map(|(g, &m)| g.iter().map(move |&x| (x - m).powi(2)))
        .sum();

    let df_between = k - 1.0;
    let df_within = n - k;

    let ms_between = ss_between / df_between;
    let ms_within = ss_within / df_within;

    let f = ms_between / ms_within;

    let dist = FDistribution::new(df_between, df_within);
    let pvalue = 1.0 - dist.cdf(f);

    TestResult::new(f, pvalue)
}

// ============================================================================
// Chi-Square Tests
// ============================================================================

/// Chi-square test for independence (contingency table).
///
/// # Arguments
/// * `observed` - 2D contingency table (row-major)
/// * `rows` - Number of rows
/// * `cols` - Number of columns
///
/// # Returns
/// Chi-square statistic and p-value
pub fn chi2_contingency(observed: &[f64], rows: usize, cols: usize) -> TestResult {
    assert_eq!(observed.len(), rows * cols);

    // Calculate row and column sums
    let row_sums: Vec<f64> = (0..rows)
        .map(|r| (0..cols).map(|c| observed[r * cols + c]).sum())
        .collect();

    let col_sums: Vec<f64> = (0..cols)
        .map(|c| (0..rows).map(|r| observed[r * cols + c]).sum())
        .collect();

    let total: f64 = row_sums.iter().sum();

    // Calculate expected frequencies and chi-square statistic
    let mut chi2 = 0.0;
    for r in 0..rows {
        for c in 0..cols {
            let expected = row_sums[r] * col_sums[c] / total;
            let obs = observed[r * cols + c];
            if expected > 0.0 {
                chi2 += (obs - expected).powi(2) / expected;
            }
        }
    }

    let df = ((rows - 1) * (cols - 1)) as f64;
    let dist = ChiSquared::new(df);
    let pvalue = 1.0 - dist.cdf(chi2);

    TestResult::new(chi2, pvalue)
}

/// Chi-square goodness of fit test.
///
/// # Arguments
/// * `observed` - Observed frequencies
/// * `expected` - Expected frequencies
///
/// # Returns
/// Chi-square statistic and p-value
pub fn chisquare(observed: &[f64], expected: &[f64]) -> TestResult {
    assert_eq!(observed.len(), expected.len());

    let chi2: f64 = observed
        .iter()
        .zip(expected.iter())
        .map(|(&o, &e)| if e > 0.0 { (o - e).powi(2) / e } else { 0.0 })
        .sum();

    let df = (observed.len() - 1) as f64;
    let dist = ChiSquared::new(df);
    let pvalue = 1.0 - dist.cdf(chi2);

    TestResult::new(chi2, pvalue)
}

// ============================================================================
// Correlation Tests
// ============================================================================

/// Pearson correlation coefficient with significance test.
///
/// # Arguments
/// * `x` - First variable (must have at least 3 elements)
/// * `y` - Second variable (must have same length as x)
///
/// # Returns
/// Correlation coefficient and p-value. Returns NaN if invalid input or zero variance.
pub fn pearsonr(x: &[f64], y: &[f64]) -> TestResult {
    // Validate inputs
    if x.len() != y.len() || x.len() < 3 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let n = x.len() as f64;

    let mean_x = x.iter().sum::<f64>() / n;
    let mean_y = y.iter().sum::<f64>() / n;

    let mut cov = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;

    for (&xi, &yi) in x.iter().zip(y.iter()) {
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    // Handle zero variance case
    if var_x == 0.0 || var_y == 0.0 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    let r = cov / (var_x * var_y).sqrt();

    // Handle perfect correlation (r = ±1)
    if (r.abs() - 1.0).abs() < 1e-15 {
        return TestResult::new(r, 0.0);
    }

    // T-statistic for testing r = 0
    let t = r * ((n - 2.0) / (1.0 - r * r)).sqrt();
    let df = n - 2.0;

    let dist = StudentT::new(df);
    let pvalue = 2.0 * (1.0 - dist.cdf(t.abs()));

    TestResult::new(r, pvalue)
}

/// Spearman rank correlation coefficient.
///
/// # Arguments
/// * `x` - First variable (must have at least 3 elements)
/// * `y` - Second variable (must have same length as x)
///
/// # Returns
/// Correlation coefficient and p-value. Returns NaN if invalid input.
pub fn spearmanr(x: &[f64], y: &[f64]) -> TestResult {
    // Validate inputs
    if x.len() != y.len() || x.len() < 3 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    // Convert to ranks
    let rank_x = to_ranks(x);
    let rank_y = to_ranks(y);

    // Compute Pearson correlation on ranks
    pearsonr(&rank_x, &rank_y)
}

/// Convert values to ranks.
fn to_ranks(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut indexed: Vec<(usize, f64)> = values.iter().enumerate().map(|(i, &v)| (i, v)).collect();
    indexed.sort_by(|a, b| a.1.total_cmp(&b.1));

    let mut ranks = vec![0.0; n];
    for (rank, &(orig_idx, _)) in indexed.iter().enumerate() {
        ranks[orig_idx] = (rank + 1) as f64;
    }
    ranks
}

// ============================================================================
// Normality Tests
// ============================================================================

/// Shapiro-Wilk test for normality.
///
/// Note: This is a simplified approximation. Most accurate for n <= 50.
///
/// # Arguments
/// * `x` - Sample data (must have 3-5000 elements)
///
/// # Returns
/// W statistic and p-value. Returns NaN if sample size out of range.
pub fn shapiro(x: &[f64]) -> TestResult {
    let n = x.len();

    // Validate sample size
    if n < 3 || n > 5000 {
        return TestResult::new(f64::NAN, f64::NAN);
    }

    // Sort the data
    let mut sorted = x.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    // Calculate mean
    let mean: f64 = sorted.iter().sum::<f64>() / n as f64;

    // Calculate SS
    let ss: f64 = sorted.iter().map(|&x| (x - mean).powi(2)).sum();

    // Calculate W statistic (simplified)
    let mut b = 0.0;
    let m = n / 2;
    for i in 0..m {
        let a = shapiro_coef(n, i);
        b += a * (sorted[n - 1 - i] - sorted[i]);
    }

    let w = (b * b) / ss;

    // Approximate p-value using normal distribution
    // This is a rough approximation
    let ln_n = (n as f64).ln();
    let mu = -1.2725 + 1.0521 * ln_n;
    let sigma = 1.0308 - 0.26758 * ln_n;

    let z = ((w).ln() - mu) / sigma;
    let normal = Normal::standard();
    let pvalue = 1.0 - normal.cdf(z);

    TestResult::new(w, pvalue)
}

/// Approximate Shapiro-Wilk coefficients.
fn shapiro_coef(n: usize, i: usize) -> f64 {
    // Very simplified approximation
    let mi = (i + 1) as f64;

    // Expected value of order statistic
    let p = (mi - 0.375) / (n as f64 + 0.25);
    let expected = normal_quantile(p);

    // Normalize
    let sum_sq: f64 = (1..=n)
        .map(|j| {
            let pj = (j as f64 - 0.375) / (n as f64 + 0.25);
            let ej = normal_quantile(pj);
            ej * ej
        })
        .sum();

    expected / sum_sq.sqrt()
}

/// Normal quantile (inverse CDF) approximation.
fn normal_quantile(p: f64) -> f64 {
    // Approximation using rational function
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }

    let p = if p > 0.5 { 1.0 - p } else { p };
    let t = (-2.0 * p.ln()).sqrt();

    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let x = t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t);

    if p > 0.5 {
        -x
    } else {
        x
    }
}

/// Kolmogorov-Smirnov test for normality.
///
/// # Arguments
/// * `x` - Sample data
///
/// # Returns
/// D statistic and p-value
pub fn kstest_normal(x: &[f64]) -> TestResult {
    let n = x.len() as f64;
    let mean: f64 = x.iter().sum::<f64>() / n;
    let std: f64 = (x.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0)).sqrt();

    let normal = Normal::standard();

    // Sort data
    let mut sorted = x.to_vec();
    sorted.sort_by(|a, b| a.total_cmp(b));

    // Calculate D statistic
    let mut d = 0.0f64;
    for (i, &xi) in sorted.iter().enumerate() {
        let z = (xi - mean) / std;
        let fn_x = (i + 1) as f64 / n;
        let f_x = normal.cdf(z);

        let d_plus = (fn_x - f_x).abs();
        let d_minus = (f_x - i as f64 / n).abs();
        d = d.max(d_plus).max(d_minus);
    }

    // Approximate p-value (asymptotic)
    let lambda = (n.sqrt() + 0.12 + 0.11 / n.sqrt()) * d;
    let pvalue = 2.0 * (-2.0 * lambda * lambda).exp();

    TestResult::new(d, pvalue.min(1.0).max(0.0))
}

// ============================================================================
// Mann-Whitney U Test
// ============================================================================

/// Mann-Whitney U test (Wilcoxon rank-sum test).
///
/// Non-parametric test for comparing two independent samples.
///
/// # Arguments
/// * `x` - First sample
/// * `y` - Second sample
///
/// # Returns
/// U statistic and p-value
pub fn mannwhitneyu(x: &[f64], y: &[f64]) -> TestResult {
    let n1 = x.len() as f64;
    let n2 = y.len() as f64;

    // Combine and rank
    let mut combined: Vec<(f64, usize)> = x
        .iter()
        .enumerate()
        .map(|(i, &v)| (v, i))
        .chain(y.iter().enumerate().map(|(i, &v)| (v, i + x.len())))
        .collect();

    combined.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Assign ranks
    let mut ranks = vec![0.0; combined.len()];
    for (rank, &(_, orig_idx)) in combined.iter().enumerate() {
        ranks[orig_idx] = (rank + 1) as f64;
    }

    // Sum of ranks for first sample
    let r1: f64 = ranks[..x.len()].iter().sum();

    // U statistic
    let u1 = n1 * n2 + n1 * (n1 + 1.0) / 2.0 - r1;
    let u2 = n1 * n2 - u1;
    let u = u1.min(u2);

    // Normal approximation for p-value
    let mean_u = n1 * n2 / 2.0;
    let std_u = (n1 * n2 * (n1 + n2 + 1.0) / 12.0).sqrt();

    let z = (u - mean_u) / std_u;
    let normal = Normal::standard();
    let pvalue = 2.0 * normal.cdf(-z.abs());

    TestResult::new(u, pvalue)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ttest_1samp() {
        let sample = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ttest_1samp(&sample, 3.0);
        assert!(result.pvalue > 0.9); // Mean is exactly 3
    }

    #[test]
    fn test_ttest_ind() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let result = ttest_ind(&a, &b, true);
        assert!(result.pvalue > 0.9); // Same samples
    }

    #[test]
    fn test_ttest_rel() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = ttest_rel(&a, &b);
        // When samples are identical, differences are all 0
        // t-statistic should be 0 or NaN, and p-value should be 1
        assert!(result.pvalue.is_nan() || result.pvalue > 0.5);
    }

    #[test]
    fn test_f_oneway() {
        let g1 = vec![1.0, 2.0, 3.0];
        let g2 = vec![1.0, 2.0, 3.0];
        let g3 = vec![1.0, 2.0, 3.0];
        let result = f_oneway(&[&g1, &g2, &g3]);
        assert!(result.pvalue > 0.9); // Same groups
    }

    #[test]
    fn test_chi2() {
        // 2x2 contingency table
        let observed = vec![10.0, 10.0, 10.0, 10.0];
        let result = chi2_contingency(&observed, 2, 2);
        assert!(result.pvalue > 0.9); // Independent
    }

    #[test]
    fn test_pearsonr() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let result = pearsonr(&x, &y);
        assert!((result.statistic - 1.0).abs() < 1e-10); // Perfect correlation
    }

    #[test]
    fn test_spearmanr() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let result = spearmanr(&x, &y);
        assert!((result.statistic - 1.0).abs() < 1e-10); // Perfect rank correlation
    }

    #[test]
    fn test_mannwhitneyu() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let result = mannwhitneyu(&x, &y);
        assert!(result.pvalue > 0.5); // Same samples
    }

    #[test]
    fn test_chisquare() {
        let observed = vec![10.0, 10.0, 10.0, 10.0];
        let expected = vec![10.0, 10.0, 10.0, 10.0];
        let result = chisquare(&observed, &expected);
        assert_eq!(result.statistic, 0.0); // Exactly as expected
        assert!(result.pvalue > 0.99);
    }

    #[test]
    fn test_shapiro() {
        // Normal-ish data
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let result = shapiro(&x);
        assert!(result.statistic > 0.0);
        assert!(result.pvalue >= 0.0 && result.pvalue <= 1.0);
    }

    #[test]
    fn test_kstest_normal() {
        // Approximately normal data
        let x = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let result = kstest_normal(&x);
        assert!(result.statistic >= 0.0 && result.statistic <= 1.0);
        assert!(result.pvalue >= 0.0 && result.pvalue <= 1.0);
    }

    #[test]
    fn test_is_significant() {
        let result = TestResult::new(2.0, 0.03);
        assert!(result.is_significant(0.05));
        assert!(!result.is_significant(0.01));
    }
}
