//! Visualization error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VizError {
    #[error("empty data: {0}")]
    EmptyData(String),

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("io error: {0}")]
    IoError(String),
}

pub type Result<T> = std::result::Result<T, VizError>;
