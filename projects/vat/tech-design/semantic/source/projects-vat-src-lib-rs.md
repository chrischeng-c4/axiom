---
id: vat-source-projects-vat-src-lib-rs
summary: >
  rust-source-unit TD AST payload for projects/vat/src/lib.rs.
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    claim: local-agent-test-runner-protocol
    coverage: partial
    rationale: "This rust-source-unit TD preserves vat source ownership while migrating #39 off group-level source replay."
---

# Standardized projects/vat/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

Rust source-unit TD for `projects/vat/src/lib.rs`, captured during #39 vat migration onto td_ast lossless source generation.

## Source
<!-- type: rust-source-unit lang: rust -->

````rust
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

pub mod cluster;
pub mod commands;
pub mod config;
#[cfg(feature = "emulator")]
pub mod emulator;
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
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/vat/src/lib.rs
    action: modify
    section: rust-source-unit
    impl_mode: codegen
    description: |
      rust-source-unit (td_ast) source for `projects/vat/src/lib.rs` captured during #39 vat standardization.
```
