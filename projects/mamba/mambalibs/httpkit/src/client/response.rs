//! Client-side response adapter.
//!
//! The user-facing response type lives in `mambalibs_http::http::Response`; this
//! module supplies the reqwest → `Response` conversion plus a small builder
//! used by tests.

pub use crate::http::Response;

use super::error::HttpResult;
use std::collections::HashMap;

/// Back-compat alias. `mambalibs_http::http::Response` is the canonical name; older
/// imports of `mambalibs_http::client::HttpResponse` keep working.
pub type HttpResponse = Response;

/// Convert a reqwest `Response` into the unified `Response`, attaching the
/// client-measured latency.
pub async fn from_reqwest(response: reqwest::Response, latency_ms: u64) -> HttpResult<Response> {
    let status_code = response.status().as_u16();
    let final_url = response.url().to_string();
    let version = format!("{:?}", response.version());

    let mut headers = HashMap::new();
    for (name, value) in response.headers().iter() {
        if let Ok(v) = value.to_str() {
            headers.insert(name.to_string(), v.to_string());
        }
    }

    let body = response.bytes().await?.to_vec();

    Ok(Response {
        status_code,
        headers,
        body,
        cookies: Vec::new(),
        media_type: String::new(),
        latency_ms,
        final_url,
        version,
    })
}

/// Builder for assembling a `Response` in tests. Production code constructs
/// responses via `from_reqwest` (client) or server-side handlers.
#[derive(Debug, Default)]
pub struct HttpResponseBuilder {
    status_code: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    latency_ms: u64,
    final_url: String,
    version: String,
}

impl HttpResponseBuilder {
    pub fn new() -> Self {
        Self {
            status_code: 200,
            version: "HTTP/1.1".to_string(),
            ..Default::default()
        }
    }

    pub fn status_code(mut self, code: u16) -> Self {
        self.status_code = code;
        self
    }

    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    pub fn body(mut self, body: Vec<u8>) -> Self {
        self.body = body;
        self
    }

    pub fn latency_ms(mut self, ms: u64) -> Self {
        self.latency_ms = ms;
        self
    }

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.final_url = url.into();
        self
    }

    pub fn build(self) -> Response {
        Response {
            status_code: self.status_code,
            headers: self.headers,
            body: self.body,
            cookies: Vec::new(),
            media_type: String::new(),
            latency_ms: self.latency_ms,
            final_url: self.final_url,
            version: self.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_status_checks() {
        let response = HttpResponseBuilder::new().status_code(200).build();
        assert!(response.is_success());
        assert!(!response.is_client_error());

        let response = HttpResponseBuilder::new().status_code(404).build();
        assert!(!response.is_success());
        assert!(response.is_client_error());

        let response = HttpResponseBuilder::new().status_code(500).build();
        assert!(response.is_server_error());
    }

    #[test]
    fn test_response_json() {
        let response = HttpResponseBuilder::new()
            .body(br#"{"name": "Alice", "age": 30}"#.to_vec())
            .build();

        let json = response.json().unwrap();
        assert_eq!(json["name"], "Alice");
        assert_eq!(json["age"], 30);
    }

    #[test]
    fn test_response_header_case_insensitive() {
        let response = HttpResponseBuilder::new()
            .header("Content-Type", "application/json")
            .build();

        assert_eq!(response.header("content-type"), Some("application/json"));
        assert_eq!(response.header("CONTENT-TYPE"), Some("application/json"));
    }

    #[test]
    fn test_response_is_json() {
        let response = HttpResponseBuilder::new()
            .header("Content-Type", "application/json; charset=utf-8")
            .build();
        assert!(response.is_json());
    }
}
