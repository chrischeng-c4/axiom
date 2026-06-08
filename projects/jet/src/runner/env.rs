// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
// CODEGEN-BEGIN
//! Environment variable injection for script runner and bundler.
//!
//! Provides two layers of environment support:
//!
//! 1. **Script execution env** (`build_env`): injects `NODE_ENV`,
//!    `JET_PROJECT_ROOT`, and prepends `node_modules/.bin` to `PATH`.
//!
//! 2. **`import.meta.env` defines** (`scan_env_files` + `import_meta_env_defines`):
//!    reads `.env`, `.env.local`, and `.env.{MODE}` files; exposes variables
//!    prefixed with `VITE_` or `JET_` as build-time replacements for
//!    `import.meta.env.*` in bundled source code.

use std::collections::HashMap;
use std::path::Path;

// ─── Script execution env ─────────────────────────────────────────────────────

/// Build environment variables for script execution.
///
/// - Prepends `node_modules/.bin` to PATH
/// - Sets `NODE_ENV` if not already set
/// - Injects `JET_PROJECT_ROOT`
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub fn build_env(project_root: &Path) -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Prepend .bin to PATH.
    //
    // GH #3582 — the prior `std::env::var("PATH").unwrap_or_default()`
    // collapsed both `NotPresent` and `NotUnicode` to `""` and then
    // produced `"<bin>:"`. On POSIX an empty PATH element means CWD,
    // so spawned processes would search the current working directory
    // for any command not provided by `.bin/` — a real privilege-
    // escalation footgun when jet runs in a directory containing
    // untrusted package files. Use the helper to elide the trailing
    // colon and warn on NotUnicode.
    let bin_dir = project_root.join("node_modules/.bin");
    let (path_value, warn) = safe_prepend_bin_to_path(&bin_dir, std::env::var("PATH"));
    if let Some(msg) = warn {
        tracing::warn!(target: "jet::runner::env", "{}", msg);
    }
    env.insert("PATH".to_string(), path_value);

    // NODE_ENV defaults to "development".
    //
    // GH #3586 — the prior `unwrap_or_else(|_| ...)` collapsed both
    // `NotPresent` and `NotUnicode` to the default. `NotPresent` is the
    // canonical "user did not configure NODE_ENV" case; `NotUnicode`
    // means the user DID set `NODE_ENV` but with non-UTF-8 bytes — a
    // misconfiguration that should warn, not silently default to
    // `"development"` (which could downgrade a production build).
    // Same family as #3582 (PATH NotPresent/NotUnicode collapse).
    let (node_env, node_env_warn) = safe_node_env(std::env::var("NODE_ENV"));
    if let Some(msg) = node_env_warn {
        tracing::warn!(target: "jet::runner::env", "{}", msg);
    }
    env.insert("NODE_ENV".to_string(), node_env);

    // JET_* variables
    env.insert(
        "JET_PROJECT_ROOT".to_string(),
        project_root.to_string_lossy().to_string(),
    );

    // Inherit npm_config_* if present
    for (key, value) in std::env::vars() {
        if key.starts_with("npm_config_") || key.starts_with("npm_package_") {
            env.insert(key, value);
        }
    }

    env
}

/// GH #3582 — prepend `bin_dir` to PATH without producing a trailing
/// empty element (which on POSIX would resolve to the current working
/// directory and let CWD-dropped binaries hijack command resolution).
///
/// Cases:
/// - `Ok(v)` → `"<bin>:<v>"` (unchanged from prior behavior),
///   `warn = None`.
/// - `Err(NotPresent)` → `"<bin>"` (no trailing colon; refuses to
///   inject CWD), `warn = None`.
/// - `Err(NotUnicode(_))` → `"<bin>"` plus a `warn` message tagged
///   `GH #3582` that the caller is expected to emit via
///   `tracing::warn!` against its own static-target macro.
///
/// The warn message is returned (rather than emitted here) so each
/// call site can use a compile-time-constant `target:` for
/// `tracing::warn!` (the `target:` arg must be a constant
/// expression).
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn safe_prepend_bin_to_path(
    bin_dir: &Path,
    current: Result<String, std::env::VarError>,
) -> (String, Option<String>) {
    match current {
        Ok(v) => (format!("{}:{}", bin_dir.display(), v), None),
        Err(std::env::VarError::NotPresent) => (bin_dir.display().to_string(), None),
        Err(std::env::VarError::NotUnicode(_)) => (
            bin_dir.display().to_string(),
            Some(format_safe_prepend_bin_to_path_warn(bin_dir, "not-unicode")),
        ),
    }
}

