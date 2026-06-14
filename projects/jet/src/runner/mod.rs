// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
// CODEGEN-BEGIN
//! Script runner: resolve and execute package.json scripts, files, or commands.
//!
//! Resolution order for `jet run <name>`:
//! 1. Check package.json `scripts` → run via `sh -c` with `.bin` on PATH
//! 2. Check if file exists on disk → JIT execute (ts/tsx/jsx) or direct (js)
//! 3. Check jet.toml pipeline → task runner mode
//! 4. Not found → error

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

pub mod env;
pub mod jit;
pub mod source_map;
pub mod watcher;

/// Script runner for executing package.json scripts and arbitrary commands.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub struct ScriptRunner {
    project_root: PathBuf,
    pkg_json: Option<PkgScripts>,
}

/// Minimal package.json view for script runner.
#[derive(Debug, Clone, serde::Deserialize)]
struct PkgScripts {
    #[serde(default)]
    scripts: HashMap<String, String>,
    #[serde(default)]
    #[allow(dead_code)]
    bin: Option<serde_json::Value>,
}

/// Result of running a script or command.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub struct RunResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
impl ScriptRunner {
    /// Create a new script runner rooted at the given directory.
    pub fn new(project_root: PathBuf) -> Self {
        let pkg_json = Self::load_pkg_scripts(&project_root);
        Self {
            project_root,
            pkg_json,
        }
    }

