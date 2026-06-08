//! HTTP client with connection pooling and async operations.
//!
//! Accepts the unified `mambalibs_http::http::Request` and returns
//! `mambalibs_http::http::Response`. Internally, requests are converted to
//! `ExtractedRequest` for the middleware/reqwest path.

use super::config::HttpClientConfig;
use super::error::{HttpError, HttpResult};
use super::request::ExtractedRequest;
use super::response::from_reqwest;
use crate::http::{HttpMethod, Request, Response};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Instant;

/// High-performance async HTTP client with connection pooling.
#[derive(Clone)]
pub struct HttpClient {
    inner: Arc<HttpClientInner>,
}

struct HttpClientInner {
    client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpClient {
    pub fn new(config: HttpClientConfig) -> HttpResult<Self> {
        let mut builder = reqwest::Client::builder()
            .timeout(config.timeout)
            .connect_timeout(config.connect_timeout)
            .pool_max_idle_per_host(config.pool_max_idle_per_host)
            .pool_idle_timeout(config.pool_idle_timeout)
            .user_agent(&config.user_agent);

        if config.follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::limited(config.max_redirects));
        } else {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        builder = builder.gzip(config.gzip).brotli(config.brotli);

        if config.danger_accept_invalid_certs {
            builder = builder.danger_accept_invalid_certs(true);
        }
        if config.danger_accept_invalid_hostnames {
            builder = builder.danger_accept_invalid_hostnames(true);
        }

        let client = builder.build()?;

        Ok(Self {
            inner: Arc::new(HttpClientInner { client, config }),
        })
    }

    pub fn default_client() -> HttpResult<Self> {
        Self::new(HttpClientConfig::default())
    }

    pub fn base_url(&self) -> Option<&str> {
        self.inner.config.base_url.as_deref()
    }

    /// Execute a pre-extracted request. Internal entry point used by
    /// middleware-aware paths.
    pub async fn execute(&self, request: ExtractedRequest) -> HttpResult<Response> {
        let start = Instant::now();

        let reqwest_builder =
            request.build_reqwest(&self.inner.client, self.inner.config.base_url.as_deref())?;

        let response = reqwest_builder.send().await?;
        let latency_ms = start.elapsed().as_millis() as u64;

        from_reqwest(response, latency_ms).await
    }

    /// Execute the unified request shape. Preferred entry point.
    pub async fn send(&self, request: Request) -> HttpResult<Response> {
        self.execute(request.into()).await
    }

    /// Back-compat alias for `send` — older callers used this name when the
    /// builder was a separate type.
    pub async fn execute_builder(&self, request: Request) -> HttpResult<Response> {
        self.send(request).await
    }

    /// Execute a streaming request and return a stream of bytes (SSE,
    /// chunked transfer, etc.).
    pub async fn execute_stream(
        &self,
        request: ExtractedRequest,
    ) -> HttpResult<Pin<Box<dyn Stream<Item = Result<Bytes, HttpError>> + Send>>> {
        let reqwest_builder =
            request.build_reqwest(&self.inner.client, self.inner.config.base_url.as_deref())?;

        let response = reqwest_builder.send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(HttpError::Status {
                code: status,
                message: error_text,
            });
        }

        let stream = response.bytes_stream();
        Ok(Box::pin(futures::StreamExt::map(
            stream,
            |result: Result<Bytes, reqwest::Error>| {
                result.map_err(|e: reqwest::Error| HttpError::Network(e.to_string()))
            },
        )))
    }

    /// Streaming variant taking the unified request shape.
    pub async fn send_stream(
        &self,
        request: Request,
    ) -> HttpResult<Pin<Box<dyn Stream<Item = Result<Bytes, HttpError>> + Send>>> {
        self.execute_stream(request.into()).await
    }

    /// Back-compat alias for `send_stream`.
    pub async fn execute_builder_stream(
        &self,
        request: Request,
    ) -> HttpResult<Pin<Box<dyn Stream<Item = Result<Bytes, HttpError>> + Send>>> {
        self.send_stream(request).await
    }

    // --- Convenience helpers ------------------------------------------------

    pub async fn get(&self, url: &str) -> HttpResult<Response> {
        self.send(Request::get(url)).await
    }

    pub async fn post(&self, url: &str, body: serde_json::Value) -> HttpResult<Response> {
        self.send(Request::post(url).json_value(body)).await
    }

    pub async fn put(&self, url: &str, body: serde_json::Value) -> HttpResult<Response> {
        self.send(Request::put(url).json_value(body)).await
    }

    pub async fn patch(&self, url: &str, body: serde_json::Value) -> HttpResult<Response> {
        self.send(Request::patch(url).json_value(body)).await
    }

    pub async fn delete(&self, url: &str) -> HttpResult<Response> {
        self.send(Request::delete(url)).await
    }

    pub async fn head(&self, url: &str) -> HttpResult<Response> {
        self.send(Request::head(url)).await
    }

    pub async fn options(&self, url: &str) -> HttpResult<Response> {
        self.send(Request::options(url)).await
    }

    pub fn request(&self, method: HttpMethod, url: &str) -> Request {
        Request::new(method, url)
    }
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient")
            .field("base_url", &self.inner.config.base_url)
            .field("timeout", &self.inner.config.timeout)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let config = HttpClientConfig::new()
            .base_url("https://api.example.com")
            .timeout_secs(30.0);

        let client = HttpClient::new(config).unwrap();
        assert_eq!(client.base_url(), Some("https://api.example.com"));
    }

    #[test]
    fn test_default_client() {
        let client = HttpClient::default_client().unwrap();
        assert!(client.base_url().is_none());
    }

    #[tokio::test]
    async fn test_request_builder() {
        let config = HttpClientConfig::new().base_url("https://httpbin.org");
        let client = HttpClient::new(config).unwrap();

        let builder = client
            .request(HttpMethod::Post, "/post")
            .header("X-Custom", "value")
            .query("foo", "bar")
            .json_value(serde_json::json!({"key": "value"}));

        assert_eq!(builder.method, HttpMethod::Post);
        assert_eq!(builder.url, "/post");
        assert!(builder.headers.contains_key("X-Custom"));
    }
}
