//! `loom run-task` — single-shot in-Job task entrypoint (#164).
//!
//! What a k8s Job runs: it reads the task identity + context from env (set by
//! the [`crate::jobcontroller`]), fetches its input from keep, runs the task
//! once via the shared handler registry, writes the result to keep, and reports
//! completion to relay (`loom.completions`) — then exits. Same execution core as
//! [`crate::worker`] ([`crate::worker::execute_task`]), in run-once mode; loom
//! folds the completion and advances the DAG, owning retry on failure.

use crate::model::KeepRef;
use crate::runner::RunnerClass;
use crate::scheduler::TaskMessage;

/// Build the [`TaskMessage`] from the `LOOM_TASK_*` env the job-controller set.
fn task_from_env() -> anyhow::Result<TaskMessage> {
    let var = |k: &str| {
        std::env::var(k).map_err(|_| anyhow::anyhow!("run-task requires {k}"))
    };
    let input_refs = std::env::var("LOOM_TASK_INPUT_REFS")
        .unwrap_or_default()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| KeepRef(s.to_string()))
        .collect();
    Ok(TaskMessage {
        run_id: var("LOOM_TASK_RUN")?,
        node_id: var("LOOM_TASK_NODE")?,
        attempt: std::env::var("LOOM_TASK_ATTEMPT").ok().and_then(|s| s.parse().ok()).unwrap_or(0),
        task_name: var("LOOM_TASK_NAME")?,
        args: serde_json::Value::Null,
        input_refs,
        runner: RunnerClass::K8sJob,
    })
}

/// Entry point for `loom run-task`.
pub fn run() -> anyhow::Result<()> {
    let relay = std::env::var("LOOM_RELAY")
        .map_err(|_| anyhow::anyhow!("loom run-task requires LOOM_RELAY"))?;
    let keep_base = std::env::var("LOOM_KEEP")
        .map_err(|_| anyhow::anyhow!("loom run-task requires LOOM_KEEP"))?;
    let task = task_from_env()?;

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let keep = crate::keep_client::KeepHttp::new(&keep_base)?;
        let sink = crate::relay_client::RelayCompletionSink::new(&relay, "loom.completions")?;
        let registry = crate::worker::default_registry();
        eprintln!(
            "loom run-task: {}::{} (attempt {}) → relay {relay}, keep {keep_base}",
            task.run_id, task.node_id, task.attempt
        );
        crate::worker::execute_task(&task, &keep, &sink, &registry).await
    })
}
