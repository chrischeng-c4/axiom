//! Thin HTTP/2 client for in-tree Rust consumers.
//!
//! Polyglot workers integrate against the OpenAPI document; this convenience
//! client is for Rust callers (e.g. queuekit's ion backend) migrating off the
//! retired raw-TCP `cclab-kv` client. Behind the `client` feature.
//!
//! It mirrors the small surface those consumers used: `connect`, `get`, `set`,
//! `delete`. Byte values use the efficient octet-stream path; structured values
//! use JSON.

use std::time::Duration;

use crate::http::models::json_to_kv;
use crate::types::KvValue;

/// Errors from the HTTP KV client.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("transport: {0}")]
    Transport(String),
    #[error("server status {0}")]
    Status(u16),
    #[error("decode: {0}")]
    Decode(String),
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        ClientError::Transport(e.to_string())
    }
}

/// HTTP/2 client for a single keep endpoint.
pub struct KvClient {
    base: String,
    http: reqwest::Client,
}

impl KvClient {
    /// Connect to a keep base URL (e.g. `http://keep:7117`) and verify liveness.
    pub async fn connect(url: &str) -> Result<Self, ClientError> {
        let http = h2c::h2c_client()?;
        let base = url.trim_end_matches('/').to_string();
        // Fail fast if the endpoint isn't reachable.
        let r = http.get(format!("{base}/healthz")).send().await?;
        if !r.status().is_success() {
            return Err(ClientError::Status(r.status().as_u16()));
        }
        Ok(Self { base, http })
    }

    /// GET a key. Byte blobs come back as `KvValue::Bytes`; structured values as
    /// the mapped `KvValue`. `None` if the key is absent.
    pub async fn get(&self, key: &str) -> Result<Option<KvValue>, ClientError> {
        let r = self
            .http
            .get(format!("{}/v1/kv/{}", self.base, key))
            .send()
            .await?;
        if r.status().as_u16() == 404 {
            return Ok(None);
        }
        if !r.status().is_success() {
            return Err(ClientError::Status(r.status().as_u16()));
        }
        let is_octet = r
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|ct| ct.starts_with("application/octet-stream"))
            .unwrap_or(false);
        if is_octet {
            let bytes = r.bytes().await?;
            return Ok(Some(KvValue::Bytes(bytes.to_vec())));
        }
        let body: serde_json::Value = r
            .json()
            .await
            .map_err(|e| ClientError::Decode(e.to_string()))?;
        let value = body
            .get("value")
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        Ok(Some(json_to_kv(value)))
    }

    /// PUT a key. Byte blobs use octet-stream (TTL via query); other values use
    /// the JSON path.
    pub async fn set(
        &self,
        key: &str,
        value: KvValue,
        ttl: Option<Duration>,
    ) -> Result<(), ClientError> {
        let url = format!("{}/v1/kv/{}", self.base, key);
        let req = match value {
            KvValue::Bytes(b) => {
                let mut rb = self
                    .http
                    .put(&url)
                    .header(reqwest::header::CONTENT_TYPE, "application/octet-stream");
                if let Some(t) = ttl {
                    rb = rb.query(&[("ttl_ms", t.as_millis() as u64)]);
                }
                rb.body(b)
            }
            other => self.http.put(&url).json(&serde_json::json!({
                "value": kv_to_json_value(other),
                "ttl_ms": ttl.map(|t| t.as_millis() as u64),
            })),
        };
        let r = req.send().await?;
        if !r.status().is_success() {
            return Err(ClientError::Status(r.status().as_u16()));
        }
        Ok(())
    }

    /// Fetch several keys (parallel to `keys`; `None` where absent). Done as
    /// individual GETs so byte blobs round-trip as `KvValue::Bytes` (keep's JSON
    /// `:mget` would array-encode bytes); fine for the modest batches consumers use.
    pub async fn mget(&self, keys: &[&str]) -> Result<Vec<Option<KvValue>>, ClientError> {
        let mut out = Vec::with_capacity(keys.len());
        for k in keys {
            out.push(self.get(k).await?);
        }
        Ok(out)
    }

    /// Liveness check against `/healthz`.
    pub async fn ping(&self) -> Result<(), ClientError> {
        let r = self
            .http
            .get(format!("{}/healthz", self.base))
            .send()
            .await?;
        if r.status().is_success() {
            Ok(())
        } else {
            Err(ClientError::Status(r.status().as_u16()))
        }
    }

    /// DELETE a key.
    pub async fn delete(&self, key: &str) -> Result<(), ClientError> {
        let r = self
            .http
            .delete(format!("{}/v1/kv/{}", self.base, key))
            .send()
            .await?;
        if !r.status().is_success() {
            return Err(ClientError::Status(r.status().as_u16()));
        }
        Ok(())
    }
}

/// Map a non-byte KvValue to its native JSON form for the request body.
fn kv_to_json_value(v: KvValue) -> serde_json::Value {
    crate::http::models::kv_to_json(v)
}
