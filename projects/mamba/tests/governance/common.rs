//! Shared plumbing for the governance umbrella binaries.
//!
//! Both `schema_gates` and `mvp_gates` register this file with
//! `#[path = "common.rs"] mod common;` (the same `#[path]` convention the
//! umbrellas use for their gate sub-modules). It collapses the trivial
//! helpers that every gate used to re-declare — TOML loading, the
//! `CARGO_MANIFEST_DIR` project root, typed-field accessors, the
//! `python3 <script>` spawn, and self-cleaning temp dirs — into one place
//! with a single error policy. The per-gate *assertions* stay in the gate
//! files; only the plumbing is shared.
//!
//! `env!("CARGO_MANIFEST_DIR")` is a compile-time macro expanded in the
//! integration-test crate, so it resolves to `projects/mamba` here exactly
//! as it did inside each gate module.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// Read and parse a TOML document, panicking with a path-tagged message on
/// any IO or parse error. This is the unified error policy for the dozens of
/// former per-gate `load_toml` copies (some of which used a bare `.unwrap()`,
/// some a tagged `panic!`); failures are unconditional panics in every case,
/// so unifying on the more informative message preserves behavior.
pub fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("manifest {} unreadable: {e}", path.display()));
    raw.parse()
        .unwrap_or_else(|e| panic!("{} parse error: {e}", path.display()))
}

/// The mamba crate root (`projects/mamba`). Former gates spelled this
/// `project_root()` / `mamba_root()`; both expanded to exactly this.
pub fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Alias kept so gates that historically called `mamba_root()` read
/// naturally; identical to [`project_root`].
pub fn mamba_root() -> PathBuf {
    project_root()
}

/// Fetch a required, non-empty string field, panicking with a context-tagged
/// message when it is missing or empty.
pub fn require_str<'a>(value: &'a toml::Value, key: &str) -> &'a str {
    value
        .get(key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| panic!("missing or empty required string `{key}`"))
}

/// Fetch a required integer field, panicking when it is missing or non-integer.
pub fn require_int(value: &toml::Value, key: &str) -> i64 {
    value
        .get(key)
        .and_then(|v| v.as_integer())
        .unwrap_or_else(|| panic!("missing required integer `{key}`"))
}

/// Fetch a required boolean field, panicking when it is missing or non-bool.
pub fn require_bool(value: &toml::Value, key: &str) -> bool {
    value
        .get(key)
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| panic!("missing required bool `{key}`"))
}

/// Spawn `python3 <script> <args...>` from the project root and capture the
/// full [`Output`]. This is the shared body of every former
/// `run_checker` / `run_validator` / `run_runner`, which all built the exact
/// same `Command` and only differed in the script path and the `.expect`
/// message. Callers convert the `Output` into whatever tuple they assert on.
pub fn run_python_script(script: &Path, args: &[&str]) -> Output {
    Command::new("python3")
        .arg(script)
        .args(args)
        .current_dir(project_root())
        .output()
        .unwrap_or_else(|e| panic!("invoke python3 {}: {e}", script.display()))
}

/// A unique temp directory that removes itself (best-effort) when dropped,
/// fixing the leak from the former `unique_dir` copies that called
/// `create_dir_all` and never cleaned up. Derefs to [`Path`] so it can be used
/// anywhere a `&Path` was expected; call [`TempDir::path`] for an owned clone.
pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::ops::Deref for TempDir {
    type Target = Path;
    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for TempDir {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Create a freshly-named temp directory tagged with `tag`, auto-cleaned via
/// [`Drop`]. Replaces the never-cleaned `unique_dir` helpers.
pub fn unique_temp_dir(tag: &str) -> TempDir {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let path = std::env::temp_dir().join(format!("mamba-gov-{tag}-{nanos}"));
    std::fs::create_dir_all(&path).expect("create tempdir");
    TempDir { path }
}
