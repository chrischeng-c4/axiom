//! Integration tests for the stats module (scipy-like statistical functions).
//!
//! These tests were extracted from the inline `#[cfg(test)]` modules in the stats source files.

#![cfg(feature = "stats")]

use scikit::stats::{
    beta_fn,
    binomial_coef,
    chi2_contingency,
    chisquare,
    describe,
    // Distributions - re-exported from distributions module
    erf,
    f_oneway,
    factorial,
    gamma_fn,
    geometric_mean,
    harmonic_mean,
    incomplete_beta,
    iqr,
    kstest_normal,
    kurtosis,
    ln_gamma,
    lower_incomplete_gamma,
    mannwhitneyu,
    median,
    median_abs_deviation,
    mode,
    moment,
    pearsonr,
    percentile,
    raw_moment,
    sem,
    shapiro,
    // Descriptive statistics
    skew,
    spearmanr,
    trim_mean,
    // Hypothesis tests
    ttest_1samp,
    ttest_ind,
    ttest_rel,
    variation,
    zscore,
    Beta,
    Binomial,
    ChiSquared,
    ContinuousDistribution,
    DiscreteDistribution,
    Exponential,
    Normal,
    Poisson,
    StudentT,
    Uniform,
};

// ============================================================================
// Descriptive Statistics tests (from descriptive.rs)
// ============================================================================

mod descriptive_tests {
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

// ============================================================================
// Hypothesis Tests (from hypothesis.rs)
// ============================================================================

mod hypothesis_tests {
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
        let result = scikit::stats::TestResult::new(2.0, 0.03);
        assert!(result.is_significant(0.05));
        assert!(!result.is_significant(0.01));
    }
}

// ============================================================================
// Distributions tests (from distributions.rs)
// ============================================================================

mod distribution_tests {
    use super::*;

    #[test]
    fn test_normal_pdf() {
        let n = Normal::standard();
        assert!((n.pdf(0.0) - 0.3989422804014327).abs() < 1e-10);
    }

    #[test]
    fn test_normal_cdf() {
        let n = Normal::standard();
        // Use reasonable tolerance for numerical approximation
        assert!((n.cdf(0.0) - 0.5).abs() < 1e-8);
        assert!((n.cdf(1.96) - 0.975).abs() < 0.01);
    }

    #[test]
    fn test_uniform() {
        let u = Uniform::new(0.0, 1.0);
        assert_eq!(u.pdf(0.5), 1.0);
        assert_eq!(u.cdf(0.5), 0.5);
        assert_eq!(u.mean(), 0.5);
    }

    #[test]
    fn test_exponential() {
        let e = Exponential::new(1.0);
        assert_eq!(e.pdf(0.0), 1.0);
        assert!((e.cdf(1.0) - 0.6321205588285577).abs() < 1e-10);
    }

    #[test]
    fn test_binomial() {
        let b = Binomial::new(10, 0.5);
        assert!((b.mean() - 5.0).abs() < 1e-10);
        assert!((b.variance() - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_poisson() {
        let p = Poisson::new(5.0);
        assert_eq!(p.mean(), 5.0);
        assert_eq!(p.variance(), 5.0);
    }

    #[test]
    fn test_beta() {
        let b = Beta::new(2.0, 5.0);
        assert!((b.mean() - 2.0 / 7.0).abs() < 1e-10);
        // PDF at 0.5
        let pdf_val = b.pdf(0.5);
        assert!(pdf_val > 0.0);
    }

    #[test]
    fn test_gamma_fn() {
        // Γ(1) = 1, Γ(2) = 1, Γ(3) = 2, Γ(4) = 6
        assert!((gamma_fn(1.0) - 1.0).abs() < 1e-10);
        assert!((gamma_fn(2.0) - 1.0).abs() < 1e-10);
        assert!((gamma_fn(3.0) - 2.0).abs() < 1e-10);
        assert!((gamma_fn(4.0) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_student_t() {
        let t = StudentT::new(10.0);
        assert_eq!(t.mean(), 0.0);
        assert!((t.variance() - 10.0 / 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_chi_squared() {
        let c = ChiSquared::new(5.0);
        assert_eq!(c.mean(), 5.0);
        assert_eq!(c.variance(), 10.0);
    }

    #[test]
    fn test_erf() {
        // erf(0) ≈ 0 (with numerical tolerance)
        assert!(erf(0.0).abs() < 1e-6);
        // erf(∞) ≈ 1
        assert!((erf(5.0) - 1.0).abs() < 1e-6);
        // erf(-x) = -erf(x) (symmetry)
        assert!((erf(1.0) + erf(-1.0)).abs() < 1e-6);
    }

    #[test]
    fn test_beta_fn() {
        // B(1, 1) = 1
        assert!((beta_fn(1.0, 1.0) - 1.0).abs() < 1e-10);
        // B(2, 2) = 1/6
        assert!((beta_fn(2.0, 2.0) - 1.0 / 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1.0);
        assert_eq!(factorial(1), 1.0);
        assert_eq!(factorial(5), 120.0);
    }

    #[test]
    fn test_binomial_coef() {
        assert_eq!(binomial_coef(5, 0), 1.0);
        assert_eq!(binomial_coef(5, 5), 1.0);
        assert_eq!(binomial_coef(5, 2), 10.0);
        assert_eq!(binomial_coef(10, 3), 120.0);
    }

    #[test]
    fn test_ln_gamma() {
        // ln(Γ(1)) = 0
        assert!(ln_gamma(1.0).abs() < 1e-10);
        // ln(Γ(2)) = 0
        assert!(ln_gamma(2.0).abs() < 1e-10);
        // ln(Γ(3)) = ln(2)
        assert!((ln_gamma(3.0) - 2.0_f64.ln()).abs() < 1e-10);
    }

    #[test]
    fn test_incomplete_beta() {
        // I_x(a, b) at x=0 should be 0
        assert!((incomplete_beta(2.0, 2.0, 0.0) - 0.0).abs() < 1e-10);
        // I_x(a, b) at x=1 should be 1
        assert!((incomplete_beta(2.0, 2.0, 1.0) - 1.0).abs() < 1e-10);
        // I_0.5(2, 2) = 0.5 (by symmetry)
        assert!((incomplete_beta(2.0, 2.0, 0.5) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lower_incomplete_gamma() {
        // γ(1, x) = 1 - e^(-x)
        let x: f64 = 1.0;
        let expected = 1.0 - (-x).exp();
        assert!((lower_incomplete_gamma(1.0, x) - expected).abs() < 1e-8);
    }
}
