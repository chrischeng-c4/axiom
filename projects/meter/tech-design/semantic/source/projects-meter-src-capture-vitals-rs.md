---
id: projects-meter-src-capture-vitals-rs
fill_sections: [overview, rust-source-unit, changes]
capability_refs:
  - id: runtime-resource-attribution
    role: primary
    gap: capture-vitals-and-measurement-contract
    claim: capture-vitals-and-measurement-contract
    coverage: full
    rationale: "The capture vitals implementation owns meter.toml parsing, wait4/rusage resource capture, measurement windows, and gate findings."
---

# Standardized projects/meter/src/capture/vitals.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/meter/src/capture/vitals.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `CaptureOutcome` | projects/meter/src/capture/vitals.rs | struct | pub | 184 |  |
| `GateConfig` | projects/meter/src/capture/vitals.rs | struct | pub | 93 |  |
| `Level` | projects/meter/src/capture/vitals.rs | enum | pub | 47 |  |
| `MeterConfig` | projects/meter/src/capture/vitals.rs | struct | pub | 105 |  |
| `Vitals` | projects/meter/src/capture/vitals.rs | struct | pub | 156 |  |
| `WindowOpts` | projects/meter/src/capture/vitals.rs | struct | pub | 169 |  |
| `as_str` | projects/meter/src/capture/vitals.rs | function | pub | 77 | as_str(&self) -> &'static str |
| `capture_window` | projects/meter/src/capture/vitals.rs | function | pub | 209 | capture_window(     target: &Target,     extra_args: &[String],     opts: &WindowOpts, ) -> Result<CaptureOutcome, SampleError> |
| `load` | projects/meter/src/capture/vitals.rs | function | pub | 126 | load(dir: &Path) -> Result<Option<MeterConfig>, String> |
| `parse` | projects/meter/src/capture/vitals.rs | function | pub | 63 | parse(s: &str) -> Result<Level, String> |
| `resolve_level` | projects/meter/src/capture/vitals.rs | function | pub | 148 | resolve_level(cli: Option<Level>, config: Option<&MeterConfig>) -> Level |
| `vitals_findings` | projects/meter/src/capture/vitals.rs | function | pub | 575 | vitals_findings(     vitals: &Vitals,     label: &str,     gate: &GateConfig,     escalate_command: &str, ) -> Vec<Finding> |
| `write_collapsed` | projects/meter/src/capture/vitals.rs | function | pub | 548 | write_collapsed(stacks: &[FoldedStack], label: &str) -> std::io::Result<PathBuf> |
## Source
<!-- type: rust-source-unit lang: rust -->

````rust
//! L1 vitals capture + the `meter.toml` per-project measurement contract (WI #3).
//!
//! @spec projects/meter/tech-design/logic/single-knob-meter-toml-level-gate-l1-vitals-in-capture-until-exi.md
//!
//! This module owns the four charter pieces of the single-knob contract:
//!
//! - **`meter.toml` parsing** — one `level` knob (`off | vitals | sample |
//!   hooks | deep`) plus an optional `[gate]` table (`max_peak_rss_mb`,
//!   `max_cpu_time_ms`). Precedence: CLI flag > meter.toml > built-in default
//!   (`vitals`). Unknown keys are rejected (`deny_unknown_fields`) so traffic
//!   knobs can never creep in.
//! - **L1 vitals** — after the spawned child is reaped via `wait4(2)`, its
//!   `rusage` becomes a `Finding{kind:vital}` carrying `cpu_time_ms`,
//!   `wall_time_ms`, and `peak_rss_bytes`. Zero injection, zero sampler.
//! - **The measurement window** — default is UNTIL CHILD EXIT (a
//!   self-terminating target is never killed mid-run); `duration_cap_secs`
//!   optionally bounds it; an opaque `drive` command's lifetime bounds the
//!   window for server-shaped targets (meter never interprets the driver).
//! - **Gate adjudication** — `[gate]` ceilings breached => severity High
//!   findings that ride the existing exit ladder, with an escalation
//!   `agent_prompt` pointing at `--level sample`.
//!
//! meter NEVER generates load: the only traffic-adjacent surface here is the
//! opaque `drive` command, which meter spawns verbatim and only borrows the
//! lifetime of.

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde::Deserialize;

