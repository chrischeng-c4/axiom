//! CLI integration tests for `mamba auth`.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_with_dir(creds: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .env("MAMBA_CREDENTIALS_DIR", creds)
        .args(args)
        .output()
        .expect("spawn mamba")
}

#[test]
fn auth_dir_prints_credentials_root_and_service_path() {
    let tmp = tempfile::tempdir().unwrap();
    let root = run_with_dir(tmp.path(), &["auth", "dir"]);
    assert!(
        root.status.success(),
        "auth dir must succeed; stderr: {}",
        String::from_utf8_lossy(&root.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&root.stdout).trim_end(),
        tmp.path().display().to_string()
    );

    let service = run_with_dir(
        tmp.path(),
        &[
            "auth",
            "dir",
            "https://Repo.EXAMPLE/simple",
            "--username",
            "alice",
        ],
    );
    assert!(
        service.status.success(),
        "auth dir service must succeed; stderr: {}",
        String::from_utf8_lossy(&service.stderr)
    );
    let stdout = String::from_utf8_lossy(&service.stdout);
    assert!(
        stdout.contains("repo.example__alice.toml"),
        "stdout: {stdout}"
    );
}

#[test]
fn auth_login_token_logout_round_trip_default_token_user() {
    let tmp = tempfile::tempdir().unwrap();
    let login = run_with_dir(
        tmp.path(),
        &["auth", "login", "repo.example", "--token", "secret-token"],
    );
    assert!(
        login.status.success(),
        "auth login must succeed; stderr: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let token = run_with_dir(tmp.path(), &["auth", "token", "repo.example"]);
    assert!(
        token.status.success(),
        "auth token must succeed; stderr: {}",
        String::from_utf8_lossy(&token.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&token.stdout), "secret-token\n");

    let logout = run_with_dir(tmp.path(), &["auth", "logout", "repo.example"]);
    assert!(
        logout.status.success(),
        "auth logout must succeed; stderr: {}",
        String::from_utf8_lossy(&logout.stderr)
    );

    let missing = run_with_dir(tmp.path(), &["auth", "token", "repo.example"]);
    assert!(!missing.status.success(), "missing token should fail");
}

#[test]
fn auth_login_normalizes_url_and_honors_username() {
    let tmp = tempfile::tempdir().unwrap();
    let login = run_with_dir(
        tmp.path(),
        &[
            "auth",
            "login",
            "https://Repo.EXAMPLE/simple",
            "--username",
            "alice",
            "--token",
            "alice-token",
        ],
    );
    assert!(
        login.status.success(),
        "auth login must succeed; stderr: {}",
        String::from_utf8_lossy(&login.stderr)
    );

    let token = run_with_dir(
        tmp.path(),
        &["auth", "token", "repo.example", "--username", "alice"],
    );
    assert!(
        token.status.success(),
        "auth token must succeed; stderr: {}",
        String::from_utf8_lossy(&token.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&token.stdout), "alice-token\n");
}