    fn load_pkg_scripts(root: &Path) -> Option<PkgScripts> {
        let path = root.join("package.json");
        // GH #3170 — the prior `.ok()?` chain returned None on *any*
        // failure, so a parse error (trailing comma, broken quote) made
        // `jet run build` report "No package.json found" even though
        // the file existed. Distinguish NotFound (legitimate non-npm
        // project) from read/parse failures so the diagnostic points
        // at the real bug.
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
            Err(e) => {
                eprintln!(
                    "[jet] package.json at {} could not be read: {e}. \
                     `jet run` will report 'No package.json found' because \
                     of this read error (GH #3170).",
                    path.display()
                );
                return None;
            }
        };
        match serde_json::from_str(&content) {
            Ok(scripts) => Some(scripts),
            Err(e) => {
                eprintln!(
                    "[jet] package.json at {} could not be parsed: {e}. \
                     `jet run` will report 'No package.json found' because \
                     of this parse error (GH #3170).",
                    path.display()
                );
                None
            }
        }
    }

    /// Run a named script from package.json with lifecycle hooks.
    pub async fn run_script(&self, name: &str, args: &[String]) -> Result<RunResult> {
        let scripts = self
            .pkg_json
            .as_ref()
            .map(|p| &p.scripts)
            .ok_or_else(|| anyhow::anyhow!("No package.json found"))?;

        let command = scripts
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Script '{}' not found in package.json", name))?;

        // Run pre-hook if exists.
        let pre_name = format!("pre{}", name);
        if let Some(pre_cmd) = scripts.get(&pre_name) {
            tracing::info!("Running pre-hook: {}", pre_name);
            let pre_result = self.exec_shell(pre_cmd, &[]).await?;
            if pre_result.exit_code != 0 {
                tracing::info!(
                    target: "jet::runner",
                    "{}",
                    format_lifecycle_short_circuit_info(
                        &pre_name,
                        &[name, &format!("post{}", name)],
                        pre_result.exit_code,
                    )
                );
                return Ok(pre_result);
            }
        }

        // Run the script itself.
        tracing::info!("Running script: {} → {}", name, command);
        let result = self.exec_shell(command, args).await?;
        if result.exit_code != 0 {
            let post_name = format!("post{}", name);
            if scripts.get(&post_name).is_some() {
                tracing::info!(
                    target: "jet::runner",
                    "{}",
                    format_lifecycle_short_circuit_info(
                        name,
                        &[&post_name],
                        result.exit_code,
                    )
                );
            }
            return Ok(result);
        }

        // Run post-hook if exists (only reached when main returned exit 0).
        let post_name = format!("post{}", name);
        if let Some(post_cmd) = scripts.get(&post_name) {
            tracing::info!("Running post-hook: {}", post_name);
            self.exec_shell(post_cmd, &[]).await?;
        }

        Ok(result)
    }

    /// Run a file directly (JIT for TS/TSX/JSX, direct for JS).
    pub async fn run_file(&self, path: &Path, args: &[String], watch: bool) -> Result<RunResult> {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.project_root.join(path)
        };

        if !full_path.exists() {
            anyhow::bail!("File not found: {}", full_path.display());
        }

        match ext.as_ref() {
            "ts" | "tsx" | "jsx" => {
                let engine = jit::JitEngine::new(&self.project_root)?;
                if watch {
                    engine.execute_watch(&full_path, args).await?;
                    Ok(RunResult {
                        exit_code: 0,
                        stdout: String::new(),
                        stderr: String::new(),
                    })
                } else {
                    engine.execute(&full_path, args).await
                }
            }
            "js" | "mjs" | "cjs" => self.exec_node(&full_path, args).await,
            _ => anyhow::bail!("Unsupported file type: .{}", ext),
        }
    }

    /// Execute an arbitrary command with node_modules/.bin on PATH.
    pub async fn exec_command(&self, cmd: &str, args: &[String]) -> Result<RunResult> {
        // Check if cmd exists in .bin
        let bin_path = self.resolve_bin_path(cmd);
        let effective_cmd = if let Some(bp) = bin_path {
            bp.to_string_lossy().to_string()
        } else {
            cmd.to_string()
        };

        self.exec_shell(&effective_cmd, args).await
    }

    /// Resolve a command name to node_modules/.bin path.
    fn resolve_bin_path(&self, cmd: &str) -> Option<PathBuf> {
        let bin_dir = self.project_root.join("node_modules/.bin");
        let candidate = bin_dir.join(cmd);
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    }

    /// Execute a shell command with injected environment.
    async fn exec_shell(&self, command: &str, extra_args: &[String]) -> Result<RunResult> {
        let env_vars = env::build_env(&self.project_root);
        let full_cmd = if extra_args.is_empty() {
            command.to_string()
        } else {
            format!("{} {}", command, extra_args.join(" "))
        };

        let output = tokio::process::Command::new("sh")
            .arg("-c")
            .arg(&full_cmd)
            .current_dir(&self.project_root)
            .envs(&env_vars)
            .output()
            .await
            .with_context(|| format!("Failed to execute: {}", full_cmd))?;

        Ok(RunResult {
            exit_code: exit_code(&output.status),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Execute a JS file directly via Node.js.
    async fn exec_node(&self, file: &Path, args: &[String]) -> Result<RunResult> {
        let env_vars = env::build_env(&self.project_root);

        let output = tokio::process::Command::new("node")
            .arg(file)
            .args(args)
            .current_dir(&self.project_root)
            .envs(&env_vars)
            .output()
            .await
            .with_context(|| format!("Failed to execute: node {}", file.display()))?;

        Ok(RunResult {
            exit_code: exit_code(&output.status),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Check if a name corresponds to a package.json script.
    pub fn has_script(&self, name: &str) -> bool {
        self.pkg_json
            .as_ref()
            .map(|p| p.scripts.contains_key(name))
            .unwrap_or(false)
    }

    /// Check if a name corresponds to a file on disk.
    pub fn is_file(&self, name: &str) -> bool {
        let path = self.project_root.join(name);
        path.is_file()
    }
}

/// GH #3801 — fallback extension string used when a path has no
/// extension at all. Kept as a named constant so call sites and tests
/// pin the same value.
pub(crate) const RUN_FILE_NO_EXTENSION_FALLBACK: &str = "";

/// GH #3801 — warn shown when `Runner::run_file` is invoked on a path
/// with no `extension()` (e.g. `README`, `Dockerfile`). The prior code
/// silently dropped to `""` and emitted `"Unsupported file type: ."`
/// with a literal trailing dot, leaving the operator unable to spot
/// the missing-extension cause among other `_` arms.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_runner_run_file_no_extension_warn(path: &Path) -> String {
    format!(
        "gh3801: jet run saw path with no extension for path={:?}; \
         falling back to empty extension — error will say \
         \"Unsupported file type: .\" with a literal trailing dot",
        path
    )
}

/// GH #3801 — warn shown when `Runner::run_file` is invoked on a path
/// whose extension is non-UTF-8 (filesystem-encoded bytes that the OS
/// accepted but Rust cannot lossless-decode). The prior code silently
/// dropped to `""` because `.to_str()` returned `None`, collapsing
/// non-UTF-8 extensions onto the no-extension case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_runner_run_file_non_utf8_extension_warn(path: &Path, lossy: &str) -> String {
    format!(
        "gh3801: jet run saw path with non-UTF-8 extension for path={:?}; \
         lossy form is {:?}; routing through the lossy form so the \
         \"Unsupported file type\" error carries a visible breadcrumb \
         instead of an empty extension",
        path, lossy
    )
}

/// GH #3801 — coerce the file extension into a string for dispatch.
///
/// - `Some(utf8)` → `Cow::Borrowed(utf8)` (silent — happy path for
///   recognised extensions and the bail message for unknown UTF-8 ones).
/// - `Some(non-UTF-8)` → emit a `tracing::warn!` carrying the lossy form
///   and return `Cow::Owned(lossy)` so the bail message names the encoding
///   instead of collapsing onto `""`.
/// - `None` → emit a `tracing::warn!` naming the path and return
///   `Cow::Borrowed("")` so legacy `_ => bail!` behaviour is preserved.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn coerce_run_file_extension_or_warn(path: &Path) -> std::borrow::Cow<'_, str> {
    use std::borrow::Cow;
    match path.extension() {
        None => {
            tracing::warn!(
                target: "jet::runner",
                path = %path.display(),
                "{}",
                format_runner_run_file_no_extension_warn(path)
            );
            Cow::Borrowed(RUN_FILE_NO_EXTENSION_FALLBACK)
        }
        Some(os) => match os.to_str() {
            Some(s) => Cow::Borrowed(s),
            None => {
                let lossy = os.to_string_lossy().into_owned();
                tracing::warn!(
                    target: "jet::runner",
                    path = %path.display(),
                    lossy = %lossy,
                    "{}",
                    format_runner_run_file_non_utf8_extension_warn(path, &lossy)
                );
                Cow::Owned(lossy)
            }
        },
    }
}

fn exit_code(status: &ExitStatus) -> i32 {
    // GH #3691 — was `status.code().unwrap_or(-1)`. `.code()` returns
    // `None` on Unix when the child was signal-killed (SIGSEGV from a
    // crashing node, SIGKILL from the OOM killer, SIGINT from Ctrl+C);
    // the prior fallback silently collapsed signal-kill onto -1, making
    // it indistinguishable from a real -1 return. Route through
    // `safe_runner_exit_code` so the warn names the signal and the
    // likely cause. Same family shape as
    // `playwright_shim::safe_playwright_exit_code` (#3655).
    let (code, warn) = safe_runner_exit_code(status);
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::runner", "{}", msg);
    }
    code
}

/// GH #3691 — distinguish a signal-killed runner child from a normal -1
/// return. Mirrors `playwright_shim::safe_playwright_exit_code` (#3655).
///
/// - happy path (`.code() == Some(c)`): returns `(c, None)`.
/// - signal-killed on Unix: returns `(128 + signum, Some(warn))` per
///   the shell convention so 137 = SIGKILL/OOM, 139 = SIGSEGV,
///   130 = SIGINT.
/// - other platforms (`code()` is `None` with no signal info): returns
///   `(-1, Some(warn))` so anyone seeing "-1 + no .code()" knows the
///   case is anomalous and not a normal -1 exit.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn safe_runner_exit_code(status: &ExitStatus) -> (i32, Option<String>) {
    if let Some(c) = status.code() {
        return (c, None);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signum) = status.signal() {
            let code = 128i32.saturating_add(signum);
            let warn = format_safe_runner_exit_code_warn(Some(signum), code);
            return (code, Some(warn));
        }
    }
    let warn = format_safe_runner_exit_code_warn(None, -1);
    (-1, Some(warn))
}