use crate::report::finding::{finding_id, Finding, Invoke, Kind, Severity};

use super::sampler::{
    parse_sample_report, resolve_target_exec, spawn_exec, FoldedStack, SampleError, SampleRun,
    Target,
};

/// The single instrumentation knob. Cumulative ladder: each level includes
/// everything below it. `Hooks` and `Deep` parse but are not yet implemented
/// (L3/L4 instrumentation epic, WI #4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub enum Level {
    /// No measurement at all.
    Off,
    /// Process vitals only: cpu_time / wall_time / peak RSS via `wait4`+`rusage`.
    Vitals,
    /// Vitals + sampled call stacks (the existing capture profile).
    Sample,
    /// Vitals + stacks + runtime injection (Python/Node) — NOT YET IMPLEMENTED.
    Hooks,
    /// Hooks + allocation / off-CPU / Rust build-time injection — NOT YET IMPLEMENTED.
    Deep,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
impl Level {
    /// Parse a CLI/meter.toml level label.
    pub fn parse(s: &str) -> Result<Level, String> {
        match s {
            "off" => Ok(Level::Off),
            "vitals" => Ok(Level::Vitals),
            "sample" => Ok(Level::Sample),
            "hooks" => Ok(Level::Hooks),
            "deep" => Ok(Level::Deep),
            other => Err(format!(
                "unknown level `{other}`; use off | vitals | sample | hooks | deep"
            )),
        }
    }

    /// The snake_case label.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Off => "off",
            Level::Vitals => "vitals",
            Level::Sample => "sample",
            Level::Hooks => "hooks",
            Level::Deep => "deep",
        }
    }
}

/// The optional `[gate]` table: per-project resource ceilings. `0` disables a
/// gate. These are the only per-project facts not derivable from `level`.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub struct GateConfig {
    /// Peak RSS ceiling in MiB for the measured child; 0 = no gate.
    #[serde(default)]
    pub max_peak_rss_mb: u64,
    /// Total cpu time (user+sys) ceiling in milliseconds; 0 = no gate.
    #[serde(default)]
    pub max_cpu_time_ms: u64,
}

/// The parsed `meter.toml` measurement contract.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub struct MeterConfig {
    /// The declared resting-state level, if any.
    pub level: Option<Level>,
    /// Gate ceilings (all-zero when absent).
    pub gate: GateConfig,
}

/// Raw serde shape for `meter.toml`. Unknown keys (and any future traffic key)
/// are hard errors by charter.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMeterToml {
    level: Option<String>,
    gate: Option<GateConfig>,
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
impl MeterConfig {
    /// Load `<dir>/meter.toml`. Absent file => `Ok(None)` (built-in defaults
    /// apply). A present-but-invalid file is a hard usage error, never
    /// silently ignored.
    pub fn load(dir: &Path) -> Result<Option<MeterConfig>, String> {
        let path = dir.join("meter.toml");
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(format!("could not read `{}`: {e}", path.display())),
        };
        let parsed: RawMeterToml = toml::from_str(&raw)
            .map_err(|e| format!("invalid meter.toml at `{}`: {e}", path.display()))?;
        let level = match parsed.level {
            Some(s) => Some(Level::parse(&s).map_err(|e| format!("meter.toml: {e}"))?),
            None => None,
        };
        Ok(Some(MeterConfig {
            level,
            gate: parsed.gate.unwrap_or_default(),
        }))
    }
}

/// Resolve the effective level: CLI flag > meter.toml > built-in `vitals`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub fn resolve_level(cli: Option<Level>, config: Option<&MeterConfig>) -> Level {
    cli.or_else(|| config.and_then(|c| c.level))
        .unwrap_or(Level::Vitals)
}

