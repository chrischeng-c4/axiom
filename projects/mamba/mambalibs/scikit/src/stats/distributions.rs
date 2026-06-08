//! Probability distributions (scipy.stats equivalent).
//!
//! Implements continuous and discrete distributions with PDF, CDF, and sampling.

use std::f64::consts::PI;

/// Trait for continuous probability distributions.
pub trait ContinuousDistribution {
    /// Probability density function at x.
    fn pdf(&self, x: f64) -> f64;

    /// Cumulative distribution function at x.
    fn cdf(&self, x: f64) -> f64;

    /// Mean of the distribution.
    fn mean(&self) -> f64;

    /// Variance of the distribution.
    fn variance(&self) -> f64;

    /// Standard deviation.
    fn std(&self) -> f64 {
        self.variance().sqrt()
    }
}

/// Trait for discrete probability distributions.
pub trait DiscreteDistribution {
    /// Probability mass function at k.
    fn pmf(&self, k: u64) -> f64;

    /// Cumulative distribution function at k.
    fn cdf(&self, k: u64) -> f64;

    /// Mean of the distribution.
    fn mean(&self) -> f64;

    /// Variance of the distribution.
    fn variance(&self) -> f64;
}

// ============================================================================
// Continuous Distributions
// ============================================================================

/// Normal (Gaussian) distribution.
#[derive(Debug, Clone, Copy)]
pub struct Normal {
    /// Mean (μ).
    pub mu: f64,
    /// Standard deviation (σ).
    pub sigma: f64,
}

impl Normal {
    /// Create a new normal distribution.
    pub fn new(mu: f64, sigma: f64) -> Self {
        assert!(sigma > 0.0, "sigma must be positive");
        Self { mu, sigma }
    }

    /// Standard normal distribution N(0, 1).
    pub fn standard() -> Self {
        Self { mu: 0.0, sigma: 1.0 }
    }
}

impl ContinuousDistribution for Normal {
    fn pdf(&self, x: f64) -> f64 {
        let z = (x - self.mu) / self.sigma;
        (-0.5 * z * z).exp() / (self.sigma * (2.0 * PI).sqrt())
    }

    fn cdf(&self, x: f64) -> f64 {
        // Standard formula: CDF(x) = 0.5 * (1 + erf((x - μ) / (σ * sqrt(2))))
        let z = (x - self.mu) / (self.sigma * std::f64::consts::SQRT_2);
        0.5 * (1.0 + erf(z))
    }

    fn mean(&self) -> f64 {
        self.mu
    }

    fn variance(&self) -> f64 {
        self.sigma * self.sigma
    }
}

/// Uniform distribution on [a, b].
#[derive(Debug, Clone, Copy)]
pub struct Uniform {
    /// Lower bound.
    pub a: f64,
    /// Upper bound.
    pub b: f64,
}

impl Uniform {
    /// Create a new uniform distribution.
    pub fn new(a: f64, b: f64) -> Self {
        assert!(a < b, "a must be less than b");
        Self { a, b }
    }
}

impl ContinuousDistribution for Uniform {
    fn pdf(&self, x: f64) -> f64 {
        if x >= self.a && x <= self.b {
            1.0 / (self.b - self.a)
        } else {
            0.0
        }
    }

    fn cdf(&self, x: f64) -> f64 {
        if x < self.a {
            0.0
        } else if x > self.b {
            1.0
        } else {
            (x - self.a) / (self.b - self.a)
        }
    }

    fn mean(&self) -> f64 {
        (self.a + self.b) / 2.0
    }

    fn variance(&self) -> f64 {
        (self.b - self.a).powi(2) / 12.0
    }
}

/// Exponential distribution.
#[derive(Debug, Clone, Copy)]
pub struct Exponential {
    /// Rate parameter (λ).
    pub lambda: f64,
}

impl Exponential {
    /// Create a new exponential distribution.
    pub fn new(lambda: f64) -> Self {
        assert!(lambda > 0.0, "lambda must be positive");
        Self { lambda }
    }
}

impl ContinuousDistribution for Exponential {
    fn pdf(&self, x: f64) -> f64 {
        if x >= 0.0 {
            self.lambda * (-self.lambda * x).exp()
        } else {
            0.0
        }
    }

    fn cdf(&self, x: f64) -> f64 {
        if x >= 0.0 {
            1.0 - (-self.lambda * x).exp()
        } else {
            0.0
        }
    }

