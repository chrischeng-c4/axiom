//! CLI integration tests for `mamba venv`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .output()
        .expect("spawn mamba")
}

#[test]
fn venv_create_refuses_existing_pyvenv_cfg_before_spawning_python() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("v");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join("pyvenv.cfg"),
        "home = /tmp\ninclude-system-site-packages = false\nversion = 3.12.0\n",
    )
    .unwrap();
    std::fs::write(root.join("marker.txt"), b"keep").unwrap();

    let out = run(&[
        "venv",
        "create",
        path_str(&root),
        "--python",
        "/definitely/missing/python",
    ]);
    assert!(!out.status.success(), "existing venv must be refused");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("refused_existing_pyvenv_cfg"),
        "stderr gives refusal reason: {stderr:?}"
    );
    assert!(
        root.join("marker.txt").exists(),
        "refusal must not mutate existing tree"
    );
}

#[test]
fn venv_remove_deletes_tree_only_when_pyvenv_cfg_exists() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("v");
    std::fs::create_dir_all(root.join("bin")).unwrap();
    std::fs::write(
        root.join("pyvenv.cfg"),
        "home = /tmp\ninclude-system-site-packages = false\nversion = 3.12.0\n",
    )
    .unwrap();

    let out = run(&["venv", "remove", path_str(&root)]);
    assert!(
        out.status.success(),
        "venv remove must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!root.exists(), "venv tree removed");
}

#[test]
fn venv_remove_refuses_non_venv_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("not-a-venv");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("data.txt"), b"keep").unwrap();

    let out = run(&["venv", "remove", path_str(&root)]);
    assert!(!out.status.success(), "non-venv directory must be refused");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("refused_no_pyvenv_cfg"),
        "stderr gives refusal reason: {stderr:?}"
    );
    assert!(
        root.join("data.txt").exists(),
        "refusal must not delete non-venv content"
    );
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("temp path is utf-8")
}
