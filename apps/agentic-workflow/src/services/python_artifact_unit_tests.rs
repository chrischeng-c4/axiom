//! Authored unit-test gate for canonical Python EC and TD projects.
//!
//! EC, TD, and generated CB artifacts own separate unit inventories. This
//! runner is intentionally small and CPython-native: the authored artifact
//! must expose at least one `tests/unit/test_*.py` file and the complete
//! inventory must pass through stdlib `unittest` discovery.

use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{path::Path, process::Command};
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PythonArtifactUnitTestReport {
    pub root: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub stdout: String,
    pub stderr: String,
}

pub fn run_authored_unit_tests(project_root: &Path) -> Result<PythonArtifactUnitTestReport> {
    run_authored_unit_tests_with_uv(project_root, Path::new("uv"))
}

fn run_authored_unit_tests_with_uv(
    project_root: &Path,
    uv_executable: &Path,
) -> Result<PythonArtifactUnitTestReport> {
    let root = project_root.canonicalize().with_context(|| {
        format!(
            "canonicalize Python artifact root {}",
            project_root.display()
        )
    })?;
    let unit_root = root.join("tests/unit");
    let files = discover_unit_test_files(&root, &unit_root)?;
    if files.is_empty() {
        bail!(
            "Python artifact has no authored unit tests; add at least one `tests/unit/test_*.py` file below {}",
            root.display()
        );
    }

    crate::services::python_artifact::ensure_uv_lock_current(&root, uv_executable)?;
    let output = Command::new(uv_executable)
        .args([
            "run",
            "--frozen",
            "--offline",
            "python",
            "-B",
            "-m",
            "unittest",
            "discover",
            "-s",
            "tests/unit",
            "-p",
            "test_*.py",
        ])
        .current_dir(&root)
        .output()
        .with_context(|| {
            format!(
                "run authored Python unit tests below {}",
                unit_root.display()
            )
        })?;
    let stdout = String::from_utf8(output.stdout)
        .context("authored Python unit-test stdout was not UTF-8")?;
    let stderr = String::from_utf8(output.stderr)
        .context("authored Python unit-test stderr was not UTF-8")?;
    if !output.status.success() {
        bail!(
            "authored Python unit tests failed below {}: exit={}; stdout={}; stderr={}",
            unit_root.display(),
            output
                .status
                .code()
                .map_or_else(|| "signal".to_string(), |code| code.to_string()),
            stdout.trim(),
            stderr.trim()
        );
    }

    Ok(PythonArtifactUnitTestReport {
        root: ".".to_string(),
        file_count: files.len(),
        files,
        stdout,
        stderr,
    })
}

fn discover_unit_test_files(root: &Path, unit_root: &Path) -> Result<Vec<String>> {
    if !unit_root.is_dir() {
        return Ok(Vec::new());
    }
    let mut files = WalkDir::new(unit_root)
        .follow_links(false)
        .into_iter()
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| format!("walk authored Python unit tests {}", unit_root.display()))?
        .into_iter()
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .file_name()
                .to_str()
                .is_some_and(|name| name.starts_with("test_") && name.ends_with(".py"))
        })
        .map(|entry| {
            entry
                .path()
                .strip_prefix(root)
                .unwrap_or(entry.path())
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect::<Vec<_>>();
    files.sort();
    files.dedup();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs, path::PathBuf};
    use tempfile::tempdir;

    fn write(path: &Path, content: &str) {
        fs::create_dir_all(path.parent().expect("test file has parent")).unwrap();
        fs::write(path, content).unwrap();
    }

    fn fake_uv(root: &Path) -> PathBuf {
        let path = root.join("fake-uv");
        write(
            &path,
            r#"#!/bin/sh
if [ "$1" = "lock" ]; then
  exit 0
fi
if [ "$1" = "run" ]; then
  shift
  shift
  shift
  exec "$@"
fi
exit 64
"#,
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }
        path
    }

    fn write_environment(root: &Path) {
        write(
            &root.join("pyproject.toml"),
            "[project]\nname = \"fixture\"\nversion = \"0.0.0\"\nrequires-python = \">=3.11\"\n",
        );
        write(&root.join("uv.lock"), "version = 1\nrevision = 3\n");
    }

    #[test]
    fn missing_inventory_fails_with_exact_remediation() {
        let temp = tempdir().unwrap();
        let error = run_authored_unit_tests(temp.path()).unwrap_err();
        assert!(error
            .to_string()
            .contains("add at least one `tests/unit/test_*.py`"));
    }

    #[test]
    fn passing_inventory_reports_discovered_files() {
        let temp = tempdir().unwrap();
        write_environment(temp.path());
        write(
            &temp.path().join("tests/unit/test_contract.py"),
            "import unittest\n\nclass ContractTest(unittest.TestCase):\n    def test_true(self):\n        self.assertTrue(True)\n",
        );
        let report = run_authored_unit_tests_with_uv(temp.path(), &fake_uv(temp.path())).unwrap();
        assert_eq!(report.file_count, 1);
        assert_eq!(report.files, vec!["tests/unit/test_contract.py"]);
    }

    #[test]
    fn failing_inventory_fails_closed() {
        let temp = tempdir().unwrap();
        write_environment(temp.path());
        write(
            &temp.path().join("tests/unit/test_contract.py"),
            "import unittest\n\nclass ContractTest(unittest.TestCase):\n    def test_false(self):\n        self.assertTrue(False)\n",
        );
        let error =
            run_authored_unit_tests_with_uv(temp.path(), &fake_uv(temp.path())).unwrap_err();
        assert!(error
            .to_string()
            .contains("authored Python unit tests failed"));
    }

    #[test]
    fn missing_uv_lock_fails_with_exact_remediation() {
        let temp = tempdir().unwrap();
        write(
            &temp.path().join("pyproject.toml"),
            "[project]\nname = \"fixture\"\nversion = \"0.0.0\"\n",
        );
        write(
            &temp.path().join("tests/unit/test_contract.py"),
            "import unittest\n\nclass ContractTest(unittest.TestCase):\n    def test_true(self):\n        self.assertTrue(True)\n",
        );

        let error = run_authored_unit_tests_with_uv(temp.path(), &fake_uv(temp.path()))
            .unwrap_err()
            .to_string();
        assert!(error.contains("requires uv.lock"), "{error}");
        assert!(error.contains("uv lock --project"), "{error}");
    }
}
