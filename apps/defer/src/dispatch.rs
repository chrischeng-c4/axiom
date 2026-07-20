// SPEC-MANAGED: apps/defer/tech-design/logic/core-scheduler-priority-rate-dispatch.md#logic
// HANDWRITE-BEGIN gap="missing-generator:logic:defer-http-dispatch" tracker="#766" reason="Committed-lease HTTP target executor with stable idempotency and optional HMAC signing."
//! HTTP push execution after committed lease ownership.

use std::time::Duration;

use anyhow::{Context, Result};
use base64::Engine as _;
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use utoipa::ToSchema;

use crate::{AttemptSettlement, DeferRaft, DispatchLease, NackOutcome, SettlementOutcome};

const IDEMPOTENCY_HEADER: &str = "idempotency-key";
const ATTEMPT_HEADER: &str = "x-defer-attempt-id";
const ATTEMPT_NUMBER_HEADER: &str = "x-defer-attempt";
const EPOCH_HEADER: &str = "x-defer-fence-epoch";
const TIMESTAMP_HEADER: &str = "x-defer-timestamp-ms";
const KEY_ID_HEADER: &str = "x-defer-key-id";
const SIGNATURE_HEADER: &str = "x-defer-signature";

#[derive(Clone)]
pub struct TargetSigningKey {
    pub key_id: String,
    secret: Vec<u8>,
}

impl TargetSigningKey {
    pub fn new(key_id: impl Into<String>, secret: impl Into<Vec<u8>>) -> Result<Self> {
        let key_id = key_id.into();
        let secret = secret.into();
        anyhow::ensure!(!key_id.trim().is_empty(), "target signing key id is empty");
        anyhow::ensure!(
            secret.len() >= 32,
            "target signing secret must be at least 32 bytes"
        );
        Ok(Self { key_id, secret })
    }

