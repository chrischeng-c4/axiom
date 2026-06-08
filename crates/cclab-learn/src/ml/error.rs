//! ML module error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MlError {
    #[error("not fitted: call fit() before predict()")]
    NotFitted,

    #[error("shape mismatch: {0}")]
    ShapeMismatch(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("convergence failed after {0} iterations")]
    ConvergenceFailed(usize),

    #[error("array error: {0}")]
    ArrayError(#[from] arraykit::array::ArrayError),
}

pub type Result<T> = std::result::Result<T, MlError>;