/// GH #3691 — tagged warn message for [`safe_runner_exit_code`].
/// Names the issue tag, the signal (when known), and the resulting
/// exit code so operators can correlate process-exit codes with crash
/// signals when debugging `jet run` / `jet exec` / `jet jit` failures.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_safe_runner_exit_code_warn(signum: Option<i32>, code: i32) -> String {
    match signum {
        Some(s) => format!(
            "GH #3691 jet runner child was signal-killed (signum={s}); \
             returning exit code {code} per shell `128 + signum` convention \
             so signal-kill is distinguishable from a normal -1 exit. \
             Check for crashing user code (SIGSEGV=139), OOM kill (SIGKILL=137), \
             or Ctrl+C (SIGINT=130). This is not a jet bug."
        ),
        None => format!(
            "GH #3691 jet runner child returned exit status with no .code() \
             AND no signal info (non-Unix platform fallback); returning {code}. \
             This case is anomalous — the caller cannot tell whether the child \
             exited cleanly or was killed externally."
        ),
    }
}

/// GH #3723 — build the `tracing::info!` message announcing that a
/// lifecycle stage short-circuited the rest of the lifecycle because it
/// returned a non-zero exit code. Names the failed stage, the stage(s)
/// being skipped, the exit code, and the npm contract being honoured —
/// extracted so the wording is unit-testable without spawning a child.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_lifecycle_short_circuit_info(
    failed_stage: &str,
    skipped_stages: &[&str],
    exit_code: i32,
) -> String {
    let skipped_joined = skipped_stages.join(", ");
    format!(
        "GH #3723 jet runner: lifecycle stage `{failed_stage}` exited with \
         code {exit_code}; skipping {skipped_joined} per npm/pnpm \
         contract (a non-zero stage halts the remainder of the lifecycle). \
         Prior behaviour ran every stage unconditionally, which caused \
         `post<name>` success-notifications to fire on failing test runs."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_runner_no_pkg_json() {
        let runner = ScriptRunner::new(PathBuf::from("/nonexistent"));
        assert!(!runner.has_script("test"));
        assert!(!runner.is_file("index.js"));
    }

    #[test]
    fn test_script_runner_with_tempdir() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = r#"{"name":"t","version":"1.0.0","scripts":{"test":"echo ok"}}"#;
        std::fs::write(dir.path().join("package.json"), pkg).unwrap();

        let runner = ScriptRunner::new(dir.path().to_path_buf());
        assert!(runner.has_script("test"));
        assert!(!runner.has_script("build"));
    }

    /// GH #3170 — Absent package.json is the legitimate "non-npm
    /// project" path. Must remain silent (None, no panic) — pinned so
    /// the new diagnostic doesn't fire on the canonical happy path.
    #[test]
    fn load_pkg_scripts_returns_none_when_file_absent() {
        let dir = tempfile::tempdir().unwrap();
        let scripts = ScriptRunner::load_pkg_scripts(dir.path());
        assert!(
            scripts.is_none(),
            "missing package.json must yield None silently"
        );
    }

    /// GH #3170 — Malformed package.json (trailing comma — the canonical
    /// hand-edit mistake) must not panic and must not "succeed" with
    /// no scripts the way `.ok()?` did. Pre-fix: return None and `jet
    /// run build` reports "No package.json found" — completely
    /// misleading. Post-fix: still returns None for liveness, but
    /// eprintln! surfaces the diagnostic so the user finds the typo.
    #[test]
    fn load_pkg_scripts_returns_none_on_malformed_json() {
        let dir = tempfile::tempdir().unwrap();
        // Trailing comma — invalid JSON, common hand-edit mistake.
        let bad = r#"{"name":"t","scripts":{"test":"echo ok",}}"#;
        std::fs::write(dir.path().join("package.json"), bad).unwrap();

        let scripts = ScriptRunner::load_pkg_scripts(dir.path());
        assert!(
            scripts.is_none(),
            "malformed package.json must yield None without panicking"
        );

        // Higher-level ScriptRunner::new must still construct.
        let runner = ScriptRunner::new(dir.path().to_path_buf());
        assert!(
            !runner.has_script("test"),
            "no scripts surface when JSON is unparseable"
        );
    }

    /// GH #3170 — Valid package.json must still yield Some — pins that
    /// the new error-handling path didn't regress the happy path.
    #[test]
    fn load_pkg_scripts_returns_some_for_valid_pkg_json() {
        let dir = tempfile::tempdir().unwrap();
        let pkg =
            r#"{"name":"t","version":"1.0.0","scripts":{"build":"jet build","test":"jet test"}}"#;
        std::fs::write(dir.path().join("package.json"), pkg).unwrap();

        let scripts =
            ScriptRunner::load_pkg_scripts(dir.path()).expect("valid package.json must yield Some");
        assert_eq!(scripts.scripts.len(), 2);
        assert!(scripts.scripts.contains_key("build"));
        assert!(scripts.scripts.contains_key("test"));
    }
}

