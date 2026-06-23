//! keep client (#14 / #167) — claim-check input/result over keep's HTTP/2 (h2c)
//! API. Implements the worker's [`KeepStore`]: `get_input` GETs
//! `/v1/inputs/{id}`, `put_result` PUTs `/v1/results/{id}`. Bytes-first
//! (octet-stream); keep is plaintext h2c so no TLS is linked.

use async_trait::async_trait;

use crate::worker::KeepStore;

/// keep claim-check store over h2c.
pub struct KeepHttp {
    client: reqwest::Client,
    base: String,
}

impl KeepHttp {
    /// Connect to a keep base URL, e.g. `http://keep:7379`.
    pub fn new(base: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().http2_prior_knowledge().build()?,
            base: base.into(),
        })
    }
}

#[async_trait]
impl KeepStore for KeepHttp {
    async fn get_input(&self, id: &str) -> anyhow::Result<Option<Vec<u8>>> {
        let resp = self
            .client
            .get(format!("{}/v1/inputs/{}", self.base, id))
            .send()
            .await?;
        if resp.status().as_u16() == 404 {
            return Ok(None);
        }
        anyhow::ensure!(resp.status().is_success(), "keep get_input: {}", resp.status());
        Ok(Some(resp.bytes().await?.to_vec()))
    }

    async fn put_result(&self, id: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        let resp = self
            .client
            .put(format!("{}/v1/results/{}", self.base, id))
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(bytes)
            .send()
            .await?;
        anyhow::ensure!(resp.status().is_success(), "keep put_result: {}", resp.status());
        Ok(())
    }
}