/// GH #3582 — build the warn message for a `NotUnicode` PATH lookup.
/// Extracted so the wording (issue tag + bin_dir + observed kind) is
/// unit-testable without provoking the actual non-UTF-8 platform case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_safe_prepend_bin_to_path_warn(bin_dir: &Path, observed_kind: &str) -> String {
    format!(
        "GH #3582 PATH env had non-UTF-8 value (observed: {observed_kind}); \
         discarding the inherited PATH to avoid producing a trailing empty \
         element (which on POSIX resolves to the current working directory \
         and would let CWD-dropped binaries hijack command resolution). \
         New PATH = {}",
        bin_dir.display()
    )
}

/// GH #3586 — resolve `NODE_ENV` without collapsing `NotPresent` and
/// `NotUnicode` into a single fallback.
///
/// Cases:
/// - `Ok(v)` → `(v, None)` (caller forwards the user's setting).
/// - `Err(NotPresent)` → `("development", None)` (canonical default).
/// - `Err(NotUnicode(_))` → `("development", Some(warn_msg))` — the
///   user DID set `NODE_ENV` but with non-UTF-8 bytes; this is a
///   misconfiguration that must not silently downgrade a production
///   build. Caller emits the warn against a compile-time-constant
///   `target:`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn safe_node_env(
    current: Result<String, std::env::VarError>,
) -> (String, Option<String>) {
    match current {
        Ok(v) => (v, None),
        Err(std::env::VarError::NotPresent) => ("development".to_string(), None),
        Err(std::env::VarError::NotUnicode(_)) => (
            "development".to_string(),
            Some(format_safe_node_env_warn("not-unicode")),
        ),
    }
}

/// GH #3586 — build the warn message for a `NotUnicode` `NODE_ENV`
/// lookup. Extracted so the wording (issue tag + observed kind +
/// downgrade consequence) is unit-testable without provoking the
/// actual non-UTF-8 platform case.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_safe_node_env_warn(observed_kind: &str) -> String {
    format!(
        "GH #3586 NODE_ENV env had non-UTF-8 value (observed: {observed_kind}); \
         the user explicitly set NODE_ENV but jet cannot decode it. Falling back \
         to \"development\" — if you intended a production build this would have \
         silently downgraded. Fix the NODE_ENV value or unset it to opt into the \
         default."
    )
}

// ─── .env file scanner ────────────────────────────────────────────────────────

/// Scan `.env` files in `project_root` and return all key=value pairs merged
/// in precedence order (later files override earlier ones):
///
/// 1. `.env`          — base defaults (lowest priority)
/// 2. `.env.local`    — local overrides (not committed)
/// 3. `.env.{mode}`   — mode-specific (e.g. `.env.production`)
///
/// Only variables prefixed with `VITE_` or `JET_` are retained since those
/// are the only ones exposed to client-side code via `import.meta.env`.
///
/// `mode` should be `"development"` or `"production"`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub fn scan_env_files(project_root: &Path, mode: &str) -> HashMap<String, String> {
    let mut merged: HashMap<String, String> = HashMap::new();

    let candidates = [
        project_root.join(".env"),
        project_root.join(".env.local"),
        project_root.join(format!(".env.{}", mode)),
    ];

    // GH #3167 — fold the `exists()` + `if let Ok(...)` pair into one
    // explicit match. Absent files (NotFound) are silent — the
    // canonical "user didn't write a .env" case. Any other read error
    // becomes a tracing::warn! so a perms-mismatch or broken-symlink
    // bug doesn't silently strip every VITE_/JET_ key from the bundle.
    for path in &candidates {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => {
                tracing::warn!(
                    target: "jet::runner::env",
                    "skipping env file {:?}: {e}; every VITE_/JET_ \
                     key it defines will NOT be present in \
                     import.meta.env (GH #3167)",
                    path
                );
                continue;
            }
        };
        for (key, value) in parse_env_file(&content) {
            merged.insert(key, value);
        }
    }

    // Retain only VITE_ and JET_ prefixed variables.
    merged.retain(|k, _| k.starts_with("VITE_") || k.starts_with("JET_"));
    merged
}