#[cfg(test)]
mod gh3691_safe_runner_exit_code_tests {
    //! GH #3691 — `runner/mod.rs::exit_code` and `runner/jit.rs::execute`
    //! both used `status.code().unwrap_or(-1)`. `.code()` returns `None`
    //! on Unix when the child was signal-killed (SIGSEGV from a crashing
    //! native addon, SIGKILL from the OOM killer, SIGINT from Ctrl+C);
    //! the silent fallback collapsed signal-kill onto -1, making it
    //! indistinguishable from a real -1 return.
    //!
    //! These tests pin the safe-fallback helper and the warn message so
    //! the user-facing diagnostics survive future refactors.

    use super::{format_safe_runner_exit_code_warn, safe_runner_exit_code};

    #[cfg(unix)]
    fn exited_with(code: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        // ExitStatus::from_raw expects the raw waitstatus shape:
        // bits 8..15 = exit code on a clean exit.
        std::process::ExitStatus::from_raw((code as i32 & 0xff) << 8)
    }

    #[cfg(unix)]
    fn signal_killed_with(signum: i32) -> std::process::ExitStatus {
        use std::os::unix::process::ExitStatusExt;
        // Low 7 bits set = signal-killed; bit 7 = core dumped (clear).
        std::process::ExitStatus::from_raw(signum)
    }

