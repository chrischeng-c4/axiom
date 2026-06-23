//! relay client (#14) — loom → relay publish.
//!
//! Implements [`Dispatcher`] over relay's HTTP/2 (h2c) `POST /v1/{subject}/publish`
//! API: the subject is the runner-class route, the body is relay's
//! `PublishRequest` ({ message_id, payload, headers }) with the JSON
//! [`TaskMessage`] as the opaque `payload` and a per-(run,node,attempt)
//! `message_id` for idempotent publish. Mirrors lumen's relay WAL client; relay
//! is plaintext h2c, so no TLS is linked.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::model::KeepRef;
use crate::scheduler::{CompletionMsg, Dispatcher, FanOutSpec, TaskMessage};
use crate::worker::{CompletionSink, LeasedTask, RelayConsumer};

/// Publishes node dispatches to a relay broker over h2c.
pub struct RelayDispatcher {
    client: reqwest::Client,
    base: String,
}

/// relay's `PublishRequest` shape (wire.rs): `payload` is opaque JSON, `headers`
/// defaults, dedupe is on `message_id`.
#[derive(Serialize)]
struct PublishBody<'a> {
    message_id: String,
    payload: &'a TaskMessage,
}

impl RelayDispatcher {
    /// Connect to a relay base URL, e.g. `http://relay:7400`.
    pub fn new(base: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().http2_prior_knowledge().build()?,
            base: base.into(),
        })
    }
}

#[async_trait]
impl Dispatcher for RelayDispatcher {
    async fn dispatch(&self, route: &str, msg: TaskMessage) -> anyhow::Result<()> {
        let url = format!("{}/v1/{}/publish", self.base, route);
        let body = PublishBody { message_id: msg.message_id(), payload: &msg };
        let resp = self.client.post(&url).json(&body).send().await?;
        anyhow::ensure!(
            resp.status().is_success(),
            "relay publish to {route} failed: {}",
            resp.status()
        );
        Ok(())
    }
}

/// A relay work-queue consumer bound to one subject, over h2c. Implements the
/// worker's [`RelayConsumer`]: `lease` returns the task body (#166), `ack`
/// completes it.
pub struct RelayWorkConsumer {
    client: reqwest::Client,
    base: String,
    subject: String,
}

#[derive(Deserialize)]
struct LeaseResp {
    lease: Option<LeaseInfo>,
    #[serde(default)]
    entry: Option<EntryInfo>,
}
#[derive(Deserialize)]
struct LeaseInfo {
    lease_id: String,
    epoch: u64,
}
#[derive(Deserialize)]
struct EntryInfo {
    payload: serde_json::Value,
}

impl RelayWorkConsumer {
    pub fn new(base: impl Into<String>, subject: impl Into<String>) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().http2_prior_knowledge().build()?,
            base: base.into(),
            subject: subject.into(),
        })
    }
}

#[async_trait]
impl RelayConsumer for RelayWorkConsumer {
    async fn lease(&self, consumer_id: &str) -> anyhow::Result<Option<LeasedTask>> {
        let url = format!("{}/v1/{}/lease", self.base, self.subject);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "consumer_id": consumer_id }))
            .send()
            .await?;
        anyhow::ensure!(resp.status().is_success(), "relay lease: {}", resp.status());
        let lr: LeaseResp = resp.json().await?;
        match (lr.lease, lr.entry) {
            (Some(l), Some(e)) => {
                let message: TaskMessage = serde_json::from_value(e.payload)
                    .map_err(|err| anyhow::anyhow!("leased payload is not a loom TaskMessage: {err}"))?;
                Ok(Some(LeasedTask { lease_id: l.lease_id, epoch: l.epoch, message }))
            }
            _ => Ok(None),
        }
    }

    async fn ack(&self, lease_id: &str, epoch: u64) -> anyhow::Result<()> {
        let url = format!("{}/v1/{}/ack", self.base, self.subject);
        let resp = self
            .client
            .post(&url)
            .json(&serde_json::json!({ "lease_id": lease_id, "epoch": epoch }))
            .send()
            .await?;
        anyhow::ensure!(resp.status().is_success(), "relay ack: {}", resp.status());
        Ok(())
    }
}

