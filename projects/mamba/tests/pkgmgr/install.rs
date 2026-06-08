//! CLI integration tests for `mamba install` — locks the tool-install
//! path the validation profile [families.install] points at.
//!
//! Pinned acceptance:
//!
//!   1. `install <pkg> --index <dir>` materializes the tool under
//!      `$MAMBA_TOOLS_DIR/<normalized>/{pkg/<name>.py, bin/<name>,
//!      manifest.toml}`.
//!   2. Re-installing the same version is a structured no-op.
//!   3. `install --list` enumerates installed tools as `<name>==<ver>`
//!      lines, sorted.
//!   4. `install --uninstall <pkg>` removes the tool dir; doing it
//!      twice is a soft no-op.
//!   5. Missing package against a configured frozen index fails with
//!      a clear diagnostic and does NOT create a tools directory.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run(tools_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .env("MAMBA_TOOLS_DIR", tools_dir)
        .output()
        .expect("spawn mamba")
}

fn normalize_pep503(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut prev_sep = false;
    for c in name.chars() {
        let is_sep = c == '-' || c == '_' || c == '.';
        if is_sep {
            if !prev_sep && !out.is_empty() {
                out.push('-');
            }
            prev_sep = true;
        } else {
            out.push(c.to_ascii_lowercase());
            prev_sep = false;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

fn stake_pkg(index: &Path, name: &str, version: &str) {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
    std::fs::write(ver_dir.join("metadata.toml"), "requires = []\n").unwrap();
}

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    stake_pkg(dir.path(), "frozen_demo_pkg", "0.1.0");
    stake_pkg(dir.path(), "frozen_demo_pkg", "0.2.0");
    dir
}

#[test]
fn install_materializes_tool_in_tools_dir() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    let out = run(
        &tools,
        &[
            "install",
            "frozen_demo_pkg",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(
        out.status.success(),
        "install must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let tool_dir = tools.join("frozen-demo-pkg");
    assert!(tool_dir.join("manifest.toml").exists(), "manifest written");
    assert!(
        tool_dir.join("pkg/frozen-demo-pkg.py").exists(),
        "pkg stub written"
    );
    assert!(tool_dir.join("bin/frozen_demo_pkg").exists(), "shim written");

    let manifest = std::fs::read_to_string(tool_dir.join("manifest.toml")).unwrap();
    assert!(manifest.contains("name = \"frozen_demo_pkg\""), "{manifest}");
    assert!(manifest.contains("version = \"0.1.0\""), "{manifest}");
}

#[test]
fn install_default_picks_latest_version_in_index() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    let out = run(
        &tools,
        &[
            "install",
            "frozen_demo_pkg",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    let manifest = std::fs::read_to_string(tools.join("frozen-demo-pkg/manifest.toml")).unwrap();
    assert!(
        manifest.contains("version = \"0.2.0\""),
        "latest version 0.2.0 picked: {manifest}"
    );
}

#[test]
fn install_same_version_twice_is_a_noop() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    assert!(
        run(
            &tools,
            &[
                "install",
                "frozen_demo_pkg",
                "--version",
                "0.1.0",
                "--index",
                index.path().to_str().unwrap(),
            ],
        )
        .status
        .success()
    );

    let out = run(
        &tools,
        &[
            "install",
            "frozen_demo_pkg",
            "--version",
            "0.1.0",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no_op"),
        "second install signals no-op: {stderr:?}"
    );
}

#[test]
fn install_list_shows_installed_tools_sorted() {
    let index = build_index();
    // Two packages to verify sorting.
    let ver_dir = index.path().join("alpha-tool").join("1.0.0");
    std::fs::create_dir_all(&ver_dir).unwrap();
    std::fs::write(ver_dir.join("metadata.toml"), "requires = []\n").unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    assert!(
        run(
            &tools,
            &[
                "install",
                "frozen_demo_pkg",
                "--version",
                "0.1.0",
                "--index",
                index.path().to_str().unwrap(),
            ],
        )
        .status
        .success()
    );
    assert!(
        run(
            &tools,
            &[
                "install",
                "alpha-tool",
                "--version",
                "1.0.0",
                "--index",
                index.path().to_str().unwrap(),
            ],
        )
        .status
        .success()
    );

    let out = run(&tools, &["install", "--list"]);
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines, vec!["alpha-tool==1.0.0", "frozen_demo_pkg==0.1.0"]);
}

#[test]
fn install_uninstall_removes_tool_dir() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    assert!(
        run(
            &tools,
            &[
                "install",
                "frozen_demo_pkg",
                "--version",
                "0.1.0",
                "--index",
                index.path().to_str().unwrap(),
            ],
        )
        .status
        .success()
    );
    let tool_dir = tools.join("frozen-demo-pkg");
    assert!(tool_dir.exists());

    let out = run(&tools, &["install", "--uninstall", "frozen_demo_pkg"]);
    assert!(out.status.success());
    assert!(!tool_dir.exists());

    // Second uninstall is a soft no-op.
    let out2 = run(&tools, &["install", "--uninstall", "frozen_demo_pkg"]);
    assert!(out2.status.success());
    let stderr = String::from_utf8_lossy(&out2.stderr);
    assert!(stderr.contains("no_op"), "uninstall replay no-op: {stderr:?}");
}

#[test]
fn install_missing_pkg_fails_cleanly() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let tools = tmp.path().join("mamba-tools");
    let out = run(
        &tools,
        &[
            "install",
            "package_that_does_not_exist",
            "--index",
            index.path().to_str().unwrap(),
        ],
    );
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("package_that_does_not_exist"),
        "stderr names missing pkg: {stderr:?}"
    );
    assert!(
        !tools.exists() || std::fs::read_dir(&tools).map(|d| d.count()).unwrap_or(0) == 0,
        "no tools created on failure"
    );
}
