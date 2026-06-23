//! Worker harness (#164) — the resident pull loop and the in-Job run-once
//! entrypoint share this core.
//!
//! The loop follows relay's worker protocol (`relay/docs/worker-protocol.md`):
//! `lease → keep get input → run handler → keep put result → ack → report
//! completion`. The transport is abstracted behind traits so the loop is tested
//! with fakes here; the real relay/keep clients wire in once the upstream gaps
//! close — **#166** (relay lease must return the message identity + payload) and
//! **#167** (keep claim-check input/result API). Completion is reported back to
//! loom (a relay publish to a completions subject loom subscribes), since
//! `done == N → next-node` is loom's job, not relay's.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use crate::model::KeepRef;
use crate::scheduler::TaskMessage;

/// A leased task as the worker needs it — the #166 contract: a lease carries the
/// message identity and the payload, not just a position.
#[derive(Debug, Clone)]
pub struct LeasedTask {
    pub lease_id: String,
    pub epoch: u64,
    pub message: TaskMessage,
}

/// relay work-queue consumer. Real impl = relay client (blocked on #166); tests
/// use a fake.
#[async_trait]
pub trait RelayConsumer: Send + Sync {
    async fn lease(&self, consumer_id: &str) -> anyhow::Result<Option<LeasedTask>>;
    async fn ack(&self, lease_id: &str, epoch: u64) -> anyhow::Result<()>;
}

/// keep claim-check store (input/result by id). Real impl = keep client (blocked
/// on #167); tests use a fake.
#[async_trait]
pub trait KeepStore: Send + Sync {
    async fn get_input(&self, id: &str) -> anyhow::Result<Option<Vec<u8>>>;
    async fn put_result(&self, id: &str, bytes: Vec<u8>) -> anyhow::Result<()>;
}

/// Where the worker reports a finished node so loom can advance the DAG (a relay
/// publish to a completions subject loom subscribes).
#[async_trait]
pub trait CompletionSink: Send + Sync {
    async fn report(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        result_ref: Option<KeepRef>,
        failed: bool,
    ) -> anyhow::Result<()>;
}

/// A task handler: fetched input bytes → result bytes. A polyglot worker
/// dispatches to the user's code/image; the Rust reference worker registers
/// handlers here by task name.
pub type Handler = Arc<dyn Fn(Vec<u8>) -> anyhow::Result<Vec<u8>> + Send + Sync>;

/// task_name → handler.
#[derive(Default, Clone)]
pub struct Registry {
    handlers: HashMap<String, Handler>,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn register(&mut self, name: impl Into<String>, handler: Handler) {
        self.handlers.insert(name.into(), handler);
    }
    fn get(&self, name: &str) -> Option<&Handler> {
        self.handlers.get(name)
    }
}

/// Lease one task (if available) and run it through the full worker loop.
/// Returns `true` when a task was processed, `false` when the lease was empty.
pub async fn run_once(
    consumer_id: &str,
    consumer: &dyn RelayConsumer,
    keep: &dyn KeepStore,
    sink: &dyn CompletionSink,
    registry: &Registry,
) -> anyhow::Result<bool> {
    let Some(task) = consumer.lease(consumer_id).await? else {
        return Ok(false);
    };
    let m = &task.message;

    // Claim-check input: the first input ref, else the message id as the key.
    let input_id = m
        .input_refs
        .first()
        .map(|r| r.0.clone())
        .unwrap_or_else(|| m.message_id());
    let input = keep.get_input(&input_id).await?.unwrap_or_default();

    let outcome = match registry.get(&m.task_name) {
        Some(handler) => handler(input),
        None => Err(anyhow::anyhow!("no handler registered for task `{}`", m.task_name)),
    };

    match outcome {
        Ok(result) => {
            let result_id = format!("{}:{}:result", m.run_id, m.node_id);
            keep.put_result(&result_id, result).await?;
            consumer.ack(&task.lease_id, task.epoch).await?;
            sink.report(&m.run_id, &m.node_id, m.attempt, Some(KeepRef(result_id)), false)
                .await?;
        }
        Err(_) => {
            // Release the lease and let loom decide retry-or-fail (loom owns the
            // DAG + retry policy; it re-dispatches a fresh attempt).
            consumer.ack(&task.lease_id, task.epoch).await?;
            sink.report(&m.run_id, &m.node_id, m.attempt, None, true).await?;
        }
    }
    Ok(true)
}

