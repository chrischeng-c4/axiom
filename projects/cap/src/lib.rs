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

// Client-side primitives live in this crate so the daemon, CLI, and any cap
// library consumers share the same wire protocol, state paths, and supervised
// run helper.

pub use daemon::is_running;

pub mod cli;
pub mod client;
pub mod command_planner;
pub mod config;
pub mod daemon;
pub mod eventlog;
pub mod hook;
pub mod hook_install;
pub mod managed_run;
pub mod paths;
pub mod protocol;
pub mod reap;
pub mod sampler;
pub mod supervisor;
pub mod throttle;
// CODEGEN-END