/// Publishes worker completions to a relay subject the controller consumes,
/// closing the loop (`done == N → next-node` is loom's job, not relay's).
pub struct RelayCompletionSink {
    client: reqwest::Client,
    base: String,
    subject: String,
    shards: u32,
}

/// Stable FNV-1a hash of a run id → shard index. Identical in the worker (which
/// publishes) and the controller (which consumes), so a run's completions always
/// land on the same shard and fold serially (#127 sharded fold).
pub fn shard_of(run_id: &str, shards: u32) -> u32 {
    if shards <= 1 {
        return 0;
    }
    let mut h: u64 = 0xcbf29ce484222325;
    for b in run_id.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    (h % shards as u64) as u32
}

impl RelayCompletionSink {
    /// Single-subject sink (no sharding) — used by tests and the in-Job run-task.
    pub fn new(base: impl Into<String>, subject: impl Into<String>) -> anyhow::Result<Self> {
        Self::new_sharded(base, subject, 1)
    }

    /// Sharded sink: completions publish to `{subject}.{shard_of(run_id)}` when
    /// `shards > 1` (else the plain `{subject}`), so the controller can fold them
    /// in parallel without same-run races.
    pub fn new_sharded(
        base: impl Into<String>,
        subject: impl Into<String>,
        shards: u32,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            client: reqwest::Client::builder().http2_prior_knowledge().build()?,
            base: base.into(),
            subject: subject.into(),
            shards: shards.max(1),
        })
    }
}

#[async_trait]
impl CompletionSink for RelayCompletionSink {
    async fn report(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        result_ref: Option<KeepRef>,
        failed: bool,
        fan_out: &[FanOutSpec],
    ) -> anyhow::Result<()> {
        let msg = CompletionMsg {
            run_id: run_id.to_string(),
            node_id: node_id.to_string(),
            attempt,
            result_ref: result_ref.map(|r| r.0),
            failed,
            fan_out: fan_out.to_vec(),
        };
        let subject = if self.shards <= 1 {
            self.subject.clone()
        } else {
            format!("{}.{}", self.subject, shard_of(run_id, self.shards))
        };
        let url = format!("{}/v1/{}/publish", self.base, subject);
        let body = serde_json::json!({
            "message_id": format!("{run_id}:{node_id}:{attempt}:done"),
            "payload": msg,
        });
        let resp = self.client.post(&url).json(&body).send().await?;
        anyhow::ensure!(
            resp.status().is_success(),
            "relay completion publish: {}",
            resp.status()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::RunnerClass;

    fn msg() -> TaskMessage {
        TaskMessage {
            run_id: "r1".into(),
            node_id: "n2".into(),
            attempt: 3,
            task_name: "ingest".into(),
            args: serde_json::json!({"k": 1}),
            input_refs: vec![KeepRef("keep/in".into())],
            runner: RunnerClass::K8sJob,
        }
    }

    #[test]
    fn publish_body_matches_relay_publish_request_shape() {
        let m = msg();
        let body = PublishBody { message_id: m.message_id(), payload: &m };
        let v = serde_json::to_value(&body).unwrap();
        // relay PublishRequest: { message_id, payload (opaque), headers? }.
        assert_eq!(v["message_id"], "r1:n2:3");
        assert_eq!(v["payload"]["task_name"], "ingest");
        assert_eq!(v["payload"]["runner"], "k8s-job");
        assert_eq!(v["payload"]["input_refs"][0], "keep/in");
    }

    #[test]
    fn constructs_h2c_client() {
        assert!(RelayDispatcher::new("http://127.0.0.1:7400").is_ok());
    }
}