/// Build the `import.meta.env.*` define map for the bundler.
///
/// Returns a map of:
/// - `import.meta.env.MODE` → `"\"development\""` or `"\"production\""`
/// - `import.meta.env.DEV`  → `"true"` or `"false"`
/// - `import.meta.env.PROD` → `"true"` or `"false"`
/// - `import.meta.env.VITE_X` → `"\"value\""` for each env var
/// - `import.meta.env.JET_X`  → `"\"value\""` for each env var
///
/// Values are JSON-encoded strings so the bundler emits valid JavaScript.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub fn import_meta_env_defines(
    env_vars: &HashMap<String, String>,
    mode: &str,
) -> HashMap<String, String> {
    let mut defines = HashMap::new();

    let is_dev = mode != "production";
    let is_prod = !is_dev;

    defines.insert("import.meta.env.MODE".to_string(), format!("\"{}\"", mode));
    defines.insert("import.meta.env.DEV".to_string(), is_dev.to_string());
    defines.insert("import.meta.env.PROD".to_string(), is_prod.to_string());

    // User-defined VITE_* and JET_* vars
    for (key, value) in env_vars {
        // GH #3564 — JSON-encode the value so it appears as a string literal
        // in the bundle. The prior `unwrap_or_else(|_| format!("\"{}\"", value))`
        // hid a possible correctness regression behind a "shouldn't happen"
        // arm: any future serializer change that emitted a real failure here
        // would silently produce invalid JS (e.g. embedded `"` would break the
        // bundler). On Err we warn and skip the define — a missing define
        // collapses to `undefined` at runtime, which is recoverable; a broken
        // define is a compile failure.
        match serde_json::to_string(value) {
            Ok(json) => {
                defines.insert(format!("import.meta.env.{}", key), json);
            }
            Err(err) => {
                tracing::warn!(
                    target: "jet::runner::env",
                    "{}",
                    format_env_define_warn(key, &err)
                );
            }
        }
    }

    // Catch-all: replace any remaining `import.meta.env` with an empty object
    // so tree-shaking can eliminate dead branches in production.
    defines.insert("import.meta.env".to_string(), "{}".to_string());

    defines
}

// ─── .env parser ─────────────────────────────────────────────────────────────

/// Parse a `.env` file into key=value pairs.
///
/// Supports:
/// - `KEY=value`
/// - `KEY="quoted value"`  (double quotes stripped)
/// - `KEY='quoted value'`  (single quotes stripped)
/// - `# comment` lines (skipped)
/// - blank lines (skipped)
fn parse_env_file(content: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();

    // GH #3717 — was `if let Some((k, v)) = line.split_once('=') { ... if
    // !key.is_empty() { ... } }`. Lines without `=` and lines with empty
    // key both fell through with no warn — a typo like `VITE_API_URL value`
    // (missing `=`) was indistinguishable downstream from the user not
    // defining VITE_API_URL at all (both showed up as `undefined` from
    // `import.meta.env.VITE_API_URL` at runtime). Surface a warn naming
    // the line number and the discriminated failure mode.
    for (idx, raw_line) in content.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim();

        // Skip blanks and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        match line.split_once('=') {
            None => {
                tracing::warn!(
                    target: "jet::runner::env",
                    "{}",
                    format_env_line_skipped_warn(
                        line_no,
                        line,
                        EnvLineSkipReason::MissingEquals,
                    )
                );
            }
            Some((key, raw_value)) => {
                let key = key.trim().to_string();
                if key.is_empty() {
                    tracing::warn!(
                        target: "jet::runner::env",
                        "{}",
                        format_env_line_skipped_warn(
                            line_no,
                            line,
                            EnvLineSkipReason::EmptyKey,
                        )
                    );
                    continue;
                }
                let value = unquote(raw_value.trim());
                pairs.push((key, value));
            }
        }
    }

    pairs
}

/// GH #3717 — discriminate the two ways a `.env` line gets dropped at parse
/// time. Surfaced through [`format_env_line_skipped_warn`] so the warn names
/// the actual failure mode instead of a generic "skipped" — the user can fix
/// the typo without re-reading the parser.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum EnvLineSkipReason {
    /// Line is non-empty, non-comment, but contains no `=` separator.
    /// e.g. `VITE_API_URL value` (typed space instead of `=`).
    MissingEquals,
    /// Line has an `=` but the key (left of `=`, trimmed) is empty.
    /// e.g. `=value` or `   =value`.
    EmptyKey,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
