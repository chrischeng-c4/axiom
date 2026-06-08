//! Preprocessing: scalers and encoders.

use super::error::{MlError, Result};
use super::traits::Transformer;

// ============================================================================
// StandardScaler (z-score normalization)
// ============================================================================

/// Standardize features by removing the mean and scaling to unit variance.
#[derive(Debug, Clone)]
pub struct StandardScaler {
    pub mean: Option<Vec<f64>>,
    pub std: Option<Vec<f64>>,
}

impl Default for StandardScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl StandardScaler {
    pub fn new() -> Self {
        Self {
            mean: None,
            std: None,
        }
    }
}

impl Transformer for StandardScaler {
    fn fit_data(&mut self, x: &[f64], n_features: usize) -> Result<()> {
        let n = x.len() / n_features;
        let mut mean = vec![0.0; n_features];
        for i in 0..n {
            for j in 0..n_features {
                mean[j] += x[i * n_features + j];
            }
        }
        for m in &mut mean {
            *m /= n as f64;
        }

        let mut std = vec![0.0; n_features];
        for i in 0..n {
            for j in 0..n_features {
                std[j] += (x[i * n_features + j] - mean[j]).powi(2);
            }
        }
        for s in &mut std {
            *s = (*s / n as f64).sqrt();
            if *s < 1e-15 {
                *s = 1.0; // Avoid division by zero
            }
        }

        self.mean = Some(mean);
        self.std = Some(std);
        Ok(())
    }

    fn transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let mean = self.mean.as_ref().ok_or(MlError::NotFitted)?;
        let std = self.std.as_ref().ok_or(MlError::NotFitted)?;
        let n = x.len() / n_features;

        let mut result = Vec::with_capacity(x.len());
        for i in 0..n {
            for j in 0..n_features {
                result.push((x[i * n_features + j] - mean[j]) / std[j]);
            }
        }
        Ok(result)
    }

    fn inverse_transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let mean = self.mean.as_ref().ok_or(MlError::NotFitted)?;
        let std = self.std.as_ref().ok_or(MlError::NotFitted)?;
        let n = x.len() / n_features;

        let mut result = Vec::with_capacity(x.len());
        for i in 0..n {
            for j in 0..n_features {
                result.push(x[i * n_features + j] * std[j] + mean[j]);
            }
        }
        Ok(result)
    }
}

// ============================================================================
// MinMaxScaler
// ============================================================================

/// Scale features to a given range (default [0, 1]).
#[derive(Debug, Clone)]
pub struct MinMaxScaler {
    pub min: Option<Vec<f64>>,
    pub max: Option<Vec<f64>>,
    pub feature_range: (f64, f64),
}

impl Default for MinMaxScaler {
    fn default() -> Self {
        Self::new()
    }
}

impl MinMaxScaler {
    pub fn new() -> Self {
        Self {
            min: None,
            max: None,
            feature_range: (0.0, 1.0),
        }
    }

    pub fn with_range(mut self, low: f64, high: f64) -> Self {
        self.feature_range = (low, high);
        self
    }
}

impl Transformer for MinMaxScaler {
    fn fit_data(&mut self, x: &[f64], n_features: usize) -> Result<()> {
        let n = x.len() / n_features;
        let mut min_vals = vec![f64::INFINITY; n_features];
        let mut max_vals = vec![f64::NEG_INFINITY; n_features];

        for i in 0..n {
            for j in 0..n_features {
                let v = x[i * n_features + j];
                min_vals[j] = min_vals[j].min(v);
                max_vals[j] = max_vals[j].max(v);
            }
        }

        self.min = Some(min_vals);
        self.max = Some(max_vals);
        Ok(())
    }

    fn transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let min_vals = self.min.as_ref().ok_or(MlError::NotFitted)?;
        let max_vals = self.max.as_ref().ok_or(MlError::NotFitted)?;
        let (low, high) = self.feature_range;
        let n = x.len() / n_features;

        let mut result = Vec::with_capacity(x.len());
        for i in 0..n {
            for j in 0..n_features {
                let range = max_vals[j] - min_vals[j];
                let scaled = if range.abs() < 1e-15 {
                    low
                } else {
                    (x[i * n_features + j] - min_vals[j]) / range * (high - low) + low
                };
                result.push(scaled);
            }
        }
        Ok(result)
    }

    fn inverse_transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let min_vals = self.min.as_ref().ok_or(MlError::NotFitted)?;
        let max_vals = self.max.as_ref().ok_or(MlError::NotFitted)?;
        let (low, high) = self.feature_range;
        let n = x.len() / n_features;

        let mut result = Vec::with_capacity(x.len());
        for i in 0..n {
            for j in 0..n_features {
                let range = max_vals[j] - min_vals[j];
                let val = if (high - low).abs() < 1e-15 {
                    min_vals[j]
                } else {
                    (x[i * n_features + j] - low) / (high - low) * range + min_vals[j]
                };
                result.push(val);
            }
        }
        Ok(result)
    }
}

