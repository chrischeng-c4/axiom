//! HTTP error types and handling

use thiserror::Error;

/// HTTP-specific errors
#[derive(Error, Debug)]
pub enum HttpError {
    /// Connection failed (DNS, TCP, TLS)
    #[error("Connection error: {0}")]
    Connection(String),

    /// Request timeout
    #[error("Request timeout: {0}")]
    Timeout(String),

    /// Invalid request configuration
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Response parsing error
    #[error("Response error: {0}")]
    ResponseError(String),

    /// Too many redirects
    #[error("Redirect error: {0}")]
    Redirect(String),

    /// URL parsing error
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(String),

    /// HTTP status error (non-2xx response)
    #[error("HTTP status {code}: {message}")]
    Status { code: u16, message: String },

    /// Network/IO error during streaming
    #[error("Network error: {0}")]
    Network(String),

    /// Generic reqwest error
    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Result type for HTTP operations
pub type HttpResult<T> = Result<T, HttpError>;

/// Error category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpErrorCategory {
    /// Connection-related errors (DNS, TCP, TLS)
    Connection,
    /// Timeout errors
    Timeout,
    /// Invalid request construction
    Request,
    /// Response parsing or status errors
    Response,
    /// Redirect errors
    Redirect,
    /// Unknown/other errors
    Unknown,
}

impl HttpError {
    /// Categorize the error for reporting
    pub fn category(&self) -> HttpErrorCategory {
        match self {
            HttpError::Connection(_) | HttpError::Network(_) => HttpErrorCategory::Connection,
            HttpError::Timeout(_) => HttpErrorCategory::Timeout,
            HttpError::InvalidRequest(_) | HttpError::InvalidUrl(_) => HttpErrorCategory::Request,
            HttpError::ResponseError(_) | HttpError::Json(_) | HttpError::Status { .. } => {
                HttpErrorCategory::Response
            }
            HttpError::Redirect(_) => HttpErrorCategory::Redirect,
            HttpError::Reqwest(e) => {
                if e.is_connect() {
                    HttpErrorCategory::Connection
                } else if e.is_timeout() {
                    HttpErrorCategory::Timeout
                } else if e.is_redirect() {
                    HttpErrorCategory::Redirect
                } else if e.is_request() {
                    HttpErrorCategory::Request
                } else {
                    HttpErrorCategory::Unknown
                }
            }
            HttpError::UrlParse(_) => HttpErrorCategory::Request,
        }
    }

    /// Sanitize error message (remove sensitive info like credentials, internal IPs)
    pub fn sanitized_message(&self) -> String {
        let msg = self.to_string();
        sanitize_error_message(&msg)
    }
}

/// Sanitize error messages by removing sensitive information
pub fn sanitize_error_message(msg: &str) -> String {
    use regex::Regex;
    use std::sync::OnceLock;

    static URL_CREDS_RE: OnceLock<Regex> = OnceLock::new();
    static INTERNAL_IP_RE: OnceLock<Regex> = OnceLock::new();
    static AUTH_HEADER_RE: OnceLock<Regex> = OnceLock::new();

    let url_creds_re =
        URL_CREDS_RE.get_or_init(|| Regex::new(r"https?://[^@:]+:[^@]+@").expect("valid regex"));
    let internal_ip_re = INTERNAL_IP_RE.get_or_init(|| {
        Regex::new(r"\b(10\.|172\.(1[6-9]|2[0-9]|3[01])\.|192\.168\.)\d+\.\d+\b")
            .expect("valid regex")
    });
    let auth_header_re = AUTH_HEADER_RE.get_or_init(|| {
        Regex::new(r"(?i)(authorization:\s*bearer|bearer|api[_-]?key|token)\s*[:=]?\s*\S+")
            .expect("valid regex")
    });

    let sanitized = url_creds_re.replace_all(msg, "https://[REDACTED]@");
    let sanitized = internal_ip_re.replace_all(&sanitized, "[INTERNAL_IP]");
    let sanitized = auth_header_re.replace_all(&sanitized, "$1: [REDACTED]");

    sanitized.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_credentials_in_url() {
        let msg = "Connection failed to https://user:password@api.example.com/path";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("password"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_internal_ip() {
        let msg = "Cannot connect to 192.168.1.100:8080";
        let sanitized = sanitize_error_message(msg);
        assert!(sanitized.contains("[INTERNAL_IP]"));
        assert!(!sanitized.contains("192.168.1.100"));
    }

    #[test]
    fn test_sanitize_auth_header() {
        let msg = "Request failed with Authorization: Bearer secret-token-123";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("secret-token-123"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let msg = "Bearer abc123xyz";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("abc123xyz"));
        assert!(sanitized.contains("[REDACTED]"));
    }

    #[test]
    fn test_sanitize_token_param() {
        let msg = "Request with token=secret123";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("secret123"));
        assert!(sanitized.contains("[REDACTED]"));
    }
}
