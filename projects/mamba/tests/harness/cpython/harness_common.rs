//! Shared primitives for the `tests/harness/cpython/*` integration-test
//! binaries (#consolidate-harness-rs).
//!
//! Before this module each harness binary re-implemented the same handful of
//! low-level helpers — `mamba_bin()` (5 copies), the subprocess
//! spawn + `try_wait` timeout/kill loop (runner.rs / lib_test.rs, with
//! drifting timeout sources and poll intervals), the fixture SHA-256 +
//! recursive `collect_files` walker (status.rs / contract.rs / perf_pin.rs),
//! and the `python3` availability probes. This file is the single source of
//! truth for those primitives. #1981 additionally moved `perf_pin.rs`'s
//! CPython-baseline loading, host-affinity gate, and `/usr/bin/time`
//! measurement primitives here, so `perf_pin.rs` (the single-pin gate) and
//! `perf_gate_report.rs` (the full-enumeration report) can never drift on
//! what "the CPU/RSS ratio" or "a usable baseline" means.
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
use std::process::{Child, Command, Output, Stdio};
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use serde::Deserialize;

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
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mamba")
        })
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
                // Hidden directories are never fixture trees — `.cache/`
                // holds machine-local artifacts (results stores, the
                // materialized oracle-env venv with thousands of .py files)
                // that must not be collected.
                let hidden = path
                    .file_name()
                    .map(|n| n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false);
                if !hidden {
                    walk(out, &path, suffix);
                }
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
// Resolve the interpreter ONCE per harness process, in preference order:
//
//   1. `MAMBA_ORACLE_PYTHON` — explicit override, always wins.
//   2. `tests/cpython/.cache/oracle-env/bin/python3` — the uv-materialized
//      oracle environment (CPython 3.12 + the pinned 3p packages from
//      tests/harness/cpython/config/oracle-env/requirements.txt), so
//      3rd-libs fixtures can satisfy the "exits 0 under CPython" contract.
//   3. The PATH-resolved `python3`'s own `sys.executable` (asked from the
//      temp dir, matching the sandboxed fixture spawn context), falling
//      back to plain "python3" (original PATH semantics) on any failure.
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
            let oracle_env = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/cpython/.cache/oracle-env/bin/python3");
            if oracle_env.is_file() {
                return oracle_env;
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
/// per-policy and acts as the CAP of an exponential backoff that starts at
/// 1ms (see [`wait_with_timeout`]), so the runner's 20ms and the seed
/// runner's 50ms remain each caller's worst-case cadence.
#[derive(Clone, Copy)]
pub struct TimeoutPolicy {
    timeout: Duration,
    poll_interval: Duration,
}

impl TimeoutPolicy {
    /// The single env-var lookup. Reads `var_name` as a positive `u64`
    /// seconds value, falling back to `default_secs` when unset, unparseable,
    /// or `0`. The poll-interval cap defaults to 20ms (the conformance
    /// runner's historical cadence) and can be overridden with
    /// [`Self::with_poll_interval`].
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

    /// Set the spawn-loop poll-interval cap. Lets each caller preserve its
    /// historical worst-case cadence (runner.rs = 20ms, lib_test.rs = 50ms).
    pub fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// The resolved timeout budget.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// The cap on the backoff interval between `try_wait` checks.
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
pub fn wait_with_timeout(mut child: Child, policy: TimeoutPolicy) -> std::io::Result<WaitOutcome> {
    let start = Instant::now();
    // Exponential backoff from 1ms up to the policy's poll interval (the
    // cap). A fixed 20ms cadence only observes exits on poll ticks, wasting
    // ~10ms per child on average (most conformance children finish within a
    // few tens of ms; two children per fixture ≈ 40-70s across a full run).
    // Brief fast polling costs negligible harness CPU next to the children's
    // own work, and the cap preserves each caller's historical worst-case
    // cadence for long-running children.
    let mut backoff = Duration::from_millis(1).min(policy.poll_interval);
    loop {
        match child.try_wait()? {
            Some(_status) => {
                return Ok(WaitOutcome::Finished(child.wait_with_output()?));
            }
            None if start.elapsed() > policy.timeout => {
                // Kill the child's whole process GROUP first (spawn sites
                // place children in their own group via setpgid in pre_exec):
                // a grandchild inheriting the stdout/stderr pipes would
                // otherwise survive the child's kill and wait_with_output
                // would block forever on pipes that never reach EOF. For
                // children that are not group leaders the group kill fails
                // harmlessly and the plain kill below still applies.
                #[cfg(unix)]
                unsafe {
                    let _ = libc::kill(-(child.id() as libc::pid_t), libc::SIGKILL);
                }
                let _ = child.kill();
                return Ok(WaitOutcome::TimedOut(child.wait_with_output()?));
            }
            None => {
                std::thread::sleep(backoff);
                backoff = (backoff * 2).min(policy.poll_interval);
            }
        }
    }
}

// ── perf-pin measurement primitives ───────────────────────────────
//
// Shared by `perf_pin.rs` (the single-pin datatest gate, `harness = false`,
// `test = false`, one `Trial` per TOML under `config/perf/pins/`) and
// `perf_gate_report.rs` (#1981: the full-enumeration report — every pin
// gets a verdict row in one run, never masked by an earlier pin's panic).
// Moved here (rather than duplicated a second time) so the two binaries can
// never drift on what "the CPU/RSS ratio" or "a usable baseline" means.

/// One `tests/harness/cpython/config/perf/pins/*.toml` entry.
#[derive(Debug, Deserialize)]
pub struct Pin {
    pub issue: u64,
    pub lib: String,
    pub fixture: String,
    pub floor: f64,
    pub samples: usize,
    #[serde(default)]
    pub prereq_imports: Vec<String>,
    /// Peak-RSS floor; the contract gate requires every pin to set it. It
    /// matches cross_runtime.rs FLOOR semantics (mem_ratio = cpython_rss /
    /// mamba_rss must be >= mem_floor, i.e. mamba uses no more peak memory
    /// than CPython at floor 1.0x).
    #[serde(default)]
    pub mem_floor: Option<f64>,
    /// Per-pin external wall-clock timeout override, in seconds (#964).
    /// Defaults to `DEFAULT_PIN_TIMEOUT_SECS` when absent. Guards against a
    /// fixture hang wedging the whole gate.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
}

/// Default per-pin external wall-clock timeout (#964): a hung fixture must
/// not wedge the whole perf-pin gate with an orphaned 100%-CPU grandchild.
/// Overridable per-pin via the `timeout_secs` TOML field.
pub const DEFAULT_PIN_TIMEOUT_SECS: u64 = 120;
/// #1024: ship-profile mamba carries a fixed runtime RSS floor of about 26 MB
/// before a fixture's own allocation pattern shows up. Small perf pins like
/// `argparse_1442`, `googleapis_common_protos_1512`, and `grpclib_1514`
/// structurally fail a raw `cpython_rss / mamba_rss` gate even when mamba's
/// workload-attributed RSS is below CPython's, so the mem gate subtracts this
/// fixed allowance from mamba before applying each pin's `mem_floor`.
pub const MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES: u64 = 26_000_000;

#[derive(Debug, Clone, Copy)]
pub struct Measurement {
    pub cpu_time_ns: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Clone, Copy)]
pub struct MemGateEvaluation {
    pub raw_ratio: f64,
    pub adjusted_ratio: f64,
    pub effective_mamba_rss_bytes: u64,
}

#[derive(Debug, Deserialize)]
pub struct CpythonPerfBaseline {
    pub pin_path: String,
    pub fixture_sha256: String,
    pub samples: usize,
    // Retained for sqlite-row deserialization compatibility; no longer used by
    // the gate (D5.2 measures external CPU time, not the fixture marker).
    #[allow(dead_code)]
    pub internal_time_ns: u64,
    pub cpu_time_ns: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
    pub python: String,
    pub captured_at_unix: u64,
    /// Recording host (#966, `platform.node()`). #1981 promotes this from a
    /// warn-only signal to an enforced gate: see [`baseline_is_same_host`] /
    /// [`load_same_host_baseline`]. `#[serde(default)]` so pre-#966 baseline
    /// rows (column added via `ALTER TABLE`, NULL on old rows) still
    /// deserialize — and, per #1981, are treated exactly like a cross-host
    /// row (honest degradation: a row with no recorded host is not
    /// verifiably this host's, so it is not used for ratio grading either).
    #[serde(default)]
    pub host: Option<String>,
}

/// Best-effort local hostname (#966), compared against a loaded baseline's
/// `host` to gate cross-host CPU/RSS ratio grading (#1981). Mirrors Python's
/// `platform.node()` closely enough for an equality check (both ultimately
/// `gethostname(2)`).
#[cfg(unix)]
pub fn local_hostname() -> Option<String> {
    let mut buf = vec![0u8; 256];
    let rc = unsafe { libc::gethostname(buf.as_mut_ptr() as *mut libc::c_char, buf.len()) };
    if rc != 0 {
        return None;
    }
    let len = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    String::from_utf8(buf[..len].to_vec()).ok()
}

#[cfg(not(unix))]
pub fn local_hostname() -> Option<String> {
    None
}

/// True iff `baseline` was recorded on this exact host (#966 host, #1981
/// enforcement). Conservative in every ambiguous direction: a `None`
/// baseline host (pre-#966 legacy row), an undetectable local hostname, and
/// an actual hostname mismatch are ALL "not the same host" — CPU/RSS ratios
/// are only ever graded against a baseline this exact machine recorded,
/// since they are not portable across machines (different CPUs, thermal
/// state, background load).
pub fn baseline_is_same_host(baseline: &CpythonPerfBaseline) -> bool {
    match (baseline.host.as_deref(), local_hostname().as_deref()) {
        (Some(baseline_host), Some(local_host)) => baseline_host == local_host,
        _ => false,
    }
}

/// Why [`load_same_host_baseline`] could not return a usable baseline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoBaselineReason {
    /// No baseline row exists for this pin at all.
    Missing,
    /// A baseline row exists but fails [`baseline_is_same_host`] — recorded
    /// on a different host, or a legacy pre-#966 row with no recorded host.
    CrossHost,
}

