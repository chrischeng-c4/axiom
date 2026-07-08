// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Spec file discovery. Walks the project tree and filters by globs.

use crate::test_runner::config::{RunnerConfig, TestEnvironment};
use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Serialize;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

/// GH #3623 — `scan` previously did
/// `path.strip_prefix(project_root).unwrap_or(path).to_path_buf()` for
/// the `SpecFile.relative` field. When the spec path was not under
/// project_root (symlinked file, `--only-files` pointing outside,
/// monorepo external test mirror), the silent fallback leaked the
/// absolute filesystem path into the public manifest (it becomes the
/// test ID in `list_manifest.rs`).
///
/// This helper distinguishes the cases:
/// - spec under root → returns the stripped relative path, no warn.
/// - spec outside root → returns the bare `file_name` only, with a warn
///   message tagged `GH #3623`. Never the absolute path.
/// - spec has no file_name (path is `/`, `..`) → returns the input path
///   unchanged but emits a warn — extreme edge case, no good fallback.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn safe_relative_path(abs: &Path, root: &Path) -> (PathBuf, Option<String>) {
    match abs.strip_prefix(root) {
        Ok(rel) => (rel.to_path_buf(), None),
        Err(err) => {
            let warn = format_safe_relative_path_warn(abs, root, &err);
            match abs.file_name() {
                Some(name) => (PathBuf::from(name), Some(warn)),
                None => (abs.to_path_buf(), Some(warn)),
            }
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub(crate) fn format_safe_relative_path_warn(
    abs: &Path,
    root: &Path,
    err: &std::path::StripPrefixError,
) -> String {
    format!(
        "GH #3623 test_runner::discovery: spec {abs:?} is not under project_root {root:?} \
         ({err}); falling back to file_name only to avoid leaking the absolute filesystem \
         path into the spec list manifest. The downstream test ID will be the bare file \
         name; pass --only-files with an in-tree path to restore the relative form."
    )
}

/// Schema version for the resolved-discovery manifest. Bumped only when
/// the JSON shape changes in a way that older consumers can't ignore.
// @spec #2709
pub const RESOLVED_DISCOVERY_SCHEMA_VERSION: u32 = 1;

/// Machine-readable error returned when the configured inputs for
/// discovery cannot be resolved to a deterministic run. Each variant
/// has a stable `code` string for tooling and a human-readable
/// `message`.
// @spec #2709
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveryConfigError {
    pub code: &'static str,
    pub message: String,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl DiscoveryConfigError {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Emit the error as a single JSON line so callers (CI, agents) can
    /// parse the failure without scraping stderr.
    // @spec #2709
    pub fn to_json(&self) -> String {
        serde_json::json!({
            "kind": "discovery_config_error",
            "code": self.code,
            "message": self.message,
        })
        .to_string()
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl std::fmt::Display for DiscoveryConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
impl std::error::Error for DiscoveryConfigError {}

/// A single spec file the runner would execute, expressed in the
/// resolved-discovery manifest.
// @spec #2709
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedSpec {
    /// Absolute path on disk.
    pub path: PathBuf,
    /// Path relative to the project root (display + stable IDs).
    pub relative: PathBuf,
}

/// Resolved-discovery manifest: the exact inputs that fed discovery
/// plus the spec list the runner would execute. Serialisable so the
/// result envelope can record what produced a run without re-walking.
// @spec #2709
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResolvedDiscovery {
    pub schema_version: u32,
    pub project_root: PathBuf,
    pub test_dir: PathBuf,
    pub test_match: Vec<String>,
    pub test_ignore: Vec<String>,
    pub environment: String,
    pub timeout_ms: u64,
    pub workers: usize,
    pub shard: Option<(u32, u32)>,
    pub only_files: Vec<PathBuf>,
    pub specs: Vec<ResolvedSpec>,
}

fn environment_str(env: TestEnvironment) -> &'static str {
    match env {
        TestEnvironment::Node => "node",
        TestEnvironment::Dom => "dom",
        TestEnvironment::Component => "component",
    }
}

/// Resolve the runner config to a deterministic discovery manifest.
///
/// Validates that the `test_dir` exists, that include/exclude globs
/// compile, and that the configured environment is one the runner can
/// actually execute. Discovery itself is delegated to [`scan`], which
/// sorts deterministically.
///
/// Returns a [`DiscoveryConfigError`] (separate from `anyhow`) so the
/// CLI can format both a human message and a stable JSON shape.
// @spec #2709
pub fn resolve_discovery(
    config: &RunnerConfig,
) -> std::result::Result<ResolvedDiscovery, DiscoveryConfigError> {
    if !config.test_dir.exists() {
        return Err(DiscoveryConfigError::new(
            "invalid_test_dir",
            format!(
                "test_dir does not exist: {} (set via project_root or RunnerConfig::test_dir)",
                config.test_dir.display()
            ),
        ));
    }
    if !config.test_dir.is_dir() {
        return Err(DiscoveryConfigError::new(
            "invalid_test_dir",
            format!("test_dir is not a directory: {}", config.test_dir.display()),
        ));
    }

    // Compile globsets up-front so an invalid pattern surfaces here
    // rather than half-way through the walk.
    let _ = build_globset(&config.test_match).map_err(|e| {
        DiscoveryConfigError::new("invalid_glob_include", format!("test_match: {e}"))
    })?;
    let _ = build_globset(&config.test_ignore).map_err(|e| {
        DiscoveryConfigError::new("invalid_glob_exclude", format!("test_ignore: {e}"))
    })?;

    config
        .environment
        .ensure_supported()
        .map_err(|e| DiscoveryConfigError::new("unsupported_environment", e))?;

    if config.workers < 1 {
        return Err(DiscoveryConfigError::new(
            "invalid_workers",
            "workers must be >= 1",
        ));
    }

    let specs = scan(config)
        .map_err(|e| DiscoveryConfigError::new("scan_failed", format!("scan failed: {e:#}")))?;
    let specs = specs
        .into_iter()
        .map(|s| ResolvedSpec {
            path: s.path,
            relative: s.relative,
        })
        .collect();

    Ok(ResolvedDiscovery {
        schema_version: RESOLVED_DISCOVERY_SCHEMA_VERSION,
        project_root: config.project_root.clone(),
        test_dir: config.test_dir.clone(),
        test_match: config.test_match.clone(),
        test_ignore: config.test_ignore.clone(),
        environment: environment_str(config.environment).to_string(),
        timeout_ms: config.timeout_ms,
        workers: config.workers,
        shard: config.shard,
        only_files: config.only_files.clone(),
        specs,
    })
}

/// A discovered spec file.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
#[derive(Debug, Clone)]
pub struct SpecFile {
    /// Absolute path.
    pub path: PathBuf,
    /// Path relative to the project root (for display).
    pub relative: PathBuf,
}

/// Walk the configured directory, return spec files matching
/// `test_match` and not `test_ignore`. Sorted deterministically.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub fn scan(config: &RunnerConfig) -> Result<Vec<SpecFile>> {
    if !config.only_files.is_empty() {
        // Explicit file list — bypass discovery.
        let mut out = Vec::new();
        for f in &config.only_files {
            let abs = if f.is_absolute() {
                f.clone()
            } else {
                config.project_root.join(f)
            };
            let abs = abs
                .canonicalize()
                .with_context(|| format!("Cannot resolve file: {}", f.display()))?;
            // GH #3623 — refuse to leak the absolute path into the spec ID.
            let (rel, warn) = safe_relative_path(&abs, &config.project_root);
            if let Some(msg) = warn {
                tracing::warn!(target: "jet::test_runner::discovery", "{}", msg);
            }
            out.push(SpecFile {
                path: abs,
                relative: rel,
            });
        }
        out.sort_by(|a, b| a.path.cmp(&b.path));
        return Ok(out);
    }

    let include = build_globset(&config.test_match)?;
    let exclude = build_globset(&config.test_ignore)?;

    let mut out = Vec::new();
    for entry in WalkDir::new(&config.test_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Never filter the walk root — tempdir names like `.tmpXXX` on
            // macOS would otherwise abort the traversal.
            if e.depth() == 0 {
                return true;
            }
            let rel = match e.path().strip_prefix(&config.test_dir) {
                Ok(r) => r,
                Err(_) => e.path(),
            };
            !is_hidden(e) && !exclude.is_match(rel)
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                // GH #3330 — surface unreachable spec files so an unreadable
                // sub-tree (chmod, broken symlink, transient IO) does not
                // silently shrink the discovered test set. Skip the entry —
                // the rest of the walk is still useful — but log the error
                // with enough context to chase the underlying problem.
                tracing::warn!(
                    target: "jet::test_runner::discovery",
                    test_dir = %config.test_dir.display(),
                    path = ?err.path(),
                    depth = err.depth(),
                    error = %err,
                    "GH #3330 walkdir entry failed during spec discovery; \
                     skipping this entry. Specs under an unreachable directory \
                     will not run. Check filesystem permissions or remove the \
                     broken entry."
                );
                continue;
            }
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        // GH #3623 — never leak the absolute path into SpecFile.relative.
        let (relative, warn) = safe_relative_path(path, &config.project_root);
        if let Some(msg) = warn {
            tracing::warn!(target: "jet::test_runner::discovery", "{}", msg);
        }
        if exclude.is_match(&relative) {
            continue;
        }
        if !include.is_match(&relative) {
            continue;
        }
        out.push(SpecFile {
            path: path.to_path_buf(),
            relative,
        });
    }
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

/// Filter `changed` to the subset that matches `config.test_match` and
/// not `config.test_ignore`. Used by watch mode to focus a rerun on the
/// spec files the user just touched instead of re-running the whole
/// suite. Paths outside the project root are dropped silently — the
/// watcher can emit them for tempfile noise.
// @spec #2712
pub fn pick_focused_specs(config: &RunnerConfig, changed: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let include = build_globset(&config.test_match)?;
    let exclude = build_globset(&config.test_ignore)?;
    let mut focused = Vec::new();
    for path in changed {
        let rel = match path.strip_prefix(&config.project_root) {
            Ok(r) => r.to_path_buf(),
            Err(_) => continue,
        };
        if exclude.is_match(&rel) {
            continue;
        }
        if !include.is_match(&rel) {
            continue;
        }
        if !focused.iter().any(|p| p == path) {
            focused.push(path.clone());
        }
    }
    focused.sort();
    Ok(focused)
}

/// Build a globset from raw patterns. Patterns are matched against paths
/// *relative* to the project root — no path-prefix manipulation.
fn build_globset(patterns: &[String]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for raw in patterns {
        let pattern = raw.trim_start_matches("./");
        let g = Glob::new(pattern).with_context(|| format!("Invalid glob pattern: {raw}"))?;
        builder.add(g);
    }
    builder.build().context("Failed to build globset")
}

/// Ignore dotfiles and dotdirs (e.g. `.git`, `.jet`). Individual patterns in
/// `test_ignore` can re-include where needed.
fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s != "." && s.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    #[test]
    fn discovers_spec_files_at_root() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.spec.ts"), "");
        write(&tmp.path().join("b.test.ts"), "");
        write(&tmp.path().join("ignore.md"), "");

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).unwrap();
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn skips_node_modules() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("node_modules/pkg/x.spec.ts"), "");
        write(&tmp.path().join("src/y.spec.ts"), "");

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].path.ends_with("src/y.spec.ts"));
    }

    #[test]
    fn skips_dist_and_target() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("dist/out.spec.ts"), "");
        write(&tmp.path().join("target/debug/x.spec.ts"), "");
        write(&tmp.path().join("tests/ok.spec.ts"), "");

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).unwrap();
        assert_eq!(found.len(), 1);
        assert!(found[0].path.ends_with("tests/ok.spec.ts"));
    }

    #[test]
    fn explicit_files_bypass_glob() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("never-matches.md"), "");
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.only_files = vec![PathBuf::from("never-matches.md")];
        let found = scan(&cfg).unwrap();
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn resolve_discovery_captures_inputs_and_specs() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("src/a.spec.ts"), "");
        write(&tmp.path().join("src/b.test.ts"), "");
        write(&tmp.path().join("node_modules/skip.spec.ts"), "");

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let resolved = resolve_discovery(&cfg).expect("resolve");

        assert_eq!(resolved.schema_version, RESOLVED_DISCOVERY_SCHEMA_VERSION);
        assert_eq!(resolved.environment, "node");
        assert_eq!(resolved.timeout_ms, 30_000);
        assert!(resolved.workers >= 1);
        assert!(resolved.shard.is_none());
        assert!(resolved.only_files.is_empty());

        // Spec list mirrors `scan` ordering + filtering.
        let rels: Vec<_> = resolved
            .specs
            .iter()
            .map(|s| s.relative.to_string_lossy().into_owned())
            .collect();
        assert_eq!(rels.len(), 2);
        assert!(rels.iter().any(|r| r.ends_with("src/a.spec.ts")));
        assert!(rels.iter().any(|r| r.ends_with("src/b.test.ts")));
        // node_modules is excluded.
        assert!(!rels.iter().any(|r| r.contains("node_modules")));
    }

    #[test]
    fn resolve_discovery_is_deterministic_across_calls() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("z.spec.ts"), "");
        write(&tmp.path().join("a.spec.ts"), "");
        write(&tmp.path().join("m.spec.ts"), "");
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();

        let a = resolve_discovery(&cfg).unwrap();
        let b = resolve_discovery(&cfg).unwrap();
        assert_eq!(a, b);

        let names: Vec<_> = a
            .specs
            .iter()
            .map(|s| s.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.spec.ts", "m.spec.ts", "z.spec.ts"]);
    }

    #[test]
    fn resolve_discovery_rejects_missing_test_dir() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.test_dir = tmp.path().join("does/not/exist");
        let err = resolve_discovery(&cfg).unwrap_err();
        assert_eq!(err.code, "invalid_test_dir");
        assert!(err.message.contains("does not exist"));
        // JSON form is parseable + carries the same code.
        let v: serde_json::Value = serde_json::from_str(&err.to_json()).unwrap();
        assert_eq!(v["kind"], "discovery_config_error");
        assert_eq!(v["code"], "invalid_test_dir");
    }

    #[test]
    fn resolve_discovery_rejects_unsupported_environment() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.environment = TestEnvironment::Dom;
        let err = resolve_discovery(&cfg).unwrap_err();
        assert_eq!(err.code, "unsupported_environment");
        assert!(err.message.to_lowercase().contains("dom"));
    }

    #[test]
    fn resolve_discovery_rejects_invalid_include_glob() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.test_match = vec!["[unclosed".to_string()];
        let err = resolve_discovery(&cfg).unwrap_err();
        assert_eq!(err.code, "invalid_glob_include");
    }

    #[test]
    fn resolve_discovery_serialises_to_stable_json() {
        // @spec #2709
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.spec.ts"), "");
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let resolved = resolve_discovery(&cfg).unwrap();
        let json = serde_json::to_value(&resolved).unwrap();
        // Top-level shape contract: keys other consumers depend on.
        for key in [
            "schema_version",
            "project_root",
            "test_dir",
            "test_match",
            "test_ignore",
            "environment",
            "timeout_ms",
            "workers",
            "shard",
            "only_files",
            "specs",
        ] {
            assert!(
                json.get(key).is_some(),
                "resolved discovery JSON missing key: {key}",
            );
        }
        assert_eq!(json["environment"], "node");
        assert_eq!(json["schema_version"], RESOLVED_DISCOVERY_SCHEMA_VERSION);
    }

    #[test]
    fn deterministic_ordering() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("z.spec.ts"), "");
        write(&tmp.path().join("a.spec.ts"), "");
        write(&tmp.path().join("m.spec.ts"), "");

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).unwrap();
        let names: Vec<_> = found
            .iter()
            .map(|s| s.path.file_name().unwrap().to_str().unwrap().to_string())
            .collect();
        assert_eq!(names, vec!["a.spec.ts", "m.spec.ts", "z.spec.ts"]);
    }

    // ── GH #3330 silent walkdir Err swallow ──────────────────────────────

    /// GH #3330 — happy path: a normal tree of spec files discovers every one.
    #[test]
    fn gh3330_scan_normal_tree_happy_path() {
        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("a.spec.ts"), "");
        write(&tmp.path().join("nested/b.spec.ts"), "");
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).unwrap();
        assert_eq!(found.len(), 2, "happy path must discover both specs");
    }

    /// GH #3330 — chmod 000 on a subtree: scan must surface the error and
    /// still return whatever specs are reachable (we verify the walk
    /// completes and the reachable spec is present).
    #[cfg(unix)]
    #[test]
    fn gh3330_scan_unreadable_subtree_surfaces_and_continues() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        // Reachable spec at the root.
        write(&tmp.path().join("ok.spec.ts"), "");
        // Unreadable subdir containing a spec we expect to be skipped.
        let locked = tmp.path().join("locked");
        std::fs::create_dir_all(&locked).unwrap();
        write(&locked.join("hidden.spec.ts"), "");
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Root may still descend — skip if we observe that.
        if std::fs::read_dir(&locked).is_ok() {
            let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));
            return;
        }

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let result = scan(&cfg);

        // Restore perms so tempdir cleanup works.
        let _ = std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755));

        let found = result.expect("scan must return Ok even when a subtree errors");
        assert!(
            found.iter().any(|s| s.path.ends_with("ok.spec.ts")),
            "reachable spec must still be discovered: {:?}",
            found
        );
        // The hidden spec must NOT appear (it lives behind chmod 000).
        assert!(
            !found.iter().any(|s| s.path.ends_with("hidden.spec.ts")),
            "spec inside chmod-000 dir must be skipped: {:?}",
            found
        );
    }

    /// GH #3330 — broken symlink is not a walkdir-fatal error: scan still
    /// completes and yields nothing surprising. Validates we don't blow up
    /// when the entry-failure path is taken on a follow-link miss.
    #[cfg(unix)]
    #[test]
    fn gh3330_scan_broken_symlink_does_not_abort() {
        use std::os::unix::fs::symlink;

        let tmp = TempDir::new().unwrap();
        write(&tmp.path().join("ok.spec.ts"), "");
        // Broken symlink pointing nowhere — WalkDir with follow_links(false)
        // typically tolerates this; the test guarantees scan never panics.
        let _ = symlink(
            tmp.path().join("does-not-exist"),
            tmp.path().join("dangling.spec.ts"),
        );

        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let found = scan(&cfg).expect("scan must tolerate broken symlinks");
        assert!(
            found.iter().any(|s| s.path.ends_with("ok.spec.ts")),
            "ok.spec.ts must still be discovered alongside the broken symlink"
        );
    }

    // ── pick_focused_specs (#2712) ───────────────────────────────────────

    #[test]
    fn pick_focused_returns_only_test_files_among_changes() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        // Use the canonicalized project_root the config already resolved
        // so strip_prefix succeeds on platforms (macOS) where tmp paths
        // contain symlinks like /var → /private/var.
        let changed = vec![
            cfg.project_root.join("a.spec.ts"),
            cfg.project_root.join("src").join("util.ts"),
            cfg.project_root.join("README.md"),
        ];
        let focused = pick_focused_specs(&cfg, &changed).unwrap();
        assert_eq!(focused, vec![cfg.project_root.join("a.spec.ts")]);
    }

    #[test]
    fn pick_focused_respects_test_ignore() {
        let tmp = TempDir::new().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.test_ignore.push("**/skip/**".to_string());
        let changed = vec![
            cfg.project_root.join("kept.spec.ts"),
            cfg.project_root.join("skip").join("nope.spec.ts"),
        ];
        let focused = pick_focused_specs(&cfg, &changed).unwrap();
        assert_eq!(focused, vec![cfg.project_root.join("kept.spec.ts")]);
    }

    #[test]
    fn pick_focused_drops_paths_outside_project_root() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        // A watcher event for a temp file outside the project must be ignored.
        let changed = vec![PathBuf::from("/var/tmp/foreign.spec.ts")];
        let focused = pick_focused_specs(&cfg, &changed).unwrap();
        assert!(focused.is_empty());
    }

    #[test]
    fn pick_focused_returns_empty_when_no_test_files_changed() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let changed = vec![
            cfg.project_root.join("src").join("util.ts"),
            cfg.project_root.join("README.md"),
        ];
        let focused = pick_focused_specs(&cfg, &changed).unwrap();
        assert!(focused.is_empty());
    }

    #[test]
    fn pick_focused_deduplicates_and_sorts() {
        let tmp = TempDir::new().unwrap();
        let cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        let root = &cfg.project_root;
        let dup = root.join("dup.spec.ts");
        let changed = vec![
            root.join("z.spec.ts"),
            dup.clone(),
            root.join("a.spec.ts"),
            dup.clone(),
        ];
        let focused = pick_focused_specs(&cfg, &changed).unwrap();
        assert_eq!(
            focused,
            vec![
                root.join("a.spec.ts"),
                root.join("dup.spec.ts"),
                root.join("z.spec.ts"),
            ]
        );
    }
}

