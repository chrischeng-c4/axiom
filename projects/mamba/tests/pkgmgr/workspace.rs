//! CLI integration tests for `mamba workspace`.

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
fn workspace_list_discovers_members_and_excludes_matches() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("pyproject.toml"),
        r#"
[tool.uv.workspace]
members = ["packages/*"]
exclude = ["packages/skip"]
"#,
    )
    .unwrap();
    write_member(tmp.path(), "packages/alpha", "Alpha_Pkg", "0.1.0");
    write_member(tmp.path(), "packages/skip", "skip", "9.9.9");

    let out = run_in(tmp.path(), &["workspace", "list"]);
    assert!(
        out.status.success(),
        "workspace list must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alpha-pkg==0.1.0"), "stdout: {stdout}");
    assert!(!stdout.contains("skip==9.9.9"), "stdout: {stdout}");
}

#[test]
fn workspace_list_json_includes_member_paths() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("pyproject.toml"),
        r#"
[tool.uv.workspace]
members = ["packages/*"]
"#,
    )
    .unwrap();
    write_member(tmp.path(), "packages/beta", "beta", "0.2.0");

    let out = run_in(tmp.path(), &["workspace", "list", "--json"]);
    assert!(
        out.status.success(),
        "workspace list --json must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"name\": \"beta\""), "stdout: {stdout}");
    assert!(
        stdout.contains("packages/beta/pyproject.toml"),
        "stdout: {stdout}"
    );
}

#[test]
fn workspace_list_returns_empty_for_non_workspace_project() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("pyproject.toml"),
        "[project]\nname = \"solo\"\nversion = \"1.0.0\"\n",
    )
    .unwrap();

    let out = run_in(tmp.path(), &["workspace", "list"]);
    assert!(
        out.status.success(),
        "non-workspace list must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout), "");
}

fn write_member(root: &Path, rel: &str, name: &str, version: &str) {
    let dir = root.join(rel);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("pyproject.toml"),
        format!("[project]\nname = {name:?}\nversion = {version:?}\n"),
    )
    .unwrap();
}
