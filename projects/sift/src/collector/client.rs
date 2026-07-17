// HANDWRITE-BEGIN gap="missing-generator:logic:d8653acc" tracker="pending-tracker" reason="POST canonical batches with x-sift-project and optional bearer, classify retryable failures, reject partial terminal outcomes, and count accepted versus duplicate."
use std::time::Duration;

use anyhow::{bail, Context, Result};
use reqwest::{StatusCode, Url};

use crate::ingest::{BatchOutcome, EventWriteRequest, EventWriteResponse};
use crate::OperationalEventV2;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DeliveryCounts {
    pub accepted: u64,
    pub duplicates: u64,
}

#[derive(Clone)]
pub struct CollectorClient {
    http: reqwest::Client,
    ingest_url: Url,
    project: String,
    token: Option<String>,
    max_retries: usize,
    initial_backoff: Duration,
}

impl CollectorClient {
    pub fn new(
        endpoint: &str,
        project: &str,
        token: Option<String>,
        max_retries: usize,
        request_timeout: Duration,
        initial_backoff: Duration,
    ) -> Result<Self> {
        if project.trim().is_empty() {
            bail!("collector project must not be empty");
        }
        let mut endpoint = Url::parse(endpoint).context("collector endpoint must be a URL")?;
        if !matches!(endpoint.scheme(), "http" | "https") || endpoint.host_str().is_none() {
            bail!("collector endpoint must use http or https and include a host");
        }
        endpoint.set_path("/v1/events:write");
        endpoint.set_query(None);
        endpoint.set_fragment(None);
        let http = reqwest::Client::builder()
            .timeout(request_timeout)
            .build()
            .context("build collector HTTP client")?;
        Ok(Self {
            http,
            ingest_url: endpoint,
            project: project.to_string(),
            token,
            max_retries,
            initial_backoff,
        })
    }

    pub async fn send(&self, events: &[OperationalEventV2]) -> Result<DeliveryCounts> {
        if events.is_empty() {
            return Ok(DeliveryCounts::default());
        }
        let request = EventWriteRequest {
            events: events
                .iter()
                .map(serde_json::to_value)
                .collect::<std::result::Result<Vec<_>, _>>()?,
        };
        let mut last_error = String::new();

        for attempt in 0..=self.max_retries {
            let mut builder = self
                .http
                .post(self.ingest_url.clone())
                .header("x-sift-project", &self.project)
                .json(&request);
            if let Some(token) = self.token.as_deref() {
                builder = builder.bearer_auth(token);
            }
            match builder.send().await {
                Ok(response) if response.status().is_success() => {
                    let response: EventWriteResponse = response
                        .json()
                        .await
                        .context("decode Sift bounded ingest response")?;
                    if response.results.len() != events.len() {
                        bail!(
                            "Sift ingest returned {} outcomes for {} collector events; checkpoint unchanged",
                            response.results.len(),
                            events.len()
                        );
                    }
                    let mut counts = DeliveryCounts::default();
                    for result in response.results {
                        match result.outcome {
                            BatchOutcome::Accepted => counts.accepted += 1,
                            BatchOutcome::Duplicate => counts.duplicates += 1,
                            BatchOutcome::Rejected => {
                                let detail = result
                                    .error
                                    .map(|error| format!("{}: {}", error.code, error.message))
                                    .unwrap_or_else(|| "missing rejection detail".to_string());
                                bail!(
                                    "Sift rejected collector item {} ({detail}); checkpoint unchanged",
                                    result.index
                                );
                            }
                        }
                    }
                    return Ok(counts);
                }
                Ok(response) => {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    last_error = format!(
                        "Sift ingest HTTP {status}: {}",
                        truncate(&body, 512)
                    );
                    if !retryable_status(status) {
                        bail!("{last_error}; verify SIFT_URL, project authorization, and SIFT_TOKEN");
                    }
                }
                Err(error) => {
                    last_error = format!("Sift ingest transport error: {error}");
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(backoff(self.initial_backoff, attempt)).await;
            }
        }

        bail!(
            "collector delivery exhausted after {} attempt(s): {}; checkpoint unchanged; verify SIFT_URL reachability and retry",
            self.max_retries + 1,
            last_error
        )
    }
}

fn retryable_status(status: StatusCode) -> bool {
    status == StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

fn backoff(initial: Duration, attempt: usize) -> Duration {
    let multiplier = 1_u32.checked_shl(attempt.min(6) as u32).unwrap_or(64);
    initial.saturating_mul(multiplier).min(Duration::from_secs(5))
}

fn truncate(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_string();
    }
    let mut end = max_bytes;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_classification_and_backoff_are_bounded() {
        assert!(retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(!retryable_status(StatusCode::BAD_REQUEST));
        assert!(!retryable_status(StatusCode::UNAUTHORIZED));
        assert_eq!(
            backoff(Duration::from_millis(50), 0),
            Duration::from_millis(50)
        );
        assert_eq!(backoff(Duration::from_secs(1), 20), Duration::from_secs(5));
    }

    #[test]
    fn client_rejects_non_http_or_missing_project_configuration() {
        assert!(CollectorClient::new(
            "file:///tmp/sift",
            "project",
            None,
            1,
            Duration::from_secs(1),
            Duration::from_millis(1)
        )
        .is_err());
        assert!(CollectorClient::new(
            "http://127.0.0.1:7380",
            "",
            None,
            1,
            Duration::from_secs(1),
            Duration::from_millis(1)
        )
        .is_err());
    }
}
// HANDWRITE-END
