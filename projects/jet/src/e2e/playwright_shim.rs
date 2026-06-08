// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Playwright compat shim — subprocess orchestration for `jet test --playwright`.
//!
//! This module is the sole entry point for the `--playwright` escape hatch.
//! It:
//!   1. Reads `JET_SUPPRESS_PLAYWRIGHT_WARNING` from the environment.
//!   2. Emits a deprecation warning on stderr (unless suppressed).
//!   3. Spawns `npx playwright test` with the forwarded file paths.
//!   4. Propagates the subprocess exit code back to the caller.
//!
//! # Deprecation timeline
//!
//! `--playwright` is deprecated in the minor release that ships this module.
//! It will be **removed** in the second subsequent minor release.
//! See `projects/jet/docs/migration-from-playwright.md` for the migration guide.

use anyhow::Result;
use std::env::VarError;
use std::path::PathBuf;

/// URL shown in the deprecation warning. Points to the migration guide.
const MIGRATION_GUIDE_URL: &str =
    "https://github.com/cclab/jet/blob/main/projects/jet/docs/migration-from-playwright.md";

/// Environment variable that suppresses the deprecation warning when set to `"1"`.
const SUPPRESS_ENV_VAR: &str = "JET_SUPPRESS_PLAYWRIGHT_WARNING";

/// GH #3606 — distinguish `Err(VarError::NotPresent)` (canonical "not set",
/// silent) from `Err(VarError::NotUnicode(_))` (real misconfiguration, warn).
/// The prior `.map(|v| v == "1").unwrap_or(false)` collapsed both into
/// `false` so a user with a non-UTF-8 value silently saw the deprecation
/// banner anyway.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn safe_suppress_playwright_warning(
    current: Result<String, VarError>,
) -> (bool, Option<String>) {
    match current {
        Ok(v) => (v == "1", None),
        Err(VarError::NotPresent) => (false, None),
        Err(VarError::NotUnicode(_)) => (false, Some(format_safe_suppress_warn("not-unicode"))),
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_safe_suppress_warn(observed_kind: &str) -> String {
    format!(
        "GH #3606 {SUPPRESS_ENV_VAR} observed as {observed_kind}; \
         the variable is being silently ignored. The deprecation banner \
         will continue to fire until the value is re-set as valid UTF-8 \
         (= \"1\" to suppress)."
    )
}

/// Args forwarded to the Playwright subprocess.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Default)]
pub struct PlaywrightArgs {
    /// Spec files to pass to `npx playwright test` (positional arguments).
    pub files: Vec<PathBuf>,
}

/// Entry point called from `cli.rs` when `--playwright` is set.
///
/// 1. Emits deprecation warning (unless `JET_SUPPRESS_PLAYWRIGHT_WARNING=1`).
/// 2. Spawns `npx playwright test [files...]`.
/// 3. Returns the subprocess exit code (0 on success, non-zero on failure).
///
/// # REQ: R1
/// # REQ: R3
/// # REQ: R4
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn run(args: &PlaywrightArgs) -> Result<i32> {
    emit_deprecation_warning();
    spawn_playwright(args)
}

/// Prints the deprecation warning to stderr unless `JET_SUPPRESS_PLAYWRIGHT_WARNING=1`.
///
/// # REQ: R3
/// # REQ: R4
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn emit_deprecation_warning() {
    let (suppressed, warn) = safe_suppress_playwright_warning(std::env::var(SUPPRESS_ENV_VAR));
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::playwright_shim", "{}", msg);
    }
    if !suppressed {
        eprintln!(
            "warning: --playwright is deprecated and will be removed; see {}",
            MIGRATION_GUIDE_URL
        );
    }
}

/// Spawns `npx playwright test [files...]` and returns the exit code.
///
/// All file paths are forwarded as positional arguments to the Playwright
/// subprocess. stdin/stdout/stderr are inherited so Playwright's output
/// passes through to the user's terminal unchanged.
///
/// # REQ: R1
/// # REQ: R2
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn spawn_playwright(args: &PlaywrightArgs) -> Result<i32> {
    let mut cmd = std::process::Command::new("npx");
    cmd.arg("playwright").arg("test");
    for file in &args.files {
        cmd.arg(file);
    }

    let status = cmd
        .spawn()
        .and_then(|mut child| child.wait())
        .map_err(|e| anyhow::anyhow!("Failed to invoke `npx playwright test`: {}", e))?;

    // GH #3655 — was `status.code().unwrap_or(1)`. `.code()` returns
    // `None` when the child was signal-killed (SIGSEGV from a crashing
    // browser, SIGKILL from the OOM killer, SIGINT from Ctrl+C). The
    // legacy `.unwrap_or(1)` made signal-kill indistinguishable from a
    // normal Playwright test failure (which also exits 1). Apply the
    // shell `128 + signum` convention on Unix and surface a tagged warn
    // so the operator can see the signal in CI logs.
    let (code, warn) = safe_playwright_exit_code(&status);
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::playwright_shim", "{}", msg);
    }
    Ok(code)
}