impl NoBaselineReason {
    pub fn as_str(self) -> &'static str {
        match self {
            NoBaselineReason::Missing => "missing",
            NoBaselineReason::CrossHost => "cross-host",
        }
    }
}

/// Load `toml_path`'s CPython baseline and apply host-affinity (#1981): a
/// baseline that exists but was not recorded on this exact host is never
/// returned, since CPU/RSS ratios recorded elsewhere are not portable here.
/// `Ok` only for a present, same-host row; `Err` distinguishes "no row at
/// all" (still eligible for a live-python3 fallback measurement — see
/// `perf_pin.rs::run_pin` and `perf_gate_report.rs::evaluate_pin`) from "a
/// row exists but isn't this host's" (never graded — #1981 requires
/// `no-baseline("cross-host")`, never a ratio, for that case).
pub fn load_same_host_baseline(toml_path: &Path) -> Result<CpythonPerfBaseline, NoBaselineReason> {
    match load_cpython_baseline(toml_path) {
        Some(baseline) if baseline_is_same_host(&baseline) => Ok(baseline),
        Some(_) => Err(NoBaselineReason::CrossHost),
        None => Err(NoBaselineReason::Missing),
    }
}

/// Put a spawned child in its own process group before exec, so a timeout
/// kill can `killpg` the whole spawn tree. Mirrors `runner.rs`'s
/// `apply_child_limits`: when the child is `/usr/bin/time`-wrapped, the
/// actual measured process is a *grandchild* that inherits the group via
/// fork (and is not itself a group leader), so killing the group takes down
/// both.
#[cfg(unix)]
pub fn own_process_group(command: &mut Command) {
    use std::os::unix::process::CommandExt;
    unsafe {
        command.pre_exec(|| {
            let _ = libc::setpgid(0, 0);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
pub fn own_process_group(_command: &mut Command) {}

/// Parse a `/usr/bin/time` stderr blob for the child's peak RSS in bytes.
/// macOS BSD `time -l` reports bytes; Linux GNU `time -v` reports kbytes.
/// Returns None if no recognised line is present. Mirrors the same parser
/// in `benches/3p/cross_runtime.rs`.
pub fn parse_peak_rss(stderr: &str) -> Option<u64> {
    for line in stderr.lines() {
        let trimmed = line.trim();
        // macOS BSD `time -l`: "<n>  maximum resident set size" (bytes).
        if let Some(rest) = trimmed.strip_suffix("maximum resident set size") {
            if let Ok(v) = rest.trim().parse::<u64>() {
                return Some(v);
            }
        }
        // Linux GNU `time -v`: "Maximum resident set size (kbytes): <n>".
        if let Some(rest) = trimmed.strip_prefix("Maximum resident set size") {
            if let Some(num) = rest.split(':').nth(1) {
                if let Ok(v) = num.trim().parse::<u64>() {
                    return Some(v.saturating_mul(1024));
                }
            }
        }
    }
    None
}

/// Parse CPU time reported by `/usr/bin/time`.
///
/// macOS BSD `time -l` emits "<real> real <user> user <sys> sys"; Linux GNU
/// `time -v` emits separate user/sys lines. The returned value is user+sys
/// CPU time in nanoseconds.
pub fn parse_cpu_time_ns(stderr: &str) -> Option<u64> {
    let mut linux_user: Option<f64> = None;
    let mut linux_sys: Option<f64> = None;

    for line in stderr.lines() {
        let trimmed = line.trim();
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() == 6 && parts[1] == "real" && parts[3] == "user" && parts[5] == "sys" {
            let user = parts[2].parse::<f64>().ok()?;
            let sys = parts[4].parse::<f64>().ok()?;
            return Some(((user + sys) * 1_000_000_000.0) as u64);
        }

        if let Some(rest) = trimmed.strip_prefix("User time (seconds):") {
            linux_user = rest.trim().parse::<f64>().ok();
        } else if let Some(rest) = trimmed.strip_prefix("System time (seconds):") {
            linux_sys = rest.trim().parse::<f64>().ok();
        }
    }

    match (linux_user, linux_sys) {
        (Some(user), Some(sys)) => Some(((user + sys) * 1_000_000_000.0) as u64),
        _ => None,
    }
}

#[cfg(unix)]
pub fn timeval_to_ns(tv: libc::timeval) -> u64 {
    (tv.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add((tv.tv_usec as u64).saturating_mul(1_000))
}

#[cfg(unix)]
pub fn child_cpu_time_ns() -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    let rc = unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, usage.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let usage = unsafe { usage.assume_init() };
    Some(timeval_to_ns(usage.ru_utime).saturating_add(timeval_to_ns(usage.ru_stime)))
}

#[cfg(not(unix))]
pub fn child_cpu_time_ns() -> Option<u64> {
    None
}

/// Build the `/usr/bin/time` argv prefix for the current platform. macOS uses
/// BSD `-l`; everywhere else assume GNU `-v`. Returns None if `/usr/bin/time`
/// does not exist (caller falls back to plain `Command::new(cmd)` and drops
/// RSS/CPU measurement — external resource gating is best-effort by design).
pub fn time_wrapper() -> Option<(&'static str, &'static str)> {
    let p = Path::new("/usr/bin/time");
    if !p.exists() {
        return None;
    }
    if cfg!(target_os = "macos") {
        Some(("/usr/bin/time", "-l"))
    } else {
        Some(("/usr/bin/time", "-v"))
    }
}

/// Run `cmd args...` once, optionally wrapped by `/usr/bin/time` so the
/// child's CPU time and peak RSS can be parsed. `timeout` bounds the whole
/// external wall-clock run (#964): the child is spawned as its own process
/// group leader and, on timeout, the whole group is `killpg`'d so a hang
/// never leaks an orphaned 100%-CPU grandchild or wedges the gate forever.
pub fn run_once_with_metrics(cmd: &str, args: &[&str], timeout: Duration) -> Measurement {
    let cpu_before = child_cpu_time_ns();
    let (spawn_cmd, spawn_args, wrapped): (&str, Vec<&str>, bool) =
        if let Some((time_bin, flag)) = time_wrapper() {
            let mut all_args: Vec<&str> = Vec::with_capacity(args.len() + 2);
            all_args.push(flag);
            all_args.push(cmd);
            all_args.extend(args.iter().copied());
            (time_bin, all_args, true)
        } else {
            (cmd, args.to_vec(), false)
        };

    let mut command = Command::new(spawn_cmd);
    command
        .args(&spawn_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    own_process_group(&mut command);
    let child = command
        .spawn()
        .unwrap_or_else(|e| panic!("failed to spawn {cmd}: {e}"));

    let policy = TimeoutPolicy::fixed(timeout);
    let wait_result = wait_with_timeout(child, policy);
    let cpu_after = child_cpu_time_ns();
    let out = match wait_result {
        Ok(WaitOutcome::Finished(output)) => output,
        Ok(WaitOutcome::TimedOut(output)) => panic!(
            "{cmd} TIMEOUT after {}s (killed process group); args={:?}\nstdout={}\nstderr={}",
            policy.timeout().as_secs(),
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ),
        Err(e) => panic!("failed to wait for {cmd}: {e}"),
    };
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "{cmd} failed: stdout={} stderr={}",
        stdout,
        stderr
    );
    // When wrapped, the child's stderr is interleaved with `time`'s memory
    // report; the exit status is the child's (preserved by `time`).
    let wrapper_cpu = if wrapped {
        parse_cpu_time_ns(&stderr)
    } else {
        None
    };
    let rusage_cpu = match (cpu_before, cpu_after) {
        (Some(before), Some(after)) => after.checked_sub(before),
        _ => None,
    };
    let peak_rss_bytes = if wrapped {
        parse_peak_rss(&stderr)
    } else {
        None
    };
    Measurement {
        cpu_time_ns: rusage_cpu.filter(|value| *value > 0).or(wrapper_cpu),
        peak_rss_bytes,
    }
}

pub fn median(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

pub fn measure_n(cmd: &str, args: &[&str], n: usize, timeout: Duration) -> Measurement {
    assert!(n > 0, "samples must be >= 1");
    let mut cpu_samples = Vec::with_capacity(n);
    let mut rss_samples = Vec::with_capacity(n);

    for _ in 0..n {
        let measurement = run_once_with_metrics(cmd, args, timeout);
        if let Some(cpu) = measurement.cpu_time_ns {
            cpu_samples.push(cpu);
        }
        if let Some(rss) = measurement.peak_rss_bytes {
            rss_samples.push(rss);
        }
    }

    Measurement {
        cpu_time_ns: if cpu_samples.is_empty() {
            None
        } else {
            Some(median(&mut cpu_samples))
        },
        peak_rss_bytes: rss_samples.into_iter().min(),
    }
}

pub fn baseline_db() -> PathBuf {
    std::env::var("MAMBA_CPYTHON_PERF_BASELINE_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/cpython/.cache/perf/cpython_baseline.sqlite")
        })
}

pub fn baseline_required() -> bool {
    std::env::var("MAMBA_REQUIRE_CPYTHON_PERF_BASELINE")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "required"
            )
        })
        .unwrap_or(false)
}

