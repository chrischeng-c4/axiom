//! Core ML traits (scikit-learn inspired).

use super::error::Result;

/// A model that can be trained on data.
pub trait Estimator {
    /// Fit the model to training data.
    ///
    /// `x` is a 2D matrix (n_samples × n_features), stored row-major as a flat slice.
    /// `y` is the target vector (n_samples).
    fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()>;
}

/// A fitted model that can make predictions.
pub trait Predictor {
    /// Predict target values for new data.
    fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
}

/// A classifier that can predict class probabilities.
pub trait Classifier: Estimator + Predictor {
    /// Predict class probabilities for each sample.
    ///
    /// Returns a flat vec of (n_samples × n_classes), row-major.
    fn predict_proba(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
}

/// A data transformer (scaler, encoder, etc.).
pub trait Transformer {
    /// Fit the transformer to data.
    fn fit_data(&mut self, x: &[f64], n_features: usize) -> Result<()>;

    /// Transform data using the fitted parameters.
    fn transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;

    /// Fit and transform in one step.
    fn fit_transform(&mut self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        self.fit_data(x, n_features)?;
        self.transform(x, n_features)
    }

    /// Inverse transform (if supported).
    fn inverse_transform(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
}
