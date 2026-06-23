//! `loom run-task` — single-shot in-Job task entrypoint (#164).
//!
//! What a k8s Job runs: receives workflow/task/attempt identity, fetches input
//! from keep, runs the task once, writes the result to keep, and reports
//! completion/failure exactly-once under retry/duplicate delivery (idempotent
//! keep write keyed by `(wf, task, attempt)`). Same harness core as
//! [`crate::worker`], in run-once mode. Implemented in P2.

/// Entry point for `loom run-task`.
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!("loom run-task: not yet implemented (#164 in-Job entrypoint, P2)")
}