pub fn baseline_tool() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/harness/cpython/tools/perf_baseline.py")
}

pub fn load_cpython_baseline(toml_path: &Path) -> Option<CpythonPerfBaseline> {
    let db = baseline_db();
    let required = baseline_required();
    if !db.exists() {
        assert!(
            !required,
            "CPython perf baseline DB missing: {}. Run `python3 tests/harness/cpython/tools/perf_baseline.py record` first.",
            db.display()
        );
        return None;
    }

    let output = Command::new(python3_bin())
        .arg(baseline_tool())
        .arg("--db")
        .arg(&db)
        .arg("get")
        .arg("--pin")
        .arg(toml_path)
        .output()
        .unwrap_or_else(|err| panic!("failed to query CPython perf baseline: {err}"));

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Some(serde_json::from_str(&stdout).unwrap_or_else(|err| {
            panic!(
                "failed to parse CPython perf baseline JSON for {}: {err}\nstdout={stdout}",
                toml_path.display()
            )
        }));
    }

    if output.status.code() == Some(2) {
        assert!(
            !required,
            "CPython perf baseline row missing for {} in {}. Run `python3 tests/harness/cpython/tools/perf_baseline.py record --pin {}` first.",
            toml_path.display(),
            db.display(),
            toml_path.display()
        );
        return None;
    }

    panic!(
        "CPython perf baseline query failed for {}: stdout={} stderr={}",
        toml_path.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn cpython_measurement_from_baseline(baseline: &CpythonPerfBaseline) -> Measurement {
    Measurement {
        cpu_time_ns: baseline.cpu_time_ns,
        peak_rss_bytes: baseline.peak_rss_bytes,
    }
}

pub fn evaluate_mem_gate(cpython_rss_bytes: u64, mamba_rss_bytes: u64) -> MemGateEvaluation {
    let effective_mamba_rss_bytes =
        mamba_rss_bytes.saturating_sub(MAMBA_FIXED_RUNTIME_RSS_FLOOR_BYTES);
    let adjusted_ratio = if effective_mamba_rss_bytes == 0 {
        f64::INFINITY
    } else {
        cpython_rss_bytes as f64 / effective_mamba_rss_bytes as f64
    };
    MemGateEvaluation {
        raw_ratio: cpython_rss_bytes as f64 / mamba_rss_bytes as f64,
        adjusted_ratio,
        effective_mamba_rss_bytes,
    }
}