impl EnvLineSkipReason {
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            EnvLineSkipReason::MissingEquals => "missing `=` separator",
            EnvLineSkipReason::EmptyKey => "empty key (no characters before `=`)",
        }
    }
}

/// GH #3717 — build the warn for a `.env` line that `parse_env_file` dropped.
/// Extracted so the wording is unit-testable. Names the 1-based line number,
/// the trimmed line content, the failure mode, and the downstream symptom
/// (`import.meta.env.KEY` reading `undefined`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_env_line_skipped_warn(
    line_no: usize,
    line: &str,
    reason: EnvLineSkipReason,
) -> String {
    format!(
        "GH #3717 .env line {line_no} skipped: {reason} — content: {line:?}. \
         The line is non-empty and non-comment but cannot be parsed as \
         KEY=value. Any VITE_/JET_ var the user intended on this line will \
         be ABSENT from `import.meta.env` at runtime (reads as `undefined`), \
         which is indistinguishable downstream from the var not being \
         defined at all. Fix the .env line.",
        reason = reason.as_str(),
    )
}

/// GH #3564 — build the warn message for a `serde_json` failure encoding a
/// VITE_/JET_ env value into an `import.meta.env.*` define. Extracted so the
/// wording is unit-testable.
///
/// Names the affected key and preserves the underlying serde error so the
/// dev can correlate the broken `.env` line with the bundler skip. The
/// `import.meta.env.{key}` consequence wording is explicit because the
/// runtime symptom is `undefined` — easy to misread as a missing `.env`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-runner.md#schema
pub(crate) fn format_env_define_warn(key: &str, err: &serde_json::Error) -> String {
    format!(
        "GH #3564 serde_json failed to encode VITE_/JET_ env var {key} into \
         an import.meta.env define: {err}; the define has been SKIPPED to \
         avoid emitting invalid JS. import.meta.env.{key} will read as \
         `undefined` at runtime — fix the .env value or report the encoder \
         failure."
    )
}

