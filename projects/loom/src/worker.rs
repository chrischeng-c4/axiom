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
use crate::scheduler::{FanOutSpec, TaskMessage};

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
/// publish to a completions subject loom subscribes). `fan_out` carries any
/// runtime children the task requested (#116).
#[async_trait]
pub trait CompletionSink: Send + Sync {
    async fn report(
        &self,
        run_id: &str,
        node_id: &str,
        attempt: u32,
        result_ref: Option<KeepRef>,
        failed: bool,
        fan_out: &[FanOutSpec],
    ) -> anyhow::Result<()>;
}

/// What a handler produces: result bytes plus any runtime fan-out children
/// (#116) — e.g. a CSV reader emits one child per chunk it discovers.
#[derive(Debug, Default, Clone)]
pub struct TaskOutput {
    pub result: Vec<u8>,
    pub fan_out: Vec<FanOutSpec>,
}

impl From<Vec<u8>> for TaskOutput {
    fn from(result: Vec<u8>) -> Self {
        Self { result, fan_out: Vec::new() }
    }
}

/// A task handler: fetched input bytes → [`TaskOutput`] (result + optional
/// fan-out). A polyglot worker dispatches to the user's code/image; the Rust
/// reference worker registers handlers here by task name.
pub type Handler = Arc<dyn Fn(Vec<u8>) -> anyhow::Result<TaskOutput> + Send + Sync>;

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
        Ok(out) => {
            let result_id = format!("{}:{}:result", m.run_id, m.node_id);
            keep.put_result(&result_id, out.result).await?;
            consumer.ack(&task.lease_id, task.epoch).await?;
            sink.report(
                &m.run_id,
                &m.node_id,
                m.attempt,
                Some(KeepRef(result_id)),
                false,
                &out.fan_out,
            )
            .await?;
        }
        Err(_) => {
            // Release the lease and let loom decide retry-or-fail (loom owns the
            // DAG + retry policy; it re-dispatches a fresh attempt).
            consumer.ack(&task.lease_id, task.epoch).await?;
            sink.report(&m.run_id, &m.node_id, m.attempt, None, true, &[]).await?;
        }
    }
    Ok(true)
}

/// Entry point for `loom worker` — a resident pull loop over real relay + keep.
/// Env: `LOOM_RELAY` (relay base), `LOOM_KEEP` (keep base), `LOOM_RUNNER`
/// (subject/runner class to lease; default `resident`). Registers a built-in
/// `echo` handler; real deployments register their task handlers.
pub fn run() -> anyhow::Result<()> {
    let relay = std::env::var("LOOM_RELAY")
        .map_err(|_| anyhow::anyhow!("loom worker requires LOOM_RELAY (relay base url)"))?;
    let keep_base = std::env::var("LOOM_KEEP")
        .map_err(|_| anyhow::anyhow!("loom worker requires LOOM_KEEP (keep base url)"))?;
    let subject = std::env::var("LOOM_RUNNER").unwrap_or_else(|_| "resident".to_string());

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let consumer = crate::relay_client::RelayWorkConsumer::new(&relay, &subject)?;
        let keep = crate::keep_client::KeepHttp::new(&keep_base)?;
        let sink = crate::relay_client::RelayCompletionSink::new(&relay, "loom.completions")?;
        let mut registry = Registry::new();
        registry.register("echo", Arc::new(|input: Vec<u8>| Ok(input.into())));
        // demo: `split` reads its input as a chunk count and fans out one `echo`
        // child per chunk at runtime (#116) — the shape a CSV reader uses (#111).
        registry.register(
            "split",
            Arc::new(|input: Vec<u8>| {
                let n: usize = std::str::from_utf8(&input)
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(3);
                let fan_out = (0..n)
                    .map(|i| FanOutSpec {
                        id: format!("chunk-{i}"),
                        task_name: "echo".to_string(),
                        input_refs: Vec::new(),
                    })
                    .collect();
                Ok(TaskOutput { result: Vec::new(), fan_out })
            }),
        );

        eprintln!("loom worker: leasing `{subject}` from relay {relay}, keep {keep_base}");
        loop {
            match run_once("loom-worker", &consumer, &keep, &sink, &registry).await {
                Ok(true) => {}
                Ok(false) => tokio::time::sleep(std::time::Duration::from_millis(200)).await,
                Err(e) => {
                    eprintln!("loom worker: tick error: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    })
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
        reports: Mutex<Vec<(String, bool, usize)>>,
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
            fan_out: &[FanOutSpec],
        ) -> anyhow::Result<()> {
            self.reports.lock().unwrap().push((node.to_string(), failed, fan_out.len()));
            Ok(())
        }
    }

    fn echo_registry() -> Registry {
        let mut r = Registry::new();
        r.register("echo", Arc::new(|input: Vec<u8>| Ok(input.into())));
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
        assert_eq!(sink.reports.lock().unwrap().as_slice(), &[("n".to_string(), false, 0)]);
    }

    #[tokio::test]
    async fn handler_fan_out_is_reported() {
        let mut reg = Registry::new();
        reg.register(
            "split",
            Arc::new(|_input: Vec<u8>| {
                Ok(TaskOutput {
                    result: Vec::new(),
                    fan_out: vec![
                        FanOutSpec { id: "c0".into(), task_name: "echo".into(), input_refs: vec![] },
                        FanOutSpec { id: "c1".into(), task_name: "echo".into(), input_refs: vec![] },
                    ],
                })
            }),
        );
        let consumer = FakeConsumer { next: Mutex::new(Some(task("split"))), ..Default::default() };
        let sink = FakeSink::default();
        run_once("w1", &consumer, &FakeKeep::default(), &sink, &reg).await.unwrap();
        // the completion carries the 2 runtime children.
        assert_eq!(sink.reports.lock().unwrap().as_slice(), &[("n".to_string(), false, 2)]);
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
        assert_eq!(sink.reports.lock().unwrap().as_slice(), &[("n".to_string(), true, 0)]);
        // still acked (lease released) so loom drives retry.
        assert_eq!(consumer.acked.lock().unwrap().len(), 1);
    }
}
