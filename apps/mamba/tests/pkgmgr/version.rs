//! CLI integration tests for `mamba version`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .current_dir(dir)
        .output()
        .expect("spawn mamba")
}

fn write_pyproject(dir: &Path, version: &str) {
    std::fs::write(
        dir.join("pyproject.toml"),
        format!("[project]\nname = \"demo\"\nversion = \"{version}\"\ndescription = \"x\"\n"),
    )
    .unwrap();
}

#[test]
fn version_prints_current_project_version() {
    let tmp = tempfile::tempdir().unwrap();
    write_pyproject(tmp.path(), "1.2.3");

    let out = run(tmp.path(), &["version"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.2.3\n");
}

#[test]
fn version_bump_patch_updates_pyproject() {
    let tmp = tempfile::tempdir().unwrap();
    write_pyproject(tmp.path(), "1.2.3");

    let out = run(tmp.path(), &["version", "--bump", "patch"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "1.2.4\n");
    let body = std::fs::read_to_string(tmp.path().join("pyproject.toml")).unwrap();
    assert!(body.contains("version = \"1.2.4\""), "{body}");
    assert!(body.contains("description = \"x\""), "{body}");
}

#[test]
fn version_dry_run_does_not_mutate_file() {
    let tmp = tempfile::tempdir().unwrap();
    write_pyproject(tmp.path(), "1.2.3");
    let before = std::fs::read_to_string(tmp.path().join("pyproject.toml")).unwrap();

    let out = run(tmp.path(), &["version", "2.0.0", "--dry-run"]);
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout), "2.0.0\n");
    let after = std::fs::read_to_string(tmp.path().join("pyproject.toml")).unwrap();
    assert_eq!(before, after);
}

#[test]
fn version_rejects_missing_pyproject() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["version"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("pyproject.toml"), "{stderr}");
}
