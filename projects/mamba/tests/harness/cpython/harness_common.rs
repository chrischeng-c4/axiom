//! Shared primitives for the `tests/harness/cpython/*` integration-test
//! binaries (#consolidate-harness-rs).
//!
//! Before this module each harness binary re-implemented the same handful of
//! low-level helpers — `mamba_bin()` (5 copies), the subprocess
//! spawn + `try_wait` timeout/kill loop (runner.rs / lib_test.rs, with
//! drifting timeout sources and poll intervals), the fixture SHA-256 +
//! recursive `collect_files` walker (status.rs / contract.rs / perf_pin.rs),
//! and the `python3` availability probes. This file is the single source of
//! truth for those primitives.
//!
//! It is wired into each consuming binary with
//!
//! ```ignore
//! #[path = "harness_common.rs"]
//! mod common;
//! ```
//!
//! the same sibling-include convention the umbrella runners
//! (`tests/pkgmgr/runner.rs`, `tests/mambalibs/runner.rs`) already use.
//!
//! IMPORTANT — this is a *consolidation*, not a behavior change. Every export
//! preserves the exact semantics of the copy it replaces:
//!   * `mamba_bin()` keeps the `option_env!` + `target/debug/mamba` fallback
//!     (a superset of the `env!`-only copies; under `cargo test` the env var
//!     is always present so the fallback path is never taken).
//!   * `collect_files()` guards on `root.exists()` (status.rs behavior); every
//!     contract.rs call site passes a directory that exists on disk, so the
//!     guard is never exercised there and the panic-on-missing-dir failure
//!     mode is unchanged for the cases that actually run.
//!   * The spawn/timeout loop is shared but each caller still supplies its own
//!     timeout duration, poll interval, and result mapping, so the 30s vs 60s
//!     budgets and the `Err`-vs-`Outcome` mappings are untouched.
//!
//! Not every harness helper is shared: the per-fixture-class runners
//! (real_world.rs's `collect_real_world_scripts`, runner.rs's directive
//! parsing / type-strict classification, lib_test.rs's outcome classification)
//! stay where they are — only the genuinely duplicated primitives move here.

#![allow(dead_code)]

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

// ── mamba binary location ─────────────────────────────────────────
//
// `CARGO_BIN_EXE_mamba` is injected by Cargo into every integration-test
// binary in this crate, so the `option_env!` branch is taken in practice.
// The `target/debug/mamba` fallback preserves the runner.rs / runtime_
// shutdown.rs copies' ability to run outside the cargo-injected env; it is a
// strict superset of the `env!("CARGO_BIN_EXE_mamba")` copies in
// perf_pin.rs / lib_test.rs / real_world.rs.
pub fn mamba_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_mamba")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mamba"))
}

// ── recursive fixture collection ──────────────────────────────────
//
// Recursively walks `root`, collecting every file whose path ends with
// `suffix`, sorted for deterministic ordering. Returns an empty vec when
// `root` does not exist (status.rs behavior); contract.rs only ever passes
// roots that exist on disk, so its panic-on-unreadable-dir behavior for the
// running cases is preserved.
pub fn collect_files(root: &Path, suffix: &str) -> Vec<PathBuf> {
    fn walk(out: &mut Vec<PathBuf>, dir: &Path, suffix: &str) {
        let entries = std::fs::read_dir(dir)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", dir.display()));
        for entry in entries {
            let path = entry.expect("read_dir entry").path();
            if path.is_dir() {
                walk(out, &path, suffix);
            } else if path.to_string_lossy().ends_with(suffix) {
                out.push(path);
            }
        }
    }

    let mut out = Vec::new();
    if root.exists() {
        walk(&mut out, root, suffix);
    }
    out.sort();
    out
}

// ── fixture content hashing ───────────────────────────────────────
//
// Streaming SHA-256 of a file's bytes, formatted lowercase-hex. This is the
// canonical form; `fixture_sha256_opt` wraps it for the status.rs call site
// that wants `Option<String>` (it silently drops unreadable files into the
// "no hash" bucket rather than surfacing the IO error).
pub fn fixture_sha256(path: &Path) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// `Option`-returning view of [`fixture_sha256`]: `None` on any IO error.
/// Matches the status.rs reporter, which treats an unreadable fixture as
/// "no hash available" rather than aborting the whole status pass.
pub fn fixture_sha256_opt(path: &Path) -> Option<String> {
    fixture_sha256(path).ok()
}

