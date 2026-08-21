//! CLI integration tests for `mamba export`.

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

fn stake_pkg(index: &Path, name: &str, version: &str, requires: &[&str]) {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
    let meta = if requires.is_empty() {
        "requires = []\n".to_string()
    } else {
        let arr = requires
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect::<Vec<_>>()
            .join(", ");
        format!("requires = [{arr}]\n")
    };
    std::fs::write(ver_dir.join("metadata.toml"), meta).unwrap();
}

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    stake_pkg(
        dir.path(),
        "frozen_demo_pkg",
        "0.1.0",
        &["frozen_demo_transitive==0.2.0"],
    );
    stake_pkg(dir.path(), "frozen_demo_transitive", "0.2.0", &[]);
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
fn export_requirements_txt_from_lockfile() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let out = run(
        &proj,
        &["export", "--no-header", "--no-hashes", "--annotate"],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("frozen_demo_pkg==0.1.0"), "{stdout}");
    assert!(stdout.contains("frozen_demo_transitive==0.2.0"), "{stdout}");
    assert!(stdout.contains("# via frozen_demo_pkg"), "{stdout}");
}

#[test]
fn export_requirements_respects_exclusion_and_output_file() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());

    let dest = proj.join("requirements.txt");
    let out = run(
        &proj,
        &[
            "export",
            "--no-header",
            "--no-hashes",
            "--no-emit-package",
            "frozen_demo_transitive",
            "--output-file",
            dest.to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    assert_eq!(out.stdout, b"");
    let body = std::fs::read_to_string(dest).unwrap();
    assert!(body.contains("frozen_demo_pkg==0.1.0"), "{body}");
    assert!(!body.contains("frozen_demo_transitive"), "{body}");
}

#[test]
fn export_pylock_uses_artifact_url_when_lockfile_has_one() {
    let tmp = tempfile::tempdir().unwrap();
    let lock = r#"
format_version = 1
input_hash = "x"

[[package]]
name = "demo"
version = "1.0.0"
sha256 = "abc123"
url = "https://example.test/demo-1.0.0-py3-none-any.whl"
source = "pypi://demo/1.0.0"
dependencies = []
"#;
    std::fs::write(tmp.path().join("mamba.lock"), lock).unwrap();

    let out = run(
        tmp.path(),
        &[
            "export",
            "--format",
            "pylock.toml",
            "--requires-python",
            ">=3.12",
            "--environment",
            "sys_platform == 'darwin'",
        ],
    );
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("requires-python = \">=3.12\""), "{stdout}");
    assert!(stdout.contains("sys_platform == 'darwin'"), "{stdout}");
    assert!(stdout.contains("wheels = ["), "{stdout}");
    assert!(
        stdout.contains("url = \"https://example.test/demo-1.0.0-py3-none-any.whl\""),
        "{stdout}"
    );
    assert!(stdout.contains("sha256 = \"abc123\""), "{stdout}");
}