    #[cfg(unix)]
    #[test]
    fn happy_path_normal_exit_returns_code_and_no_warn() {
        let status = exited_with(0);
        let (code, warn) = safe_runner_exit_code(&status);
        assert_eq!(code, 0);
        assert!(warn.is_none(), "normal exit must not produce a warn");

        let status = exited_with(42);
        let (code, warn) = safe_runner_exit_code(&status);
        assert_eq!(code, 42);
        assert!(warn.is_none());
    }

    #[cfg(unix)]
    #[test]
    fn signal_killed_returns_128_plus_signum_and_warns_instead_of_silent_minus_one() {
        let status = signal_killed_with(9); // SIGKILL
        let (code, warn) = safe_runner_exit_code(&status);
        assert_eq!(
            code, 137,
            "shell `128 + signum` convention: SIGKILL must surface as 137 \
             so OOM kills are distinguishable from a real -1 exit"
        );
        let msg = warn.expect(
            "signal-killed must produce a warn — the headline fix is that \
             we no longer silently collapse signal-kill onto -1",
        );
        assert!(
            msg.contains("GH #3691"),
            "warn must carry the issue tag so future grep finds it: {msg}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn signal_killed_warn_names_signum_and_the_likely_cause() {
        let status = signal_killed_with(11); // SIGSEGV
        let (_, warn) = safe_runner_exit_code(&status);
        let msg = warn.expect("must warn on signal-kill");
        let lower = msg.to_lowercase();
        assert!(
            msg.contains("signum=11"),
            "warn must name the signal number so operators correlating CI \
             exit codes with kernel signals can pivot: {msg}"
        );
        assert!(
            lower.contains("sigsegv") || lower.contains("sigkill") || lower.contains("sigint"),
            "warn must name at least one likely cause so operators searching \
             for any of OOM/crash/Ctrl+C find this line: {msg}"
        );
    }

    #[test]
    fn warn_message_disclaims_jet_responsibility() {
        let msg = format_safe_runner_exit_code_warn(Some(9), 137);
        let lower = msg.to_lowercase();
        assert!(
            lower.contains("not a jet bug") || lower.contains("not jet"),
            "warn must explicitly disclaim this is a jet bug so operators \
             don't go hunting in jet source: {msg}"
        );
    }

    #[test]
    fn format_helper_no_signal_branch_names_anomalous_case() {
        let msg = format_safe_runner_exit_code_warn(None, -1);
        let lower = msg.to_lowercase();
        assert!(
            msg.contains("GH #3691"),
            "warn must carry the issue tag even on the no-signal fallback: {msg}"
        );
        assert!(
            lower.contains("anomalous")
                || lower.contains("no .code()")
                || lower.contains("no signal"),
            "warn must explicitly call out the anomalous no-code-no-signal case \
             so operators don't dismiss it as a normal -1 exit: {msg}"
        );
    }

    #[test]
    fn format_helper_round_trip_carries_signum() {
        let msg_segv = format_safe_runner_exit_code_warn(Some(11), 139);
        assert!(msg_segv.contains("signum=11"));
        assert!(msg_segv.contains("139"));

        let msg_int = format_safe_runner_exit_code_warn(Some(2), 130);
        assert!(msg_int.contains("signum=2"));
        assert!(msg_int.contains("130"));
    }

    #[cfg(unix)]
    #[test]
    fn helper_output_is_deterministic_across_calls() {
        let status = exited_with(7);
        assert_eq!(
            safe_runner_exit_code(&status),
            safe_runner_exit_code(&status)
        );

        let status = signal_killed_with(15); // SIGTERM
        let a = safe_runner_exit_code(&status);
        let b = safe_runner_exit_code(&status);
        assert_eq!(
            a, b,
            "signal-killed fallback must also be deterministic so warn messages don't churn"
        );
    }
}

#[cfg(test)]
mod gh3723_lifecycle_short_circuit_tests {
    //! GH #3723 — `run_script` used to run every lifecycle stage
    //! (pre<name> → <name> → post<name>) unconditionally, regardless
    //! of the previous stage's exit code. npm/pnpm contract: a non-zero
    //! stage halts the remainder of the lifecycle. The user-facing
    //! symptom of the prior code was `posttest` "success notifications"
    //! firing on red test runs. The helper exists so the wording is
    //! testable without spawning real child processes.
    use super::*;