    fn mean(&self) -> f64 {
        1.0 / self.lambda
    }

    fn variance(&self) -> f64 {
        1.0 / (self.lambda * self.lambda)
    }
}

/// Gamma distribution.
#[derive(Debug, Clone, Copy)]
pub struct Gamma {
    /// Shape parameter (k or α).
    pub shape: f64,
    /// Scale parameter (θ).
    pub scale: f64,
}

impl Gamma {
    /// Create a new gamma distribution.
    pub fn new(shape: f64, scale: f64) -> Self {
        assert!(shape > 0.0, "shape must be positive");
        assert!(scale > 0.0, "scale must be positive");
        Self { shape, scale }
    }
}

impl ContinuousDistribution for Gamma {
    fn pdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let k = self.shape;
        let theta = self.scale;
        x.powf(k - 1.0) * (-x / theta).exp() / (theta.powf(k) * gamma_fn(k))
    }

    fn cdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        lower_incomplete_gamma(self.shape, x / self.scale) / gamma_fn(self.shape)
    }

    fn mean(&self) -> f64 {
        self.shape * self.scale
    }

    fn variance(&self) -> f64 {
        self.shape * self.scale * self.scale
    }
}

/// Beta distribution.
#[derive(Debug, Clone, Copy)]
pub struct Beta {
    /// First shape parameter (α).
    pub alpha: f64,
    /// Second shape parameter (β).
    pub beta: f64,
}

impl Beta {
    /// Create a new beta distribution.
    pub fn new(alpha: f64, beta: f64) -> Self {
        assert!(alpha > 0.0, "alpha must be positive");
        assert!(beta > 0.0, "beta must be positive");
        Self { alpha, beta }
    }
}

impl ContinuousDistribution for Beta {
    fn pdf(&self, x: f64) -> f64 {
        if x < 0.0 || x > 1.0 {
            return 0.0;
        }
        let b = beta_fn(self.alpha, self.beta);
        x.powf(self.alpha - 1.0) * (1.0 - x).powf(self.beta - 1.0) / b
    }

    fn cdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        if x >= 1.0 {
            return 1.0;
        }
        incomplete_beta(self.alpha, self.beta, x)
    }

    fn mean(&self) -> f64 {
        self.alpha / (self.alpha + self.beta)
    }

    fn variance(&self) -> f64 {
        let ab = self.alpha + self.beta;
        (self.alpha * self.beta) / (ab * ab * (ab + 1.0))
    }
}

/// Student's t-distribution.
#[derive(Debug, Clone, Copy)]
pub struct StudentT {
    /// Degrees of freedom (ν).
    pub df: f64,
}

impl StudentT {
    /// Create a new Student's t-distribution.
    pub fn new(df: f64) -> Self {
        assert!(df > 0.0, "df must be positive");
        Self { df }
    }
}

impl ContinuousDistribution for StudentT {
    fn pdf(&self, x: f64) -> f64 {
        let v = self.df;
        let coef = gamma_fn((v + 1.0) / 2.0) / (gamma_fn(v / 2.0) * (v * PI).sqrt());
        coef * (1.0 + x * x / v).powf(-(v + 1.0) / 2.0)
    }

    fn cdf(&self, x: f64) -> f64 {
        let v = self.df;
        let t = v / (v + x * x);
        if x >= 0.0 {
            1.0 - 0.5 * incomplete_beta(v / 2.0, 0.5, t)
        } else {
            0.5 * incomplete_beta(v / 2.0, 0.5, t)
        }
    }

    fn mean(&self) -> f64 {
        if self.df > 1.0 {
            0.0
        } else {
            f64::NAN
        }
    }

    fn variance(&self) -> f64 {
        if self.df > 2.0 {
            self.df / (self.df - 2.0)
        } else if self.df > 1.0 {
            f64::INFINITY
        } else {
            f64::NAN
        }
    }
}

/// Chi-squared distribution.
#[derive(Debug, Clone, Copy)]
pub struct ChiSquared {
    /// Degrees of freedom (k).
    pub df: f64,
}

impl ChiSquared {
    /// Create a new chi-squared distribution.
    pub fn new(df: f64) -> Self {
        assert!(df > 0.0, "df must be positive");
        Self { df }
    }
}