/// GH #3655 — `spawn_playwright` previously did
/// `status.code().unwrap_or(1)`, silently collapsing signal-killed
/// children (`None` from `.code()`) into exit code 1 — indistinguishable
/// from a normal Playwright test failure.
///
/// This helper distinguishes:
/// - happy path (`.code() == Some(c)`): returns `(c, None)`
/// - signal-killed on Unix: returns `(128 + signum, Some(warn))` per the
///   shell convention, so 137 = SIGKILL/OOM, 139 = SIGSEGV, 130 = SIGINT.
/// - other platforms (`code()` is `None` with no signal info): returns
///   `(1, Some(warn))` — the legacy value, but with a warn so anyone
///   seeing "code 1 + no .code()" knows the case is anomalous.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn safe_playwright_exit_code(
    status: &std::process::ExitStatus,
) -> (i32, Option<String>) {
    if let Some(c) = status.code() {
        return (c, None);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signum) = status.signal() {
            let code = 128i32.saturating_add(signum);
            let warn = format_safe_playwright_exit_code_warn(Some(signum), code);
            return (code, Some(warn));
        }
    }
    let warn = format_safe_playwright_exit_code_warn(None, 1);
    (1, Some(warn))
}

/// GH #3655 — tagged warn message for [`safe_playwright_exit_code`].
/// Names the issue, the signal (if known), and the resulting exit code
/// so an operator can correlate CI exit codes with crash signals.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub(crate) fn format_safe_playwright_exit_code_warn(signum: Option<i32>, code: i32) -> String {
    match signum {
        Some(s) => format!(
            "GH #3655 jet playwright shim child was signal-killed \
             (signum={s}); returning exit code {code} per shell `128 + signum` \
             convention so signal-kill is distinguishable from test-failure-exit-1. \
             Check for crashing browser (SIGSEGV=139), OOM kill (SIGKILL=137), \
             or Ctrl+C (SIGINT=130)."
        ),
        None => format!(
            "GH #3655 jet playwright shim child ExitStatus.code() returned \
             None with no signal info available on this platform; falling \
             back to legacy exit code {code}. The case is anomalous — check \
             whether the spawn actually launched (e.g. npx playwright missing)."
        ),
    }
}

/// Validates that no native-only flags are combined with `--playwright`.
///
/// Returns `Ok(())` when the flag set is valid, or `Err` with an error message
/// and suggested exit code 2.
///
/// Called from `cli.rs` **before** `playwright_shim::run()`.
///
/// # REQ: R6
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn validate_no_native_flags(
    reporter: bool,
    trace: bool,
    workers: bool,
    shard: bool,
    report_dir: bool,
) -> Result<(), (String, i32)> {
    let conflicts = [
        (reporter, "--reporter"),
        (trace, "--trace"),
        (workers, "--workers"),
        (shard, "--shard"),
        (report_dir, "--report-dir"),
    ];

    for (set, flag) in conflicts {
        if set {
            return Err((
                format!(
                    "error: {} cannot be combined with --playwright (see migration guide: {})",
                    flag, MIGRATION_GUIDE_URL
                ),
                2,
            ));
        }
    }

    Ok(())
}

