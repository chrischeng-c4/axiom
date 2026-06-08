//! Py3.12 runtime conformance test harness (#752).
//!
//! Discovers `.py` fixtures under `tests/cpython/` and runs each through the
//! Mamba JIT pipeline, comparing captured stdout against the LIVE CPython 3.12
//! oracle (D5.6 capstone proved this reproduces the retired `.expected` goldens).
//!
//! Generator / iterator protocol conformance (#756) lives in a sister
//! binary: `tests/cpython_generators.rs` + `tests/cpython/fixtures/core/generators/`.
//! That binary uses the default `#[test]` harness (not datatest_stable) so
//! it can express per-scenario Python-source assertions rather than golden
//! file diffs. Run it with:
//!
//!   cargo test -p mamba --test conformance_generators generators::
//!
//! @spec .aw/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md
//!
//! Fixture areas:
//!   - `core/` — language semantics: data structures, builtins, classes,
//!     iterators/generators, exceptions, pattern matching, dunders
//!   - `pep/` — PEP-numbered language features
//!   - `builtin-libs/` — methods on builtin types
//!   - `std-libs/` — json, math, re, collections, etc.
//!   - `3rd-libs/` — third-party PyPI libraries
//!   - `type-strict/` — runtime-typing contract (mamba MUST raise where
//!     CPython accepts); driven by `# mamba-strict-type:` directives
//!
//! Directives (in `.py` file comments):
//!   `# mamba-xfail: <reason>` — mark as expected failure
//!   `# mamba-strict-type: TypeError` — mamba must reject wrong-typed code
//!
//! Goldens retired (D5.6): the live CPython oracle replaces the static
//! `.expected` files; this harness no longer reads goldens or `regen_golden.py`.

use datatest_stable::harness;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

const DEFAULT_TIMEOUT_SECS: u64 = 30;
const DEFAULT_CPU_SECS: u64 = DEFAULT_TIMEOUT_SECS * 2;
const DEFAULT_MEM_MB: u64 = 1024;

// ── Directive parsing ─────────────────────────────────────────────

struct Directives {
    xfail: Option<String>,
    strict_type: bool,
}

fn parse_directives(src: &str) -> Directives {
    let mut xfail = None;
    let mut strict_type = false;
    for line in src.lines() {
        let t = line.trim();
        if let Some(reason) = t.strip_prefix("# mamba-xfail:") {
            xfail = Some(reason.trim().to_string());
        } else if t.starts_with("# mamba-strict-type:") {
            strict_type = true;
        }
    }
    Directives { xfail, strict_type }
}

fn has_pipeline_run_directive(src: &str) -> bool {
    src.lines()
        .any(|line| line.trim_start().starts_with("# RUN:"))
}

// ── CLI execution with output capture ─────────────────────────────

fn mamba_bin() -> PathBuf {
    option_env!("CARGO_BIN_EXE_mamba")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/mamba")
        })
}

fn status_detail(status: ExitStatus) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return match signal {
                libc::SIGXCPU => format!("CPU_LIMIT signal {signal}"),
                libc::SIGKILL => format!("OOM_OR_KILLED signal {signal}"),
                _ => format!("signal {signal}"),
            };
        }
    }

    match status.code() {
        Some(code) => format!("exit code {code}"),
        None => "unknown process status".to_string(),
    }
}

fn terminated_by_signal(status: ExitStatus) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        return status.signal().is_some();
    }

    #[cfg(not(unix))]
    {
        let _ = status;
        false
    }
}

fn env_u64(name: &str, default: u64) -> u64 {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn timeout() -> Duration {
    Duration::from_secs(env_u64(
        "MAMBA_CONFORMANCE_TIMEOUT_SECS",
        DEFAULT_TIMEOUT_SECS,
    ))
}

#[cfg(unix)]
fn child_cpu_secs() -> libc::rlim_t {
    env_u64("MAMBA_CONFORMANCE_CPU_SECS", DEFAULT_CPU_SECS) as libc::rlim_t
}

#[cfg(unix)]
fn child_mem_bytes() -> libc::rlim_t {
    env_u64("MAMBA_CONFORMANCE_MEM_MB", DEFAULT_MEM_MB)
        .saturating_mul(1024)
        .saturating_mul(1024) as libc::rlim_t
}

#[cfg(unix)]
fn set_limit(which: libc::c_int, current: libc::rlim_t, maximum: libc::rlim_t) {
    let limit = libc::rlimit {
        rlim_cur: current,
        rlim_max: maximum,
    };
    unsafe {
        let _ = libc::setrlimit(which, &limit);
    }
}

#[cfg(unix)]
fn apply_child_limits(command: &mut Command) {
    use std::os::unix::process::CommandExt;

    let mem_bytes = child_mem_bytes();
    let cpu_secs = child_cpu_secs();

    unsafe {
        command.pre_exec(move || {
            set_limit(libc::RLIMIT_AS, mem_bytes, mem_bytes);
            set_limit(libc::RLIMIT_DATA, mem_bytes, mem_bytes);
            set_limit(libc::RLIMIT_CPU, cpu_secs, cpu_secs);
            set_limit(libc::RLIMIT_CORE, 0, 0);
            Ok(())
        });
    }
}

#[cfg(not(unix))]
fn apply_child_limits(_command: &mut Command) {}

fn resource_failure(status: ExitStatus, stderr: &str) -> Option<String> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            if signal == libc::SIGXCPU {
                return Some(format!(
                    "CPU_LIMIT: mamba exceeded {} CPU seconds",
                    env_u64("MAMBA_CONFORMANCE_CPU_SECS", DEFAULT_CPU_SECS)
                ));
            }
            if signal == libc::SIGKILL {
                return Some("OOM_OR_KILLED: mamba was killed by SIGKILL".to_string());
            }
        }
    }

    if stderr.contains("MemoryError") {
        return Some("OOM: mamba hit the conformance memory limit".to_string());
    }

    None
}

