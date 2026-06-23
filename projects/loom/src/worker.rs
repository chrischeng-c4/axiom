//! `loom worker` — resident pull-loop worker harness (#164).
//!
//! The thin harness for `runner = resident` tasks: lease from relay → fetch
//! input from keep → run the user's `handle(input) -> output` → write result to
//! keep → ack relay exactly-once, heartbeating throughout. The author writes
//! only the task function; the harness owns the error-prone loop. Shares its
//! core with [`crate::runtask`] (single-shot k8s-Job mode). Rust reference
//! first; polyglot workers use generated OpenAPI clients + the documented
//! protocol. Implemented in P2.

/// Entry point for `loom worker`.
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!("loom worker: not yet implemented (#164 worker harness, P2)")
}
