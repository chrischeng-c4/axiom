// REQ: Tick 19 — shared PEP 517 / PEP 660 subprocess plumbing.
//
// Both `sdist::build_wheel_from_sdist` (Tick 18) and
// `editable::build_editable_wheel` (Tick 19) need to:
//   1. Spawn an isolated venv from a host Python.
//   2. Install the project's declared build requires into that venv.
//   3. Invoke a PEP 517 hook (`build_wheel` or `build_editable`) via a
//      one-shot Python frontend script, capturing the produced wheel
//      basename from stdout.
//
// The previous version inlined these helpers inside `sdist.rs`. Hoisting
// them here keeps both builders honest about the shared contract and lets
// the editable path reuse the same isolated env semantics that uv ships.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Configuration shared by every PEP 517 builder.
#[derive(Debug, Clone)]
pub struct BuildConfig {
    /// Host interpreter used as the **base** for the isolated build venv.
    /// Must support `-m venv` (≥3.3, in practice ≥3.8 for modern backends).
    pub python: PathBuf,
    /// Work directory in which the isolated venv lives. Callers append
    /// build-specific subdirs (e.g. `src/` for sdist unpacking).
    pub work_dir: PathBuf,
}

/// Path to the python interpreter inside a venv, regardless of platform.
pub fn venv_python_path(venv: &Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("python.exe")
    } else {
        venv.join("bin").join("python")
    }
}

/// Create a fresh isolated venv at `venv_dir` using `python`. Removes any
/// existing directory at the target location first so callers can drive
/// a clean rebuild without stale artifacts.
pub fn create_isolated_venv(python: &Path, venv_dir: &Path) -> Result<(), IndexError> {
    if venv_dir.exists() {
        std::fs::remove_dir_all(venv_dir).map_err(|e| IndexError::CacheIo {
            path: venv_dir.display().to_string(),
            detail: format!("clean stale build venv: {e}"),
        })?;
    }
    run_subprocess(
        python,
        &["-m", "venv", venv_dir.to_string_lossy().as_ref()],
        None,
        "create build venv",
    )
}

/// Install a list of build requires into a venv using the venv's pip.
/// Each item in `requires` is passed verbatim to `pip install`, so PEP 440
/// specifiers and other markers are honored.
pub fn install_build_requires(
    venv_python: &Path,
    requires: &[String],
) -> Result<(), IndexError> {
    if requires.is_empty() {
        return Ok(());
    }
    let mut args: Vec<String> = vec![
        "-m".into(),
        "pip".into(),
        "install".into(),
        "--disable-pip-version-check".into(),
    ];
    args.extend(requires.iter().cloned());
    let args_ref: Vec<&str> = args.iter().map(String::as_str).collect();
    run_subprocess(
        venv_python,
        &args_ref,
        None,
        "pip install build-system.requires",
    )
}

/// Construct a one-shot Python script that imports `backend` and runs the
/// hook named `hook` with `out_dir` as the single positional argument. The
/// script prints exactly the returned filename (one line, no extras).
///
/// When `fallback_hook` is `Some`, the script catches `AttributeError`
/// from `hook` and falls back to the secondary hook. This is how PEP 660
/// `build_editable` degrades to legacy `build_wheel` when the backend
/// hasn't implemented the editable hook yet.
pub fn pep517_frontend_script(
    backend: &str,
    hook: &str,
    fallback_hook: Option<&str>,
    out_dir: &Path,
) -> String {
    // backend is e.g. "setuptools.build_meta" or "flit_core.buildapi:legacy".
    let (module, attr) = match backend.split_once(':') {
        Some((m, a)) => (m, a),
        None => (backend, ""),
    };
    let out_str = out_dir.to_string_lossy().replace('\\', "\\\\");

    let attempt_primary = format!("result = backend.{hook}({out_str:?})");

    let body = match fallback_hook {
        None => format!("{attempt_primary}\n"),
        Some(fb) => format!(
            "try:\n    {attempt_primary}\nexcept AttributeError:\n    result = backend.{fb}({out_str:?})\n",
        ),
    };

    format!(
        "import importlib, sys\n\
         mod = importlib.import_module({module:?})\n\
         backend = getattr(mod, {attr:?}) if {attr:?} else mod\n\
         {body}\
         sys.stdout.write(result + '\\n')\n\
         sys.stdout.flush()\n",
    )
}

/// Run a subprocess. Any non-zero exit becomes `IndexError::NetworkError`
/// (uniform with the rest of the `pkgmgr` error surface). Stdout/stderr
/// are captured and discarded — stderr is surfaced in the error detail on
/// failure.
pub fn run_subprocess(
    program: &Path,
    args: &[&str],
    cwd: Option<&Path>,
    label: &str,
) -> Result<(), IndexError> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(c) = cwd {
        cmd.current_dir(c);
    }
    let output = cmd.output().map_err(|e| IndexError::NetworkError {
        url: format!("<{label}>"),
        detail: format!("spawn {}: {e}", program.display()),
    })?;
    if !output.status.success() {
        return Err(IndexError::NetworkError {
            url: format!("<{label}>"),
            detail: format!(
                "{} exited with {}; stderr: {}",
                program.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    Ok(())
}

/// Like [`run_subprocess`] but returns captured stdout on success.
pub fn run_subprocess_capture(
    program: &Path,
    args: &[&str],
    cwd: Option<&Path>,
    label: &str,
) -> Result<String, IndexError> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(c) = cwd {
        cmd.current_dir(c);
    }
    let output = cmd.output().map_err(|e| IndexError::NetworkError {
        url: format!("<{label}>"),
        detail: format!("spawn {}: {e}", program.display()),
    })?;
    if !output.status.success() {
        return Err(IndexError::NetworkError {
            url: format!("<{label}>"),
            detail: format!(
                "{} exited with {}; stderr: {}",
                program.display(),
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn venv_python_path_picks_platform_layout() {
        let venv = Path::new("/tmp/v");
        let p = venv_python_path(venv);
        if cfg!(windows) {
            assert!(p.ends_with("Scripts/python.exe") || p.ends_with("Scripts\\python.exe"));
        } else {
            assert_eq!(p, Path::new("/tmp/v/bin/python"));
        }
    }

    #[test]
    fn frontend_script_no_fallback_module_only() {
        let s = pep517_frontend_script(
            "setuptools.build_meta",
            "build_wheel",
            None,
            Path::new("/tmp/out"),
        );
        assert!(s.contains("\"setuptools.build_meta\""));
        assert!(s.contains("backend.build_wheel("));
        assert!(!s.contains("except AttributeError"));
        // module-only backend → empty attr placeholder
        assert!(s.contains("\"\""));
    }

    #[test]
    fn frontend_script_with_fallback_handles_attribute_error() {
        let s = pep517_frontend_script(
            "setuptools.build_meta",
            "build_editable",
            Some("build_wheel"),
            Path::new("/tmp/out"),
        );
        assert!(s.contains("backend.build_editable("));
        assert!(s.contains("except AttributeError"));
        assert!(s.contains("backend.build_wheel("));
    }

    #[test]
    fn frontend_script_module_colon_attr_form() {
        let s = pep517_frontend_script(
            "flit_core.buildapi:legacy",
            "build_wheel",
            None,
            Path::new("/tmp/out"),
        );
        assert!(s.contains("\"flit_core.buildapi\""));
        assert!(s.contains("\"legacy\""));
    }
}
