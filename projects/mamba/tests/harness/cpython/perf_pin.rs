//! Consolidated perf-pin regression gate runner (#1265 Goal 2).
//!
//! Replaces 119 standalone `<lib>_perf_pin_<issue>.rs` binaries. Each pin is
//! described declaratively by a TOML file under
//! `tests/harness/cpython/config/perf/pins/`:
//!
//! ```toml
//! issue = 1447
//! lib   = "abc"
//! fixture = "tests/cpython/fixtures/std-libs/abc/bench/get_cache_token_hot.py"
//! floor   = 1.0
//! mem_floor = 1.0
//! samples = 1            # 1 = single shot; N>=3 = median-of-N
//! prereq_imports = []    # e.g. ["aiofiles", "google.protobuf"]
//! ```
//!
//! The runner is `#[ignore]`-equivalent by default: it lives in an integration
//! test binary registered with `harness = false`, so it does not run unless the
//! G3.1 selector explicitly opts in:
//!
//!     cargo test -p mamba --release --test perf_pin_runner -- perf_pin
//!
//! For each TOML entry it loads the CPython baseline from the local SQLite
//! database created by `tests/harness/cpython/tools/perf_baseline.py record`, then
//! spawns `mamba run <fixture>`, measures the child's CPU time externally
//! (getrusage / `/usr/bin/time`), and asserts the mamba/cpython CPU-time ratio
//! `<= floor` (D5.2: the harness owns measurement; fixtures stay pure — no
//! self-emitted timing marker). When the baseline is absent, the
//! runner falls back to live `python3 <fixture>` measurement unless
//! `MAMBA_REQUIRE_CPYTHON_PERF_BASELINE=1` is set.
//!
//! Each pin's emitted test name is `perf_pin::<lib>_<issue>` which lets the
//! `perf_pin` substring filter match every pin in one go.

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

use datatest_stable::harness;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Pin {
    issue: u64,
    lib: String,
    fixture: String,
    floor: f64,
    samples: usize,
    #[serde(default)]
    prereq_imports: Vec<String>,
    /// Peak-RSS floor; the contract gate requires every pin to set it. It
    /// matches cross_runtime.rs FLOOR semantics (mem_ratio = cpython_rss /
    /// mamba_rss must be >= mem_floor, i.e. mamba uses no more peak memory
    /// than CPython at floor 1.0x).
    #[serde(default)]
    mem_floor: Option<f64>,
}

