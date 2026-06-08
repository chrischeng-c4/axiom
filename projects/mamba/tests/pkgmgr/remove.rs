//! CLI integration tests for `mamba remove` — closes the runtime side of
//! tests/governance/gates/pkgmgr/remove/manifest.toml (#2680).
//!
//! Pinned acceptance:
//!
//!   1. Removed dep no longer appears in mamba.toml.
//!   2. mamba.lock is updated deterministically (byte-identical on replay).
//!   3. Other deps and the project's name/version/python-requires are
//!      preserved.

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

fn init_with_one_dep(workdir: &Path, spec: &str) {
    assert!(run(workdir, &["init"]).status.success());
    assert!(run(workdir, &["add", spec, "--offline"]).status.success());
}

#[test]
fn remove_strips_dep_from_manifest_and_lockfile() {
    let tmp = tempfile::tempdir().unwrap();
    init_with_one_dep(tmp.path(), "frozen_demo_pkg==0.1.0");

    let out = run(tmp.path(), &["remove", "frozen_demo_pkg"]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(
        !manifest.contains("frozen_demo_pkg"),
        "manifest must not contain removed dep: {manifest}"
    );

    let lock = std::fs::read_to_string(tmp.path().join("mamba.lock")).unwrap();
    assert!(
        !lock.contains("frozen_demo_pkg"),
        "lock must not contain removed dep: {lock}"
    );
}

#[test]
fn remove_preserves_other_deps_and_project_fields() {
    let tmp = tempfile::tempdir().unwrap();
    let pdir = tmp.path().join("widget");
    std::fs::create_dir(&pdir).unwrap();
    assert!(run(&pdir, &["init"]).status.success());
    assert!(run(&pdir, &["add", "foo==1.0.0", "--offline"]).status.success());
    assert!(run(&pdir, &["add", "bar==2.0.0", "--offline"]).status.success());

    let out = run(&pdir, &["remove", "foo"]);
    assert!(out.status.success());

    let manifest = std::fs::read_to_string(pdir.join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("name = \"widget\""),
        "project name preserved: {manifest}"
    );
    assert!(
        manifest.contains("version = \"0.1.0\""),
        "project version preserved: {manifest}"
    );
    assert!(
        manifest.contains("python-requires = \">=3.12\""),
        "python-requires preserved: {manifest}"
    );
    assert!(
        manifest.contains("\"bar==2.0.0\""),
        "other dep survives: {manifest}"
    );
    assert!(
        !manifest.contains("\"foo==1.0.0\""),
        "removed dep stripped: {manifest}"
    );
}

#[test]
fn remove_is_byte_identical_on_replay() {
    let tmp = tempfile::tempdir().unwrap();
    let pdir = tmp.path().join("demo");
    std::fs::create_dir(&pdir).unwrap();
    init_with_one_dep(&pdir, "frozen_demo_pkg==0.1.0");

    assert!(
        run(&pdir, &["remove", "frozen_demo_pkg"])
            .status
            .success()
    );
    let m_a = std::fs::read(pdir.join("mamba.toml")).unwrap();
    let l_a = std::fs::read(pdir.join("mamba.lock")).unwrap();

    // Replaying remove against the now-stripped state must be a no-op
    // with byte-identical files.
    assert!(
        run(&pdir, &["remove", "frozen_demo_pkg"])
            .status
            .success()
    );
    let m_b = std::fs::read(pdir.join("mamba.toml")).unwrap();
    let l_b = std::fs::read(pdir.join("mamba.lock")).unwrap();

    assert_eq!(m_a, m_b, "manifest byte-identical on replay");
    assert_eq!(l_a, l_b, "lockfile byte-identical on replay");
}

#[test]
fn remove_unknown_dep_is_a_soft_noop() {
    let tmp = tempfile::tempdir().unwrap();
    assert!(run(tmp.path(), &["init"]).status.success());
    let m_before = std::fs::read(tmp.path().join("mamba.toml")).unwrap();

    let out = run(tmp.path(), &["remove", "not_installed_pkg"]);
    assert!(
        out.status.success(),
        "removing unknown dep is a no-op success"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no-op") || stderr.contains("not recorded"),
        "stderr signals no-op, got: {stderr:?}"
    );

    // Manifest dep list stays empty, so byte-content is stable.
    let m_after = std::fs::read(tmp.path().join("mamba.toml")).unwrap();
    assert_eq!(m_before, m_after);
}

#[test]
fn remove_requires_initialized_project() {
    let tmp = tempfile::tempdir().unwrap();
    let out = run(tmp.path(), &["remove", "foo"]);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("mamba init"), "stderr hints at init: {stderr:?}");
}
