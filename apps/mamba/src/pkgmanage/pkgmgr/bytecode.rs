// Post-install bytecode pre-compilation (Tick 26).
//
// CPython lazily byte-compiles `.py` → `__pycache__/<mod>.cpython-XY.pyc`
// on first import. That cost shows up as first-run startup latency for
// large dependency trees (think `requests`, `boto3`, `sqlalchemy`). uv
// pre-compiles after install so the cost is paid once during `sync`
// instead of repeatedly at runtime.
//
// uv's strategy: spawn Python with `compileall`/`py_compile`, which is
// the canonical way to generate `.pyc` files because it owns the magic
// number + serialization format. Re-implementing the `.pyc` writer in
// Rust would tightly couple us to CPython's internal format, which
// changes minor-version-to-minor-version. Delegating to the target
// interpreter sidesteps that.
//
// This module ships:
//   - `compile_bytecode(target, python, opts)` — runs
//     `python -m compileall <target>` with the right flags. Returns a
//     typed report with file counts and stderr captured.
//   - `BytecodeOptions { workers, quiet, force }` mirror uv's user-facing
//     knobs: `-j` workers (0 = use CPU count), `-q` quiet, `-f` force
//     recompile even when `.pyc` is newer than `.py`.
//   - `count_py_files(target)` — best-effort estimate used to soft-skip
//     when there's nothing to compile and to make the report informative.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::pkgmanage::pkgmgr::types::IndexError;

/// Knobs for the compileall pass.
#[derive(Debug, Clone)]
pub struct BytecodeOptions {
    /// `-j` workers passed to `compileall`. `0` means "use CPU count"
    /// (compileall's own convention). uv defaults to all cores.
    pub workers: u32,
    /// `-q` quiet — suppress per-file output. uv defaults to quiet.
    pub quiet: bool,
    /// `-f` force — recompile even when an up-to-date `.pyc` already
    /// exists. Default off: idempotent re-syncs skip already-compiled
    /// files.
    pub force: bool,
}

impl Default for BytecodeOptions {
    fn default() -> Self {
        Self {
            workers: 0, // CPU count via compileall.
            quiet: true,
            force: false,
        }
    }
}

/// Report from one `compileall` invocation.
#[derive(Debug, Clone)]
pub struct CompileReport {
    /// Number of `.py` files discovered under `target` *before* the
    /// compileall run. Useful for "did anything actually happen?" checks.
    pub py_files_seen: usize,
    /// Was the run skipped because `py_files_seen == 0`?
    pub skipped_empty: bool,
    /// Captured stderr (truncated to 32 KiB by `compileall` itself in
    /// most cases). Empty when `skipped_empty` is true.
    pub stderr: String,
}

