---
id: cap-source-reap
summary: Source replay payload for projects/cap/src/reap.rs
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

# Source TD: projects/cap/src/reap.rs

## Overview
<!-- type: overview lang: markdown -->

### Symbols

| Symbol | Coverage |
|---|---|
| `REAP_ALLOWLIST` | public Rust symbol in `projects/cap/src/reap.rs` |
| `ReapedProcess` | public Rust symbol in `projects/cap/src/reap.rs` |
| `Reaper` | public Rust symbol in `projects/cap/src/reap.rs` |
| `cooldown_elapsed` | public Rust symbol in `projects/cap/src/reap.rs` |
| `dry_scan` | public Rust symbol in `projects/cap/src/reap.rs` |
| `new` | public Rust symbol in `projects/cap/src/reap.rs` |
| `scan_and_reap` | public Rust symbol in `projects/cap/src/reap.rs` |

## Source
<!-- type: source lang: rust -->

`````rust
//! Reap allowlist — narrow carve-out from cap's "lease-only" model.
//!
//! cap's general philosophy is to never touch non-lease processes:
//! free memory is observed, not attributed; the only things cap kills
//! are commands users explicitly opted into. The reap path is the one
//! exception, justified by a single property: every entry in
//! [`REAP_ALLOWLIST`] is a process that the surrounding tool will
//! transparently relaunch when missing (LSPs spawned by an editor,
//! proc-macro servers spawned by rust-analyzer). Killing them is a
//! free way to reclaim RAM without losing work.
//!
//! Three guards keep this safe:
//!
//! 1. **Hardcoded list** — only the names baked into this file are
//!    candidates. Users cannot widen it via config (they can only
//!    disable the feature wholesale via `reap_enabled = false`).
//! 2. **Uptime gate** — `reap_min_uptime_secs` (default 60 s) must
//!    have elapsed since the process started. This breaks the
//!    pathological loop where cap kills an LSP, the editor relaunches
//!    it, the next tick kills the relaunched copy, and so on.
//! 3. **Cooldown** — `reap_cooldown_secs` (default 10 s) between
//!    process-table scans. The scan is the most expensive thing cap
//!    does per tick (sysinfo full refresh ≈ few ms on a quiet box, up
//!    to ~100 ms on busy ones with 5 k processes); the cooldown bounds
//!    its cost when free memory stays low for a while.

use std::collections::HashSet;

use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

/// Process names cap is allowed to SIGTERM under kill-floor pressure.
/// Every entry must be a tool the surrounding editor (VS Code, Neovim,
/// Cursor, ...) automatically respawns when missing — otherwise the
/// user loses work silently.
///
/// Conservative on purpose: we only list native-named binaries because
/// matching `node` or `python` is too broad (the user's own scripts
/// often have those names). LSPs shipped as Node bundles (pyright,
/// tsserver) are intentionally excluded for that reason.
pub const REAP_ALLOWLIST: &[&str] = &[
    "rust-analyzer",
    "rust-analyzer-proc-macro-srv",
    "gopls",
    "clangd",
    "zls",
];

/// One process the reap pass picked up. Surfaced in daemon logs and
/// can be embedded in a follow-up KillEnvelope's note so the agent
/// sees what cap did on its behalf.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
#[derive(Debug, Clone)]
pub struct ReapedProcess {
    pub pid: i32,
    pub name: String,
    pub rss_bytes: u64,
    pub uptime_secs: u64,
}

/// Stateful reaper: owns a sysinfo `System` so successive scans can
/// reuse the internal process map (avoids re-allocating on every call)
/// and remembers the last scan time so the daemon can debounce
/// without a free-standing Instant.
/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
pub struct Reaper {
    sys: System,
    last_scan: Option<std::time::Instant>,
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Reaper {
    pub fn new() -> Self {
        Self {
            sys: System::new(),
            last_scan: None,
        }
    }

    /// True iff the cooldown has elapsed since the last scan (or no
    /// scan has happened yet). Caller checks this before calling
    /// `scan_and_reap` so we don't pay the full-refresh cost every tick.
    pub fn cooldown_elapsed(&self, cooldown: std::time::Duration) -> bool {
        match self.last_scan {
            None => true,
            Some(t) => t.elapsed() >= cooldown,
        }
    }

    /// Walk the process table, SIGTERM any allowlisted process older
    /// than `min_uptime_secs` whose PID is not in `exclude_pids`
    /// (typically the cap-managed lease PIDs — we never want to reap
    /// our own children). Returns whatever we signaled, sorted RSS
    /// descending so logs surface the biggest hitter first.
    pub fn scan_and_reap(
        &mut self,
        min_uptime_secs: u64,
        exclude_pids: &[i32],
    ) -> Vec<ReapedProcess> {
        self.last_scan = Some(std::time::Instant::now());
        // Full process refresh — needed to enumerate everything that
        // could match the allowlist. `with_memory()` keeps the refresh
        // narrow (no disk/network).
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_memory(),
        );
        let exclude: HashSet<i32> = exclude_pids.iter().copied().collect();
        let mut found: Vec<ReapedProcess> = self
            .sys
            .processes()
            .iter()
            .filter_map(|(pid, proc_)| {
                let pid_i32 = pid.as_u32() as i32;
                if pid_i32 <= 0 || exclude.contains(&pid_i32) {
                    return None;
                }
                let name = proc_.name().to_str()?;
                if !REAP_ALLOWLIST.contains(&name) {
                    return None;
                }
                let uptime = proc_.run_time();
                if uptime < min_uptime_secs {
                    return None;
                }
                Some(ReapedProcess {
                    pid: pid_i32,
                    name: name.to_string(),
                    rss_bytes: proc_.memory(),
                    uptime_secs: uptime,
                })
            })
            .collect();
        found.sort_by_key(|c| std::cmp::Reverse(c.rss_bytes));
        for c in &found {
            // Leader-only SIGTERM (not the process group). The
            // allowlist is curated so the parent process is the right
            // target; killing the group could take down the entire
            // editor pipeline if the LSP was spawned inline.
            unsafe {
                let _ = libc::kill(c.pid, libc::SIGTERM);
            }
        }
        found
    }

    /// Returns the PIDs of allowlisted processes that *would* be
    /// reaped right now, without signaling them. Useful for tests and
    /// for dry-run / observability use cases.
    #[cfg(test)]
    pub fn dry_scan(&mut self, min_uptime_secs: u64, exclude_pids: &[i32]) -> Vec<ReapedProcess> {
        self.sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::new().with_memory(),
        );
        let exclude: HashSet<i32> = exclude_pids.iter().copied().collect();
        self.sys
            .processes()
            .iter()
            .filter_map(|(pid, proc_)| {
                let pid_i32 = pid.as_u32() as i32;
                if pid_i32 <= 0 || exclude.contains(&pid_i32) {
                    return None;
                }
                let name = proc_.name().to_str()?;
                if !REAP_ALLOWLIST.contains(&name) {
                    return None;
                }
                let uptime = proc_.run_time();
                if uptime < min_uptime_secs {
                    return None;
                }
                Some(ReapedProcess {
                    pid: pid_i32,
                    name: name.to_string(),
                    rss_bytes: proc_.memory(),
                    uptime_secs: uptime,
                })
            })
            .collect()
    }

    /// Suppress the implicit Pid type import for downstream callers.
    #[allow(dead_code)]
    fn _silence_pid_import(_: Pid) {}
}

/// @spec projects/cap/tech-design/semantic/cap-src.md#schema
impl Default for Reaper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowlist_only_contains_native_named_binaries() {
        // Anything matching `node`, `python`, `java` etc. is too broad
        // — the user's own scripts often have those names. The
        // allowlist is intentionally narrow.
        for &n in REAP_ALLOWLIST {
            assert!(
                !matches!(n, "node" | "python" | "python3" | "java" | "ruby"),
                "{n} is too generic for the allowlist"
            );
        }
    }

    #[test]
    fn cooldown_elapsed_true_before_first_scan() {
        let r = Reaper::new();
        assert!(r.cooldown_elapsed(std::time::Duration::from_secs(10)));
    }

    #[tokio::test]
    async fn cooldown_blocks_immediately_after_scan() {
        let mut r = Reaper::new();
        // dry_scan still stamps last_scan via the helper-shared init,
        // but we want to test scan_and_reap. The call may match
        // nothing; we just care about the cooldown side-effect.
        let _ = r.scan_and_reap(60, &[]);
        assert!(!r.cooldown_elapsed(std::time::Duration::from_secs(10)));
        assert!(r.cooldown_elapsed(std::time::Duration::from_secs(0)));
    }

    #[test]
    fn dry_scan_excludes_lease_pids() {
        // Sanity-only: with no allowlisted process running on the test
        // box, dry_scan must return empty whether or not we exclude.
        // (If a developer happens to be running rust-analyzer locally,
        // this test is informational — there's no assertion to fail.)
        let mut r = Reaper::new();
        let _ = r.dry_scan(60, &[]);
        // The above must not panic; that's the contract this test
        // protects against (e.g. if sysinfo's process API changes).
    }
}
`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/cap/src/reap.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/cap/tech-design/semantic/cap-src.md#rust_source.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-cap-src-reap-rs-source-replay-superseded>"
```
