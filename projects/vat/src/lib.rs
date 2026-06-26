// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-src-lib-rs.md#rust-source-unit
// CODEGEN-BEGIN
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
// CODEGEN-END
// SPEC-MANAGED: projects/vat/tech-design/logic/vat-td-ast-promote-remaining-grouped-source-units.md#rust-source-unit
// CODEGEN-BEGIN

// CODEGEN-END