    /// `v1` HMAC over the immutable delivery identity and exact JSON bytes.
    pub fn signature(&self, lease: &DispatchLease, timestamp_ms: i64, body: &[u8]) -> String {
        let mut mac =
            Hmac::<Sha256>::new_from_slice(&self.secret).expect("HMAC accepts arbitrary-size keys");
        let timestamp = timestamp_ms.to_string();
        for field in [
            lease.idempotency_key.as_bytes(),
            lease.attempt_id.as_bytes(),
            lease.target.url.as_bytes(),
            timestamp.as_bytes(),
            body,
        ] {
            mac.update(&(field.len() as u64).to_be_bytes());
            mac.update(field);
        }
        format!(
            "v1={}",
            base64::engine::general_purpose::STANDARD.encode(mac.finalize().into_bytes())
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub enum DispatchDisposition {
    Acked,
    Retried { next_at: DateTime<Utc> },
    DeadLettered,
    LostOwnership,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DispatchReport {
    pub task_id: String,
    pub attempt_id: String,
    pub target_status: Option<u16>,
    pub transport_error: Option<String>,
    pub disposition: DispatchDisposition,
}

pub struct HttpDispatcher {
    client: reqwest::Client,
    signing_key: Option<TargetSigningKey>,
}

struct ExecutedAttempt {
    lease: DispatchLease,
    target_status: Option<u16>,
    transport_error: Option<String>,
    success: bool,
    completed_at: DateTime<Utc>,
}

impl HttpDispatcher {
    pub fn new(timeout: Duration, signing_key: Option<TargetSigningKey>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .http2_adaptive_window(true)
            .timeout(timeout)
            .build()
            .context("build Defer target HTTP client")?;
        Ok(Self {
            client,
            signing_key,
        })
    }

    /// Lease and execute at most one due task. `None` means the queue had no
    /// eligible work. HTTP success is any 2xx status; transport errors and
    /// non-2xx responses commit nack/retry/DLQ. A false ack/nack means this
    /// executor lost the committed fence while the external effect was in
    /// flight and is reported as `LostOwnership`.
    pub async fn dispatch_one(
        &self,
        raft: &DeferRaft,
        queue: &str,
        lease_at: DateTime<Utc>,
    ) -> Result<Option<DispatchReport>> {
        Ok(self
            .dispatch_batch(raft, queue, lease_at, 1, 1)
            .await?
            .pop())
    }

    /// Commit one lease batch, execute the HTTP effects with a shared bounded
    /// runner, then commit one fenced settlement batch. This keeps ownership
    /// and terminal state authoritative without paying one Raft/fsync round
    /// trip per task transition.
    pub async fn dispatch_batch(
        &self,
        raft: &DeferRaft,
        queue: &str,
        lease_at: DateTime<Utc>,
        requested: usize,
        max_concurrency: usize,
    ) -> Result<Vec<DispatchReport>> {
        let leases = raft
            .lease_due(queue.to_string(), lease_at, requested)
            .await?;
        if leases.is_empty() {
            return Ok(Vec::new());
        }
        let executions = service_executor::run_bounded(leases, max_concurrency, |lease| {
            self.execute_lease(lease, lease_at)
        })
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        let attempts = executions
            .iter()
            .map(|execution| AttemptSettlement {
                attempt_id: execution.lease.attempt_id.clone(),
                epoch: execution.lease.epoch,
                completed_at: execution.completed_at,
                success: execution.success,
            })
            .collect();
        let outcomes = raft.settle_batch(queue.to_string(), attempts).await?;
        anyhow::ensure!(
            outcomes.len() == executions.len(),
            "settlement outcome count mismatch"
        );
        Ok(executions
            .into_iter()
            .zip(outcomes)
            .map(|(execution, outcome)| {
                let disposition = match outcome {
                    SettlementOutcome::Acked(true) => DispatchDisposition::Acked,
                    SettlementOutcome::Acked(false) => DispatchDisposition::LostOwnership,
                    SettlementOutcome::Nacked(Some(NackOutcome::Retried { next_at })) => {
                        DispatchDisposition::Retried { next_at }
                    }
                    SettlementOutcome::Nacked(Some(NackOutcome::DeadLettered)) => {
                        DispatchDisposition::DeadLettered
                    }
                    SettlementOutcome::Nacked(None) => DispatchDisposition::LostOwnership,
                };
                DispatchReport {
                    task_id: execution.lease.task_id,
                    attempt_id: execution.lease.attempt_id,
                    target_status: execution.target_status,
                    transport_error: execution.transport_error,
                    disposition,
                }
            })
            .collect())
    }

    async fn execute_lease(
        &self,
        lease: DispatchLease,
        lease_at: DateTime<Utc>,
    ) -> Result<ExecutedAttempt> {
        let body = serde_json::to_vec(&lease.payload).context("encode Defer target payload")?;
        let method = reqwest::Method::from_bytes(lease.target.method.as_bytes());

        let response = match method {
            Ok(method) => {
                let mut request = self
                    .client
                    .request(method, &lease.target.url)
                    .header(IDEMPOTENCY_HEADER, &lease.idempotency_key)
                    .header(ATTEMPT_HEADER, &lease.attempt_id)
                    .header(ATTEMPT_NUMBER_HEADER, lease.attempt.to_string())
                    .header(EPOCH_HEADER, lease.epoch.to_string())
                    .header("content-type", "application/json");
                for (name, value) in &lease.target.headers {
                    request = request.header(name, value);
                }
                if let Some(key) = &self.signing_key {
                    let timestamp_ms = lease_at.timestamp_millis();
                    request = request
                        .header(TIMESTAMP_HEADER, timestamp_ms.to_string())
                        .header(KEY_ID_HEADER, &key.key_id)
                        .header(SIGNATURE_HEADER, key.signature(&lease, timestamp_ms, &body));
                }
                request
                    .body(body)
                    .send()
                    .await
                    .map_err(|error| error.to_string())
            }
            Err(error) => Err(error.to_string()),
        };

        let (target_status, transport_error, success) = match response {
            Ok(response) => {
                let status = response.status();
                (Some(status.as_u16()), None, status.is_success())
            }
            Err(error) => (None, Some(error), false),
        };

        // Tests and catch-up workers may propose a due time just ahead of the
        // local wall clock; never commit a completion timestamp before the
        // committed lease timestamp.
        let completed_at = std::cmp::max(Utc::now(), lease_at);
        Ok(ExecutedAttempt {
            lease,
            target_status,
            transport_error,
            success,
            completed_at,
        })
    }
}
// HANDWRITE-END
