//! `loom job-controller` — relay → k8s Job bridge (#164).
//!
//! A relay consumer for `runner = k8s-job` tasks: for each leased task it
//! creates a k8s Job that runs `loom run-task`, heartbeats relay while the Job
//! runs, and acks on success (lease expiry → redeliver → new attempt). It is
//! the *only* component that touches the k8s API, keeping loom's core
//! cluster-free and testable. Implemented in P3 (#164).

/// Entry point for `loom job-controller`.
pub fn run() -> anyhow::Result<()> {
    anyhow::bail!("loom job-controller: not yet implemented (#164 k8s Job bridge, P3)")
}