/// Strip surrounding single or double quotes from a value.
fn unquote(s: &str) -> String {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_env_has_path() {
        let env = build_env(&PathBuf::from("/tmp/project"));
        let path = env.get("PATH").unwrap();
        assert!(path.starts_with("/tmp/project/node_modules/.bin:"));
    }

    #[test]
    fn test_build_env_has_node_env() {
        let env = build_env(&PathBuf::from("/tmp/project"));
        assert!(env.contains_key("NODE_ENV"));
    }

    #[test]
    fn test_build_env_has_jet_root() {
        let env = build_env(&PathBuf::from("/tmp/project"));
        assert_eq!(env.get("JET_PROJECT_ROOT").unwrap(), "/tmp/project");
    }

    #[test]
    fn test_parse_env_file_basic() {
        let content = "VITE_API_URL=http://localhost:3200\nJET_MODE=dev\n";
        let pairs = parse_env_file(content);
        assert_eq!(pairs.len(), 2);
        assert_eq!(
            pairs[0],
            (
                "VITE_API_URL".to_string(),
                "http://localhost:3200".to_string()
            )
        );
        assert_eq!(pairs[1], ("JET_MODE".to_string(), "dev".to_string()));
    }

    #[test]
    fn test_parse_env_file_quoted() {
        let content = r#"VITE_API_URL="http://localhost:3200""#;
        let pairs = parse_env_file(content);
        assert_eq!(pairs[0].1, "http://localhost:3200");
    }

    #[test]
    fn test_parse_env_file_comments() {
        let content = "# comment\nVITE_X=1\n";
        let pairs = parse_env_file(content);
        assert_eq!(pairs.len(), 1);
    }

    #[test]
    fn test_scan_env_files_only_vite_jet() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".env"),
            "VITE_API=http://api\nSECRET=hidden\nJET_FLAG=1\n",
        )
        .unwrap();

        let vars = scan_env_files(dir.path(), "development");
        assert!(vars.contains_key("VITE_API"));
        assert!(vars.contains_key("JET_FLAG"));
        assert!(!vars.contains_key("SECRET"));
    }

    #[test]
    fn test_scan_env_files_precedence() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".env"), "VITE_URL=base\n").unwrap();
        std::fs::write(dir.path().join(".env.local"), "VITE_URL=local\n").unwrap();

        let vars = scan_env_files(dir.path(), "development");
        assert_eq!(vars.get("VITE_URL").map(String::as_str), Some("local"));
    }

    /// GH #3167 — Absent `.env` files yield empty map silently (no
    /// crash, no log). The canonical "no env file" branch.
    #[test]
    fn scan_env_files_returns_empty_when_no_files_exist() {
        let dir = tempfile::tempdir().unwrap();
        let vars = scan_env_files(dir.path(), "development");
        assert!(
            vars.is_empty(),
            "no .env files → empty map without diagnostic"
        );
    }

    /// GH #3167 — A `.env` file that exists but is unreadable (chmod 000)
    /// must not crash scan_env_files, and a sibling readable `.env.local`
    /// must still be merged. Unix-only — self-skips when chmod is
    /// effectively a no-op (root in a container).
    #[cfg(unix)]
    #[test]
    fn scan_env_files_skips_unreadable_file_but_merges_siblings() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".env"), "VITE_API=http://from-env\n").unwrap();
        std::fs::write(
            dir.path().join(".env.local"),
            "VITE_LOCAL=http://from-local\n",
        )
        .unwrap();

        let bad = dir.path().join(".env");
        std::fs::set_permissions(&bad, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Self-skip when chmod is effectively ignored.
        if std::fs::read_to_string(&bad).is_ok() {
            return;
        }

        let vars = scan_env_files(dir.path(), "development");
        assert_eq!(
            vars.get("VITE_LOCAL").map(String::as_str),
            Some("http://from-local"),
            "sibling readable .env.local must still be merged when one file is unreadable (GH #3167)"
        );
        assert!(
            !vars.contains_key("VITE_API"),
            "unreadable .env contents must be absent — not silently faked"
        );

        // Restore perms for tempdir cleanup.
        std::fs::set_permissions(&bad, std::fs::Permissions::from_mode(0o644)).unwrap();
    }

    #[test]
    fn test_scan_env_files_mode_override() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".env"), "VITE_URL=base\n").unwrap();
        std::fs::write(dir.path().join(".env.local"), "VITE_URL=local\n").unwrap();
        std::fs::write(dir.path().join(".env.production"), "VITE_URL=prod\n").unwrap();

        let vars = scan_env_files(dir.path(), "production");
        assert_eq!(vars.get("VITE_URL").map(String::as_str), Some("prod"));
    }

    #[test]
    fn test_import_meta_env_defines_dev() {
        let vars = HashMap::from([(
            "VITE_API_URL".to_string(),
            "http://localhost:3200".to_string(),
        )]);
        let defines = import_meta_env_defines(&vars, "development");

        assert_eq!(
            defines.get("import.meta.env.MODE").map(String::as_str),
            Some("\"development\"")
        );
        assert_eq!(
            defines.get("import.meta.env.DEV").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            defines.get("import.meta.env.PROD").map(String::as_str),
            Some("false")
        );
        assert!(defines.contains_key("import.meta.env.VITE_API_URL"));
    }

    #[test]
    fn test_import_meta_env_defines_prod() {
        let vars = HashMap::new();
        let defines = import_meta_env_defines(&vars, "production");

        assert_eq!(
            defines.get("import.meta.env.MODE").map(String::as_str),
            Some("\"production\"")
        );
        assert_eq!(
            defines.get("import.meta.env.DEV").map(String::as_str),
            Some("false")
        );
        assert_eq!(
            defines.get("import.meta.env.PROD").map(String::as_str),
            Some("true")
        );
    }

    // ─── GH #3564: silent serde_json define-encoder swallow ────────────────

    /// GH #3564 — the env-define warn message must name the affected key,
    /// preserve the underlying serde error verbatim, AND include the
    /// GH #3564 tag so log readers can land on this fix from a CI grep.
    #[test]
    fn gh3564_env_define_warn_names_tag_key_and_error() {
        // Synthesize a real serde_json::Error so the test exercises the
        // actual Display impl rather than an `anyhow::anyhow!` stand-in.
        let parse_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();

        let msg = format_env_define_warn("VITE_BROKEN", &parse_err);

        assert!(
            msg.contains("GH #3564"),
            "must include searchable issue tag, got: {msg}"
        );
        assert!(
            msg.contains("VITE_BROKEN"),
            "must name the affected key, got: {msg}"
        );
        // serde_json errors include "expected" or similar diagnostic — the
        // message must include the full Display so the dev can correlate
        // the failure with their .env line.
        let err_text = format!("{parse_err}");
        assert!(
            msg.contains(&err_text),
            "must preserve underlying serde error verbatim, got: {msg}"
        );
    }

    /// GH #3564 — the warning must explain the runtime consequence so the
    /// dev does not misread a skipped define as "the .env file wasn't read":
    /// `import.meta.env.{key}` must be named as the symptom site.
    #[test]
    fn gh3564_env_define_warn_explains_runtime_consequence() {
        let parse_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>("not json").unwrap_err();

        let msg = format_env_define_warn("JET_API", &parse_err);

        assert!(
            msg.contains("import.meta.env.JET_API"),
            "must name the symptom site at runtime, got: {msg}"
        );
        assert!(
            msg.contains("undefined") || msg.contains("SKIPPED"),
            "must explain the SKIPPED/undefined branch so the dev does not \
             misread the warning as a .env read failure, got: {msg}"
        );
    }

    /// GH #3564 — happy-path contract: `import_meta_env_defines` must
    /// JSON-encode a value containing a literal `"` so the bundler sees
    /// a syntactically valid string. This is the bug the silent
    /// `unwrap_or_else` was hiding: any future encoder regression
    /// emitting `"a"b"` would now be SKIPPED with a warning rather than
    /// quietly producing broken JS.
    #[test]
    fn gh3564_quote_in_env_value_is_json_escaped_not_silently_broken() {
        let vars = HashMap::from([("JET_FOO".to_string(), r#"a"b"#.to_string())]);
        let defines = import_meta_env_defines(&vars, "development");

        let encoded = defines
            .get("import.meta.env.JET_FOO")
            .expect("define must be present for valid string value");

        assert_eq!(
            encoded, r#""a\"b""#,
            "value with embedded quote must be JSON-escaped"
        );
        // Sanity: the broken-fallback shape that the prior code would
        // have emitted is exactly the thing we must NOT see.
        assert_ne!(
            encoded, r#""a"b""#,
            "must not emit the unescaped broken-fallback shape"
        );
    }

    // ─── GH #3582: PATH-prepend trailing-colon (CWD in PATH) bug ──────

    /// GH #3582 — `Ok` case is unchanged: the bin dir is prepended with
    /// a colon and the inherited PATH is preserved verbatim.
    #[test]
    fn gh3582_safe_prepend_bin_to_path_ok_case_preserves_inherited() {
        let bin = std::path::Path::new("/proj/node_modules/.bin");
        let (out, warn) = safe_prepend_bin_to_path(bin, Ok("/usr/bin:/bin".to_string()));
        assert_eq!(out, "/proj/node_modules/.bin:/usr/bin:/bin");
        assert!(warn.is_none(), "Ok case must not emit a warn");
    }

    /// GH #3582 — `NotPresent` must NOT produce a trailing-colon PATH.
    /// Pre-fix: `unwrap_or_default()` returned `""`, format!("{}:{}", bin, "") yielded
    /// "bin:" — and an empty PATH element on POSIX resolves to CWD,
    /// letting any binary dropped in CWD hijack command resolution.
    #[test]
    fn gh3582_safe_prepend_bin_to_path_not_present_has_no_trailing_colon() {
        let bin = std::path::Path::new("/proj/node_modules/.bin");
        let (out, warn) = safe_prepend_bin_to_path(bin, Err(std::env::VarError::NotPresent));
        assert_eq!(out, "/proj/node_modules/.bin");
        assert!(
            !out.ends_with(':'),
            "trailing colon would resolve to CWD on POSIX (GH #3582), got: {out:?}"
        );
        assert!(
            warn.is_none(),
            "NotPresent is not a misconfiguration; no warn"
        );
    }

    /// GH #3582 — `NotUnicode` must also avoid the trailing-colon
    /// failure mode AND surface a warn message so the operator sees
    /// that PATH was discarded.
    #[test]
    fn gh3582_safe_prepend_bin_to_path_not_unicode_warns_and_drops_trailing_colon() {
        let bin = std::path::Path::new("/proj/node_modules/.bin");
        let raw = std::ffi::OsString::from("ignored");
        let (out, warn) = safe_prepend_bin_to_path(bin, Err(std::env::VarError::NotUnicode(raw)));
        assert_eq!(out, "/proj/node_modules/.bin");
        assert!(
            !out.ends_with(':'),
            "trailing colon would resolve to CWD on POSIX (GH #3582), got: {out:?}"
        );
        let msg = warn.expect("NotUnicode must produce a warn message");
        assert!(
            msg.contains("GH #3582"),
            "warn must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("not-unicode"),
            "warn must name observed kind, got: {msg}"
        );
    }

    /// GH #3582 — `format_safe_prepend_bin_to_path_warn` must include
    /// the tag, the bin_dir path, and the observed kind so the warn
    /// is greppable.
    #[test]
    fn gh3582_format_safe_prepend_bin_to_path_warn_names_tag_path_and_kind() {
        let bin = std::path::Path::new("/proj/node_modules/.bin");
        let msg = format_safe_prepend_bin_to_path_warn(bin, "not-unicode");
        assert!(msg.contains("GH #3582"), "must include tag, got: {msg}");
        assert!(
            msg.contains("/proj/node_modules/.bin"),
            "must name the bin dir, got: {msg}"
        );
        assert!(
            msg.contains("not-unicode"),
            "must name the kind, got: {msg}"
        );
    }

    // ─── GH #3586: NODE_ENV NotPresent/NotUnicode collapse ────────────

    /// GH #3586 — `Ok` case is unchanged: the user-set value passes
    /// through verbatim with no warn.
    #[test]
    fn gh3586_safe_node_env_ok_case_passes_through() {
        let (out, warn) = safe_node_env(Ok("production".to_string()));
        assert_eq!(out, "production");
        assert!(warn.is_none(), "Ok case must not emit a warn");
    }

    /// GH #3586 — `NotPresent` is the canonical "user did not set
    /// NODE_ENV" case. Default to "development" silently.
    #[test]
    fn gh3586_safe_node_env_not_present_defaults_silently() {
        let (out, warn) = safe_node_env(Err(std::env::VarError::NotPresent));
        assert_eq!(out, "development");
        assert!(
            warn.is_none(),
            "NotPresent is canonical default, no warn (GH #3586)"
        );
    }

    /// GH #3586 — `NotUnicode` is a real misconfiguration: the user
    /// DID set NODE_ENV, but with non-UTF-8 bytes. Falling back to
    /// "development" is the safe default but MUST produce a warn so
    /// a production build does not silently downgrade.
    #[test]
    fn gh3586_safe_node_env_not_unicode_warns_and_defaults() {
        let raw = std::ffi::OsString::from("ignored");
        let (out, warn) = safe_node_env(Err(std::env::VarError::NotUnicode(raw)));
        assert_eq!(out, "development");
        let msg = warn.expect("NotUnicode must produce a warn message");
        assert!(
            msg.contains("GH #3586"),
            "warn must include issue tag, got: {msg}"
        );
        assert!(
            msg.contains("not-unicode"),
            "warn must name observed kind, got: {msg}"
        );
    }

    /// GH #3586 — the warn message must distinguish itself from the
    /// silent NotPresent default and must name the downgrade
    /// consequence (NODE_ENV cannot be decoded, falling back to
    /// development) so a CI grep can land here.
    #[test]
    fn gh3586_format_safe_node_env_warn_names_tag_kind_and_consequence() {
        let msg = format_safe_node_env_warn("not-unicode");
        assert!(msg.contains("GH #3586"), "must include tag, got: {msg}");
        assert!(
            msg.contains("not-unicode"),
            "must name the kind, got: {msg}"
        );
        assert!(
            msg.contains("NODE_ENV"),
            "must name the env var, got: {msg}"
        );
        assert!(
            msg.contains("development"),
            "must name the downgrade target so the operator sees the fallback, got: {msg}"
        );
    }

    /// GH #3586 — happy-path regression: `build_env` continues to set
    /// NODE_ENV when it is unset in the actual process env. (The
    /// existing `test_build_env_has_node_env` asserts presence; this
    /// one nails the canonical fallback value.)
    #[test]
    fn gh3586_build_env_node_env_falls_back_to_development_when_unset() {
        // Run only when NODE_ENV is unset in the test process — avoids
        // tearing through other tests' env state.
        if std::env::var_os("NODE_ENV").is_some() {
            return;
        }
        let env = build_env(&PathBuf::from("/tmp/project"));
        assert_eq!(
            env.get("NODE_ENV").map(String::as_str),
            Some("development"),
            "NotPresent must canonically default to development (GH #3586)"
        );
    }
}

#[cfg(test)]
mod gh3717_env_line_skipped_warn_tests {
    //! GH #3717 — `parse_env_file` used to drop any line that lacked an `=`
    //! separator, and any line whose key (left of `=`, trimmed) was empty.
    //! Both drops were silent — no warn, no error. A typo like
    //! `VITE_API_URL value` (space instead of `=`) was indistinguishable
    //! downstream from the user not defining VITE_API_URL at all (both
    //! showed up as `undefined` from `import.meta.env.VITE_API_URL` at
    //! runtime). These tests pin: malformed lines stay dropped (don't abort
    //! the parse), but the warn names the line number, the trimmed line
    //! content, and the discriminated failure mode.
    use super::*;

    #[test]
    fn missing_equals_helper_tags_gh_and_line_no() {
        let msg =
            format_env_line_skipped_warn(7, "VITE_API_URL value", EnvLineSkipReason::MissingEquals);
        assert!(msg.contains("GH #3717"), "msg: {msg}");
        assert!(
            msg.contains("line 7"),
            "msg must name 1-based line no: {msg}"
        );
        assert!(
            msg.contains("VITE_API_URL value"),
            "msg must echo content: {msg}"
        );
        assert!(msg.contains("missing"), "msg must name the reason: {msg}");
    }

    #[test]
    fn empty_key_helper_distinguishes_from_missing_equals() {
        let missing =
            format_env_line_skipped_warn(1, "VITE_X value", EnvLineSkipReason::MissingEquals);
        let empty = format_env_line_skipped_warn(2, "=orphan", EnvLineSkipReason::EmptyKey);
        assert_ne!(
            missing, empty,
            "two reasons must produce distinct warn text"
        );
        assert!(
            empty.contains("empty key"),
            "empty-key warn must name it: {empty}"
        );
    }

    #[test]
    fn helper_names_downstream_undefined_symptom() {
        let msg = format_env_line_skipped_warn(3, "VITE_X value", EnvLineSkipReason::MissingEquals);
        assert!(
            msg.contains("undefined") && msg.contains("import.meta.env"),
            "warn must name the runtime symptom so users find it: {msg}"
        );
    }

    #[test]
    fn helper_is_deterministic_for_fixed_inputs() {
        let a = format_env_line_skipped_warn(5, "BROKEN line", EnvLineSkipReason::MissingEquals);
        let b = format_env_line_skipped_warn(5, "BROKEN line", EnvLineSkipReason::MissingEquals);
        assert_eq!(a, b);
    }

    #[test]
    fn missing_equals_line_is_dropped_not_aborted() {
        let content = "VITE_OK=present\nVITE_API_URL value\nJET_ALSO_OK=2\n";
        let pairs = parse_env_file(content);
        // The malformed middle line is dropped; the others survive.
        let keys: Vec<_> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"VITE_OK"));
        assert!(keys.contains(&"JET_ALSO_OK"));
        assert!(
            !keys.contains(&"VITE_API_URL"),
            "malformed line must not silently parse as key-only"
        );
    }

    #[test]
    fn empty_key_line_is_dropped() {
        let content = "VITE_OK=present\n=orphan-value\nJET_OK=1\n";
        let pairs = parse_env_file(content);
        let keys: Vec<_> = pairs.iter().map(|(k, _)| k.as_str()).collect();
        assert!(keys.contains(&"VITE_OK"));
        assert!(keys.contains(&"JET_OK"));
        assert_eq!(pairs.len(), 2, "empty-key line must be dropped, not stored");
    }

    #[test]
    fn whitespace_only_key_line_is_dropped() {
        // `   =value` → `split_once('=')` gives `("   ", "value")` →
        // `key.trim()` is `""` → EmptyKey, dropped.
        let content = "   =value\nVITE_KEEP=1\n";
        let pairs = parse_env_file(content);
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0].0, "VITE_KEEP");
    }

    #[test]
    fn reason_as_str_is_stable() {
        assert_eq!(
            EnvLineSkipReason::MissingEquals.as_str(),
            "missing `=` separator"
        );
        assert_eq!(
            EnvLineSkipReason::EmptyKey.as_str(),
            "empty key (no characters before `=`)"
        );
    }
}
// CODEGEN-END
