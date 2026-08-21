//! Middleware system for request/response processing
//!
//! This module provides a middleware trait and chain for extensible
//! request processing, enabling features like retries, logging, and authentication.
//!
//! # Architecture
//!
//! Middlewares form a chain where each middleware can:
//! - Modify the request before passing it to the next middleware
//! - Modify the response after receiving it from the next middleware
//! - Short-circuit the chain (e.g., returning a cached response)
//!
//! # Example
//!
//! ```ignore
//! use cclab_fetch::middleware::{Middleware, MiddlewareChain, LoggingMiddleware};
//!
//! let chain = MiddlewareChain::new()
//!     .with(LoggingMiddleware)
//!     .with(RetryMiddleware::default());
//! ```

use super::error::HttpError;
use super::request::ExtractedRequest;
use super::response::HttpResponse;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// ============================================================================
// Middleware Trait
// ============================================================================

/// Result type for middleware operations
pub type MiddlewareResult = Result<HttpResponse, HttpError>;

/// Future type for async middleware
pub type MiddlewareFuture = Pin<Box<dyn Future<Output = MiddlewareResult> + Send>>;

/// Trait for implementing HTTP middleware
///
/// Middlewares can intercept requests and responses to add cross-cutting
/// concerns like logging, retries, authentication, and caching.
pub trait Middleware: Send + Sync {
    /// Process a request and return a response
    ///
    /// # Arguments
    /// * `request` - The HTTP request to process
    /// * `next` - Function to call the next middleware in the chain
    ///
    /// # Returns
    /// The HTTP response (possibly modified)
    fn handle(&self, request: ExtractedRequest, next: Next) -> MiddlewareFuture;

    /// Get the middleware name for debugging
    fn name(&self) -> &'static str {
        "Middleware"
    }
}

/// Next function to call the remaining middleware chain
pub type Next = Arc<dyn Fn(ExtractedRequest) -> MiddlewareFuture + Send + Sync>;

// ============================================================================
// Middleware Chain
// ============================================================================

/// A chain of middlewares executed in order
#[derive(Clone, Default)]
pub struct MiddlewareChain {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareChain {
    /// Create a new empty middleware chain
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a middleware to the chain
    pub fn with<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Arc::new(middleware));
        self
    }

    /// Add a boxed middleware to the chain
    pub fn with_boxed(mut self, middleware: Arc<dyn Middleware>) -> Self {
        self.middlewares.push(middleware);
        self
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Get the number of middlewares in the chain
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }

    /// Get middleware names for debugging
    pub fn names(&self) -> Vec<&'static str> {
        self.middlewares.iter().map(|m| m.name()).collect()
    }
}

// ============================================================================
// Built-in Middlewares
// ============================================================================

/// Middleware that logs requests and responses
#[derive(Debug, Clone, Copy, Default)]
pub struct LoggingMiddleware;

impl Middleware for LoggingMiddleware {
    fn handle(&self, request: ExtractedRequest, next: Next) -> MiddlewareFuture {
        Box::pin(async move {
            let method = request.method.clone();
            let url = request.url.clone();

            // Log request
            tracing::info!(method = %method, url = %url, "HTTP Request");

            let start = std::time::Instant::now();
            let result = next(request).await;
            let elapsed = start.elapsed();

            // Log response
            match &result {
                Ok(response) => {
                    tracing::info!(
                        method = %method,
                        url = %url,
                        status = response.status_code,
                        elapsed_ms = elapsed.as_millis(),
                        "HTTP Response"
                    );
                }
                Err(e) => {
                    tracing::error!(
                        method = %method,
                        url = %url,
                        error = %e,
                        elapsed_ms = elapsed.as_millis(),
                        "HTTP Error"
                    );
                }
            }

            result
        })
    }

    fn name(&self) -> &'static str {
        "LoggingMiddleware"
    }
}

/// Configuration for retry middleware
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries (0 = no retries)
    pub max_retries: u32,
    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,
    /// Backoff multiplier for exponential backoff
    pub backoff_multiplier: f64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// HTTP status codes that should trigger a retry
    pub retry_status_codes: Vec<u16>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 10000,
            retry_status_codes: vec![429, 500, 502, 503, 504],
        }
    }
}

/// Middleware that automatically retries failed requests
#[derive(Debug, Clone)]
pub struct RetryMiddleware {
    config: RetryConfig,
}

impl RetryMiddleware {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    #[cfg(test)]
    fn should_retry(&self, status: u16) -> bool {
        self.config.retry_status_codes.contains(&status)
    }

