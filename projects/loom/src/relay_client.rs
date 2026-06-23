//! relay client (#14) — loom → relay publish.
//!
//! Implements [`Dispatcher`] over relay's HTTP/2 (h2c) `POST /v1/{subject}/publish`
//! API: the subject is the runner-class route, the body is relay's
//! `PublishRequest` ({ message_id, payload, headers }) with the JSON
//! [`TaskMessage`] as the opaque `payload` and a per-(run,node,attempt)
//! `message_id` for idempotent publish. Mirrors lumen's relay WAL client; relay
//! is plaintext h2c, so no TLS is linked.

use async_trait::async_trait;
use serde::Serialize;

use crate::scheduler::{Dispatcher, TaskMessage};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::KeepRef;
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
