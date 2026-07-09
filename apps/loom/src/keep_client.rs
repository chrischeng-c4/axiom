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
        // A claim-check ref resolves whether it names a producer-supplied input
        // or an upstream node's result: try /v1/inputs, then fall back to
        // /v1/results. This is what makes inter-node data flow work (a downstream
        // node's input_ref is an upstream node's result_ref).
        for ns in ["inputs", "results"] {
            let resp =
                self.client.get(format!("{}/v1/{}/{}", self.base, ns, id)).send().await?;
            if resp.status().as_u16() == 404 {
                continue;
            }
            anyhow::ensure!(resp.status().is_success(), "keep get {ns}: {}", resp.status());
            return Ok(Some(resp.bytes().await?.to_vec()));
        }
        Ok(None)
    }

    async fn put_input(&self, id: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
        let resp = self
            .client
            .put(format!("{}/v1/inputs/{}", self.base, id))
            .header(reqwest::header::CONTENT_TYPE, "application/octet-stream")
            .body(bytes)
            .send()
            .await?;
        anyhow::ensure!(resp.status().is_success(), "keep put_input: {}", resp.status());
        Ok(())
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