/// L1 process vitals for one reaped child, from `wait4(2)`'s `rusage`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub struct Vitals {
    /// user+sys CPU time in milliseconds.
    pub cpu_time_ms: u64,
    /// Wall-clock duration of the measurement window in milliseconds.
    pub wall_time_ms: u64,
    /// Peak resident set size in bytes (ru_maxrss normalized: macOS reports
    /// bytes, Linux kilobytes).
    pub peak_rss_bytes: u64,
}

/// Options for one capture window.
#[derive(Debug, Clone, Default)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub struct WindowOpts {
    /// Effective level (only `Vitals`/`Sample` reach the window; `Off`/
    /// `Hooks`/`Deep` are handled by the caller before spawning anything).
    pub attach_sampler: bool,
    /// Optional cap on the window in seconds; `None` = until child exit.
    pub duration_cap_secs: Option<u64>,
    /// Opaque driver command: spawned via `sh -c`, its exit ends the window.
    pub drive: Option<String>,
    /// Sampling rate override (Hz) for the sampler.
    pub hz: Option<u64>,
}

/// Everything one capture window produced.
#[derive(Debug)]
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub struct CaptureOutcome {
    /// Folded stacks (only when the sampler was attached).
    pub sample: Option<SampleRun>,
    /// The child's L1 vitals (always present).
    pub vitals: Vitals,
    /// The child's exit code, when it exited normally (None = killed at
    /// window end / by the cap).
    pub child_exit: Option<i32>,
    /// The opaque driver command, when one bounded the window.
    pub driver_command: Option<String>,
}

/// Default macOS sampling interval (ms) when no `--hz` is given; mirrors the
/// sampler module's hot-spot default.
const SAMPLE_INTERVAL_MS_DEFAULT: u64 = 4;
/// Effectively-unbounded sample duration for until-exit windows; `sample`
/// terminates early when the target exits.
const SAMPLE_UNTIL_EXIT_SECS: u64 = 86_400;
/// Poll granularity for the window wait loop.
const POLL_INTERVAL: Duration = Duration::from_millis(20);

/// Run one capture window over `target`: spawn the child, optionally attach
/// the platform stack sampler, bound the window (driver lifetime > duration
/// cap > child exit), reap via `wait4`, and return stacks + vitals.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub fn capture_window(
    target: &Target,
    extra_args: &[String],
    opts: &WindowOpts,
) -> Result<CaptureOutcome, SampleError> {
    let exec_path = resolve_target_exec(target)?;
    let wall_start = Instant::now();
    let mut child = spawn_exec(&exec_path, extra_args)?;
    let pid = child.id();

    // Attach the sampler (as a sibling process) before the window starts.
    let sampler = if opts.attach_sampler {
        Some(attach_sampler(pid, opts)?)
    } else {
        None
    };

    // The window: driver lifetime when given, else child exit, both bounded by
    // the optional duration cap.
    let deadline = opts
        .duration_cap_secs
        .map(|s| wall_start + Duration::from_secs(s));
    let mut child_exit: Option<i32> = None;
    let mut reaped: Option<libc::rusage> = None;

    if let Some(drive_cmd) = &opts.drive {
        // Opaque driver: meter spawns it verbatim and only borrows its
        // lifetime as the measurement window. Never interpreted.
        let mut driver = Command::new("/bin/sh")
            .arg("-c")
            .arg(drive_cmd)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| SampleError::Spawn(format!("could not spawn --drive command: {e}")))?;
        wait_with_deadline_child(&mut driver, deadline);
        // Window over: the SUT is meter's child; terminate and reap it.
        let _ = child.kill();
        if let Some((status, ru)) = wait4_blocking(pid) {
            child_exit = exit_code_of(status);
            reaped = Some(ru);
        }
    } else {
        // Until-exit default: wait for the child itself, capped if asked.
        let (status, ru, killed) = wait4_with_deadline(&mut child, deadline);
        if let (Some(status), false) = (status, killed) {
            child_exit = exit_code_of(status);
        }
        reaped = ru;
    }

    let wall = wall_start.elapsed();
    let vitals = reaped
        .map(|ru| vitals_from_rusage(&ru, wall))
        .unwrap_or(Vitals {
            cpu_time_ms: 0,
            wall_time_ms: wall.as_millis() as u64,
            peak_rss_bytes: 0,
        });

    // Collect the sampler's stacks now that the window is over.
    let sample = match sampler {
        Some(s) => Some(s.finish()?),
        None => None,
    };

    Ok(CaptureOutcome {
        sample,
        vitals,
        child_exit,
        driver_command: opts.drive.clone(),
    })
}

