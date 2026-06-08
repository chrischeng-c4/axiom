//! Error types for pulsar-markup.

use thiserror::Error;

/// Markup processing errors.
#[derive(Debug, Error)]
pub enum MarkupError {
    /// Parse error with position info.
    #[error("parse error at line {line}, col {col}: {message}")]
    ParseError {
        line: usize,
        col: usize,
        message: String,
    },

    /// Invalid selector syntax.
    #[error("invalid selector: {0}")]
    InvalidSelector(String),

    /// Invalid XPath expression.
    #[error("invalid XPath: {0}")]
    InvalidXPath(String),

    /// Invalid XSLT stylesheet.
    #[error("invalid XSLT: {0}")]
    InvalidXslt(String),

    /// Node not found.
    #[error("node not found")]
    NodeNotFound,

    /// Attribute not found.
    #[error("attribute not found: {0}")]
    AttributeNotFound(String),

    /// Invalid operation.
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, MarkupError>;
