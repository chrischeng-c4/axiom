// `mamba run` package-manager preflight and interpreter routing — closes
// #2684; #4207 drops the `PYTHONPATH` injection this module used to own;
// #4210 makes a file's default route the resolved interpreter instead of
// the Mamba compiler.
//
// Acceptance (tests/governance/gates/pkgmgr/run/manifest.toml, schema gate
// pkgmgr_run_fixture_2684.rs; apps/mamba/e2e/pkgmgr_run_file_venv.rs):
//
//   - Running before sync fails with "environment is not synced".
//   - Running after sync proceeds: the venv's own `bin` (or `Scripts`)
//     directory goes first on the child's `PATH`, so the venv's own
//     interpreter resolves its own site-packages through `site`, the
//     way every other Python tool does — never through `PYTHONPATH`.
//   - No global PATH or user shell env mutation: env changes are scoped
//     to the spawned child (`mamba run -- <cmd>` or `mamba run <file>`).
//   - Offline against the frozen local index (sync owns that).
//   - `mamba run <file>` executes the file on the interpreter
//     `resolve_run_interpreter` selects for the current `Mode`, never the
//     compiler, unless `--compile` was passed or `<file>` is the `-` stdin
//     sentinel (the compiler's own routes, decided by `routes_to_compiler`
//     in `main.rs`, not re-derived here).
//
// Hook shape: callers invoke `preflight(project_dir)` BEFORE handing the
// file off to an interpreter or the compiler, then pass the resulting
// `Mode` to `resolve_run_interpreter` to pick the child to spawn. Legacy
// `mamba run <file>` outside a mamba project is untouched (`preflight`
// returns `Mode::Legacy`, and `resolve_run_interpreter` walks `PATH`).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pkgmanage::sync::{parse_locked_packages, resolve_site_packages};

const MANIFEST_FILE: &str = "mamba.toml";
const LOCKFILE_FILE: &str = "mamba.lock";
const VENV_DIR: &str = ".venv";

#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// No mamba.toml here — caller should run the file with legacy
    /// semantics (no env contract).
    Legacy,
    /// mamba.toml present; venv is in sync with the lockfile. The
    /// venv's own `bin` (or `Scripts`) directory goes first on the
    /// child's `PATH` so its own interpreter resolves its own
    /// site-packages through `site`, never `PYTHONPATH`.
    Project { site_packages: PathBuf },
    /// mamba.toml present, lockfile has zero packages, so no env
    /// is required.
    EmptyLock,
}

/// Project-aware preflight for `mamba run`. Returns the execution mode.
/// Bails when the project has a non-empty lockfile but the venv has not
/// been synced (no `pyvenv.cfg` yet). Sets no environment variable —
/// `configure_command_environment` scopes every change to the spawned
/// child.
pub fn preflight(project_dir: &Path) -> Result<Mode> {
    let manifest = project_dir.join(MANIFEST_FILE);
    if !manifest.exists() {
        return Ok(Mode::Legacy);
    }
    let lock_path = project_dir.join(LOCKFILE_FILE);
    if !lock_path.exists() {
        // mamba.toml without a lockfile is a still-initializing
        // project. Don't gate run on it; treat as legacy.
        return Ok(Mode::Legacy);
    }
    let lock_src = std::fs::read_to_string(&lock_path)
        .with_context(|| format!("read {}", lock_path.display()))?;
    let packages = parse_locked_packages(&lock_src)?;
    if packages.is_empty() {
        return Ok(Mode::EmptyLock);
    }

    let venv_dir = project_dir.join(VENV_DIR);
    if !venv_dir.join("pyvenv.cfg").exists() {
        bail!(
            "environment is not synced — run `mamba sync` to install \
             {n} locked package(s) into {venv}",
            n = packages.len(),
            venv = venv_dir.display()
        );
    }

    Ok(Mode::Project {
        site_packages: resolve_site_packages(&venv_dir),
    })
}

/// Apply the project environment to a subprocess for `mamba run -- <cmd>`.
/// This never mutates the user's shell and never sets `PYTHONPATH`.
/// Executables from `.venv/bin` (or `.venv/Scripts`) win over host PATH
/// when a synced venv exists, so the venv's own interpreter resolves its
/// own site-packages the way every other Python tool does.
pub fn configure_command_environment(command: &mut Command, project_dir: &Path, _mode: &Mode) {
    let venv = project_dir.join(VENV_DIR);
    if venv.join("pyvenv.cfg").exists() {
        command.env("VIRTUAL_ENV", &venv);
        let bin_dirs = [venv.join("bin"), venv.join("Scripts")]
            .into_iter()
            .filter(|path| path.is_dir());
        if let Some(joined) = joined_path_front("PATH", bin_dirs) {
            command.env("PATH", joined);
        }
    }
}

