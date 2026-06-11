//! Unified error handling for Argus
//!
//! This module provides a comprehensive error type that covers all error cases
//! in the Argus static analysis tool.

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for Argus operations
pub type Result<T> = std::result::Result<T, ArgusError>;

/// Unified error type for all Argus operations
#[derive(Error, Debug)]
pub enum ArgusError {
    /// Parser initialization or operation failed
    #[error("Parser error: {0}")]
    Parser(String),

    /// AST cache operation failed
    #[error("AST cache error: file not found in cache: {0}")]
    AstCacheNotFound(PathBuf),

    /// File I/O error
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid identifier or name
    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(String),

    /// Definition not found during refactoring
    #[error("Definition not found: {0}")]
    DefinitionNotFound(String),

    /// Type system error
    #[error("Type error: {0}")]
    Type(String),

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    /// Tree-sitter language error
    #[error("Tree-sitter language error: {0}")]
    TreeSitterLanguage(#[from] tree_sitter::LanguageError),

    /// Generic error with message
    #[error("{0}")]
    Other(String),
}

impl ArgusError {
    /// Create a parser error
    pub fn parser(msg: impl Into<String>) -> Self {
        Self::Parser(msg.into())
    }

    /// Create an AST cache not found error
    pub fn ast_cache_not_found(path: PathBuf) -> Self {
        Self::AstCacheNotFound(path)
    }

    /// Create an invalid identifier error
    pub fn invalid_identifier(id: impl Into<String>) -> Self {
        Self::InvalidIdentifier(id.into())
    }

    /// Create a definition not found error
    pub fn definition_not_found(name: impl Into<String>) -> Self {
        Self::DefinitionNotFound(name.into())
    }

    /// Create a type error
    pub fn type_error(msg: impl Into<String>) -> Self {
        Self::Type(msg.into())
    }

    /// Create a generic error
    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ArgusError::parser("Failed to initialize");
        assert_eq!(err.to_string(), "Parser error: Failed to initialize");

        let err = ArgusError::ast_cache_not_found(PathBuf::from("test.py"));
        assert!(err.to_string().contains("test.py"));

        let err = ArgusError::invalid_identifier("123invalid");
        assert_eq!(err.to_string(), "Invalid identifier: 123invalid");
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ArgusError = io_err.into();
        assert!(matches!(err, ArgusError::Io(_)));
    }
}
