---
id: cap-source-lib
summary: Source replay payload for projects/cap/src/lib.rs
fill_sections: [source, changes]
capability_refs:
  - id: command-lease-throttling
    role: primary
    gap: lease-admission-and-process-supervision
    claim: lease-admission-and-process-supervision
    coverage: full
    rationale: "The cap source group implements command wrapping, daemon leases, throttling, and structured run outcomes."
  - id: agent-hook-installation
    role: primary
    gap: claude-and-codex-hook-installation
    claim: claude-and-codex-hook-installation
    coverage: full
    rationale: "The cap source group contains the installer logic for Claude Code and Codex CLI hook registration."
  - id: agent-hook-installation
    role: primary
    gap: hook-payload-rewrite-adapters
    claim: hook-payload-rewrite-adapters
    coverage: full
    rationale: "The same source group contains hook rewrite and hook installation adapters."
  - id: command-lease-throttling
    role: primary
    gap: memory-and-cpu-pressure-sampling
    claim: memory-and-cpu-pressure-sampling
    coverage: full
    rationale: "The sampler and throttle modules in this source group implement memory and CPU pressure sampling."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: daemon-process-lifecycle
    claim: daemon-process-lifecycle
    coverage: full
    rationale: "The daemon, status, wait, and ping surfaces are implemented in this source group."
  - id: daemon-lifecycle-and-status
    role: primary
    gap: cli-status-and-wait-surfaces
    claim: cli-status-and-wait-surfaces
    coverage: full
    rationale: "The CLI module in this source group implements status and wait command surfaces."
  - id: config-logging-and-reap-policy
    role: primary
    gap: configuration-defaults-and-compatibility
    claim: configuration-defaults-and-compatibility
    coverage: full
    rationale: "Configuration, JSONL logging, and reap policy modules live in this source group."
  - id: config-logging-and-reap-policy
    role: primary
    gap: run-log-persistence
    claim: run-log-persistence
    coverage: full
    rationale: "The event log module in this source group implements JSONL run-log persistence."
  - id: config-logging-and-reap-policy
    role: primary
    gap: reap-allowlist-policy
    claim: reap-allowlist-policy
    coverage: full
    rationale: "The reap module in this source group implements bounded allowlist-based process reaping."
---

# Source TD: projects/cap/src/lib.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `cli` | public Rust symbol in `projects/cap/src/lib.rs` |
| `config` | public Rust symbol in `projects/cap/src/lib.rs` |
| `daemon` | public Rust symbol in `projects/cap/src/lib.rs` |
| `eventlog` | public Rust symbol in `projects/cap/src/lib.rs` |
| `hook` | public Rust symbol in `projects/cap/src/lib.rs` |
| `hook_install` | public Rust symbol in `projects/cap/src/lib.rs` |
| `reap` | public Rust symbol in `projects/cap/src/lib.rs` |
| `sampler` | public Rust symbol in `projects/cap/src/lib.rs` |
| `throttle` | public Rust symbol in `projects/cap/src/lib.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
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
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/lib.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#exports.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-lib-rs-source-replay-superseded>"
```