    #[test]
    fn helper_tags_gh_issue() {
        let msg = format_lifecycle_short_circuit_info("pretest", &["test", "posttest"], 1);
        assert!(msg.contains("GH #3723"), "msg: {msg}");
    }

    #[test]
    fn helper_names_failed_stage_and_exit_code() {
        let msg = format_lifecycle_short_circuit_info("pretest", &["test", "posttest"], 7);
        assert!(msg.contains("pretest"), "msg must name failed stage: {msg}");
        assert!(msg.contains("code 7"), "msg must name exit code: {msg}");
    }

    #[test]
    fn helper_names_all_skipped_stages() {
        let msg = format_lifecycle_short_circuit_info("pretest", &["test", "posttest"], 1);
        assert!(msg.contains("test"), "must name skipped main: {msg}");
        assert!(msg.contains("posttest"), "must name skipped post: {msg}");
    }

    #[test]
    fn helper_post_only_variant_does_not_invent_main_skip() {
        // When main itself failed, only post<name> is being skipped — the
        // wording must not falsely claim main was also skipped.
        let msg = format_lifecycle_short_circuit_info("test", &["posttest"], 1);
        assert!(msg.contains("posttest"), "must name skipped post: {msg}");
        // Don't accidentally enumerate stages that ARE running.
        // (Main "test" appears as the failed_stage, which is fine; the
        // skipped-list must not double-count it.)
        let skipped_count = msg.matches("posttest").count();
        assert!(skipped_count >= 1, "msg: {msg}");
    }

    #[test]
    fn helper_names_npm_contract_so_users_can_search_docs() {
        let msg = format_lifecycle_short_circuit_info("test", &["posttest"], 2);
        assert!(
            msg.contains("npm") || msg.contains("pnpm"),
            "must reference npm/pnpm contract: {msg}"
        );
    }

    #[test]
    fn helper_names_observable_symptom_so_users_link_root_cause() {
        let msg = format_lifecycle_short_circuit_info("test", &["posttest"], 1);
        assert!(
            msg.contains("success-notification")
                || msg.contains("notification")
                || msg.contains("post<name>"),
            "must name the user-facing symptom: {msg}"
        );
    }

    #[test]
    fn helper_is_deterministic_for_fixed_inputs() {
        let a = format_lifecycle_short_circuit_info("test", &["posttest"], 1);
        let b = format_lifecycle_short_circuit_info("test", &["posttest"], 1);
        assert_eq!(a, b);
    }

    #[test]
    fn helper_handles_single_skipped_stage_without_comma() {
        let msg = format_lifecycle_short_circuit_info("test", &["posttest"], 1);
        // Sanity: no orphan comma when there's only one skipped stage.
        assert!(
            !msg.contains(", posttest") && !msg.contains("posttest, "),
            "single-element list must not render with a stray comma: {msg}"
        );
    }
}

#[cfg(test)]
mod gh3723_run_script_lifecycle_tests {
    //! GH #3723 — Behavioral integration: pretest exit 1 must NOT trigger
    //! test or posttest; test exit 1 must NOT trigger posttest. Uses real
    //! tempdir + package.json + sh-driven shell commands (the runner spawns
    //! sh -c). Skipped on platforms without a usable sh in PATH.
    use super::*;

