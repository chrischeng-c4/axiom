---
id: cap-source-sampler
summary: Source replay payload for projects/cap/src/sampler.rs
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

# Source TD: projects/cap/src/sampler.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `LoadSampler` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `MemorySampler` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `RssSampler` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `free_gb` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `load_per_core` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `new` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `rss_bytes` | public Rust symbol in `projects/cap/src/sampler.rs` |
| `total_gb` | public Rust symbol in `projects/cap/src/sampler.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
//! Thin wrapper around `sysinfo` so the rest of the crate doesn't
//! depend on it directly (makes the throttler easy to test with
//! synthetic numbers).
//!
//! Per-OS quirks:
//!
//! - **Linux**: `sysinfo::System::available_memory()` reads
//!   `/proc/meminfo`'s `MemAvailable`, which is the kernel's
//!   authoritative "how much we can allocate without going to swap".
//!   Use it directly.
//! - **macOS**: `available_memory()` is effectively `free_memory()`
//!   and reports ~0 on any active machine, because macOS aggressively
//!   parks RAM as inactive / compressed / file cache.
//!   `used_memory()` excludes those reclaimable pages, so
//!   `total - used` is a much better proxy for "memory I can hand
//!   out without paging".
//!
//! This split keeps the throttler's input meaningful on the platform
//! we actually develop on.

use std::collections::HashMap;

use sysinfo::{MemoryRefreshKind, Pid, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System};

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct MemorySampler {
    sys: System,
}

/// Per-process RSS lookup, scoped to a caller-provided PID list so we
/// don't pay the cost of scanning the entire process table each tick.
/// The throttler only needs RSS at kill time, but Slice 3 refreshes
/// every tick — N is typically ≤ 8 (one entry per active lease).
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct RssSampler {
    sys: System,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl RssSampler {
    pub fn new() -> Self {
        // Start with a bare System; we only ever poke processes.
        Self { sys: System::new() }
    }

    /// Refresh the named PIDs and return RSS in bytes for each one we
    /// could read. Dead / unknown PIDs are simply absent from the map.
    pub fn rss_bytes(&mut self, pids: &[i32]) -> HashMap<i32, u64> {
        if pids.is_empty() {
            return HashMap::new();
        }
        let pid_list: Vec<Pid> = pids
            .iter()
            .filter(|p| **p > 0)
            .map(|p| Pid::from(*p as usize))
            .collect();
        if pid_list.is_empty() {
            return HashMap::new();
        }
        // `remove_dead_processes = true` keeps the internal map from
        // growing across calls when lease PIDs come and go.
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::Some(&pid_list),
            true,
            ProcessRefreshKind::new().with_memory(),
        );
        pids.iter()
            .copied()
            .filter(|p| *p > 0)
            .filter_map(|p| {
                self.sys
                    .process(Pid::from(p as usize))
                    .map(|proc| (p, proc.memory()))
            })
            .collect()
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for RssSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// 1-minute load average normalized by core count. > 1.0 means the
/// machine is oversubscribed; cap's CPU pause floor is a fraction of
/// that (default 0.80 = "stop submitting once load > 80% of nproc").
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct LoadSampler {
    nproc: f64,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl LoadSampler {
    pub fn new() -> Self {
        // available_parallelism rounds physical+SMT cores the same way
        // every other tool the user thinks in does (htop, top -1). Fall
        // back to 1.0 to keep the math defined on weird platforms.
        let nproc = std::thread::available_parallelism()
            .map(|n| n.get() as f64)
            .unwrap_or(1.0);
        Self { nproc }
    }

    pub fn load_per_core(&self) -> f64 {
        // sysinfo's load_average is a static-method-style read of the
        // OS-wide rolling averages; doesn't need a System instance.
        let load = sysinfo::System::load_average();
        if self.nproc > 0.0 {
            load.one / self.nproc
        } else {
            load.one
        }
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for LoadSampler {
    fn default() -> Self {
        Self::new()
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl MemorySampler {
    pub fn new() -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );
        Self { sys }
    }

    pub fn free_gb(&mut self) -> f64 {
        self.sys.refresh_memory();
        bytes_to_gb(self.available_bytes())
    }

    /// Total installed RAM (GB). Stable for the life of the process,
    /// so callers cache the value at startup.
    pub fn total_gb(&mut self) -> f64 {
        self.sys.refresh_memory();
        bytes_to_gb(self.sys.total_memory())
    }

    #[cfg(target_os = "macos")]
    fn available_bytes(&self) -> u64 {
        self.sys
            .total_memory()
            .saturating_sub(self.sys.used_memory())
    }

    #[cfg(not(target_os = "macos"))]
    fn available_bytes(&self) -> u64 {
        self.sys.available_memory()
    }
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for MemorySampler {
    fn default() -> Self {
        Self::new()
    }
}

fn bytes_to_gb(bytes: u64) -> f64 {
    bytes as f64 / 1024.0 / 1024.0 / 1024.0
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/sampler.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#rust_source.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-sampler-rs-source-replay-superseded>"
```