/// Compile bytecode for every `.py` file under `target`.
///
/// `python` should point to the *target environment's* interpreter (the
/// one that will later import these modules). Compiling with a different
/// minor version produces a `.pyc` with the wrong magic number, which
/// CPython will ignore at import time — wasted work, not incorrectness,
/// but still a bug.
///
/// Returns a `CompileReport`. Soft-skips with `skipped_empty=true` when
/// no `.py` files are found under `target` (very common on `bin/`-only
/// distributions). Returns `Err(IndexError::NetworkError)` if the python
/// subprocess fails — the variant name is a bit of a misnomer here but
/// it carries the stderr we need in `detail`.
pub fn compile_bytecode(
    target: &Path,
    python: &Path,
    opts: &BytecodeOptions,
) -> Result<CompileReport, IndexError> {
    let py_files_seen = count_py_files(target)?;
    if py_files_seen == 0 {
        return Ok(CompileReport {
            py_files_seen: 0,
            skipped_empty: true,
            stderr: String::new(),
        });
    }

    let mut cmd = Command::new(python);
    cmd.arg("-I"); // ignore PYTHON* env, match uv
    cmd.arg("-m").arg("compileall");
    if opts.quiet {
        cmd.arg("-q");
    }
    if opts.force {
        cmd.arg("-f");
    }
    cmd.arg(format!("-j{}", opts.workers));
    cmd.arg(target);

    let output: Output = cmd.output().map_err(|err| IndexError::NetworkError {
        url: python.display().to_string(),
        detail: format!("spawning python -m compileall: {err}"),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(IndexError::NetworkError {
            url: python.display().to_string(),
            detail: format!(
                "python -m compileall failed (status {:?}): {}",
                output.status.code(),
                stderr.trim()
            ),
        });
    }

    Ok(CompileReport {
        py_files_seen,
        skipped_empty: false,
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

/// Count `.py` files under `target` (depth-first). Used to estimate work
/// and to detect the "nothing to compile" case so we don't bother
/// spawning Python.
///
/// Hidden directories are *not* skipped — uv processes everything under
/// site-packages including `.dist-info` (which doesn't contain `.py`
/// files in practice, but the principle is "no surprise omissions").
pub fn count_py_files(target: &Path) -> Result<usize, IndexError> {
    if !target.exists() {
        return Err(IndexError::CacheIo {
            path: target.display().to_string(),
            detail: "compile target does not exist".into(),
        });
    }
    let mut count = 0usize;
    walk_for_py(target, &mut count)?;
    Ok(count)
}

fn walk_for_py(dir: &Path, count: &mut usize) -> Result<(), IndexError> {
    let read_dir = std::fs::read_dir(dir).map_err(|err| IndexError::CacheIo {
        path: dir.display().to_string(),
        detail: format!("reading directory: {err}"),
    })?;
    let mut subdirs: Vec<PathBuf> = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|err| IndexError::CacheIo {
            path: dir.display().to_string(),
            detail: format!("reading directory entry: {err}"),
        })?;
        let path = entry.path();
        if path.is_dir() {
            // Skip `__pycache__` — those are *outputs* of compileall, not
            // sources. Walking into them just inflates the count.
            if path.file_name().and_then(|s| s.to_str()) == Some("__pycache__") {
                continue;
            }
            subdirs.push(path);
        } else if path.extension().and_then(|s| s.to_str()) == Some("py") {
            *count += 1;
        }
    }
    for sub in subdirs {
        walk_for_py(&sub, count)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn first_python_on_path() -> Option<PathBuf> {
        let path_var = std::env::var_os("PATH")?;
        for dir in std::env::split_paths(&path_var) {
            for name in ["python3", "python"] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
        }
        None
    }

    #[test]
    fn count_py_files_recurses() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.py"), "pass\n").unwrap();
        fs::create_dir_all(dir.path().join("sub/nested")).unwrap();
        fs::write(dir.path().join("sub/b.py"), "pass\n").unwrap();
        fs::write(dir.path().join("sub/nested/c.py"), "pass\n").unwrap();
        // Non-py files don't count.
        fs::write(dir.path().join("README.txt"), "no").unwrap();
        let n = count_py_files(dir.path()).unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn count_py_files_skips_pycache() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.py"), "pass\n").unwrap();
        fs::create_dir_all(dir.path().join("__pycache__")).unwrap();
        // A `.py` inside __pycache__ should be invisible to the counter
        // (compileall ignores it too).
        fs::write(dir.path().join("__pycache__/ghost.py"), "pass\n").unwrap();
        let n = count_py_files(dir.path()).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn count_py_files_errors_on_missing_path() {
        let dir = TempDir::new().unwrap();
        let bogus = dir.path().join("definitely-not-there");
        let err = count_py_files(&bogus).unwrap_err();
        assert!(format!("{err}").contains("does not exist"));
    }

    #[test]
    fn compile_bytecode_soft_skips_empty_target() {
        let dir = TempDir::new().unwrap();
        // No .py files at all.
        fs::write(dir.path().join("notes.txt"), "nope").unwrap();
        // We don't need a real python for this branch — we never spawn one.
        let report = compile_bytecode(
            dir.path(),
            Path::new("/this/is/never/used"),
            &BytecodeOptions::default(),
        )
        .unwrap();
        assert!(report.skipped_empty);
        assert_eq!(report.py_files_seen, 0);
    }

    #[test]
    fn compile_bytecode_end_to_end() {
        // Real end-to-end: spawn the system python, compile two modules,
        // assert .pyc files materialize under __pycache__.
        let Some(python) = first_python_on_path() else {
            eprintln!("(no python3 on PATH — skipping compile_bytecode_end_to_end)");
            return;
        };

        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.py"), "x = 1\n").unwrap();
        fs::create_dir_all(dir.path().join("pkg")).unwrap();
        fs::write(dir.path().join("pkg/__init__.py"), "y = 2\n").unwrap();
        fs::write(dir.path().join("pkg/mod.py"), "z = 3\n").unwrap();

        let report = compile_bytecode(dir.path(), &python, &BytecodeOptions::default())
            .expect("compile should succeed");

        assert!(!report.skipped_empty);
        assert_eq!(report.py_files_seen, 3);

        // __pycache__/ should now exist with .pyc files in it.
        let top_cache = dir.path().join("__pycache__");
        assert!(top_cache.is_dir(), "top-level __pycache__ should exist");
        let pkg_cache = dir.path().join("pkg/__pycache__");
        assert!(pkg_cache.is_dir(), "pkg/__pycache__ should exist");

        let pyc_count = std::fs::read_dir(&top_cache)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("pyc"))
            .count();
        assert!(
            pyc_count >= 1,
            "expected at least one .pyc under {top_cache:?}"
        );
    }

    #[test]
    fn compile_bytecode_propagates_python_failure() {
        // Point at a bogus python — spawn should fail and surface a clean
        // NetworkError.
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.py"), "pass\n").unwrap();
        let err = compile_bytecode(
            dir.path(),
            Path::new("/nonexistent/python/binary"),
            &BytecodeOptions::default(),
        )
        .unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("spawning python"), "got: {msg}");
    }
}