/// Returns `true` if the given source text imports `@playwright/test`.
///
/// Used to route spec files to the Playwright subprocess or emit a helpful
/// error when `--playwright` is absent.
///
/// # REQ: R2
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn imports_playwright_test(source: &str) -> bool {
    source.contains("@playwright/test")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mutex that serializes all tests that read/write env vars to prevent
    /// inter-test pollution when `cargo test` runs tests in parallel.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    // REQ: R4
    #[test]
    fn test_suppress_warning_env_var_suppresses() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // We can't easily capture stderr in a unit test without a subprocess,
        // but we can verify the env-var read logic directly.
        std::env::set_var(SUPPRESS_ENV_VAR, "1");
        let suppressed = std::env::var(SUPPRESS_ENV_VAR)
            .map(|v| v == "1")
            .unwrap_or(false);
        std::env::remove_var(SUPPRESS_ENV_VAR);
        assert!(suppressed);
    }

    // REQ: R4
    #[test]
    fn test_non_one_value_does_not_suppress() {
        let _guard = ENV_MUTEX.lock().unwrap();
        std::env::remove_var(SUPPRESS_ENV_VAR); // ensure clean state
        std::env::set_var(SUPPRESS_ENV_VAR, "true");
        let suppressed = std::env::var(SUPPRESS_ENV_VAR)
            .map(|v| v == "1")
            .unwrap_or(false);
        std::env::remove_var(SUPPRESS_ENV_VAR);
        assert!(!suppressed);
    }

    // REQ: R6
    #[test]
    fn test_reporter_flag_rejected() {
        let result = validate_no_native_flags(true, false, false, false, false);
        assert!(result.is_err());
        let (msg, code) = result.unwrap_err();
        assert_eq!(code, 2);
        assert!(msg.contains("--reporter"));
        assert!(msg.contains("--playwright"));
    }

    // REQ: R6
    #[test]
    fn test_trace_flag_rejected() {
        let result = validate_no_native_flags(false, true, false, false, false);
        assert!(result.is_err());
        let (msg, code) = result.unwrap_err();
        assert_eq!(code, 2);
        assert!(msg.contains("--trace"));
    }

    // REQ: R6
    #[test]
    fn test_workers_flag_rejected() {
        let result = validate_no_native_flags(false, false, true, false, false);
        assert!(result.is_err());
        let (msg, code) = result.unwrap_err();
        assert_eq!(code, 2);
        assert!(msg.contains("--workers"));
    }

    // REQ: R6
    #[test]
    fn test_shard_flag_rejected() {
        let result = validate_no_native_flags(false, false, false, true, false);
        assert!(result.is_err());
        let (msg, code) = result.unwrap_err();
        assert_eq!(code, 2);
        assert!(msg.contains("--shard"));
    }

    // REQ: R6
    #[test]
    fn test_report_dir_flag_rejected() {
        let result = validate_no_native_flags(false, false, false, false, true);
        assert!(result.is_err());
        let (msg, code) = result.unwrap_err();
        assert_eq!(code, 2);
        assert!(msg.contains("--report-dir"));
    }

    // REQ: R6
    #[test]
    fn test_no_conflict_passes() {
        let result = validate_no_native_flags(false, false, false, false, false);
        assert!(result.is_ok());
    }

    // REQ: R2
    #[test]
    fn test_imports_playwright_test_detected() {
        let source = r#"import { test, expect } from '@playwright/test';"#;
        assert!(imports_playwright_test(source));
    }

    // REQ: R2
    #[test]
    fn test_imports_playwright_test_not_detected() {
        let source = r#"import { describe, it } from 'vitest';"#;
        assert!(!imports_playwright_test(source));
    }

    // REQ: R2
    #[test]
    fn test_imports_playwright_test_double_quotes() {
        let source = r#"import { test } from "@playwright/test";"#;
        assert!(imports_playwright_test(source));
    }
}

#[cfg(test)]
mod gh3606_suppress_warn_tests {
    //! GH #3606 — env var `JET_SUPPRESS_PLAYWRIGHT_WARNING` must
    //! distinguish `NotPresent` (canonical, silent) from `NotUnicode`
    //! (misconfiguration, warn). Prior `.unwrap_or(false)` collapsed
    //! both into `false`.
    use super::*;

    #[test]
    fn ok_one_suppresses_with_no_warn() {
        let (suppressed, warn) = safe_suppress_playwright_warning(Ok("1".to_string()));
        assert!(suppressed);
        assert!(warn.is_none());
    }

    #[test]
    fn ok_other_value_does_not_suppress() {
        let (suppressed, warn) = safe_suppress_playwright_warning(Ok("true".to_string()));
        assert!(!suppressed, "only literal \"1\" must suppress");
        assert!(warn.is_none());
    }

    #[test]
    fn not_present_silently_does_not_suppress() {
        let (suppressed, warn) = safe_suppress_playwright_warning(Err(VarError::NotPresent));
        assert!(!suppressed);
        assert!(
            warn.is_none(),
            "NotPresent is canonical, must not emit a warn"
        );
    }

    #[test]
    fn not_unicode_does_not_suppress_and_warns() {
        let raw = std::ffi::OsString::from("ignored");
        let (suppressed, warn) = safe_suppress_playwright_warning(Err(VarError::NotUnicode(raw)));
        assert!(
            !suppressed,
            "NotUnicode must not suppress (no usable value)"
        );
        let msg = warn.expect("NotUnicode must emit a warn");
        assert!(msg.contains("GH #3606"), "msg: {msg}");
        assert!(msg.contains("not-unicode"), "msg: {msg}");
        assert!(msg.contains(SUPPRESS_ENV_VAR), "msg: {msg}");
    }