#[derive(Debug, Clone, Copy)]
struct Measurement {
    cpu_time_ns: Option<u64>,
    peak_rss_bytes: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct CpythonPerfBaseline {
    pin_path: String,
    fixture_sha256: String,
    samples: usize,
    // Retained for sqlite-row deserialization compatibility; no longer used by
    // the gate (D5.2 measures external CPU time, not the fixture marker).
    #[allow(dead_code)]
    internal_time_ns: u64,
    cpu_time_ns: Option<u64>,
    peak_rss_bytes: Option<u64>,
    python: String,
    captured_at_unix: u64,
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn python3_can_import(module: &str) -> bool {
    Command::new("python3")
        .args(["-c", &format!("import {module}")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parse a `/usr/bin/time` stderr blob for the child's peak RSS in bytes.
/// macOS BSD `time -l` reports bytes; Linux GNU `time -v` reports kbytes.
/// Returns None if no recognised line is present. Mirrors the same parser
/// in `benches/3p/cross_runtime.rs`.
fn parse_peak_rss(stderr: &str) -> Option<u64> {
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
fn parse_cpu_time_ns(stderr: &str) -> Option<u64> {
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
fn timeval_to_ns(tv: libc::timeval) -> u64 {
    (tv.tv_sec as u64)
        .saturating_mul(1_000_000_000)
        .saturating_add((tv.tv_usec as u64).saturating_mul(1_000))
}

#[cfg(unix)]
fn child_cpu_time_ns() -> Option<u64> {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    let rc = unsafe { libc::getrusage(libc::RUSAGE_CHILDREN, usage.as_mut_ptr()) };
    if rc != 0 {
        return None;
    }
    let usage = unsafe { usage.assume_init() };
    Some(timeval_to_ns(usage.ru_utime).saturating_add(timeval_to_ns(usage.ru_stime)))
}

#[cfg(not(unix))]
fn child_cpu_time_ns() -> Option<u64> {
    None
}

/// Build the `/usr/bin/time` argv prefix for the current platform. macOS uses
/// BSD `-l`; everywhere else assume GNU `-v`. Returns None if `/usr/bin/time`
/// does not exist (caller falls back to plain `Command::new(cmd)` and drops
/// RSS/CPU measurement — external resource gating is best-effort by design).
fn time_wrapper() -> Option<(&'static str, &'static str)> {
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
/// child's CPU time and peak RSS can be parsed.
fn run_once_with_metrics(cmd: &str, args: &[&str]) -> Measurement {
    let cpu_before = child_cpu_time_ns();
    let (out, wrapped) = if let Some((time_bin, flag)) = time_wrapper() {
        let mut all_args: Vec<&str> = Vec::with_capacity(args.len() + 2);
        all_args.push(flag);
        all_args.push(cmd);
        all_args.extend(args.iter().copied());
        (Command::new(time_bin).args(&all_args).output(), true)
    } else {
        (Command::new(cmd).args(args).output(), false)
    };
    let cpu_after = child_cpu_time_ns();
    let out = out.unwrap_or_else(|e| panic!("failed to spawn {cmd}: {e}"));
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

fn median(values: &mut [u64]) -> u64 {
    values.sort_unstable();
    values[values.len() / 2]
}

fn measure_n(cmd: &str, args: &[&str], n: usize) -> Measurement {
    assert!(n > 0, "samples must be >= 1");
    let mut cpu_samples = Vec::with_capacity(n);
    let mut rss_samples = Vec::with_capacity(n);

    for _ in 0..n {
        let measurement = run_once_with_metrics(cmd, args);
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

fn baseline_db() -> PathBuf {
    std::env::var("MAMBA_CPYTHON_PERF_BASELINE_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            manifest_dir().join("tests/cpython/.cache/perf/cpython_baseline.sqlite")
        })
}

fn baseline_required() -> bool {
    std::env::var("MAMBA_REQUIRE_CPYTHON_PERF_BASELINE")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "required"
            )
        })
        .unwrap_or(false)
}

fn baseline_tool() -> PathBuf {
    manifest_dir().join("tests/harness/cpython/tools/perf_baseline.py")
}

fn fixture_sha256(path: &Path) -> std::io::Result<String> {
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

fn load_cpython_baseline(toml_path: &Path) -> Option<CpythonPerfBaseline> {
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

    let output = Command::new("python3")
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

fn cpython_measurement_from_baseline(baseline: &CpythonPerfBaseline) -> Measurement {
    Measurement {
        cpu_time_ns: baseline.cpu_time_ns,
        peak_rss_bytes: baseline.peak_rss_bytes,
    }
}

fn run_pin(toml_path: &Path) -> datatest_stable::Result<()> {
    let raw = std::fs::read_to_string(toml_path)?;
    let pin: Pin = toml::from_str(&raw)?;

    let fixture = manifest_dir().join(&pin.fixture);
    assert!(
        fixture.exists(),
        "#{} {} fixture missing: {}",
        pin.issue,
        pin.lib,
        fixture.display()
    );

    if !python3_available() {
        eprintln!(
            "python3 not available; skipping #{} {} perf gate (mamba-only \
             run is meaningless without the CPython baseline)",
            pin.issue, pin.lib
        );
        return Ok(());
    }

    let baseline = load_cpython_baseline(toml_path);
    if let Some(baseline) = &baseline {
        let actual_hash = fixture_sha256(&fixture)?;
        assert_eq!(
            baseline.fixture_sha256,
            actual_hash,
            "#{} {} CPython perf baseline is stale for {}. Re-run `python3 tests/harness/cpython/tools/perf_baseline.py record --pin {}`.",
            pin.issue,
            pin.lib,
            fixture.display(),
            toml_path.display()
        );
        eprintln!(
            "#{} {} CPython perf baseline: {} samples={} python={} captured_at={} \
             internal={} ns cpu={:?} rss={:?}",
            pin.issue,
            pin.lib,
            baseline.pin_path,
            baseline.samples,
            baseline.python,
            baseline.captured_at_unix,
            baseline.internal_time_ns,
            baseline.cpu_time_ns,
            baseline.peak_rss_bytes
        );
    } else {
        eprintln!(
            "#{} {} CPython perf baseline missing; falling back to live python3 measurement",
            pin.issue, pin.lib
        );
        for imp in &pin.prereq_imports {
            if !python3_can_import(imp) {
                eprintln!(
                    "python3 lacks `{imp}`; skipping #{} {} perf gate \
                     (CPython baseline unavailable on this host)",
                    pin.issue, pin.lib
                );
                return Ok(());
            }
        }
    }

    let fixture_str = fixture.to_str().expect("fixture path is not valid UTF-8");
    let mamba_bin_path = mamba_bin();
    let mamba_bin_str = mamba_bin_path
        .to_str()
        .expect("mamba binary path is not valid UTF-8");

    let samples = pin.samples.max(1);
    let cpy = if let Some(baseline) = &baseline {
        cpython_measurement_from_baseline(baseline)
    } else {
        measure_n("python3", &[fixture_str], samples)
    };
    let mb = measure_n(mamba_bin_str, &["run", fixture_str], samples);

    let mode = if samples <= 1 {
        "single-shot".to_string()
    } else {
        format!("median-of-{samples}")
    };

    // D5.2: the gate is the EXTERNAL CPU-time ratio (getrusage / /usr/bin/time),
    // not a fixture-emitted self-timing marker. Process-startup cost is
    // included; warmup/median (samples) damps it. See PRODUCTION-GATE.md D5.2.
    match (cpy.cpu_time_ns, mb.cpu_time_ns) {
        (Some(cpy_cpu), Some(mb_cpu)) if cpy_cpu > 0 => {
            let cpu_ratio = mb_cpu as f64 / cpy_cpu as f64;
            eprintln!(
                "#{} {} CPU gate ({mode}): mamba/cpython CPU-time ratio = {:.3}x \
                 (mamba {} ns vs cpython {} ns)",
                pin.issue, pin.lib, cpu_ratio, mb_cpu, cpy_cpu
            );
            assert!(
                cpu_ratio <= pin.floor,
                "#{} {} CPU gate FAIL: ratio = {:.2}x exceeds floor of {:.2}x \
                 (mamba {} ns vs cpython {} ns) [{mode}]",
                pin.issue,
                pin.lib,
                cpu_ratio,
                pin.floor,
                mb_cpu,
                cpy_cpu,
            );
        }
        _ => {
            eprintln!(
                "#{} {} CPU gate skipped: CPU-time measurement unavailable \
                 (cpython={:?}, mamba={:?})",
                pin.issue, pin.lib, cpy.cpu_time_ns, mb.cpu_time_ns
            );
        }
    }

    // OPTIONAL peak-RSS gate. A pin without `mem_floor` behaves exactly as
    // before (no assertion). When present, assert mem_ratio = cpython_rss /
    // mamba_rss >= mem_floor (mamba uses no more peak memory than CPython at
    // floor 1.0x — matches cross_runtime.rs FLOOR semantics). The CPython side
    // comes from the SQLite baseline when present; otherwise it is measured
    // live as the compatibility fallback.
    if let Some(mem_floor) = pin.mem_floor {
        match (cpy.peak_rss_bytes, mb.peak_rss_bytes) {
            (Some(cpy_b), Some(mb_b)) if mb_b > 0 => {
                let mem_ratio = cpy_b as f64 / mb_b as f64;
                eprintln!(
                    "#{} {} mem gate: cpython/mamba peak-RSS ratio = {:.3}x \
                     (mamba {} B vs cpython {} B)",
                    pin.issue, pin.lib, mem_ratio, mb_b, cpy_b
                );
                assert!(
                    mem_ratio >= mem_floor,
                    "#{} {} mem gate FAIL: cpython/mamba peak-RSS ratio = \
                     {:.2}x below floor of {:.2}x (mamba {} B vs cpython {} B)",
                    pin.issue,
                    pin.lib,
                    mem_ratio,
                    mem_floor,
                    mb_b,
                    cpy_b,
                );
            }
            _ => {
                eprintln!(
                    "#{} {} mem gate skipped: peak-RSS measurement unavailable \
                     (cpython={:?}, mamba={:?}); mem_floor={:.2}x left unenforced",
                    pin.issue, pin.lib, cpy.peak_rss_bytes, mb.peak_rss_bytes, mem_floor
                );
            }
        }
    }
    Ok(())
}

harness!(
    run_pin,
    "tests/harness/cpython/config/perf/pins",
    r"^.*\.toml$"
);
