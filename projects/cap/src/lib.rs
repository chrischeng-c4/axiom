// SPEC-MANAGED: projects/cap/tech-design/semantic/cap-src.md#exports
// CODEGEN-BEGIN
//! cap — local resource-protection daemon + CLI wrapper.
//!
//! Goal: stop heavy commands (`cargo test`, `uv run`, ...) from eating
//! the whole machine. NOT about environment isolation — only about
//! live memory pressure.
//!
//! Architecture:
//!
//! ```text
//!   cap <cmd>              cap <cmd>             cap <cmd>
//!       │                      │                     │
//!       └──── Acquire ─────────┴──── Spawned ────────┘
//!                                  │
//!                                  ▼
//!                            cap daemon
//!                            (UDS RPC + sampler loop)
//!                                  │
//!               every sample_interval_ms:
//!                 free = OS available_memory()
//!                 free ≥ floor  → SIGCONT oldest paused
//!                 free < floor  → SIGSTOP newest running
//!                 only one left → SIGKILL it
//! ```
//!
//! No declared budgets. No per-command estimates. The OS's idea of
//! "free memory" is the only input.

// Client-side primitives now live in `cap-core` (shared with vat and other
// tools that register leases). Re-exported so `crate::{paths,protocol,client,
// supervisor}` keep resolving across this crate's daemon/CLI code.

pub use cap_core::{client, paths, protocol, supervisor};

pub mod cli;
pub mod config;
pub mod daemon;
pub mod eventlog;
pub mod hook;
pub mod hook_install;
pub mod reap;
pub mod sampler;
pub mod throttle;
// CODEGEN-END