#[cfg(test)]
mod gh3623_safe_relative_path_tests {
    //! GH #3623 — `scan` previously did
    //! `path.strip_prefix(project_root).unwrap_or(path).to_path_buf()`
    //! for `SpecFile.relative`. When the spec was not under project_root
    //! (symlinked, external --only-files, monorepo mirror), the silent
    //! fallback leaked the absolute path into the public list manifest
    //! (via `list_manifest.rs`'s `id = relative.to_string_lossy()`).
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn under_root_returns_stripped_no_warn() {
        let root = PathBuf::from("/proj");
        let abs = PathBuf::from("/proj/tests/foo.spec.ts");
        let (rel, warn) = safe_relative_path(&abs, &root);
        assert_eq!(rel, PathBuf::from("tests/foo.spec.ts"));
        assert!(warn.is_none(), "under-root must not warn");
    }

    #[test]
    fn outside_root_falls_back_to_file_name_only() {
        let root = PathBuf::from("/proj");
        let abs = PathBuf::from("/elsewhere/external.spec.ts");
        let (rel, warn) = safe_relative_path(&abs, &root);
        assert_eq!(
            rel,
            PathBuf::from("external.spec.ts"),
            "outside-root must reduce to file_name only — NEVER leak abs path"
        );
        let msg = warn.expect("outside-root must warn");
        assert!(msg.contains("GH #3623"), "msg: {msg}");
        assert!(msg.contains("/elsewhere/external.spec.ts"), "msg: {msg}");
        assert!(msg.contains("/proj"), "msg: {msg}");
    }

