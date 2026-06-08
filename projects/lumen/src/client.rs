//! Typed HTTP client for the lumen public API.
//!
//! Mirrors the endpoints exposed by `crate::api::router` and reuses the
//! exact wire types from `crate::types` so consumers never see a parallel
//! schema. The client is shard-aware via Layer 1 routing
//! ([`crate::routing::shard_index`]) — set [`Client::with_shard_routing`]
//! to drive a base URL that contains the `{shard}` placeholder.
//!
//! Example:
//!
//! ```no_run
//! # async fn run() -> Result<(), lumen::client::ClientError> {
//! use lumen::client::Client;
//! use lumen::types::*;
//! use std::collections::BTreeMap;
//!
//! let client = Client::new("http://lumen-svc:8080")
//!     .with_bearer("token-abc");
//!
//! let mut fields = BTreeMap::new();
//! fields.insert(
//!     "email".to_string(),
//!     FieldSpec {
//!         field_type: FieldType::Keyword,
//!         analyzer: None,
//!         multi: None,
//!         dim: None,
//!         metric: None,
//!         backend: None,
//!         quantize: None,
//!     },
//! );
//! client.create_collection("users", fields).await?;
//! # Ok(()) }
//! ```

use std::collections::BTreeMap;

use reqwest::{header, StatusCode};
use thiserror::Error;

use crate::routing::shard_index;
use crate::types::{
    CreateCollectionRequest, CreateCollectionResponse, DuplicatesRequest, DuplicatesResponse,
    FieldSpec, IndexItem, IndexRequest, IndexResponse, SearchRequest, SearchResponse,
    StatsResponse,
};

/// Errors produced by [`Client`].
#[derive(Debug, Error)]
pub enum ClientError {
    #[error("transport: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("server returned status {0}: {1}")]
    Server(u16, String),
    #[error("invalid base url: {0}")]
    BadBaseUrl(String),
}

/// Typed HTTP client over the lumen public API.
///
/// Construction is cheap; share a single instance across tasks — the
/// underlying [`reqwest::Client`] holds a connection pool.
#[derive(Debug, Clone)]
pub struct Client {
    base_url: String,
    http: reqwest::Client,
    bearer: Option<String>,
    shard_count: Option<u32>,
}

impl Client {
    /// Build a client against a single base URL (no shard fan-out).
    ///
    /// Use [`Self::with_shard_routing`] if the base URL contains a
    /// `{shard}` placeholder.
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
            bearer: None,
            shard_count: None,
        }
    }

    /// Attach a bearer token sent on every request.
    pub fn with_bearer(mut self, token: impl Into<String>) -> Self {
        self.bearer = Some(token.into());
        self
    }

    /// Enable client-side shard routing. When set, every method
    /// substitutes `{shard}` in the base URL with
    /// `shard_index(collection_id, shard_count)`.
    pub fn with_shard_routing(mut self, shard_count: u32) -> Self {
        self.shard_count = Some(shard_count);
        self
    }

    // -------------------------------------------------------------------
    // URL building
    // -------------------------------------------------------------------

    /// Resolve the per-collection base URL, applying shard routing if
    /// configured. Trailing slashes on `base_url` are stripped.
    fn collection_base(&self, collection_id: &str) -> Result<String, ClientError> {
        let resolved = match self.shard_count {
            Some(count) => {
                if count == 0 {
                    return Err(ClientError::BadBaseUrl(
                        "shard_count must be > 0".to_string(),
                    ));
                }
                if !self.base_url.contains("{shard}") {
                    return Err(ClientError::BadBaseUrl(format!(
                        "shard routing enabled but base URL `{}` has no {{shard}} placeholder",
                        self.base_url
                    )));
                }
                let shard = shard_index(collection_id, count);
                self.base_url.replace("{shard}", &shard.to_string())
            }
            None => {
                if self.base_url.contains("{shard}") {
                    return Err(ClientError::BadBaseUrl(format!(
                        "base URL `{}` contains {{shard}} but shard routing is disabled",
                        self.base_url
                    )));
                }
                self.base_url.clone()
            }
        };
        let trimmed = resolved.trim_end_matches('/').to_string();
        if trimmed.is_empty() {
            return Err(ClientError::BadBaseUrl("base URL is empty".to_string()));
        }
        Ok(trimmed)
    }

    /// Apply bearer token to an outgoing request builder, if present.
    fn with_auth(&self, rb: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.bearer {
            Some(t) => rb.header(header::AUTHORIZATION, format!("Bearer {t}")),
            None => rb,
        }
    }

    // -------------------------------------------------------------------
    // Endpoints
    // -------------------------------------------------------------------

    /// `PUT /collections/{id}` — create or upsert-extend a collection schema.
    pub async fn create_collection(
        &self,
        collection_id: &str,
        fields: BTreeMap<String, FieldSpec>,
    ) -> Result<CreateCollectionResponse, ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}");
        let req = CreateCollectionRequest { fields };
        let resp = self
            .with_auth(self.http.put(&url).json(&req))
            .send()
            .await?;
        decode_json(resp).await
    }

    /// `DELETE /collections/{id}` — drop a collection.
    pub async fn drop_collection(&self, collection_id: &str) -> Result<(), ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}");
        let resp = self.with_auth(self.http.delete(&url)).send().await?;
        ensure_success(resp).await.map(|_| ())
    }

    /// `GET /collections` — list collection IDs visible to this caller.
    pub async fn list_collections(&self) -> Result<Vec<String>, ClientError> {
        // `list_collections` is unsharded — we still need a concrete base URL.
        if self.shard_count.is_some() {
            return Err(ClientError::BadBaseUrl(
                "list_collections is not shard-routable; build a Client without with_shard_routing"
                    .to_string(),
            ));
        }
        let base = self.base_url.trim_end_matches('/').to_string();
        if base.is_empty() {
            return Err(ClientError::BadBaseUrl("base URL is empty".to_string()));
        }
        let url = format!("{base}/collections");
        let resp = self.with_auth(self.http.get(&url)).send().await?;
        decode_json(resp).await
    }

    /// `POST /collections/{id}/index` — bulk index items.
    pub async fn index(
        &self,
        collection_id: &str,
        items: Vec<IndexItem>,
        request_id: Option<String>,
    ) -> Result<IndexResponse, ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}/index");
        let req = IndexRequest { items, request_id };
        let resp = self
            .with_auth(self.http.post(&url).json(&req))
            .send()
            .await?;
        decode_json(resp).await
    }

    /// `DELETE /collections/{id}/index/{external_id}` — drop one
    /// external_id (optionally restricted to a single field).
    pub async fn delete(
        &self,
        collection_id: &str,
        external_id: &str,
        field: Option<&str>,
    ) -> Result<(), ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}/index/{external_id}");
        let mut rb = self.http.delete(&url);
        if let Some(f) = field {
            rb = rb.query(&[("field", f)]);
        }
        let resp = self.with_auth(rb).send().await?;
        ensure_success(resp).await.map(|_| ())
    }

    /// `POST /collections/{id}/search`.
    pub async fn search(
        &self,
        collection_id: &str,
        request: SearchRequest,
    ) -> Result<SearchResponse, ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}/search");
        let resp = self
            .with_auth(self.http.post(&url).json(&request))
            .send()
            .await?;
        decode_json(resp).await
    }

    /// `POST /collections/{id}/duplicates`.
    pub async fn duplicates(
        &self,
        collection_id: &str,
        request: DuplicatesRequest,
    ) -> Result<DuplicatesResponse, ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}/duplicates");
        let resp = self
            .with_auth(self.http.post(&url).json(&request))
            .send()
            .await?;
        decode_json(resp).await
    }

    /// `GET /collections/{id}/stats`.
    pub async fn stats(&self, collection_id: &str) -> Result<StatsResponse, ClientError> {
        let base = self.collection_base(collection_id)?;
        let url = format!("{base}/collections/{collection_id}/stats");
        let resp = self.with_auth(self.http.get(&url)).send().await?;
        decode_json(resp).await
    }
}