/// Entry point for `loom worker` / `loom run-task`.
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!(
        "loom worker: harness loop implemented and tested; real relay/keep wiring \
         is blocked on #166 (relay lease must return message identity + payload) \
         and #167 (keep claim-check input/result API)"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::RunnerClass;
    use std::sync::Mutex;

    fn task(name: &str) -> LeasedTask {
        LeasedTask {
            lease_id: "L1".into(),
            epoch: 7,
            message: TaskMessage {
                run_id: "r".into(),
                node_id: "n".into(),
                attempt: 1,
                task_name: name.into(),
                args: serde_json::Value::Null,
                input_refs: vec![KeepRef("in/n".into())],
                runner: RunnerClass::Resident,
            },
        }
    }

    #[derive(Default)]
    struct FakeConsumer {
        next: Mutex<Option<LeasedTask>>,
        acked: Mutex<Vec<(String, u64)>>,
    }
    #[async_trait]
    impl RelayConsumer for FakeConsumer {
        async fn lease(&self, _c: &str) -> anyhow::Result<Option<LeasedTask>> {
            Ok(self.next.lock().unwrap().take())
        }
        async fn ack(&self, lease_id: &str, epoch: u64) -> anyhow::Result<()> {
            self.acked.lock().unwrap().push((lease_id.to_string(), epoch));
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeKeep {
        results: Mutex<HashMap<String, Vec<u8>>>,
    }
    #[async_trait]
    impl KeepStore for FakeKeep {
        async fn get_input(&self, _id: &str) -> anyhow::Result<Option<Vec<u8>>> {
            Ok(Some(b"hello".to_vec()))
        }
        async fn put_result(&self, id: &str, bytes: Vec<u8>) -> anyhow::Result<()> {
            self.results.lock().unwrap().insert(id.to_string(), bytes);
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeSink {
        reports: Mutex<Vec<(String, bool)>>,
    }
    #[async_trait]
    impl CompletionSink for FakeSink {
        async fn report(
            &self,
            _run: &str,
            node: &str,
            _attempt: u32,
            _result: Option<KeepRef>,
            failed: bool,
        ) -> anyhow::Result<()> {
            self.reports.lock().unwrap().push((node.to_string(), failed));
            Ok(())
        }
    }

    fn echo_registry() -> Registry {
        let mut r = Registry::new();
        r.register("echo", Arc::new(|input: Vec<u8>| Ok(input)));
        r
    }

    #[tokio::test]
    async fn runs_task_end_to_end_and_acks() {
        let consumer = FakeConsumer { next: Mutex::new(Some(task("echo"))), ..Default::default() };
        let keep = FakeKeep::default();
        let sink = FakeSink::default();

        let did = run_once("w1", &consumer, &keep, &sink, &echo_registry()).await.unwrap();
        assert!(did);
        assert_eq!(keep.results.lock().unwrap().get("r:n:result").unwrap(), b"hello");
        assert_eq!(consumer.acked.lock().unwrap().as_slice(), &[("L1".to_string(), 7)]);
        assert_eq!(sink.reports.lock().unwrap().as_slice(), &[("n".to_string(), false)]);
    }

    #[tokio::test]
    async fn empty_lease_is_noop() {
        let consumer = FakeConsumer::default();
        let did = run_once("w1", &consumer, &FakeKeep::default(), &FakeSink::default(), &echo_registry())
            .await
            .unwrap();
        assert!(!did);
    }

    #[tokio::test]
    async fn missing_handler_reports_failure() {
        let consumer = FakeConsumer { next: Mutex::new(Some(task("unknown"))), ..Default::default() };
        let sink = FakeSink::default();
        run_once("w1", &consumer, &FakeKeep::default(), &sink, &echo_registry()).await.unwrap();
        assert_eq!(sink.reports.lock().unwrap().as_slice(), &[("n".to_string(), true)]);
        // still acked (lease released) so loom drives retry.
        assert_eq!(consumer.acked.lock().unwrap().len(), 1);
    }
}
