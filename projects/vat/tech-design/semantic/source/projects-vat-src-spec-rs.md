---
id: vat-source-projects-vat-src-spec-rs
summary: Source replay payload for projects/vat/src/spec.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/spec.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Base` | projects/vat/src/spec.rs | enum | pub | 73 |  |
| `EnvSpec` | projects/vat/src/spec.rs | struct | pub | 18 |  |
| `GpuRequest` | projects/vat/src/spec.rs | enum | pub | 100 |  |
| `Isolation` | projects/vat/src/spec.rs | enum | pub | 84 |  |
| `Limits` | projects/vat/src/spec.rs | struct | pub | 113 |  |
## Source
<!-- type: source lang: rust -->

`````rust
//! Declarative environment spec.
//!
//! Not a Dockerfile. A vat's spec is data an agent reads and rewrites: where
//! the workspace comes from, what env to inject, what to run on creation, how
//! tightly to sandbox, and whether the GPU is required. It serializes to JSON
//! (stored inside `meta.json`) and can be authored as JSON on `vat run`.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Full declarative description of a vat's environment.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvSpec {
    /// Where the workspace is cloned from. `None` for an empty workspace.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base: Option<Base>,

    /// Working directory *inside* the rootfs the command runs in.
    #[serde(default = "default_workdir")]
    pub workdir: PathBuf,

    /// Extra environment variables injected into the run.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,

    /// Commands run once at creation time (e.g. `pip install -r req.txt`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub setup: Vec<String>,

    /// How tightly to isolate the process.
    #[serde(default)]
    pub isolation: Isolation,

    /// GPU expectation for this vat.
    #[serde(default)]
    pub gpu: GpuRequest,

    /// Advisory resource ceilings recorded for agents and wrappers. Vat does
    /// not schedule workloads; run vat under an external scheduler such as cap
    /// when admission control or throttling is required.
    #[serde(default)]
    pub limits: Limits,
}

/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
impl Default for EnvSpec {
    fn default() -> Self {
        EnvSpec {
            base: None,
            workdir: default_workdir(),
            env: BTreeMap::new(),
            setup: Vec::new(),
            isolation: Isolation::default(),
            gpu: GpuRequest::default(),
            limits: Limits::default(),
        }
    }
}

fn default_workdir() -> PathBuf {
    PathBuf::from(".")
}

/// Source of a vat's initial workspace.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "ref")]
pub enum Base {
    /// Copy-on-write clone of a host directory.
    Dir(PathBuf),
    /// Fork of another vat's rootfs (carries lineage).
    Vat(String),
}

/// Process isolation strength. v1 ships `None` and `Seatbelt` (macOS).
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Isolation {
    /// No syscall sandbox: just the copy-on-write workspace + injected env.
    /// The default, because it keeps full native GPU/IO with zero friction.
    #[default]
    None,
    /// macOS seatbelt profile: reads allowed broadly, writes confined to the
    /// rootfs + temp. Opt-in; Metal still works (it's a host process).
    Seatbelt,
}

/// Whether the vat wants the GPU. Vat never *removes* GPU access (it can't —
/// the process is native); this only drives a pre-flight check and what the
/// agent is told.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum GpuRequest {
    /// Use the GPU if present, don't fail if absent. The sensible default.
    #[default]
    Auto,
    /// Fail fast at creation if no accessible GPU is detected.
    Required,
    /// Caller doesn't care about the GPU.
    None,
}

/// Advisory limits echoed in state for the agent or an external scheduler.
/// @spec projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Limits {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_mb: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_s: Option<u64>,
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/spec.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-spec-rs-source-replay-superseded>"
```