/// Whether `mamba run <file>` should hand `<file>` to the Mamba compiler
/// instead of the interpreter `resolve_run_interpreter` selects: the
/// explicit `--compile` opt-in, or the `-` stdin sentinel, which the
/// compiler still owns because there is no file on disk to hand to a
/// spawned interpreter.
pub fn routes_to_compiler(compile_flag: bool, file: &str) -> bool {
    compile_flag || file == "-"
}

/// The interpreter path a synced project's own venv lays down —
/// `<project>/.venv/bin/python` on POSIX, `<project>/.venv/Scripts/python.exe`
/// on Windows. Returned un-resolved (never `canonicalize`d): the caller
/// spawns this launch path directly, the way every other Python tool does.
pub fn venv_python_path(project_dir: &Path) -> PathBuf {
    let venv = project_dir.join(VENV_DIR);
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Select the interpreter `mamba run <file>` should spawn for the given
/// preflight `Mode`. Never re-derives project state (no second
/// `mamba.toml`/lockfile read) — `mode` already carries every decision this
/// needs, which is what lets the negative control that pins `Mode::Legacy`
/// unconditionally prove this function actually reads it.
///
///   - `Mode::Project { .. }`: the project's own venv interpreter.
///   - `Mode::EmptyLock`: the venv interpreter when a venv exists (a project
///     with no locked packages may still have run `mamba venv` by hand),
///     otherwise the `PATH` interpreter.
///   - `Mode::Legacy`: the `PATH` interpreter only, `python3` before
///     `python`, ignoring any `.venv` that happens to sit in `project_dir`.
///
/// `path_value` is the raw `PATH` environment value to search — a parameter,
/// not a direct `std::env::var_os` read, so callers can pass a fixture PATH
/// under test.
pub fn resolve_run_interpreter(
    project_dir: &Path,
    mode: &Mode,
    path_value: Option<&std::ffi::OsStr>,
) -> Result<PathBuf> {
    match mode {
        Mode::Project { .. } => Ok(venv_python_path(project_dir)),
        Mode::EmptyLock => {
            let venv = project_dir.join(VENV_DIR);
            if venv.join("pyvenv.cfg").exists() {
                Ok(venv_python_path(project_dir))
            } else {
                resolve_path_interpreter(path_value)
            }
        }
        Mode::Legacy => resolve_path_interpreter(path_value),
    }
}

/// Walk `path_value` (a `PATH`-shaped environment value) for the first
/// `python3`, falling back to `python` in the same directory before moving
/// on to the next — so a directory carrying both never yields `python` over
/// `python3`. Bails naming both candidate names when neither resolves
/// anywhere on `path_value`.
pub fn resolve_path_interpreter(path_value: Option<&std::ffi::OsStr>) -> Result<PathBuf> {
    if let Some(path) = path_value {
        for dir in std::env::split_paths(path) {
            for name in ["python3", "python"] {
                let candidate = dir.join(name);
                #[cfg(windows)]
                let candidate = candidate.with_extension("exe");
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }
    bail!(
        "no `python3` or `python` found on PATH; `mamba run` needs one of \
         them to execute a file outside a synced project"
    );
}

fn joined_path_front<I>(key: &str, front: I) -> Option<std::ffi::OsString>
where
    I: IntoIterator<Item = PathBuf>,
{
    let existing = std::env::var_os(key);
    let mut paths: Vec<PathBuf> = Vec::new();
    for path in front {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    if let Some(v) = existing {
        for p in std::env::split_paths(&v) {
            if !paths.contains(&p) {
                paths.push(p);
            }
        }
    }
    std::env::join_paths(paths.iter()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_when_no_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::Legacy);
    }

    #[test]
    fn legacy_when_no_lockfile() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("mamba.toml"),
            "[project]\nname = \"x\"\nversion = \"0.1.0\"\npython-requires = \">=3.12\"\ndependencies = []\ndev-dependencies = []\n",
        )
        .unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::Legacy);
    }

    #[test]
    fn empty_lock_does_not_require_venv() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mamba.toml"), "[project]\n").unwrap();
        std::fs::write(
            tmp.path().join("mamba.lock"),
            "format_version = 1\ninput_hash = \"x\"\n",
        )
        .unwrap();
        assert_eq!(preflight(tmp.path()).unwrap(), Mode::EmptyLock);
    }

    #[test]
    fn non_empty_lock_without_venv_bails() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("mamba.toml"), "[project]\n").unwrap();
        std::fs::write(
            tmp.path().join("mamba.lock"),
            "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"foo\"\nversion = \"1.0\"\nsha256 = \"\"\nsource = \"pypi://foo/1.0\"\ndependencies = []\n",
        )
        .unwrap();
        let err = preflight(tmp.path()).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("environment is not synced"), "{msg}");
    }
}
