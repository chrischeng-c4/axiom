//! Feature engineering: PolynomialFeatures, CountVectorizer.
//!
//! These complement the existing preprocessing (StandardScaler, MinMaxScaler,
//! LabelEncoder, OneHotEncoder) from `preprocessing.rs`.

use super::error::{MlError, Result};
use super::traits::Transformer;
use std::collections::BTreeMap;

// ============================================================================
// PolynomialFeatures
// ============================================================================

/// Generate polynomial and interaction features.
///
/// For degree=2 and 2 features `[a, b]`, generates `[1, a, b, a^2, a*b, b^2]`.
/// Set `include_bias=false` to omit the leading 1.
/// Set `interaction_only=true` to omit pure powers (a^2, b^2, etc.).
#[derive(Debug, Clone)]
pub struct PolynomialFeatures {
    pub degree: usize,
    pub include_bias: bool,
    pub interaction_only: bool,
    n_input_features: Option<usize>,
    /// Cached combination powers: each entry is a vec of exponents (one per input feature).
    combos: Vec<Vec<usize>>,
}

impl Default for PolynomialFeatures {
    fn default() -> Self {
        Self::new(2, true, false)
    }
}

impl PolynomialFeatures {
    pub fn new(degree: usize, include_bias: bool, interaction_only: bool) -> Self {
        assert!(degree >= 1, "degree must be >= 1");
        Self {
            degree,
            include_bias,
            interaction_only,
            n_input_features: None,
            combos: Vec::new(),
        }
    }

    /// Number of output features after fit.
    pub fn n_output_features(&self) -> Option<usize> {
        if self.combos.is_empty() && self.n_input_features.is_none() {
            None
        } else {
            Some(self.combos.len())
        }
    }

    /// Generate all combinations of exponents that sum to <= degree.
    ///
    /// Output is sorted by total degree, then lexicographically (matching
    /// scikit-learn convention):
    /// For 2 features, degree 2: `[1, a, b, a^2, ab, b^2]`.
    fn generate_combos(
        n_features: usize,
        degree: usize,
        include_bias: bool,
        interaction_only: bool,
    ) -> Vec<Vec<usize>> {
        let mut combos = Vec::new();
        let mut current = vec![0usize; n_features];
        Self::enumerate_combos(
            &mut combos,
            &mut current,
            0,
            degree,
            include_bias,
            interaction_only,
        );
        // Sort by total degree, then graded reverse lexicographic order:
        // higher exponents on earlier features come first.
        // This matches scikit-learn: [1, x0, x1, x0^2, x0*x1, x1^2].
        combos.sort_by(|a, b| {
            let da: usize = a.iter().sum();
            let db: usize = b.iter().sum();
            da.cmp(&db).then_with(|| b.cmp(a))
        });
        combos
    }

    fn enumerate_combos(
        combos: &mut Vec<Vec<usize>>,
        current: &mut Vec<usize>,
        feature_idx: usize,
        remaining_degree: usize,
        include_bias: bool,
        interaction_only: bool,
    ) {
        let n = current.len();
        if feature_idx == n {
            let total: usize = current.iter().sum();
            if total == 0 && !include_bias {
                return;
            }
            if interaction_only {
                // Check no exponent > 1
                if current.iter().any(|&e| e > 1) {
                    return;
                }
            }
            combos.push(current.clone());
            return;
        }

        let max_power = if interaction_only {
            remaining_degree.min(1)
        } else {
            remaining_degree
        };

        for power in 0..=max_power {
            current[feature_idx] = power;
            Self::enumerate_combos(
                combos,
                current,
                feature_idx + 1,
                remaining_degree - power,
                include_bias,
                interaction_only,
            );
        }
        current[feature_idx] = 0;
    }

    /// Apply the polynomial transform to a single sample.
    fn transform_sample(&self, sample: &[f64]) -> Vec<f64> {
        self.combos
            .iter()
            .map(|exponents| {
                let mut val = 1.0;
                for (i, &exp) in exponents.iter().enumerate() {
                    if exp > 0 {
                        val *= sample[i].powi(exp as i32);
                    }
                }
                val
            })
            .collect()
    }
}

impl Transformer for PolynomialFeatures {
    fn fit_data(&mut self, _x: &[f64], n_features: usize) -> Result<()> {
        self.n_input_features = Some(n_features);
        self.combos = Self::generate_combos(
            n_features,
            self.degree,
            self.include_bias,
            self.interaction_only,
        );
        Ok(())
    }