impl ContinuousDistribution for ChiSquared {
    fn pdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let k = self.df;
        let coef = 1.0 / (2.0_f64.powf(k / 2.0) * gamma_fn(k / 2.0));
        coef * x.powf(k / 2.0 - 1.0) * (-x / 2.0).exp()
    }

    fn cdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        lower_incomplete_gamma(self.df / 2.0, x / 2.0) / gamma_fn(self.df / 2.0)
    }

    fn mean(&self) -> f64 {
        self.df
    }

    fn variance(&self) -> f64 {
        2.0 * self.df
    }
}

/// F-distribution.
#[derive(Debug, Clone, Copy)]
pub struct FDistribution {
    /// First degrees of freedom (d1).
    pub d1: f64,
    /// Second degrees of freedom (d2).
    pub d2: f64,
}

impl FDistribution {
    /// Create a new F-distribution.
    pub fn new(d1: f64, d2: f64) -> Self {
        assert!(d1 > 0.0, "d1 must be positive");
        assert!(d2 > 0.0, "d2 must be positive");
        Self { d1, d2 }
    }
}

impl ContinuousDistribution for FDistribution {
    fn pdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let d1 = self.d1;
        let d2 = self.d2;
        let num = ((d1 * x).powf(d1) * d2.powf(d2) / (d1 * x + d2).powf(d1 + d2)).sqrt();
        let den = x * beta_fn(d1 / 2.0, d2 / 2.0);
        num / den
    }

    fn cdf(&self, x: f64) -> f64 {
        if x <= 0.0 {
            return 0.0;
        }
        let d1 = self.d1;
        let d2 = self.d2;
        incomplete_beta(d1 / 2.0, d2 / 2.0, d1 * x / (d1 * x + d2))
    }

    fn mean(&self) -> f64 {
        if self.d2 > 2.0 {
            self.d2 / (self.d2 - 2.0)
        } else {
            f64::NAN
        }
    }

    fn variance(&self) -> f64 {
        if self.d2 > 4.0 {
            let d1 = self.d1;
            let d2 = self.d2;
            (2.0 * d2 * d2 * (d1 + d2 - 2.0)) / (d1 * (d2 - 2.0).powi(2) * (d2 - 4.0))
        } else {
            f64::NAN
        }
    }
}

// ============================================================================
// Discrete Distributions
// ============================================================================

/// Binomial distribution.
#[derive(Debug, Clone, Copy)]
pub struct Binomial {
    /// Number of trials.
    pub n: u64,
    /// Probability of success.
    pub p: f64,
}

impl Binomial {
    /// Create a new binomial distribution.
    pub fn new(n: u64, p: f64) -> Self {
        assert!((0.0..=1.0).contains(&p), "p must be between 0 and 1");
        Self { n, p }
    }
}

impl DiscreteDistribution for Binomial {
    fn pmf(&self, k: u64) -> f64 {
        if k > self.n {
            return 0.0;
        }
        let coef = binomial_coef(self.n, k);
        coef * self.p.powi(k as i32) * (1.0 - self.p).powi((self.n - k) as i32)
    }

    fn cdf(&self, k: u64) -> f64 {
        (0..=k).map(|i| self.pmf(i)).sum()
    }

    fn mean(&self) -> f64 {
        self.n as f64 * self.p
    }

    fn variance(&self) -> f64 {
        self.n as f64 * self.p * (1.0 - self.p)
    }
}

/// Poisson distribution.
#[derive(Debug, Clone, Copy)]
pub struct Poisson {
    /// Rate parameter (λ).
    pub lambda: f64,
}

impl Poisson {
    /// Create a new Poisson distribution.
    pub fn new(lambda: f64) -> Self {
        assert!(lambda > 0.0, "lambda must be positive");
        Self { lambda }
    }
}

impl DiscreteDistribution for Poisson {
    fn pmf(&self, k: u64) -> f64 {
        self.lambda.powi(k as i32) * (-self.lambda).exp() / factorial(k)
    }

    fn cdf(&self, k: u64) -> f64 {
        (0..=k).map(|i| self.pmf(i)).sum()
    }

    fn mean(&self) -> f64 {
        self.lambda
    }

    fn variance(&self) -> f64 {
        self.lambda
    }
}

/// Geometric distribution (number of trials until first success).
#[derive(Debug, Clone, Copy)]
pub struct Geometric {
    /// Probability of success.
    pub p: f64,
}