// ── oracle interpreter location ───────────────────────────────────
//
// `Command::new("python3")` re-resolves through $PATH on every spawn; on
// pyenv machines that lands on the bash shim, which costs ~470ms/exec vs
// ~25ms for the real binary (measured ~65% of a full conformance run).
// Resolve the interpreter ONCE per harness process: honor
// `MAMBA_ORACLE_PYTHON` when set, else ask the PATH-resolved `python3` for
// its `sys.executable` (from the temp dir, so pyenv version selection
// matches the sandboxed fixture spawns, which also run under $TMPDIR), and
// fall back to plain "python3" (original PATH semantics) if resolution
// fails. Same interpreter as before — just reached without the shim tax.
pub fn python3_bin() -> &'static Path {
    static PYTHON3: OnceLock<PathBuf> = OnceLock::new();
    PYTHON3
        .get_or_init(|| {
            if let Ok(overridden) = std::env::var("MAMBA_ORACLE_PYTHON") {
                let overridden = overridden.trim();
                if !overridden.is_empty() {
                    return PathBuf::from(overridden);
                }
            }
            let resolved = Command::new("python3")
                .args(["-c", "import sys; print(sys.executable)"])
                .current_dir(std::env::temp_dir())
                .output();
            match resolved {
                Ok(out) if out.status.success() => {
                    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    if path.is_empty() {
                        PathBuf::from("python3")
                    } else {
                        PathBuf::from(path)
                    }
                }
                _ => PathBuf::from("python3"),
            }
        })
        .as_path()
}

// ── python3 availability probes ───────────────────────────────────

/// True iff the resolved oracle interpreter runs `--version` with exit 0.
pub fn python3_available() -> bool {
    Command::new(python3_bin())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// True iff `<oracle python3> -c "import <module>"` exits 0.
pub fn python3_can_import(module: &str) -> bool {
    Command::new(python3_bin())
        .args(["-c", &format!("import {module}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

// ── timeout policy + bounded subprocess wait ──────────────────────

/// Outcome of [`wait_with_timeout`]: either the child finished on its own
/// and we collected its `Output`, or the budget elapsed and we killed it
/// (still collecting whatever output had been buffered).
pub enum WaitOutcome {
    /// The child exited before the timeout. Carries its captured `Output`.
    Finished(Output),
    /// The budget elapsed; the child was killed. Carries the (partial)
    /// captured `Output` so the caller can include stdout/stderr in its
    /// timeout report.
    TimedOut(Output),
}

/// A single-source-of-truth timeout budget.
///
/// `TimeoutPolicy::from_env` performs the ONE env-var lookup
/// (`MAMBA_CONFORMANCE_TIMEOUT_SECS`) used by the conformance runner; values
/// that are non-numeric or `0` fall back to the supplied default. Callers
/// with a fixed budget (e.g. lib_test.rs's 60s seed budget) use
/// `TimeoutPolicy::fixed` and never read the env. The poll interval is
/// per-policy so the runner's 20ms cadence and the seed runner's 50ms cadence
/// are both preserved exactly.
#[derive(Clone, Copy)]
pub struct TimeoutPolicy {
    timeout: Duration,
    poll_interval: Duration,
}

impl TimeoutPolicy {
    /// The single env-var lookup. Reads `var_name` as a positive `u64`
    /// seconds value, falling back to `default_secs` when unset, unparseable,
    /// or `0`. The poll interval defaults to 20ms (the conformance runner's
    /// cadence) and can be overridden with [`Self::with_poll_interval`].
    pub fn from_env(var_name: &str, default_secs: u64) -> Self {
        let secs = std::env::var(var_name)
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(default_secs);
        Self {
            timeout: Duration::from_secs(secs),
            poll_interval: Duration::from_millis(20),
        }
    }

    /// A fixed budget with no env lookup. Poll interval defaults to 20ms;
    /// override with [`Self::with_poll_interval`].
    pub fn fixed(timeout: Duration) -> Self {
        Self {
            timeout,
            poll_interval: Duration::from_millis(20),
        }
    }

    /// Set the spawn-loop poll interval. Lets each caller preserve its
    /// historical cadence (runner.rs = 20ms, lib_test.rs = 50ms).
    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// The resolved timeout budget.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The poll interval between `try_wait` checks.
    pub fn poll_interval(&self) -> Duration {
        self.poll_interval
    }
}

/// Drive an already-spawned `child` to completion under `policy`, polling
/// with `try_wait` and killing it if the budget elapses. This is the shared
/// body of the previously-duplicated `spawn_mamba` / `spawn_python` /
/// `run_seed` loops; each caller keeps its own pre-spawn `Command` setup and
/// its own mapping of [`WaitOutcome`] into the caller's error/outcome type.
///
/// Returns an IO error only if `try_wait` itself fails or the post-mortem
/// `wait_with_output` fails — i.e. the same `Err` cases the old loops
/// surfaced. A normal exit yields `WaitOutcome::Finished`; an elapsed budget
/// yields `WaitOutcome::TimedOut`.
pub fn wait_with_timeout(
    mut child: Child,
    policy: TimeoutPolicy,
) -> std::io::Result<WaitOutcome> {
    let start = Instant::now();
    loop {
        match child.try_wait()? {
            Some(_status) => {
                return Ok(WaitOutcome::Finished(child.wait_with_output()?));
            }
            None if start.elapsed() > policy.timeout => {
                let _ = child.kill();
                return Ok(WaitOutcome::TimedOut(child.wait_with_output()?));
            }
            None => std::thread::sleep(policy.poll_interval),
        }
    }
}