// ============================================================================
// LabelEncoder
// ============================================================================

/// Encode string labels as integers.
#[derive(Debug, Clone)]
pub struct LabelEncoder {
    pub classes: Option<Vec<String>>,
}

impl Default for LabelEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelEncoder {
    pub fn new() -> Self {
        Self { classes: None }
    }

    pub fn fit(&mut self, labels: &[&str]) {
        let mut unique: Vec<String> = labels.iter().map(|s| s.to_string()).collect();
        unique.sort();
        unique.dedup();
        self.classes = Some(unique);
    }

    pub fn transform(&self, labels: &[&str]) -> Result<Vec<usize>> {
        let classes = self.classes.as_ref().ok_or(MlError::NotFitted)?;
        labels
            .iter()
            .map(|&label| {
                classes
                    .iter()
                    .position(|c| c == label)
                    .ok_or_else(|| MlError::InvalidParameter(format!("unknown label: {}", label)))
            })
            .collect()
    }

    pub fn inverse_transform(&self, encoded: &[usize]) -> Result<Vec<String>> {
        let classes = self.classes.as_ref().ok_or(MlError::NotFitted)?;
        encoded
            .iter()
            .map(|&idx| {
                classes
                    .get(idx)
                    .cloned()
                    .ok_or_else(|| MlError::InvalidParameter(format!("index {} out of range", idx)))
            })
            .collect()
    }
}

// ============================================================================
// OneHotEncoder
// ============================================================================

/// One-hot encode categorical integer labels.
#[derive(Debug, Clone)]
pub struct OneHotEncoder {
    pub n_categories: Option<usize>,
}

impl Default for OneHotEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl OneHotEncoder {
    pub fn new() -> Self {
        Self { n_categories: None }
    }

    pub fn fit(&mut self, labels: &[usize]) {
        self.n_categories = labels.iter().max().map(|&m| m + 1);
    }

    pub fn transform(&self, labels: &[usize]) -> Result<Vec<f64>> {
        let n_cat = self.n_categories.ok_or(MlError::NotFitted)?;
        let mut result = vec![0.0; labels.len() * n_cat];
        for (i, &label) in labels.iter().enumerate() {
            if label >= n_cat {
                return Err(MlError::InvalidParameter(format!(
                    "label {} >= n_categories {}",
                    label, n_cat
                )));
            }
            result[i * n_cat + label] = 1.0;
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_scaler() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let mut scaler = StandardScaler::new();
        let transformed = scaler.fit_transform(&x, 2).unwrap();
        // Mean should be ~0 for each feature
        let mean_f0 = (transformed[0] + transformed[2] + transformed[4]) / 3.0;
        assert!(mean_f0.abs() < 1e-10);

        let recovered = scaler.inverse_transform(&transformed, 2).unwrap();
        for (orig, rec) in x.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 1e-10);
        }
    }

    #[test]
    fn test_minmax_scaler() {
        let x = vec![1.0, 10.0, 5.0, 20.0, 9.0, 30.0];
        let mut scaler = MinMaxScaler::new();
        let transformed = scaler.fit_transform(&x, 2).unwrap();
        assert!((transformed[0] - 0.0).abs() < 1e-10); // min → 0
        assert!((transformed[4] - 1.0).abs() < 1e-10); // max → 1

        let recovered = scaler.inverse_transform(&transformed, 2).unwrap();
        for (orig, rec) in x.iter().zip(recovered.iter()) {
            assert!((orig - rec).abs() < 1e-10);
        }
    }

    #[test]
    fn test_label_encoder() {
        let mut enc = LabelEncoder::new();
        enc.fit(&["cat", "dog", "cat", "bird"]);
        let encoded = enc.transform(&["bird", "cat", "dog"]).unwrap();
        assert_eq!(encoded, vec![0, 1, 2]);
        let decoded = enc.inverse_transform(&encoded).unwrap();
        assert_eq!(decoded, vec!["bird", "cat", "dog"]);
    }

    #[test]
    fn test_onehot_encoder() {
        let mut enc = OneHotEncoder::new();
        enc.fit(&[0, 1, 2]);
        let encoded = enc.transform(&[0, 2, 1]).unwrap();
        assert_eq!(encoded, vec![1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_scaler_not_fitted() {
        let scaler = StandardScaler::new();
        assert!(scaler.transform(&[1.0], 1).is_err());
    }
}