impl Geometric {
    /// Create a new geometric distribution.
    pub fn new(p: f64) -> Self {
        assert!((0.0..=1.0).contains(&p) && p > 0.0, "p must be between 0 and 1");
        Self { p }
    }
}

impl DiscreteDistribution for Geometric {
    fn pmf(&self, k: u64) -> f64 {
        if k == 0 {
            return 0.0;
        }
        (1.0 - self.p).powi((k - 1) as i32) * self.p
    }

    fn cdf(&self, k: u64) -> f64 {
        if k == 0 {
            return 0.0;
        }
        1.0 - (1.0 - self.p).powi(k as i32)
    }

    fn mean(&self) -> f64 {
        1.0 / self.p
    }

    fn variance(&self) -> f64 {
        (1.0 - self.p) / (self.p * self.p)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Error function (erf).
pub fn erf(x: f64) -> f64 {
    // Approximation using Horner's method
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;

    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();

    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();

    sign * y
}

/// Gamma function using Lanczos approximation.
pub fn gamma_fn(x: f64) -> f64 {
    if x <= 0.0 && x == x.floor() {
        return f64::INFINITY;
    }

    // Use reflection formula for negative values
    if x < 0.5 {
        return PI / ((PI * x).sin() * gamma_fn(1.0 - x));
    }

    let x = x - 1.0;

    // Lanczos coefficients
    let g = 7.0;
    let c = [
        0.99999999999980993,
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278686905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];

    let mut sum = c[0];
    for (i, &ci) in c.iter().enumerate().skip(1) {
        sum += ci / (x + i as f64);
    }

    let t = x + g + 0.5;
    (2.0 * PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * sum
}

/// Log gamma function.
pub fn ln_gamma(x: f64) -> f64 {
    gamma_fn(x).ln()
}

/// Beta function B(a, b) = Γ(a)Γ(b)/Γ(a+b).
pub fn beta_fn(a: f64, b: f64) -> f64 {
    gamma_fn(a) * gamma_fn(b) / gamma_fn(a + b)
}

/// Lower incomplete gamma function γ(s, x).
pub fn lower_incomplete_gamma(s: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }

    // Series expansion
    let mut sum = 0.0;
    let mut term = 1.0 / s;
    sum += term;

    for n in 1..100 {
        term *= x / (s + n as f64);
        sum += term;
        if term.abs() < 1e-15 * sum.abs() {
            break;
        }
    }

    x.powf(s) * (-x).exp() * sum
}

/// Regularized incomplete beta function I_x(a, b).
pub fn incomplete_beta(a: f64, b: f64, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // Use continued fraction expansion
    let bt = if x == 0.0 || x == 1.0 {
        0.0
    } else {
        (ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b) + a * x.ln() + b * (1.0 - x).ln()).exp()
    };

    if x < (a + 1.0) / (a + b + 2.0) {
        bt * beta_cf(a, b, x) / a
    } else {
        1.0 - bt * beta_cf(b, a, 1.0 - x) / b
    }
}

/// Continued fraction for incomplete beta.
fn beta_cf(a: f64, b: f64, x: f64) -> f64 {
    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < 1e-30 {
        d = 1e-30;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=100 {
        let m = m as f64;
        let m2 = 2.0 * m;

        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        h *= d * c;

        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-30 {
            d = 1e-30;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-30 {
            c = 1e-30;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() < 1e-10 {
            break;
        }
    }

    h
}

/// Factorial n!
pub fn factorial(n: u64) -> f64 {
    if n <= 1 {
        return 1.0;
    }
    (2..=n).fold(1.0, |acc, x| acc * x as f64)
}

/// Binomial coefficient C(n, k).
pub fn binomial_coef(n: u64, k: u64) -> f64 {
    if k > n {
        return 0.0;
    }
    if k == 0 || k == n {
        return 1.0;
    }

    let k = k.min(n - k);
    let mut result = 1.0;
    for i in 0..k {
        result *= (n - i) as f64 / (i + 1) as f64;
    }
    result
}

#[cfg(test)]
mod tests {
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
        assert!((b.mean() - 2.0/7.0).abs() < 1e-10);
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
        assert!((t.variance() - 10.0/8.0).abs() < 1e-10);
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
        assert!((beta_fn(2.0, 2.0) - 1.0/6.0).abs() < 1e-10);
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
