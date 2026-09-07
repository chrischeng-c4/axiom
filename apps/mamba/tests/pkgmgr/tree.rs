//! CLI integration tests for `mamba tree`.

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

use crate::fixtures::fixture_pkg;

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    fixture_pkg(
        dir.path(),
        "frozen_demo_pkg",
        "0.1.0",
        &["frozen_demo_transitive==0.2.0"],
    );
    fixture_pkg(dir.path(), "frozen_demo_transitive", "0.2.0", &[]);
    dir
}

fn setup_locked_project(proj: &Path, index: &Path) {
    assert!(run(proj, &["init"]).status.success());
    assert!(run(
        proj,
        &[
            "add",
            "frozen_demo_pkg==0.1.0",
            "--index",
            index.to_str().unwrap()
        ]
    )
    .status
    .success());
    assert!(run(proj, &["lock", "--index", index.to_str().unwrap()])
        .status
        .success());
}

#[test]
fn tree_renders_forward_dependency_graph() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let out = run(&proj, &["tree"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("frozen_demo_pkg v0.1.0"), "{stdout}");
    assert!(stdout.contains("frozen_demo_transitive v0.2.0"), "{stdout}");
}

#[test]
fn tree_supports_focus_invert_and_depth() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let focus = run(
        &proj,
        &["tree", "--package", "frozen_demo_pkg", "--depth", "0"],
    );
    assert!(focus.status.success());
    let focus_stdout = String::from_utf8_lossy(&focus.stdout);
    assert!(
        focus_stdout.contains("frozen_demo_pkg v0.1.0"),
        "{focus_stdout}"
    );
    assert!(
        !focus_stdout.contains("frozen_demo_transitive"),
        "{focus_stdout}"
    );

    let inverted = run(
        &proj,
        &["tree", "--package", "frozen_demo_transitive", "--invert"],
    );
    assert!(inverted.status.success());
    let inverted_stdout = String::from_utf8_lossy(&inverted.stdout);
    assert!(
        inverted_stdout.contains("frozen_demo_transitive v0.2.0"),
        "{inverted_stdout}"
    );
    assert!(
        inverted_stdout.contains("frozen_demo_pkg v0.1.0"),
        "{inverted_stdout}"
    );
}

#[test]
fn tree_without_lockfile_fails_cleanly() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["tree"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mamba.lock"), "{stderr}");
}