    #[test]
    fn warn_helper_tags_issue_and_kind() {
        let msg = format_safe_suppress_warn("not-unicode");
        assert!(msg.contains("GH #3606"), "msg: {msg}");
        assert!(msg.contains("not-unicode"), "msg: {msg}");
        assert!(msg.contains(SUPPRESS_ENV_VAR), "msg: {msg}");
    }
}

#[cfg(test)]
mod gh3655_safe_playwright_exit_code_tests {
    //! GH #3655 — `spawn_playwright` used to do `status.code().unwrap_or(1)`,
    //! collapsing signal-killed children (e.g. SIGSEGV from a crashing
    //! browser, SIGKILL from OOM) into exit code 1 indistinguishable from
    //! a regular Playwright test failure. Safe helper applies the shell
    //! `128 + signum` convention.
    use super::*;

    fn run_to_status(args: &[&str]) -> std::process::ExitStatus {
        // Use `sh -c "<args>"` so we can shape the child's exit behavior.
        // `sh` is portable across the macOS/Linux dev/CI surface this
        // project actually targets.
        let mut cmd = std::process::Command::new("sh");
        cmd.arg("-c").args(args);
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
        cmd.spawn().expect("spawn sh").wait().expect("wait sh")
    }

    #[test]
    fn exit_zero_is_happy_path() {
        let status = run_to_status(&["exit 0"]);
        let (code, warn) = safe_playwright_exit_code(&status);
        assert_eq!(code, 0);
        assert!(warn.is_none(), "exit 0 must not emit a warn");
    }

    #[test]
    fn exit_one_is_happy_path() {
        let status = run_to_status(&["exit 1"]);
        let (code, warn) = safe_playwright_exit_code(&status);
        assert_eq!(code, 1);
        assert!(warn.is_none(), "exit 1 (test-failure) must not emit warn");
    }

    #[test]
    fn exit_two_is_happy_path() {
        let status = run_to_status(&["exit 2"]);
        let (code, warn) = safe_playwright_exit_code(&status);
        assert_eq!(code, 2);
        assert!(warn.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn sigterm_returns_143_with_warn() {
        // `sh -c 'kill -TERM $$'` self-signals so we exercise the
        // signal branch without depending on `kill <pid>` timing.
        let status = run_to_status(&["kill -TERM $$"]);
        let (code, warn) = safe_playwright_exit_code(&status);
        assert_eq!(code, 128 + 15, "SIGTERM → 143 per shell convention");
        let msg = warn.expect("signal-killed must emit a warn");
        assert!(msg.contains("GH #3655"), "msg: {msg}");
        assert!(msg.contains("signum=15"), "msg: {msg}");
        assert!(msg.contains("143"), "msg: {msg}");
    }

    #[cfg(unix)]
    #[test]
    fn sigkill_returns_137_with_warn() {
        let status = run_to_status(&["kill -KILL $$"]);
        let (code, warn) = safe_playwright_exit_code(&status);
        assert_eq!(code, 128 + 9, "SIGKILL → 137 (OOM kill convention)");
        let msg = warn.expect("signal-killed must emit a warn");
        assert!(msg.contains("GH #3655"), "msg: {msg}");
        assert!(msg.contains("signum=9"), "msg: {msg}");
    }

    #[cfg(unix)]
    #[test]
    fn warn_formatter_includes_tag_and_signum() {
        let msg = format_safe_playwright_exit_code_warn(Some(11), 139);
        assert!(msg.contains("GH #3655"), "msg: {msg}");
        assert!(msg.contains("signum=11"), "msg: {msg}");
        assert!(msg.contains("139"), "msg: {msg}");
    }

    #[test]
    fn warn_formatter_no_signum_path_includes_tag_and_anomaly_note() {
        let msg = format_safe_playwright_exit_code_warn(None, 1);
        assert!(msg.contains("GH #3655"), "msg: {msg}");
        assert!(msg.contains("None"), "msg: {msg}");
        assert!(msg.contains("anomalous"), "msg: {msg}");
    }

    #[test]
    fn helper_name_pin() {
        // Pin: family convention is `safe_*`. If a future rename breaks
        // this, the loop's grep tooling needs to know.
        let _ = safe_playwright_exit_code as fn(&std::process::ExitStatus) -> (i32, Option<String>);
    }
}
// CODEGEN-END