    fn transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        if self.n_input_features.is_none() {
            return Err(MlError::NotFitted);
        }
        let n_in = self.n_input_features.unwrap();
        if n_features != n_in {
            return Err(MlError::InvalidParameter(format!(
                "expected {} features, got {}",
                n_in, n_features
            )));
        }
        let n_samples = x.len() / n_features;
        let n_out = self.combos.len();
        let mut result = Vec::with_capacity(n_samples * n_out);
        for i in 0..n_samples {
            let sample = &x[i * n_features..(i + 1) * n_features];
            result.extend(self.transform_sample(sample));
        }
        Ok(result)
    }

    fn inverse_transform(&self, _x: &[f64], _n_features: usize) -> Result<Vec<f64>> {
        Err(MlError::InvalidParameter(
            "PolynomialFeatures does not support inverse_transform".into(),
        ))
    }
}

// ============================================================================
// CountVectorizer (simple bag-of-words)
// ============================================================================

/// Simple bag-of-words text vectorizer.
///
/// Converts a collection of text documents to a matrix of token counts.
/// Tokens are whitespace-separated, lowercased words.
#[derive(Debug, Clone)]
pub struct CountVectorizer {
    /// Vocabulary: word -> index mapping.
    pub vocabulary: Option<BTreeMap<String, usize>>,
    /// Maximum number of features (vocabulary size limit). None = unlimited.
    pub max_features: Option<usize>,
}

impl Default for CountVectorizer {
    fn default() -> Self {
        Self::new(None)
    }
}

impl CountVectorizer {
    pub fn new(max_features: Option<usize>) -> Self {
        Self {
            vocabulary: None,
            max_features,
        }
    }

