---
id: vat-source-projects-vat-src-state-rs
summary: Source replay payload for projects/vat/src/state.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/state.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/state.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ArtifactRecord` | projects/vat/src/state.rs | struct | pub | 133 |  |
| `ChangeSet` | projects/vat/src/state.rs | struct | pub | 157 |  |
| `ChangeSummary` | projects/vat/src/state.rs | struct | pub | 203 |  |
| `ConfigRef` | projects/vat/src/state.rs | struct | pub | 80 |  |
| `ProcessStatus` | projects/vat/src/state.rs | enum | pub | 121 |  |
| `RunRecord` | projects/vat/src/state.rs | struct | pub | 44 |  |
| `RunnerRunRecord` | projects/vat/src/state.rs | struct | pub | 105 |  |
| `ServiceRunRecord` | projects/vat/src/state.rs | struct | pub | 88 |  |
| `Status` | projects/vat/src/state.rs | enum | pub | 30 |  |
| `TestRunEvidence` | projects/vat/src/state.rs | struct | pub | 142 |  |
| `VatMeta` | projects/vat/src/state.rs | struct | pub | 59 |  |
| `VatState` | projects/vat/src/state.rs | struct | pub | 228 |  |
| `WorkspaceInfo` | projects/vat/src/state.rs | struct | pub | 218 |  |
| `is_empty` | projects/vat/src/state.rs | function | pub | 169 | is_empty(&self) -> bool |
| `oneline` | projects/vat/src/state.rs | function | pub | 174 | oneline(&self) -> String |
| `summary` | projects/vat/src/state.rs | function | pub | 185 | summary(&self, sample: usize) -> ChangeSummary |
| `total` | projects/vat/src/state.rs | function | pub | 165 | total(&self) -> usize |
## Source
<!-- type: source lang: rust -->

`````rust
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

/// Captured service state for one run-scoped dependency process.
/// @spec projects/vat/tech-design/logic/local-agent-test-runner-protocol.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRunRecord {
    pub id: String,
    pub command: Vec<String>,
    pub status: ProcessStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ready_http: Option<String>,
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

`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/state.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-state-rs-source-replay-superseded>"
```