    #[test]
    fn outside_root_relative_does_not_contain_abs_prefix() {
        // The whole point: list_manifest.rs builds ID = relative.to_string_lossy().
        // It must NOT start with "/elsewhere" or contain the abs prefix.
        let root = PathBuf::from("/proj");
        let abs = PathBuf::from("/elsewhere/sub/dir/external.spec.ts");
        let (rel, _) = safe_relative_path(&abs, &root);
        let id = rel.to_string_lossy();
        assert!(
            !id.starts_with("/elsewhere"),
            "ID must not leak abs path, got: {id}"
        );
        assert!(
            !id.contains("/sub/dir"),
            "ID must not leak intermediate dirs, got: {id}"
        );
        assert_eq!(id, "external.spec.ts");
    }

    #[test]
    fn helper_message_includes_tag_and_paths() {
        let root = PathBuf::from("/proj");
        let abs = PathBuf::from("/elsewhere/x.spec.ts");
        let err = abs.strip_prefix(&root).unwrap_err();
        let msg = format_safe_relative_path_warn(&abs, &root, &err);
        assert!(msg.contains("GH #3623"), "msg: {msg}");
        assert!(msg.contains("/elsewhere/x.spec.ts"), "msg: {msg}");
        assert!(msg.contains("/proj"), "msg: {msg}");
        assert!(msg.contains("file_name"), "msg: {msg}");
    }

    #[test]
    fn path_equal_to_root_returns_empty_relative() {
        let root = PathBuf::from("/proj");
        let abs = PathBuf::from("/proj");
        let (rel, warn) = safe_relative_path(&abs, &root);
        assert_eq!(rel, PathBuf::from(""));
        assert!(warn.is_none());
    }
}
// CODEGEN-END
