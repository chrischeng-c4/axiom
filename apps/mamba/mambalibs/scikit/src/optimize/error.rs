//! Error types for the optimize module.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum OptimizeError {
    #[error("convergence failed after {0} iterations")]
    ConvergenceFailed(usize),

    #[error("invalid bounds: {0}")]
    InvalidBounds(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}

pub type Result<T> = std::result::Result<T, OptimizeError>;
