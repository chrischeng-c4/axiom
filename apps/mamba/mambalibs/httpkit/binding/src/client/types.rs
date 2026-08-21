//! Opaque types for the `mambalibs.http` FFI layer.

use mambalibs_http::client::{HttpClient, HttpClientConfig};

/// A Mamba-visible HTTP client handle.
///
/// Stores the pooled core `mambalibs_http::client::HttpClient` plus the small amount of
/// configuration that existing binding tests inspect directly.
#[derive(Clone)]
pub struct MbHttpClient {
    /// Base URL prepended to relative paths (e.g. `"https://api.example.com"`).
    pub base_url: String,
    /// Optional headers as key-value pairs.
    pub headers: Vec<(String, String)>,
    /// Request timeout in seconds.
    pub timeout_secs: f64,
    /// Shared core client. HTTP execution must go through this object so the
    /// binding inherits pooling, redirects, compression, and error handling.
    pub client: HttpClient,
}

impl MbHttpClient {
    pub fn new(base_url: impl Into<String>, timeout_secs: f64) -> Self {
        let base_url = base_url.into();
        let mut config = HttpClientConfig::new().timeout_secs(timeout_secs);
        if !base_url.is_empty() {
            config = config.base_url(base_url.clone());
        }
        let client = HttpClient::new(config).expect("mambalibs.http client config must be valid");
        Self {
            base_url,
            headers: Vec::new(),
            timeout_secs,
            client,
        }
    }
}

impl std::fmt::Debug for MbHttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MbHttpClient")
            .field("base_url", &self.base_url)
            .field("headers", &self.headers)
            .field("timeout_secs", &self.timeout_secs)
            .finish()
    }
}

/// A Mamba-visible HTTP response.
#[derive(Debug, Clone)]
pub struct MbHttpResponse {
    /// HTTP status code (0 on error).
    pub status: u16,
    /// Raw response body as text.
    pub body: String,
}

impl MbHttpResponse {
    pub fn ok(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }

    pub fn error() -> Self {
        Self {
            status: 0,
            body: "error".to_string(),
        }
    }
}