    #[cfg(test)]
    fn calculate_backoff(&self, attempt: u32) -> u64 {
        let backoff = self.config.initial_backoff_ms as f64
            * self.config.backoff_multiplier.powi(attempt as i32);
        (backoff as u64).min(self.config.max_backoff_ms)
    }
}

impl Default for RetryMiddleware {
    fn default() -> Self {
        Self::new(RetryConfig::default())
    }
}

impl Middleware for RetryMiddleware {
    fn handle(&self, request: ExtractedRequest, next: Next) -> MiddlewareFuture {
        let config = self.config.clone();
        let next = next.clone();

        Box::pin(async move {
            let mut attempt = 0;
            let max_retries = config.max_retries;

            loop {
                // Clone request for retry
                let req = request.clone();
                let result = next(req).await;

                match &result {
                    Ok(response) => {
                        let should_retry =
                            config.retry_status_codes.contains(&response.status_code);

                        if should_retry && attempt < max_retries {
                            attempt += 1;
                            let backoff = config.initial_backoff_ms as f64
                                * config.backoff_multiplier.powi(attempt as i32);
                            let backoff = (backoff as u64).min(config.max_backoff_ms);

                            tracing::warn!(
                                status = response.status_code,
                                attempt = attempt,
                                backoff_ms = backoff,
                                "Retrying request"
                            );

                            tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                            continue;
                        }
                        return result;
                    }
                    Err(_) if attempt < max_retries => {
                        attempt += 1;
                        let backoff = config.initial_backoff_ms as f64
                            * config.backoff_multiplier.powi(attempt as i32);
                        let backoff = (backoff as u64).min(config.max_backoff_ms);

                        tracing::warn!(
                            attempt = attempt,
                            backoff_ms = backoff,
                            "Retrying request after error"
                        );

                        tokio::time::sleep(std::time::Duration::from_millis(backoff)).await;
                        continue;
                    }
                    Err(_) => return result,
                }
            }
        })
    }

    fn name(&self) -> &'static str {
        "RetryMiddleware"
    }
}

/// Middleware that adds authentication headers
#[derive(Debug, Clone)]
pub struct AuthMiddleware {
    /// Authorization header value
    header_value: String,
}

impl AuthMiddleware {
    /// Create a new bearer token auth middleware
    pub fn bearer(token: impl Into<String>) -> Self {
        Self {
            header_value: format!("Bearer {}", token.into()),
        }
    }

    /// Create a new basic auth middleware
    pub fn basic(username: impl AsRef<str>, password: impl AsRef<str>) -> Self {
        use base64::Engine;
        let credentials = format!("{}:{}", username.as_ref(), password.as_ref());
        let encoded = base64::engine::general_purpose::STANDARD.encode(credentials);
        Self {
            header_value: format!("Basic {}", encoded),
        }
    }

    /// Create a custom auth middleware
    pub fn custom(header_value: impl Into<String>) -> Self {
        Self {
            header_value: header_value.into(),
        }
    }
}

impl Middleware for AuthMiddleware {
    fn handle(&self, mut request: ExtractedRequest, next: Next) -> MiddlewareFuture {
        let header_value = self.header_value.clone();

        Box::pin(async move {
            request
                .headers
                .push(("authorization".to_string(), header_value));
            next(request).await
        })
    }

    fn name(&self) -> &'static str {
        "AuthMiddleware"
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_middleware_chain() {
        let chain = MiddlewareChain::new()
            .with(LoggingMiddleware)
            .with(RetryMiddleware::default());

        assert_eq!(chain.len(), 2);
        assert!(!chain.is_empty());

        let names = chain.names();
        assert_eq!(names, vec!["LoggingMiddleware", "RetryMiddleware"]);
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_backoff_ms, 100);

        let middleware = RetryMiddleware::new(config.clone());
        assert!(middleware.should_retry(429));
        assert!(middleware.should_retry(503));
        assert!(!middleware.should_retry(200));
        assert!(!middleware.should_retry(404));
    }

    #[test]
    fn test_retry_backoff() {
        let middleware = RetryMiddleware::default();

        assert_eq!(middleware.calculate_backoff(0), 100);
        assert_eq!(middleware.calculate_backoff(1), 200);
        assert_eq!(middleware.calculate_backoff(2), 400);
        assert_eq!(middleware.calculate_backoff(10), 10000); // Capped at max
    }

    #[test]
    fn test_auth_middleware_bearer() {
        let auth = AuthMiddleware::bearer("my-token");
        assert_eq!(auth.header_value, "Bearer my-token");
    }

    #[test]
    fn test_auth_middleware_basic() {
        let auth = AuthMiddleware::basic("user", "pass");
        assert!(auth.header_value.starts_with("Basic "));
    }
}
