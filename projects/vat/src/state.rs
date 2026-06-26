// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! The state model — vat's reason to exist.
//!
//! Two shapes live here:
//!
//! - [`VatMeta`] is what's **persisted** to `meta.json`: identity, status,
//!   spec, lineage, and the last run. It's small and changes on transitions.
//! - [`VatState`] is the **projection** an agent reads: meta plus things
//!   computed on demand — the live filesystem [`ChangeSet`] vs. base, recent
//!   [`events`](crate::event), workspace size, and the [`gpu`](crate::gpu) the
//!   vat can see. One `vat state <id>` returns the whole document.
//!
//! The contract is: *an agent should never have to parse logs to understand a
//! vat.* If understanding the environment needs a fact, it belongs in
//! [`VatState`].

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::config::RetentionPolicy;
use crate::event::Event;
use crate::gpu::GpuInfo;
use crate::spec::EnvSpec;

/// Lifecycle status of a vat.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum Status {
    /// Created, never run.
    Created,
    /// A command is currently executing.
    Running,
    /// Last command finished with this exit code.
    Exited { code: i32 },
    /// A frozen, read-only label (produced by `vat snapshot`).
    Snapshot,
}

/// Persisted record of the most recent run.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    /// The program and its arguments, as invoked.
    pub command: Vec<String>,
    pub started_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

/// Persisted, on-disk record of a vat. Stored as `meta.json`.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatMeta {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub spec: EnvSpec,
    /// Ancestor vat ids, oldest first — the fork tree this vat sits in.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lineage: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_run: Option<RunRecord>,
    /// Evidence for a vat.toml runner invocation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub test_run: Option<TestRunEvidence>,
}

/// vat.toml config reference captured for one runner invocation.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRef {
    pub path: String,
    pub digest: String,
}

/// Captured state of a local Kubernetes cluster backing a `cluster` service.
/// @spec projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterRunRecord {
    /// Backend that provisioned the cluster: "kind", "k3d", or "minikube".
    pub backend: String,
    /// Cluster name as known to the backend.
    pub name: String,
    /// Path to the isolated kubeconfig exported to the runner.
    pub kubeconfig: String,
    /// Number of nodes requested for the cluster.
    pub node_count: u32,
    /// Time from create to first readiness, when measured.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_ms: Option<u64>,
}

/// Captured service state for one run-scoped dependency process.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRunRecord {
    pub id: String,
    pub command: Vec<String>,
    pub status: ProcessStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preset: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepare_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prepare_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exported_env: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_http: Option<String>,
    /// Present when this service is a local Kubernetes cluster.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cluster: Option<ClusterRunRecord>,
    pub stdout_log: String,
    pub stderr_log: String,
}

/// Captured runner process state.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerRunRecord {
    pub id: String,
    pub command: Vec<String>,
    pub status: ProcessStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub stdout_log: String,
    pub stderr_log: String,
}

/// Process status used inside test-run evidence.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessStatus {
    Created,
    Running,
    Ready,
    Exited,
    Failed,
    Timeout,
}

/// Artifact captured from a runner workspace.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

/// Complete evidence bundle for one vat.toml runner invocation.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunEvidence {
    pub config: ConfigRef,
    pub runner_id: String,
    pub retention: RetentionPolicy,
    pub services: Vec<ServiceRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runner: Option<RunnerRunRecord>,
    /// Every runner of a concurrent `vat run a b ...` set; `runner` keeps the
    /// first record for backward compatibility. Empty on legacy metadata.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub runners: Vec<RunnerRunRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ArtifactRecord>,
}

/// Filesystem changes vs. the base manifest. Full lists; the projection
/// samples them for compactness.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeSet {
    pub added: Vec<String>,
    pub modified: Vec<String>,
    pub deleted: Vec<String>,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
impl ChangeSet {
    pub fn total(&self) -> usize {
        self.added.len() + self.modified.len() + self.deleted.len()
    }

    pub fn is_empty(&self) -> bool {
        self.total() == 0
    }

    /// One-line summary, e.g. `+3 ~1 -0`.
    pub fn oneline(&self) -> String {
        format!(
            "+{} ~{} -{}",
            self.added.len(),
            self.modified.len(),
            self.deleted.len()
        )
    }

    /// Compact summary for [`VatState`]: counts plus a bounded sample so the
    /// JSON stays token-cheap even when thousands of files changed.
    pub fn summary(&self, sample: usize) -> ChangeSummary {
        let take = |v: &[String]| v.iter().take(sample).cloned().collect::<Vec<_>>();
        ChangeSummary {
            added: self.added.len(),
            modified: self.modified.len(),
            deleted: self.deleted.len(),
            total: self.total(),
            truncated: self.total() > sample * 3,
            sample_added: take(&self.added),
            sample_modified: take(&self.modified),
            sample_deleted: take(&self.deleted),
        }
    }
}

/// Bounded change view embedded in [`VatState`].
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub added: usize,
    pub modified: usize,
    pub deleted: usize,
    pub total: usize,
    /// True when sample lists omit entries (full lists via `vat diff`).
    pub truncated: bool,
    pub sample_added: Vec<String>,
    pub sample_modified: Vec<String>,
    pub sample_deleted: Vec<String>,
}

/// Workspace footprint.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceInfo {
    pub rootfs: String,
    pub file_count: usize,
    pub size_bytes: u64,
}

/// The full, agent-legible projection of a vat. This is what `vat state`
/// prints and what an agent should read to understand the environment.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-state-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VatState {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub spec: EnvSpec,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub lineage: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_run: Option<RunRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_run: Option<TestRunEvidence>,
    pub workspace: WorkspaceInfo,
    pub changes: ChangeSummary,
    /// The GPU this vat can reach — the headline contrast with Docker-in-VM.
    pub gpu: GpuInfo,
    pub events_tail: Vec<Event>,
}
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
