//! CLI integration tests for `mamba python`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_in(dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .current_dir(dir)
        .args(args)
        .output()
        .expect("spawn mamba")
}

#[test]
fn python_pin_writes_python_version_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_in(tmp.path(), &["python", "pin", "3.12"]);
    assert!(
        out.status.success(),
        "python pin must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let body = std::fs::read_to_string(tmp.path().join(".python-version")).unwrap();
    assert_eq!(body, "3.12\n");
}

#[test]
fn python_pin_rejects_any_request() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run_in(tmp.path(), &["python", "pin", "any"]);
    assert!(!out.status.success(), "pinning any must fail");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unconstrained"),
        "stderr explains rejection: {stderr:?}"
    );
}

#[test]
fn python_dir_honors_uv_data_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let data = tmp.path().join("data");
    let out = Command::new(mamba_bin())
        .args(["python", "dir"])
        .env("UV_DATA_DIR", &data)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "python dir must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        data.join("python").to_string_lossy()
    );
}

#[test]
fn python_list_succeeds_when_path_has_no_interpreters() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_path = tmp.path().join("empty");
    std::fs::create_dir_all(&empty_path).unwrap();
    let out = Command::new(mamba_bin())
        .args(["python", "list"])
        .env("PATH", &empty_path)
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "python list must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

#[test]
fn python_find_fails_cleanly_when_no_interpreter_matches() {
    let tmp = tempfile::tempdir().unwrap();
    let empty_path = tmp.path().join("empty");
    std::fs::create_dir_all(&empty_path).unwrap();
    std::fs::write(tmp.path().join(".python-version"), "3.12\n").unwrap();
    let out = Command::new(mamba_bin())
        .current_dir(tmp.path())
        .args(["python", "find"])
        .env("PATH", &empty_path)
        .output()
        .expect("spawn mamba");
    assert!(!out.status.success(), "find must fail without a match");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no installed Python matches 3.12"),
        "stderr names missing request: {stderr:?}"
    );
}