fn has_line_prefix(text: &str, prefix: &str) -> bool {
    text.lines().any(|line| line.starts_with(prefix))
}

fn is_type_strict_path(path: &Path) -> bool {
    path.components()
        .any(|component| component.as_os_str() == "type-strict")
}

fn is_compile_time_type_error(output: &Output) -> bool {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    stderr.contains("TypeError")
        || stderr.contains("type error")
        || stdout.contains("TypeError")
        || stdout.contains("type error")
}

fn run_type_strict(path: &Path) -> datatest_stable::Result<()> {
    let output = spawn_mamba(path)?;
    let stderr = String::from_utf8_lossy(&output.stderr);

    if terminated_by_signal(output.status) {
        return Err(format!(
            "{}: mamba run ended with {}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            status_detail(output.status),
            String::from_utf8_lossy(&output.stdout),
            stderr
        )
        .into());
    }

    if let Some(reason) = resource_failure(output.status, &stderr) {
        return Err(format!(
            "{}: {reason}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            stderr
        )
        .into());
    }

    if !output.status.success() {
        if is_compile_time_type_error(&output) {
            eprintln!(
                "  [STRICT_TYPE_OK] {}: compile-time type error",
                path.display()
            );
            return Ok(());
        }
        return Err(format!(
            "{}: STRICT_TYPE_WRONG_EXCEPTION: mamba failed with {}, not TypeError\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            status_detail(output.status),
            String::from_utf8_lossy(&output.stdout),
            stderr
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let has_typeerror = has_line_prefix(&stdout, "typeerror:");
    let has_no_typeerror = has_line_prefix(&stdout, "no_typeerror:");

    if has_typeerror && !has_no_typeerror {
        eprintln!("  [STRICT_TYPE_OK] {}: runtime TypeError", path.display());
        return Ok(());
    }
    if has_no_typeerror && !has_typeerror {
        return Err(format!(
            "{}: MAMBA_TYPE_LEAKED: mamba accepted wrong-typed code without TypeError\nstdout:\n{}",
            path.display(),
            stdout
        )
        .into());
    }

    Err(format!(
        "{}: malformed type-strict fixture output; expected exactly one of `typeerror:` or `no_typeerror:`\nstdout:\n{}",
        path.display(),
        stdout
    )
    .into())
}

fn spawn_mamba(path: &Path) -> Result<Output, String> {
    let fixture = absolute_fixture_path(path);
    let sandbox = temp_sandbox(path)?;
    let mut command = Command::new(mamba_bin());
    command
        .arg("run")
        .arg(&fixture)
        .current_dir(sandbox.path())
        .env("TMPDIR", sandbox.path())
        .env("TEMP", sandbox.path())
        .env("TMP", sandbox.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_child_limits(&mut command);

    let mut child = command
        .spawn()
        .map_err(|err| format!("{}: failed to execute mamba: {err}", path.display()))?;

    let timeout = timeout();
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child.wait_with_output().map_err(|err| {
                    format!("{}: failed to collect mamba output: {err}", path.display())
                });
            }
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let output = child.wait_with_output().map_err(|err| {
                    format!(
                        "{}: TIMEOUT after {}s; failed to collect mamba output: {err}",
                        path.display(),
                        timeout.as_secs()
                    )
                })?;
                return Err(format!(
                    "{}: TIMEOUT after {}s\nstdout:\n{}\nstderr:\n{}",
                    path.display(),
                    timeout.as_secs(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(err) => {
                return Err(format!(
                    "{}: failed to wait for mamba: {err}",
                    path.display()
                ));
            }
        }
    }
}

fn spawn_python(path: &Path) -> Result<Output, String> {
    let fixture = absolute_fixture_path(path);
    let sandbox = temp_sandbox(path)?;
    let mut command = Command::new("python3");
    command
        .arg(&fixture)
        .current_dir(sandbox.path())
        .env("TMPDIR", sandbox.path())
        .env("TEMP", sandbox.path())
        .env("TMP", sandbox.path())
        .env("PYTHONBREAKPOINT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    apply_child_limits(&mut command);

    let mut child = command
        .spawn()
        .map_err(|err| format!("{}: failed to execute python3: {err}", path.display()))?;

    let timeout = timeout();
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => {
                return child.wait_with_output().map_err(|err| {
                    format!(
                        "{}: failed to collect python3 output: {err}",
                        path.display()
                    )
                });
            }
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let output = child.wait_with_output().map_err(|err| {
                    format!(
                        "{}: TIMEOUT after {}s; failed to collect python3 output: {err}",
                        path.display(),
                        timeout.as_secs()
                    )
                })?;
                return Err(format!(
                    "{}: CPython TIMEOUT after {}s\nstdout:\n{}\nstderr:\n{}",
                    path.display(),
                    timeout.as_secs(),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(err) => {
                return Err(format!(
                    "{}: failed to wait for python3: {err}",
                    path.display()
                ));
            }
        }
    }
}

fn absolute_fixture_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn temp_sandbox(path: &Path) -> Result<tempfile::TempDir, String> {
    tempfile::Builder::new()
        .prefix("mamba-cpython-harness-")
        .tempdir_in(env::temp_dir())
        .map_err(|err| {
            format!(
                "{}: failed to create temp sandbox in {}: {err}",
                path.display(),
                env::temp_dir().display()
            )
        })
}

// ── Harness runner ────────────────────────────────────────────────

fn run_conformance(path: &Path) -> datatest_stable::Result<()> {
    // bench/*.py fixtures are owned by perf-pin Rust tests (shell-outs to
    // python3 + mamba run); they have no `.expected` goldens by design.
    // Skip them here so the conformance harness doesn't false-fail on
    // their absence. See #2239.
    if path.components().any(|c| c.as_os_str() == "bench") {
        eprintln!("  [bench-skip] {}", path.display());
        return Ok(());
    }

    let src = std::fs::read_to_string(path)?;
    let directives = parse_directives(&src);

    if has_pipeline_run_directive(&src) {
        eprintln!("  [pipeline-skip] {}", path.display());
        return Ok(());
    }

    // Skip xfail tests entirely — avoids hangs from unimplemented features
    // (e.g., generators with `while True:` compiling to infinite loops).
    // Remove `# mamba-xfail` directive to re-enable a test.
    if let Some(reason) = &directives.xfail {
        eprintln!("  [xfail] {}: {reason}", path.display());
        return Ok(());
    }

    if directives.strict_type || is_type_strict_path(path) {
        return run_type_strict(path);
    }

    // D5.6 capstone proved the dynamic CPython oracle reproduces every static
    // .expected golden (668/668 cpython-side), so conformance runs the LIVE
    // oracle for every fixture — no golden files, no regen_golden.py. The
    // fixture must exit 0 under CPython, mamba must exit 0, and stdout must match.
    let expected = spawn_python(path)?;
    if !expected.status.success() {
        return Err(format!(
            "{}: INVALID fixture: CPython ended with {}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            status_detail(expected.status),
            String::from_utf8_lossy(&expected.stdout),
            String::from_utf8_lossy(&expected.stderr)
        )
        .into());
    }

    let output = spawn_mamba(path)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let detail = resource_failure(output.status, &stderr)
            .unwrap_or_else(|| status_detail(output.status));
        return Err(format!(
            "{}: mamba run ended with {}\nstdout:\n{}\nstderr:\n{}",
            path.display(),
            detail,
            String::from_utf8_lossy(&output.stdout),
            stderr
        )
        .into());
    }

    let expected_stdout = String::from_utf8_lossy(&expected.stdout);
    let actual_stdout = String::from_utf8_lossy(&output.stdout);
    if actual_stdout != expected_stdout {
        let diff = format_diff(&expected_stdout, &actual_stdout);
        return Err(format!(
            "{}: output mismatch\n\n--- expected (CPython)\n+++ actual (mamba)\n{}",
            path.display(),
            diff
        )
        .into());
    }
    Ok(())
}

/// Simple line-by-line diff for readable test output.
fn format_diff(expected: &str, actual: &str) -> String {
    let mut out = String::new();
    let exp_lines: Vec<&str> = expected.lines().collect();
    let act_lines: Vec<&str> = actual.lines().collect();
    let max = exp_lines.len().max(act_lines.len());

    for i in 0..max {
        let e = exp_lines.get(i).copied().unwrap_or("");
        let a = act_lines.get(i).copied().unwrap_or("");
        if e != a {
            out.push_str(&format!(
                "  line {}: expected {:?}, got {:?}\n",
                i + 1,
                e,
                a
            ));
        }
    }

    if exp_lines.len() != act_lines.len() {
        out.push_str(&format!(
            "  (expected {} lines, got {} lines)\n",
            exp_lines.len(),
            act_lines.len()
        ));
    }

    out
}

harness!(run_conformance, "tests/cpython/fixtures", r".*\.py$");