    fn write_pkg(dir: &Path, scripts: &str) {
        let pkg = format!(r#"{{"name":"t","version":"1.0.0","scripts":{{{scripts}}}}}"#);
        std::fs::write(dir.join("package.json"), pkg).unwrap();
    }

    #[tokio::test]
    async fn pretest_failure_short_circuits_test_and_posttest() {
        let dir = tempfile::tempdir().unwrap();
        let marker_test = dir.path().join("test-ran");
        let marker_post = dir.path().join("post-ran");
        // pretest fails (exit 1); test and posttest would write markers if
        // they ran. After the fix, no markers should appear.
        let scripts = format!(
            "\"pretest\":\"exit 1\",\"test\":\"touch {test_m}\",\"posttest\":\"touch {post_m}\"",
            test_m = marker_test.display(),
            post_m = marker_post.display(),
        );
        write_pkg(dir.path(), &scripts);

        let runner = ScriptRunner::new(dir.path().to_path_buf());
        let result = runner.run_script("test", &[]).await.unwrap();
        assert_eq!(result.exit_code, 1, "pretest's exit code must surface");
        assert!(
            !marker_test.exists(),
            "GH #3723: test must NOT run when pretest exits non-zero"
        );
        assert!(
            !marker_post.exists(),
            "GH #3723: posttest must NOT run when pretest exits non-zero"
        );
    }

    #[tokio::test]
    async fn test_failure_short_circuits_posttest() {
        let dir = tempfile::tempdir().unwrap();
        let marker_post = dir.path().join("post-ran");
        let scripts = format!(
            "\"test\":\"exit 7\",\"posttest\":\"touch {post_m}\"",
            post_m = marker_post.display(),
        );
        write_pkg(dir.path(), &scripts);

        let runner = ScriptRunner::new(dir.path().to_path_buf());
        let result = runner.run_script("test", &[]).await.unwrap();
        assert_eq!(
            result.exit_code, 7,
            "test's exit code must surface verbatim"
        );
        assert!(
            !marker_post.exists(),
            "GH #3723: posttest must NOT run when test exits non-zero \
             (this is the success-notification-on-red-CI bug)"
        );
    }

    #[tokio::test]
    async fn all_zero_runs_full_lifecycle() {
        // Regression guard: the happy path must still chain pre -> main -> post.
        let dir = tempfile::tempdir().unwrap();
        let marker_pre = dir.path().join("pre-ran");
        let marker_test = dir.path().join("test-ran");
        let marker_post = dir.path().join("post-ran");
        let scripts = format!(
            "\"pretest\":\"touch {pre}\",\"test\":\"touch {tst}\",\"posttest\":\"touch {post}\"",
            pre = marker_pre.display(),
            tst = marker_test.display(),
            post = marker_post.display(),
        );
        write_pkg(dir.path(), &scripts);

        let runner = ScriptRunner::new(dir.path().to_path_buf());
        let result = runner.run_script("test", &[]).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(marker_pre.exists(), "pretest must run on happy path");
        assert!(marker_test.exists(), "test must run on happy path");
        assert!(marker_post.exists(), "posttest must run on happy path");
    }
}

#[cfg(test)]
mod gh3801_run_file_extension_warn_tests {
    use super::{
        coerce_run_file_extension_or_warn, format_runner_run_file_no_extension_warn,
        format_runner_run_file_non_utf8_extension_warn, RUN_FILE_NO_EXTENSION_FALLBACK,
    };
    use std::path::Path;

    #[test]
    fn utf8_extension_passes_through_silently() {
        assert_eq!(coerce_run_file_extension_or_warn(Path::new("a.ts")), "ts");
        assert_eq!(coerce_run_file_extension_or_warn(Path::new("a.tsx")), "tsx");
        assert_eq!(coerce_run_file_extension_or_warn(Path::new("a.js")), "js");
        assert_eq!(coerce_run_file_extension_or_warn(Path::new("a.mjs")), "mjs");
    }