/// A sampler attached to a live pid; `finish()` collects its folded stacks
/// after the window closes.
struct AttachedSampler {
    backend: Backend,
    proc_: Child,
    effective_hz: f64,
    argv: Vec<String>,
}

enum Backend {
    MacosSample {
        report_path: PathBuf,
    },
    #[allow(dead_code)]
    LinuxPerf {
        data_path: PathBuf,
    },
}

/// Attach the platform sampler to `pid` (macOS `/usr/bin/sample <pid>`,
/// Linux `perf record -p <pid>`).
fn attach_sampler(pid: u32, opts: &WindowOpts) -> Result<AttachedSampler, SampleError> {
    let interval_ms = match opts.hz {
        Some(h) if h > 0 => (1000 / h).max(1),
        _ => SAMPLE_INTERVAL_MS_DEFAULT,
    };
    let effective_hz = 1000.0 / interval_ms as f64;
    let duration = opts.duration_cap_secs.unwrap_or(SAMPLE_UNTIL_EXIT_SECS);

    if cfg!(target_os = "macos") {
        if !Path::new("/usr/bin/sample").exists() {
            return Err(SampleError::NoBackend(
                "/usr/bin/sample not found (expected on macOS)".to_string(),
            ));
        }
        let report_path = std::env::temp_dir().join(format!("meter-sample-{pid}.txt"));
        let argv = vec![
            "/usr/bin/sample".to_string(),
            pid.to_string(),
            duration.to_string(),
            interval_ms.to_string(),
            "-file".to_string(),
            report_path.display().to_string(),
            "-mayDie".to_string(),
        ];
        let proc_ = Command::new("/usr/bin/sample")
            .args(&argv[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| SampleError::Sampler(format!("/usr/bin/sample failed to spawn: {e}")))?;
        Ok(AttachedSampler {
            backend: Backend::MacosSample { report_path },
            proc_,
            effective_hz,
            argv,
        })
    } else if cfg!(target_os = "linux") {
        let freq = opts.hz.unwrap_or(250).max(1);
        let data_path = std::env::temp_dir().join(format!("meter-perf-{pid}.data"));
        let argv = vec![
            "perf".to_string(),
            "record".to_string(),
            "-F".to_string(),
            freq.to_string(),
            "-g".to_string(),
            "-p".to_string(),
            pid.to_string(),
        ];
        let proc_ = Command::new("perf")
            .args([
                "record",
                "-F",
                &freq.to_string(),
                "-g",
                "-p",
                &pid.to_string(),
                "-o",
            ])
            .arg(&data_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| SampleError::NoBackend(format!("`perf` failed to spawn: {e}")))?;
        Ok(AttachedSampler {
            backend: Backend::LinuxPerf { data_path },
            proc_,
            effective_hz: freq as f64,
            argv,
        })
    } else {
        Err(SampleError::NoBackend(format!(
            "platform `{}` has no supported stack sampler (macOS `sample` / Linux `perf`)",
            std::env::consts::OS
        )))
    }
}

/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
impl AttachedSampler {
    /// Wait for the sampler process and fold its report into stacks.
    fn finish(mut self) -> Result<SampleRun, SampleError> {
        let status = self
            .proc_
            .wait()
            .map_err(|e| SampleError::Sampler(format!("sampler wait failed: {e}")))?;
        match self.backend {
            Backend::MacosSample { report_path } => {
                if !status.success() {
                    let _ = std::fs::remove_file(&report_path);
                    return Err(SampleError::Sampler(format!(
                        "/usr/bin/sample exited {status}"
                    )));
                }
                let content = std::fs::read_to_string(&report_path).map_err(|e| {
                    SampleError::Sampler(format!(
                        "could not read sample report `{}`: {e}",
                        report_path.display()
                    ))
                })?;
                let _ = std::fs::remove_file(&report_path);
                let stacks = parse_sample_report(&content);
                if stacks.is_empty() {
                    return Err(SampleError::Sampler(format!(
                        "sample report contained no call-graph stacks (target may have exited \
                         before sampling attached, or ran too briefly); report was {} bytes",
                        content.len()
                    )));
                }
                Ok(SampleRun {
                    stacks,
                    backend: "macos-sample".to_string(),
                    effective_hz: self.effective_hz,
                    command: self.argv,
                })
            }
            Backend::LinuxPerf { data_path } => {
                let script = Command::new("perf")
                    .arg("script")
                    .arg("-i")
                    .arg(&data_path)
                    .output()
                    .map_err(|e| SampleError::Sampler(format!("perf script failed: {e}")))?;
                let _ = std::fs::remove_file(&data_path);
                let text = String::from_utf8_lossy(&script.stdout);
                let stacks = super::sampler::parse_perf_script(&text);
                if stacks.is_empty() {
                    return Err(SampleError::Sampler(
                        "perf script produced no stacks".to_string(),
                    ));
                }
                Ok(SampleRun {
                    stacks,
                    backend: "linux-perf".to_string(),
                    effective_hz: self.effective_hz,
                    command: self.argv,
                })
            }
        }
    }
}

/// Wait for `child` (reaping via `wait4` so we get `rusage`), killing it if
/// `deadline` passes first. Returns `(raw status, rusage, killed_by_cap)`.
fn wait4_with_deadline(
    child: &mut Child,
    deadline: Option<Instant>,
) -> (Option<i32>, Option<libc::rusage>, bool) {
    let pid = child.id() as libc::pid_t;
    let mut killed = false;
    loop {
        let mut status: libc::c_int = 0;
        let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
        let r = unsafe { libc::wait4(pid, &mut status, libc::WNOHANG, &mut ru) };
        if r == pid {
            return (Some(status), Some(ru), killed);
        }
        if r == -1 {
            // No such child (already reaped elsewhere) — nothing more to learn.
            return (None, None, killed);
        }
        if let Some(d) = deadline {
            if !killed && Instant::now() >= d {
                let _ = child.kill();
                killed = true;
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Blocking `wait4` reap for an already-terminating pid.
fn wait4_blocking(pid: u32) -> Option<(i32, libc::rusage)> {
    let mut status: libc::c_int = 0;
    let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
    let r = unsafe { libc::wait4(pid as libc::pid_t, &mut status, 0, &mut ru) };
    if r == pid as libc::pid_t {
        Some((status, ru))
    } else {
        None
    }
}

/// Wait for a plain child (the driver) with an optional deadline; on expiry
/// the driver is killed (the window is over either way).
fn wait_with_deadline_child(child: &mut Child, deadline: Option<Instant>) {
    let mut killed = false;
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => {}
            Err(_) => return,
        }
        if let Some(d) = deadline {
            if !killed && Instant::now() >= d {
                let _ = child.kill();
                killed = true;
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    }
}

/// Decode a raw `wait4` status into an exit code (None for signals).
fn exit_code_of(status: i32) -> Option<i32> {
    if libc::WIFEXITED(status) {
        Some(libc::WEXITSTATUS(status))
    } else {
        None
    }
}

/// Normalize an `rusage` + wall duration into [`Vitals`]. `ru_maxrss` is bytes
/// on macOS and kilobytes on Linux.
fn vitals_from_rusage(ru: &libc::rusage, wall: Duration) -> Vitals {
    let tv_ms =
        |tv: &libc::timeval| -> u64 { (tv.tv_sec as u64) * 1000 + (tv.tv_usec as u64) / 1000 };
    let maxrss = ru.ru_maxrss.max(0) as u64;
    let peak_rss_bytes = if cfg!(target_os = "macos") {
        maxrss
    } else {
        maxrss * 1024
    };
    Vitals {
        cpu_time_ms: tv_ms(&ru.ru_utime) + tv_ms(&ru.ru_stime),
        wall_time_ms: wall.as_millis() as u64,
        peak_rss_bytes,
    }
}

/// Sanitize a target label into a filename-safe slug.
fn safe_slug(label: &str) -> String {
    label
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

/// Write the folded stacks as a collapsed artifact under `.meter/` (relative
/// to the cwd, like the persisted report) and return its path. One line per
/// stack: `frame;frame;leaf count`.
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub fn write_collapsed(stacks: &[FoldedStack], label: &str) -> std::io::Result<PathBuf> {
    write_collapsed_in(Path::new("."), stacks, label)
}

/// Testable core of [`write_collapsed`]: write under `<base>/.meter/`.
fn write_collapsed_in(
    base: &Path,
    stacks: &[FoldedStack],
    label: &str,
) -> std::io::Result<PathBuf> {
    let dir = base.join(".meter");
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.collapsed", safe_slug(label)));
    let mut content = String::new();
    for s in stacks {
        content.push_str(&s.to_folded_line());
        content.push('\n');
    }
    std::fs::write(&path, content)?;
    Ok(path)
}

/// Produce the `kind=vital` findings for one capture window: one Info finding
/// carrying the vitals evidence, plus one High finding per breached `[gate]`
/// ceiling. `escalate_command` is the literal next command suggested when a
/// gate breach needs root-causing (the `--level sample` escalation funnel).
/// @spec projects/meter/tech-design/semantic/source/projects-meter-src-capture-vitals-rs.md#source
pub fn vitals_findings(
    vitals: &Vitals,
    label: &str,
    gate: &GateConfig,
    escalate_command: &str,
) -> Vec<Finding> {
    let slug = safe_slug(label);
    let evidence = serde_json::json!({
        "cpu_time_ms": vitals.cpu_time_ms,
        "wall_time_ms": vitals.wall_time_ms,
        "peak_rss_bytes": vitals.peak_rss_bytes,
    });
    let mut out = vec![Finding {
        id: finding_id(Kind::Vital, &slug),
        severity: Severity::Info,
        kind: Kind::Vital,
        title: format!("process vitals for {label}"),
        detail: format!(
            "cpu_time {} ms, wall_time {} ms, peak RSS {} bytes (getrusage after child wait)",
            vitals.cpu_time_ms, vitals.wall_time_ms, vitals.peak_rss_bytes
        ),
        remediation: "Informational: these are the measured per-run resource vitals.".to_string(),
        invoke: Invoke::command("meter report"),
        evidence,
        location: None,
    }];

    if gate.max_peak_rss_mb > 0 {
        let limit_bytes = gate.max_peak_rss_mb * 1024 * 1024;
        if vitals.peak_rss_bytes > limit_bytes {
            out.push(gate_breach_finding(
                &slug,
                label,
                "max_peak_rss_mb",
                limit_bytes,
                vitals.peak_rss_bytes,
                "bytes",
                escalate_command,
            ));
        }
    }
    if gate.max_cpu_time_ms > 0 && vitals.cpu_time_ms > gate.max_cpu_time_ms {
        out.push(gate_breach_finding(
            &slug,
            label,
            "max_cpu_time_ms",
            gate.max_cpu_time_ms,
            vitals.cpu_time_ms,
            "ms",
            escalate_command,
        ));
    }
    out
}

/// One breached-gate finding (severity High => exit ladder rung 1).
fn gate_breach_finding(
    slug: &str,
    label: &str,
    gate_key: &str,
    limit: u64,
    observed: u64,
    unit: &str,
    escalate_command: &str,
) -> Finding {
    Finding {
        id: finding_id(Kind::Vital, format!("{slug}:{gate_key}")),
        severity: Severity::High,
        kind: Kind::Vital,
        title: format!("[gate] {gate_key} breached for {label}"),
        detail: format!("observed {observed} {unit} > declared ceiling {limit} {unit} (meter.toml [gate].{gate_key})"),
        remediation: format!(
            "The run exceeded the declared resource ceiling. Locate the cost by escalating one \
             level: `{escalate_command}`."
        ),
        invoke: Invoke::command(escalate_command),
        evidence: serde_json::json!({
            "gate": gate_key,
            "limit": limit,
            "observed": observed,
            "unit": unit,
        }),
        location: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let p = std::env::temp_dir().join(format!("meter-vitals-{tag}-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        p
    }

    #[test]
    fn level_parses_and_orders() {
        assert_eq!(Level::parse("vitals").unwrap(), Level::Vitals);
        assert_eq!(Level::parse("off").unwrap(), Level::Off);
        assert!(Level::parse("turbo").is_err());
        assert!(Level::Sample > Level::Vitals);
        assert!(Level::Deep > Level::Hooks);
        assert_eq!(Level::Sample.as_str(), "sample");
    }

    #[test]
    fn config_level_only_file_is_valid_and_absent_file_is_none() {
        let dir = tmp_dir("cfg-min");
        std::fs::write(dir.join("meter.toml"), "level = \"vitals\"\n").unwrap();
        let cfg = MeterConfig::load(&dir).unwrap().unwrap();
        assert_eq!(cfg.level, Some(Level::Vitals));
        assert_eq!(cfg.gate, GateConfig::default());

        let empty = tmp_dir("cfg-absent");
        assert!(MeterConfig::load(&empty).unwrap().is_none());
    }

    #[test]
    fn config_gate_table_parses_and_unknown_keys_are_rejected() {
        let dir = tmp_dir("cfg-gate");
        std::fs::write(
            dir.join("meter.toml"),
            "level = \"sample\"\n[gate]\nmax_peak_rss_mb = 512\nmax_cpu_time_ms = 2000\n",
        )
        .unwrap();
        let cfg = MeterConfig::load(&dir).unwrap().unwrap();
        assert_eq!(cfg.level, Some(Level::Sample));
        assert_eq!(cfg.gate.max_peak_rss_mb, 512);
        assert_eq!(cfg.gate.max_cpu_time_ms, 2000);

        // Traffic keys (or any unknown key) must be hard errors, per charter.
        std::fs::write(
            dir.join("meter.toml"),
            "level = \"vitals\"\n[gate]\nmax_latency_p99_ms = 5\n",
        )
        .unwrap();
        assert!(MeterConfig::load(&dir).is_err());
        std::fs::write(dir.join("meter.toml"), "rps = 100\n").unwrap();
        assert!(MeterConfig::load(&dir).is_err());
    }

    #[test]
    fn level_precedence_is_cli_then_toml_then_default() {
        let toml_cfg = MeterConfig {
            level: Some(Level::Sample),
            gate: GateConfig::default(),
        };
        // CLI wins over meter.toml.
        assert_eq!(
            resolve_level(Some(Level::Deep), Some(&toml_cfg)),
            Level::Deep
        );
        // meter.toml wins over the default.
        assert_eq!(resolve_level(None, Some(&toml_cfg)), Level::Sample);
        // Default is vitals.
        assert_eq!(resolve_level(None, None), Level::Vitals);
        let no_level = MeterConfig::default();
        assert_eq!(resolve_level(None, Some(&no_level)), Level::Vitals);
    }

    #[test]
    fn vitals_window_until_exit_reaps_a_self_terminating_child() {
        // A self-terminating target must NOT be killed mid-run and must yield
        // real vitals. /bin/sh is universally present.
        let target = Target::Exec(PathBuf::from("/bin/sh"));
        let outcome = capture_window(
            &target,
            &["-c".to_string(), "sleep 0.05; exit 7".to_string()],
            &WindowOpts::default(),
        )
        .expect("vitals window");
        assert_eq!(outcome.child_exit, Some(7), "child ran to completion");
        assert!(outcome.sample.is_none(), "no sampler at vitals level");
        assert!(
            outcome.vitals.wall_time_ms >= 40,
            "window covered the sleep"
        );
        assert!(outcome.vitals.peak_rss_bytes > 0, "rusage maxrss captured");
    }

    #[test]
    fn duration_cap_bounds_a_long_running_child() {
        let target = Target::Exec(PathBuf::from("/bin/sh"));
        let start = Instant::now();
        let outcome = capture_window(
            &target,
            &["-c".to_string(), "sleep 30".to_string()],
            &WindowOpts {
                duration_cap_secs: Some(1),
                ..Default::default()
            },
        )
        .expect("capped window");
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "cap ended the window"
        );
        assert_eq!(
            outcome.child_exit, None,
            "capped child was killed, not exited"
        );
    }

    #[test]
    fn drive_lifetime_bounds_the_window_for_a_server_shaped_child() {
        // The SUT never exits on its own; the opaque driver's exit must end
        // the window and the driver command must be recorded.
        let target = Target::Exec(PathBuf::from("/bin/sh"));
        let start = Instant::now();
        let outcome = capture_window(
            &target,
            &["-c".to_string(), "sleep 30".to_string()],
            &WindowOpts {
                drive: Some("sleep 0.1".to_string()),
                ..Default::default()
            },
        )
        .expect("driven window");
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "driver ended the window"
        );
        assert_eq!(outcome.driver_command.as_deref(), Some("sleep 0.1"));
        assert!(
            outcome.vitals.wall_time_ms >= 90,
            "window spans driver lifetime"
        );
    }

    #[test]
    fn gate_breach_produces_high_finding_with_escalation() {
        let vitals = Vitals {
            cpu_time_ms: 5000,
            wall_time_ms: 6000,
            peak_rss_bytes: 600 * 1024 * 1024,
        };
        let gate = GateConfig {
            max_peak_rss_mb: 512,
            max_cpu_time_ms: 2000,
        };
        let findings = vitals_findings(
            &vitals,
            "exec:/x",
            &gate,
            "meter profile --exec /x --level sample",
        );
        // 1 info vital + 2 gate breaches.
        assert_eq!(findings.len(), 3);
        assert!(findings.iter().all(|f| f.kind == Kind::Vital));
        let breaches: Vec<_> = findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .collect();
        assert_eq!(breaches.len(), 2);
        assert!(breaches
            .iter()
            .all(|f| f.invoke.command.contains("--level sample")));
        assert!(breaches.iter().any(|f| f.id.contains("max_peak_rss_mb")));
    }

    #[test]
    fn gate_zero_means_disabled() {
        let vitals = Vitals {
            cpu_time_ms: 999_999,
            wall_time_ms: 1,
            peak_rss_bytes: u64::MAX / 2,
        };
        let findings = vitals_findings(&vitals, "x", &GateConfig::default(), "meter report");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, Severity::Info);
    }

    #[test]
    fn collapsed_artifact_writes_one_line_per_stack() {
        // No chdir: cwd is process-global and other lib tests read relative
        // paths; exercise the path-taking core directly.
        let dir = tmp_dir("collapsed");
        let stacks = vec![
            FoldedStack::new(vec!["a".into(), "b".into()], 5),
            FoldedStack::new(vec!["a".into(), "c".into()], 2),
        ];
        let path = write_collapsed_in(&dir, &stacks, "exec:/tmp/x").unwrap();
        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, "a;b 5\na;c 2\n");
        assert!(path.to_string_lossy().contains(".meter"));
        assert!(path.to_string_lossy().ends_with(".collapsed"));
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/meter/src/capture/vitals.rs"
    action: modify
    section: rust-source-unit
    description: "Regenerate the capture vitals implementation from a TD-owned rust source unit."
    impl_mode: codegen
```