    /// Tokenize a document into lowercase words.
    fn tokenize(doc: &str) -> Vec<String> {
        doc.split_whitespace()
            .map(|w| {
                w.chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|w| !w.is_empty())
            .collect()
    }

    /// Fit the vectorizer on a list of documents.
    pub fn fit(&mut self, documents: &[&str]) {
        // Count document frequency for each word
        let mut word_counts: BTreeMap<String, usize> = BTreeMap::new();
        for doc in documents {
            let tokens = Self::tokenize(doc);
            // Use set to count each word once per document
            let unique: std::collections::BTreeSet<String> = tokens.into_iter().collect();
            for word in unique {
                *word_counts.entry(word).or_insert(0) += 1;
            }
        }

        // Sort by frequency (descending), then alphabetically
        let mut words: Vec<(String, usize)> = word_counts.into_iter().collect();
        words.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        // Apply max_features limit
        if let Some(max) = self.max_features {
            words.truncate(max);
        }

        // Build vocabulary index
        let mut vocab = BTreeMap::new();
        for (idx, (word, _)) in words.into_iter().enumerate() {
            vocab.insert(word, idx);
        }
        self.vocabulary = Some(vocab);
    }

    /// Transform documents into a flat feature matrix (n_docs x vocab_size).
    ///
    /// Returns `(data, n_features)` where data is row-major.
    pub fn transform(&self, documents: &[&str]) -> Result<(Vec<f64>, usize)> {
        let vocab = self.vocabulary.as_ref().ok_or(MlError::NotFitted)?;
        let n_features = vocab.len();
        let n_docs = documents.len();
        let mut result = vec![0.0; n_docs * n_features];

        for (doc_idx, doc) in documents.iter().enumerate() {
            let tokens = Self::tokenize(doc);
            for token in tokens {
                if let Some(&feat_idx) = vocab.get(&token) {
                    result[doc_idx * n_features + feat_idx] += 1.0;
                }
            }
        }

        Ok((result, n_features))
    }

    /// Fit and transform in one step.
    pub fn fit_transform(&mut self, documents: &[&str]) -> Result<(Vec<f64>, usize)> {
        self.fit(documents);
        self.transform(documents)
    }

    /// Get vocabulary size.
    pub fn vocab_size(&self) -> Option<usize> {
        self.vocabulary.as_ref().map(|v| v.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- PolynomialFeatures ---

    #[test]
    fn test_poly_degree2_with_bias() {
        let mut pf = PolynomialFeatures::new(2, true, false);
        // 2 features: [a, b] -> [1, a, b, a^2, ab, b^2]
        let x = vec![2.0, 3.0];
        pf.fit_data(&x, 2).unwrap();
        let out = pf.transform(&x, 2).unwrap();
        assert_eq!(pf.n_output_features(), Some(6));
        // Expected: [1, 2, 3, 4, 6, 9]
        let expected = vec![1.0, 2.0, 3.0, 4.0, 6.0, 9.0];
        assert_eq!(out.len(), expected.len());
        for (a, b) in out.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-10, "expected {}, got {}", b, a);
        }
    }

    #[test]
    fn test_poly_degree2_no_bias() {
        let mut pf = PolynomialFeatures::new(2, false, false);
        let x = vec![2.0, 3.0];
        pf.fit_data(&x, 2).unwrap();
        let out = pf.transform(&x, 2).unwrap();
        // [a, b, a^2, ab, b^2] = [2, 3, 4, 6, 9]
        assert_eq!(pf.n_output_features(), Some(5));
        let expected = vec![2.0, 3.0, 4.0, 6.0, 9.0];
        for (a, b) in out.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_poly_interaction_only() {
        let mut pf = PolynomialFeatures::new(2, false, true);
        let x = vec![2.0, 3.0, 4.0];
        pf.fit_data(&x, 3).unwrap();
        let out = pf.transform(&x, 3).unwrap();
        // interaction_only: no a^2, b^2 etc., only cross terms
        // degree<=2: [a, b, c, ab, ac, bc]
        let expected = vec![2.0, 3.0, 4.0, 6.0, 8.0, 12.0];
        assert_eq!(out.len(), expected.len());
        for (a, b) in out.iter().zip(expected.iter()) {
            assert!((a - b).abs() < 1e-10, "expected {}, got {}", b, a);
        }
    }

    #[test]
    fn test_poly_degree3() {
        let mut pf = PolynomialFeatures::new(3, true, false);
        let x = vec![2.0];
        pf.fit_data(&x, 1).unwrap();
        let out = pf.transform(&x, 1).unwrap();
        // Single feature, degree 3: [1, x, x^2, x^3]
        assert_eq!(out, vec![1.0, 2.0, 4.0, 8.0]);
    }

    #[test]
    fn test_poly_multiple_samples() {
        let mut pf = PolynomialFeatures::new(2, true, false);
        let x = vec![1.0, 2.0, 3.0, 4.0]; // 2 samples, 2 features
        pf.fit_data(&x, 2).unwrap();
        let out = pf.transform(&x, 2).unwrap();
        let n_out = pf.n_output_features().unwrap();
        assert_eq!(out.len(), 2 * n_out);
        // First sample: [1, 1, 2, 1, 2, 4]
        assert!((out[0] - 1.0).abs() < 1e-10); // bias
        assert!((out[1] - 1.0).abs() < 1e-10); // a=1
        assert!((out[2] - 2.0).abs() < 1e-10); // b=2
    }

    #[test]
    fn test_poly_not_fitted() {
        let pf = PolynomialFeatures::new(2, true, false);
        assert!(pf.transform(&[1.0, 2.0], 2).is_err());
    }

    // --- CountVectorizer ---

    #[test]
    fn test_count_vectorizer_basic() {
        let mut cv = CountVectorizer::new(None);
        let docs = vec!["hello world", "hello rust"];
        let (data, n_features) = cv.fit_transform(&docs).unwrap();
        assert_eq!(n_features, 3); // hello, rust, world
        assert_eq!(data.len(), 2 * 3);
    }

    #[test]
    fn test_count_vectorizer_counts() {
        let mut cv = CountVectorizer::new(None);
        let docs = vec!["the cat sat on the mat"];
        cv.fit(&docs);
        let (data, n_features) = cv.transform(&docs).unwrap();
        // "the" appears 2 times
        let vocab = cv.vocabulary.as_ref().unwrap();
        let the_idx = vocab.get("the").unwrap();
        assert!((data[*the_idx] - 2.0).abs() < 1e-10);
        assert!(n_features > 0);
    }

    #[test]
    fn test_count_vectorizer_max_features() {
        let mut cv = CountVectorizer::new(Some(2));
        let docs = vec!["a b c d e", "a b c"];
        cv.fit(&docs);
        assert_eq!(cv.vocab_size(), Some(2));
    }

    #[test]
    fn test_count_vectorizer_not_fitted() {
        let cv = CountVectorizer::new(None);
        assert!(cv.transform(&["hello"]).is_err());
    }

    #[test]
    fn test_count_vectorizer_empty_doc() {
        let mut cv = CountVectorizer::new(None);
        let docs = vec!["hello world", ""];
        let (data, n_features) = cv.fit_transform(&docs).unwrap();
        // Second doc should be all zeros
        let row2_sum: f64 = data[n_features..].iter().sum();
        assert!((row2_sum - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_count_vectorizer_punctuation() {
        let mut cv = CountVectorizer::new(None);
        let docs = vec!["Hello, World!", "hello world"];
        cv.fit(&docs);
        let vocab = cv.vocabulary.as_ref().unwrap();
        // Both should normalize to "hello" and "world"
        assert!(vocab.contains_key("hello"));
        assert!(vocab.contains_key("world"));
    }
}
