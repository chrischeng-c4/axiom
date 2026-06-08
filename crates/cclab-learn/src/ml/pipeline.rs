//! ML Pipeline — chain transformers and a final estimator.

use super::error::{MlError, Result};

/// A step in the pipeline.
pub enum PipelineStep {
    /// A transformer step (e.g., scaler).
    Transform(Box<dyn PipelineTransformer>),
    /// Final estimator (predictor).
    Predict(Box<dyn PipelinePredictor>),
}

/// Trait for pipeline-compatible transformers.
pub trait PipelineTransformer {
    fn fit_transform_pipeline(&mut self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
    fn transform_pipeline(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
}

/// Trait for pipeline-compatible predictors.
pub trait PipelinePredictor {
    fn fit_pipeline(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()>;
    fn predict_pipeline(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>>;
}

/// A machine learning pipeline: sequence of transforms + final predictor.
pub struct Pipeline {
    transforms: Vec<Box<dyn PipelineTransformer>>,
    predictor: Option<Box<dyn PipelinePredictor>>,
    n_features: usize,
}

impl Pipeline {
    pub fn new() -> Self {
        Self {
            transforms: Vec::new(),
            predictor: None,
            n_features: 0,
        }
    }

    /// Add a transformer step.
    pub fn add_transform(mut self, t: Box<dyn PipelineTransformer>) -> Self {
        self.transforms.push(t);
        self
    }

    /// Set the final predictor.
    pub fn set_predictor(mut self, p: Box<dyn PipelinePredictor>) -> Self {
        self.predictor = Some(p);
        self
    }

    /// Fit the entire pipeline.
    pub fn fit(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        self.n_features = n_features;
        let mut data = x.to_vec();

        for transform in &mut self.transforms {
            data = transform.fit_transform_pipeline(&data, n_features)?;
        }

        if let Some(ref mut predictor) = self.predictor {
            predictor.fit_pipeline(&data, n_features, y)?;
        }
        Ok(())
    }

    /// Predict using the fitted pipeline.
    pub fn predict(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        let mut data = x.to_vec();

        for transform in &self.transforms {
            data = transform.transform_pipeline(&data, n_features)?;
        }

        self.predictor
            .as_ref()
            .ok_or(MlError::NotFitted)?
            .predict_pipeline(&data, n_features)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

// Blanket implementations for existing traits
impl<T: super::traits::Transformer> PipelineTransformer for T {
    fn fit_transform_pipeline(&mut self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        self.fit_data(x, n_features)?;
        self.transform(x, n_features)
    }
    fn transform_pipeline(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        self.transform(x, n_features)
    }
}

impl<T: super::traits::Estimator + super::traits::Predictor> PipelinePredictor for T {
    fn fit_pipeline(&mut self, x: &[f64], n_features: usize, y: &[f64]) -> Result<()> {
        self.fit(x, n_features, y)
    }
    fn predict_pipeline(&self, x: &[f64], n_features: usize) -> Result<Vec<f64>> {
        self.predict(x, n_features)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ml::linear::LinearRegression;
    use crate::ml::preprocessing::{MinMaxScaler, StandardScaler};

    // ── Pipeline::new() ────────────────────────────────────────────────

    #[test]
    fn test_new_defaults() {
        let p = Pipeline::new();
        assert!(p.transforms.is_empty());
        assert!(p.predictor.is_none());
        assert_eq!(p.n_features, 0);
    }

    #[test]
    fn test_default_same_as_new() {
        let p = Pipeline::default();
        assert!(p.transforms.is_empty());
        assert!(p.predictor.is_none());
    }

    // ── add_transform() ────────────────────────────────────────────────

    #[test]
    fn test_add_single_transform() {
        let p = Pipeline::new().add_transform(Box::new(StandardScaler::new()));
        assert_eq!(p.transforms.len(), 1);
    }

    #[test]
    fn test_add_multiple_transforms() {
        let p = Pipeline::new()
            .add_transform(Box::new(StandardScaler::new()))
            .add_transform(Box::new(MinMaxScaler::new()));
        assert_eq!(p.transforms.len(), 2);
    }

    // ── set_predictor() ────────────────────────────────────────────────

    #[test]
    fn test_set_predictor() {
        let p = Pipeline::new().set_predictor(Box::new(LinearRegression::new()));
        assert!(p.predictor.is_some());
    }

    #[test]
    fn test_set_predictor_replaces() {
        let p = Pipeline::new()
            .set_predictor(Box::new(LinearRegression::new()))
            .set_predictor(Box::new(LinearRegression::new()));
        assert!(p.predictor.is_some());
    }

    // ── fit() ──────────────────────────────────────────────────────────

    #[test]
    fn test_fit_happy_scaler_regression() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mut pipeline = Pipeline::new()
            .add_transform(Box::new(StandardScaler::new()))
            .set_predictor(Box::new(LinearRegression::new()));
        pipeline.fit(&x, 1, &y).unwrap();
        let pred = pipeline.predict(&[3.0], 1).unwrap();
        assert!((pred[0] - 6.0).abs() < 1.0);
    }

    #[test]
    fn test_fit_no_predictor_ok() {
        // fit() should succeed even without a predictor — only transforms run
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mut pipeline = Pipeline::new().add_transform(Box::new(StandardScaler::new()));
        assert!(pipeline.fit(&x, 1, &y).is_ok());
    }

    #[test]
    fn test_fit_empty_pipeline() {
        // No transforms, no predictor — fit should still succeed
        let mut pipeline = Pipeline::new();
        assert!(pipeline.fit(&[1.0, 2.0], 1, &[3.0, 4.0]).is_ok());
    }

    #[test]
    fn test_fit_predictor_only() {
        // No transforms, just predictor
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mut pipeline = Pipeline::new().set_predictor(Box::new(LinearRegression::new()));
        pipeline.fit(&x, 1, &y).unwrap();
        let pred = pipeline.predict(&[3.0], 1).unwrap();
        assert!((pred[0] - 6.0).abs() < 1e-6);
    }

    #[test]
    fn test_fit_multi_feature() {
        // y = 1*x1 + 2*x2 + 3
        let x = vec![1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 2.0, 3.0];
        let y = vec![4.0, 5.0, 6.0, 11.0];
        let mut pipeline = Pipeline::new().set_predictor(Box::new(LinearRegression::new()));
        pipeline.fit(&x, 2, &y).unwrap();
        let pred = pipeline.predict(&[1.0, 1.0], 2).unwrap();
        assert!((pred[0] - 6.0).abs() < 0.1);
    }

    #[test]
    fn test_fit_chained_transforms() {
        // StandardScaler + MinMaxScaler + LinearRegression
        let x = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mut pipeline = Pipeline::new()
            .add_transform(Box::new(StandardScaler::new()))
            .add_transform(Box::new(MinMaxScaler::new()))
            .set_predictor(Box::new(LinearRegression::new()));
        pipeline.fit(&x, 1, &y).unwrap();
        let pred = pipeline.predict(&[30.0], 1).unwrap();
        // Should predict roughly 6.0
        assert!((pred[0] - 6.0).abs() < 1.5);
    }

    // ── predict() ──────────────────────────────────────────────────────

    #[test]
    fn test_predict_no_predictor_error() {
        let mut pipeline = Pipeline::new().add_transform(Box::new(StandardScaler::new()));
        pipeline.fit(&[1.0, 2.0, 3.0], 1, &[1.0, 2.0, 3.0]).unwrap();
        let err = pipeline.predict(&[2.0], 1);
        assert!(err.is_err());
        assert!(matches!(err.unwrap_err(), MlError::NotFitted));
    }

    #[test]
    fn test_predict_unfitted_predictor_error() {
        // Predictor set but never fitted
        let pipeline = Pipeline::new().set_predictor(Box::new(LinearRegression::new()));
        let err = pipeline.predict(&[1.0], 1);
        assert!(err.is_err());
    }

    #[test]
    fn test_predict_multiple_samples() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let mut pipeline = Pipeline::new().set_predictor(Box::new(LinearRegression::new()));
        pipeline.fit(&x, 1, &y).unwrap();
        let pred = pipeline.predict(&[1.0, 2.0, 3.0], 1).unwrap();
        assert_eq!(pred.len(), 3);
        assert!((pred[0] - 2.0).abs() < 0.5);
        assert!((pred[1] - 4.0).abs() < 0.5);
        assert!((pred[2] - 6.0).abs() < 0.5);
    }

    #[test]
    fn test_predict_empty_pipeline_no_predictor() {
        let pipeline = Pipeline::new();
        assert!(pipeline.predict(&[1.0], 1).is_err());
    }
}
