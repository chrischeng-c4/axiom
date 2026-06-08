//! Error types for jieba operations.

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum JiebaError {
    #[error("Dictionary load error: {0}")]
    DictLoadError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("HMM model error: {0}")]
    HmmError(String),
}

pub type Result<T> = std::result::Result<T, JiebaError>;
