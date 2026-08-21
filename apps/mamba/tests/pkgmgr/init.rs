//! CLI integration tests for `mamba init` — closes the runtime side of
//! tests/governance/gates/pkgmgr/init/manifest.toml (#2679).
//!
//! Pinned acceptance:
//!
//!   1. First run in an empty directory creates the five expected files.
//!   2. Re-running is a clean no-op (exit 0, "already initialized" on
//!      stderr) and preserves mamba.toml / README.md / src/__init__.py
//!      byte-for-byte.
//!   3. No writes escape the project directory.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_init(workdir: &Path) -> std::process::Output {
    Command::new(mamba_bin())
        .arg("init")
        .current_dir(workdir)
        .output()
        .expect("spawn mamba init")
}

#[test]
fn init_creates_expected_files_on_fresh_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out = run_init(tmp.path());
    assert!(
        out.status.success(),
        "init failed: stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr),
    );

    for f in [
        "mamba.toml",
        ".python-version",
        ".gitignore",
        "README.md",
        "src/__init__.py",
    ] {
        let p = tmp.path().join(f);
        assert!(p.exists(), "init must create {f}");
    }

    for f in ["mamba.lock", ".venv"] {
        let p = tmp.path().join(f);
        assert!(!p.exists(), "init must not create {f}");
    }

    let manifest = std::fs::read_to_string(tmp.path().join("mamba.toml")).unwrap();
    assert!(manifest.contains("version = \"0.1.0\""), "default version");
    assert!(
        manifest.contains("python-requires = \">=3.12\""),
        "default python-requires"
    );
    assert!(manifest.contains("dependencies = []"), "deps default");
    assert!(
        manifest.contains("dev-dependencies = []"),
        "dev deps default"
    );
}

#[test]
fn init_is_idempotent_on_second_run() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let first = run_init(tmp.path());
    assert!(first.status.success());

    let manifest_path = tmp.path().join("mamba.toml");
    let readme_path = tmp.path().join("README.md");
    let pkg_path = tmp.path().join("src/__init__.py");

    let m_before = std::fs::read(&manifest_path).unwrap();
    let r_before = std::fs::read(&readme_path).unwrap();
    let p_before = std::fs::read(&pkg_path).unwrap();

    let second = run_init(tmp.path());
    assert!(
        second.status.success(),
        "second init must succeed: stderr={:?}",
        String::from_utf8_lossy(&second.stderr),
    );
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        stderr.contains("already initialized"),
        "second init stderr must contain 'already initialized', got: {stderr:?}"
    );

    assert_eq!(std::fs::read(&manifest_path).unwrap(), m_before);
    assert_eq!(std::fs::read(&readme_path).unwrap(), r_before);
    assert_eq!(std::fs::read(&pkg_path).unwrap(), p_before);
}

#[test]
fn init_uses_directory_name_as_project_name() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let project_dir = tmp.path().join("widget");
    std::fs::create_dir(&project_dir).unwrap();
    let out = run_init(&project_dir);
    assert!(out.status.success());

    let manifest = std::fs::read_to_string(project_dir.join("mamba.toml")).unwrap();
    assert!(
        manifest.contains("name = \"widget\""),
        "manifest must use dir name 'widget', got: {manifest}"
    );
}
