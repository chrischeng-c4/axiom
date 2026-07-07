---
id: vat-source-projects-vat-src-spec-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/spec.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/spec.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `Base` | projects/vat/src/spec.rs | enum | pub | 78 |  |
| `EgressPolicy` | projects/vat/src/spec.rs | enum | pub | 105 |  |
| `EnvSpec` | projects/vat/src/spec.rs | struct | pub | 18 |  |
| `GpuRequest` | projects/vat/src/spec.rs | enum | pub | 122 |  |
| `Isolation` | projects/vat/src/spec.rs | enum | pub | 89 |  |
| `Limits` | projects/vat/src/spec.rs | struct | pub | 135 |  |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-spec-rs.md#rust-source-unit
// CODEGEN-BEGIN
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

    /// Outbound network egress policy (seatbelt-enforced).
    #[serde(default)]
    pub egress: EgressPolicy,

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
            egress: EgressPolicy::default(),
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

/// Outbound network egress policy, enforced by the seatbelt backend
/// (`sandbox-exec`). Only enforceable under `Isolation::Seatbelt`; with
/// `Isolation::None` it is advisory (vat warns it cannot confine egress).
/// @spec projects/vat/tech-design/logic/vat-network-sandbox-v3-seatbelt-egress-policy-deny-outbound-exce.md#schema
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EgressPolicy {
    /// No network restriction (current behaviour).
    #[default]
    Open,
    /// Deny outbound network except localhost (loopback + unix sockets) — the
    /// hermetic-with-routing mode: vat's local emulators/proxy stay reachable.
    LocalhostOnly,
    /// Deny all outbound network, including localhost.
    Deny,
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
// CODEGEN-END
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/spec.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/spec.rs` captured during #39 vat standardization.
```
