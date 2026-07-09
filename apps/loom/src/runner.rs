//! Runner selection (#164): per-task execution-environment choice.
//!
//! loom *selects* the runner (task/stage-level metadata); it does **not** own
//! execution. Every task is published to relay tagged with its runner class;
//! relay routes by class and execution is realized by whoever consumes the
//! lease — a resident polyglot worker, the `loom job-controller` (which creates
//! a k8s Job), or a local dev runner. relay's lease/ack is unchanged for all.

use serde::{Deserialize, Serialize};

/// Where a task runs. Stored as task/stage metadata, not a worker-global setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RunnerClass {
    /// Long-running pull-loop worker (Celery-style); IO-bound tasks.
    #[default]
    #[serde(rename = "resident")]
    Resident,
    /// One k8s Job per task/attempt, created by `loom job-controller`;
    /// CPU-bound, isolated, or dependency-heavy tasks.
    #[serde(rename = "k8s-job")]
    K8sJob,
    /// Local process/container runner for dev and tests.
    #[serde(rename = "local")]
    Local,
}

impl RunnerClass {
    /// The relay routing key (queue/topic suffix) for this runner class.
    pub fn relay_route(self) -> &'static str {
        match self {
            RunnerClass::Resident => "resident",
            RunnerClass::K8sJob => "k8s-job",
            RunnerClass::Local => "local",
        }
    }
}