    #[test]
    fn unrecognised_utf8_extension_still_passes_silently_for_bail_message() {
        // The bail message includes the extension, so an unknown UTF-8
        // extension like `.exe` MUST still flow through silently so the
        // operator sees the actual extension in "Unsupported file type: .exe".
        assert_eq!(coerce_run_file_extension_or_warn(Path::new("a.exe")), "exe");
    }

    #[test]
    fn no_extension_falls_back_to_empty_string() {
        let ext = coerce_run_file_extension_or_warn(Path::new("README"));
        assert_eq!(ext, RUN_FILE_NO_EXTENSION_FALLBACK);
        assert_eq!(ext, "");
    }

    #[cfg(unix)]
    #[test]
    fn non_utf8_extension_produces_lossy_form_not_empty() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;
        // 0xFF is not valid UTF-8; the lossy converter substitutes U+FFFD
        // so the bail message carries a visible breadcrumb instead of "".
        let bytes = b"file.\xFFabc";
        let p = PathBuf::from(OsStr::from_bytes(bytes));
        let ext = coerce_run_file_extension_or_warn(&p);
        assert!(
            !ext.is_empty(),
            "non-UTF-8 extension must produce a lossy form, not collapse onto empty"
        );
        assert_ne!(ext, RUN_FILE_NO_EXTENSION_FALLBACK);
    }

    #[cfg(unix)]
    #[test]
    fn two_distinct_non_utf8_extensions_do_not_collide() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;
        let p1 = PathBuf::from(OsStr::from_bytes(b"a.\xFFA"));
        let p2 = PathBuf::from(OsStr::from_bytes(b"a.\xFEB"));
        let e1 = coerce_run_file_extension_or_warn(&p1);
        let e2 = coerce_run_file_extension_or_warn(&p2);
        assert_ne!(
            e1, e2,
            "two distinct non-UTF-8 extensions must NOT lossy onto the same string"
        );
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("mod.rs");
        assert!(src.contains("fn format_runner_run_file_no_extension_warn"));
        assert!(src.contains("fn format_runner_run_file_non_utf8_extension_warn"));
        assert!(src.contains("fn coerce_run_file_extension_or_warn"));
        assert!(src.contains("RUN_FILE_NO_EXTENSION_FALLBACK"));
    }

    #[test]
    fn each_warn_string_carries_gh3801_tag() {
        let no_ext = format_runner_run_file_no_extension_warn(Path::new("README"));
        assert!(
            no_ext.starts_with("gh3801:"),
            "missing gh3801 tag: {no_ext:?}"
        );
        let nonutf8 =
            format_runner_run_file_non_utf8_extension_warn(Path::new("a.x"), "lossy-form");
        assert!(
            nonutf8.starts_with("gh3801:"),
            "missing gh3801 tag: {nonutf8:?}"
        );
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let no_ext = format_runner_run_file_no_extension_warn(Path::new("README"));
        let nonutf8 = format_runner_run_file_non_utf8_extension_warn(Path::new("a.x"), "lossy");
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795", "gh3797", "gh3799",
        ] {
            assert!(
                !no_ext.contains(tag),
                "no-ext warn must not carry {tag}: {no_ext:?}"
            );
            assert!(
                !nonutf8.contains(tag),
                "non-utf8 warn must not carry {tag}: {nonutf8:?}"
            );
        }
    }

    #[test]
    fn two_sibling_warns_are_mutually_distinct() {
        // The two arms (no-extension vs non-UTF-8-extension) must not
        // emit the same wording — they describe different operator
        // mistakes and need to be triagable separately.
        let no_ext = format_runner_run_file_no_extension_warn(Path::new("README"));
        let nonutf8 = format_runner_run_file_non_utf8_extension_warn(Path::new("a.x"), "lossy");
        assert_ne!(no_ext, nonutf8);
        assert!(no_ext.contains("no extension"));
        assert!(nonutf8.contains("non-UTF-8 extension"));
    }

    #[test]
    fn happy_path_silent_on_recognised_ts_extension() {
        // Pins that the recognised .ts dispatch path emits no warn —
        // critical because run_file is on the hot path for every
        // `jet run script.ts` invocation.
        let ext = coerce_run_file_extension_or_warn(Path::new("/proj/src/script.ts"));
        assert_eq!(ext, "ts");
    }
}
// CODEGEN-END