// -----------------------------------------------------------------------
// Response helpers
// -----------------------------------------------------------------------

async fn ensure_success(resp: reqwest::Response) -> Result<reqwest::Response, ClientError> {
    let status = resp.status();
    if status.is_success() || status == StatusCode::NO_CONTENT {
        return Ok(resp);
    }
    let code = status.as_u16();
    let body = resp.text().await.unwrap_or_default();
    Err(ClientError::Server(code, body))
}

async fn decode_json<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
) -> Result<T, ClientError> {
    let resp = ensure_success(resp).await?;
    Ok(resp.json::<T>().await?)
}

// -----------------------------------------------------------------------
// Tests — URL shaping only. Wire-format coverage lives in
// `tests/client_e2e.rs` against a real axum TestServer.
// -----------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shard_routing_substitutes_placeholder() {
        let c = Client::new("http://lumen-{shard}.lumen-peer:8080").with_shard_routing(3);
        // Pick a collection_id and verify both the resolved URL and the
        // underlying shard math agree.
        let cid = "users";
        let url = c.collection_base(cid).unwrap();
        let expected_shard = shard_index(cid, 3);
        assert_eq!(
            url,
            format!("http://lumen-{expected_shard}.lumen-peer:8080")
        );
    }

    #[test]
    fn no_shard_routing_keeps_base_url_as_is() {
        let c = Client::new("http://lumen-svc:8080/");
        // Trailing slash is normalised away.
        assert_eq!(c.collection_base("users").unwrap(), "http://lumen-svc:8080");
    }

    #[test]
    fn shard_routing_rejects_base_url_without_placeholder() {
        let c = Client::new("http://lumen-svc:8080").with_shard_routing(3);
        let err = c.collection_base("users").unwrap_err();
        assert!(
            matches!(err, ClientError::BadBaseUrl(_)),
            "expected BadBaseUrl, got {err:?}"
        );
    }

    #[test]
    fn placeholder_without_routing_is_rejected() {
        // {shard} left in the base URL without enabling shard routing is
        // almost always a config bug — fail loudly instead of sending a
        // literal `{shard}` to DNS.
        let c = Client::new("http://lumen-{shard}.lumen-peer:8080");
        let err = c.collection_base("users").unwrap_err();
        assert!(matches!(err, ClientError::BadBaseUrl(_)));
    }

    #[test]
    fn shard_routing_zero_count_is_rejected() {
        let c = Client::new("http://lumen-{shard}.lumen-peer:8080").with_shard_routing(0);
        let err = c.collection_base("users").unwrap_err();
        assert!(matches!(err, ClientError::BadBaseUrl(_)));
    }

    #[test]
    fn shard_routing_is_deterministic_per_collection() {
        let c = Client::new("http://lumen-{shard}.lumen-peer:8080").with_shard_routing(5);
        let a = c.collection_base("data-table:42").unwrap();
        let b = c.collection_base("data-table:42").unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn bearer_token_is_stored() {
        let c = Client::new("http://lumen-svc:8080").with_bearer("secret-abc");
        assert_eq!(c.bearer.as_deref(), Some("secret-abc"));
    }
}
