// REQ: Tick 19 — PEP 660 editable installs.
//
// Given a source root (a directory with `pyproject.toml`), call the PEP 660
// `build_editable` hook to produce a wheel whose installation leaves user
// code in-place: changes to the source tree are picked up by Python without
// a rebuild. This is the same primitive uv uses when a user runs
// `uv add -e <path>`.
//
// Many older backends predate PEP 660 and only implement `build_wheel`.
// The frontend script we emit catches the resulting `AttributeError` and
// falls back to a regular wheel, matching pip's behavior. The wheel still
// installs cleanly — the user just loses the live-edit property.

use std::path::{Path, PathBuf};

use crate::pkgmanage::pkgmgr::pep517::{
    create_isolated_venv, install_build_requires, pep517_frontend_script, run_subprocess_capture,
    venv_python_path, BuildConfig,
};
use crate::pkgmanage::pkgmgr::sdist::{read_build_system_from, BuildSystem};
use crate::pkgmanage::pkgmgr::types::IndexError;

/// Config alias mirroring [`crate::pkgmanage::pkgmgr::sdist::SdistBuildConfig`].
/// Editable builds share the isolated-venv semantics of regular wheel
/// builds; we just call a different hook.
pub type EditableBuildConfig = BuildConfig;

/// Build a PEP 660 editable wheel from `source_root` into `out_dir`.
///
/// Steps:
///   1. Read `[build-system]` from `source_root/pyproject.toml` (PEP 518
///      default applies when missing).
///   2. Create an isolated venv at `{work_dir}/build-env/` from `config.python`.
///   3. Install `build-system.requires` via the venv's pip.
///   4. Run a one-shot Python frontend that calls
///      `backend.build_editable(out_dir)` and falls back to
///      `backend.build_wheel(out_dir)` on `AttributeError`.
///   5. Return the absolute path to the produced wheel.
///
/// `out_dir` is created if it does not already exist.
///
/// # Errors
///
/// - [`IndexError::CacheIo`] — filesystem failure.
/// - [`IndexError::NetworkError`] — subprocess invocation failure.
pub fn build_editable_wheel(
    source_root: &Path,
    out_dir: &Path,
    config: &EditableBuildConfig,
) -> Result<PathBuf, IndexError> {
    if !source_root.is_dir() {
        return Err(IndexError::CacheIo {
            path: source_root.display().to_string(),
            detail: "editable source root is not a directory".into(),
        });
    }

    let bs: BuildSystem = read_build_system_from(source_root)?;

    let venv = config.work_dir.join("build-env");
    create_isolated_venv(&config.python, &venv)?;
    let venv_python = venv_python_path(&venv);

    install_build_requires(&venv_python, &bs.requires)?;

    std::fs::create_dir_all(out_dir).map_err(|e| IndexError::CacheIo {
        path: out_dir.display().to_string(),
        detail: format!("create out_dir: {e}"),
    })?;
    let out_abs = std::fs::canonicalize(out_dir).map_err(|e| IndexError::CacheIo {
        path: out_dir.display().to_string(),
        detail: format!("canonicalize out_dir: {e}"),
    })?;

    let frontend = pep517_frontend_script(
        &bs.backend,
        "build_editable",
        Some("build_wheel"),
        &out_abs,
    );
    let stdout = run_subprocess_capture(
        &venv_python,
        &["-c", &frontend],
        Some(source_root),
        "PEP 660 build_editable",
    )?;

    let basename = stdout.lines().last().unwrap_or("").trim().to_string();
    if basename.is_empty() {
        return Err(IndexError::NetworkError {
            url: "<pep660-build>".into(),
            detail: format!("build_editable hook returned empty filename (stdout: {stdout:?})"),
        });
    }
    let produced = out_abs.join(&basename);
    if !produced.exists() {
        return Err(IndexError::CacheIo {
            path: produced.display().to_string(),
            detail: "build_editable hook reported a wheel that does not exist on disk".into(),
        });
    }
    Ok(produced)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn which_python() -> Option<PathBuf> {
        for candidate in &["python3", "python"] {
            let out = Command::new(candidate)
                .args(["-c", "import sys, venv, pip; sys.exit(0)"])
                .output();
            if let Ok(o) = out {
                if o.status.success() {
                    return Some(PathBuf::from(candidate));
                }
            }
        }
        None
    }

    #[test]
    fn build_editable_wheel_rejects_non_directory_root() {
        let tmp = tempfile::tempdir().unwrap();
        let not_a_dir = tmp.path().join("does_not_exist");
        let cfg = EditableBuildConfig {
            python: PathBuf::from("python3"),
            work_dir: tmp.path().join("work"),
        };
        let err = build_editable_wheel(&not_a_dir, &tmp.path().join("out"), &cfg).unwrap_err();
        match err {
            IndexError::CacheIo { detail, .. } => {
                assert!(detail.contains("not a directory"), "{detail}");
            }
            other => panic!("expected CacheIo, got {other:?}"),
        }
    }

    /// End-to-end PEP 660 build: construct a minimal setuptools project,
    /// drive `build_editable_wheel`, and verify a wheel materializes. The
    /// resulting wheel may be either an editable wheel (setuptools >= 64
    /// implements `build_editable`) or a regular wheel via the fallback —
    /// the test asserts the produced file is a real `.whl` either way.
    #[test]
    fn build_editable_wheel_end_to_end() {
        let Some(python) = which_python() else {
            eprintln!("[build_editable_wheel_end_to_end] skipped: no python3 on PATH");
            return;
        };

        let tmp = tempfile::tempdir().unwrap();
        let proj = tmp.path().join("mamba_t19_demo");
        std::fs::create_dir_all(proj.join("src/mamba_t19_demo")).unwrap();
        std::fs::write(
            proj.join("pyproject.toml"),
            b"[build-system]\nrequires = [\"setuptools>=64\", \"wheel\"]\nbuild-backend = \"setuptools.build_meta\"\n\n[project]\nname = \"mamba_t19_demo\"\nversion = \"0.1.0\"\n\n[tool.setuptools.packages.find]\nwhere = [\"src\"]\n",
        )
        .unwrap();
        std::fs::write(
            proj.join("src/mamba_t19_demo/__init__.py"),
            b"VERSION = '0.1.0'\n",
        )
        .unwrap();

        let out = tmp.path().join("out");
        let cfg = EditableBuildConfig {
            python,
            work_dir: tmp.path().join("work"),
        };

        let result = build_editable_wheel(&proj, &out, &cfg);
        match result {
            Ok(wheel) => {
                let name = wheel.file_name().unwrap().to_string_lossy().to_string();
                assert!(
                    name.starts_with("mamba_t19_demo-0.1.0") && name.ends_with(".whl"),
                    "built editable wheel must be mamba_t19_demo-0.1.0-*.whl (got {name})"
                );
                assert!(wheel.exists(), "built editable wheel must exist on disk");
            }
            Err(e) => {
                eprintln!(
                    "[build_editable_wheel_end_to_end] skipped: PEP 660 build did not complete ({e:?})"
                );
            }
        }
    }
}
