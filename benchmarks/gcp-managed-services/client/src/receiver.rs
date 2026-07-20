use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Deserialize)]
pub struct ReceiverStats {
    pub requests: usize,
    pub unique: usize,
    pub duplicates: usize,
}

#[derive(Clone)]
pub struct Receiver {
    client: Client,
    url: String,
    secret: String,
}

impl Receiver {
    pub fn new(client: Client, url: String, secret: String) -> Self {
        Self {
            client,
            url,
            secret,
        }
    }

    pub async fn warm(&self) -> Result<()> {
        // Cloud Run can report the service ready before the public invoker IAM
        // binding has propagated to every frontend. Treat early 404/403/5xx
        // responses like cold-start connection failures and retry within one
        // bounded window; the benchmark must not confuse control-plane
        // convergence with a Defer/Cloud Tasks domain failure.
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            let last_error = match self
                .client
                .get(format!("{}/healthz", self.url))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => return Ok(()),
                Ok(response) => format!("receiver health returned {}", response.status()),
                Err(error) => format!("warm Cloud Run receiver: {error}"),
            };
            if Instant::now() >= deadline {
                bail!(last_error);
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    pub async fn reset(&self, backend: &str) -> Result<()> {
        let response = self
            .client
            .post(format!("{}/reset", self.url))
            .query(&[("backend", backend)])
            .header("x-axiom-bench-secret", &self.secret)
            .send()
            .await
            .with_context(|| format!("reset receiver backend {backend}"))?;
        if !response.status().is_success() {
            bail!("receiver reset returned {}", response.status());
        }
        Ok(())
    }

    pub async fn stats(&self, backend: &str) -> Result<ReceiverStats> {
        self.client
            .get(format!("{}/stats", self.url))
            .query(&[("backend", backend)])
            .header("x-axiom-bench-secret", &self.secret)
            .send()
            .await
            .with_context(|| format!("read receiver backend {backend}"))?
            .error_for_status()?
            .json()
            .await
            .context("decode receiver stats")
    }

    pub async fn wait_unique(
        &self,
        backend: &str,
        expected: usize,
        timeout: Duration,
    ) -> Result<ReceiverStats> {
        let deadline = Instant::now() + timeout;
        loop {
            let stats = self.stats(backend).await?;
            if stats.unique >= expected {
                return Ok(stats);
            }
            if Instant::now() >= deadline {
                bail!(
                    "receiver backend {backend} stopped at {}/{} unique requests",
                    stats.unique,
                    expected
                );
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}
