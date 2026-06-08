---
id: vat-source-projects-vat-src-lib-rs
summary: Source replay payload for projects/vat/src/lib.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: copy-on-write-fork-and-snapshot-lifecycle
    claim: copy-on-write-fork-and-snapshot-lifecycle
    coverage: full
    rationale: "This source replay TD preserves vat's copy-on-write workspace, agent-legible state, resource isolation, and host GPU behavior."
---

# Source TD: projects/vat/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/src/lib.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `VERSION` | projects/vat/src/lib.rs | constant | pub | 49 |  |
| `cli` | projects/vat/src/lib.rs | module | pub | 46 |  |
| `commands` | projects/vat/src/lib.rs | module | pub | 34 |  |
| `config` | projects/vat/src/lib.rs | module | pub | 35 |  |
| `event` | projects/vat/src/lib.rs | module | pub | 36 |  |
| `gpu` | projects/vat/src/lib.rs | module | pub | 37 |  |
| `id` | projects/vat/src/lib.rs | module | pub | 38 |  |
| `overlay` | projects/vat/src/lib.rs | module | pub | 39 |  |
| `paths` | projects/vat/src/lib.rs | module | pub | 40 |  |
| `sandbox` | projects/vat/src/lib.rs | module | pub | 41 |  |
| `spec` | projects/vat/src/lib.rs | module | pub | 42 |  |
| `state` | projects/vat/src/lib.rs | module | pub | 43 |  |
| `store` | projects/vat/src/lib.rs | module | pub | 44 |  |
## Source
<!-- type: source lang: rust -->

`````rust
//! vat — agent-native, GPU-native dev containers.
//!
//! ## What vat is
//!
//! A container runtime for the one user who never gets a say in Docker's
//! design: a coding/ML **agent**. Two things make it different from "Docker
//! minus the GUI":
//!
//! 1. **Agent-legible state.** Every vat projects its full current state as
//!    one compact, structured [`state::VatState`] JSON value — what's
//!    installed, what changed on disk vs. its base, the last run, recent
//!    events, the GPU it can see, its fork lineage. An agent reads *one*
//!    document to understand "what is this environment right now" instead of
//!    parsing the scrollback of `docker ps/inspect/diff/logs`.
//!
//! 2. **GPU-native because there is no VM.** On Apple Silicon, Docker runs
//!    Linux containers inside a Linux VM, and Metal has no compute passthrough
//!    into that guest — so the M-series GPU is invisible to the container.
//!    vat does not use a VM. A vat is a **sandboxed host process** over a
//!    copy-on-write workspace, so the workload runs natively on macOS and the
//!    Apple GPU (Metal / MPS / MLX) is simply present. See [`gpu`].
//!
//! ## The model
//!
//! A *vat* = a copy-on-write workspace ([`overlay`]) + a declarative
//! [`spec::EnvSpec`] + an append-only [`event`] log + projected
//! [`state::VatState`]. Vats are cheap to [`snapshot`](commands::snapshot) and
//! to **fork** (try two approaches from one starting point), like git for a
//! running environment. Isolation is a pluggable [`sandbox::Sandbox`] backend;
//! v1 ships a host-process backend with an opt-in macOS seatbelt profile.

pub mod commands;
pub mod config;
pub mod event;
pub mod gpu;
pub mod id;
pub mod overlay;
pub mod paths;
pub mod sandbox;
pub mod spec;
pub mod state;
pub mod store;

pub mod cli;

/// Crate version, surfaced by `vat --version`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/src/lib.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-src.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-src-lib-rs-source-replay-superseded>"
```
