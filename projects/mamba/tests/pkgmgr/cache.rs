//! CLI integration tests for `mamba cache` — closes the runtime side
//! of tests/governance/gates/pkgmgr/cache/manifest.toml (#2685).
//!
//! Pinned acceptance:
//!
//!   1. `$MAMBA_CACHE_DIR` overrides the platform default so the gate
//!      can isolate the cache under a tempdir.
//!   2. `cache dir` prints the resolved cache root on its own line.
//!   3. `cache clean` removes entries under the root, keeps the root
//!      itself, and is idempotent (no-op when root is missing).
//!   4. `cache size` reports exact bytes and `cache prune` supports dry-run
//!      and package-targeted removal.

use std::path::{Path, PathBuf};
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

fn run_with_cache(cache_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .env("MAMBA_CACHE_DIR", cache_dir)
        .output()
        .expect("spawn mamba")
}

fn run_with_empty_env(cache_dir: &Path, args: &[&str]) -> std::process::Output {
    Command::new(mamba_bin())
        .args(args)
        .env("MAMBA_CACHE_DIR", cache_dir)
        .env_remove("XDG_CACHE_HOME")
        .output()
        .expect("spawn mamba")
}

#[test]
fn cache_dir_prints_resolved_root() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    let out = run_with_empty_env(&cache_root, &["cache", "dir"]);
    assert!(
        out.status.success(),
        "cache dir must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let printed = stdout.trim_end();
    assert_eq!(
        printed,
        cache_root.to_string_lossy().as_ref(),
        "cache dir output must match MAMBA_CACHE_DIR"
    );
}

#[test]
fn cache_clean_removes_contents_but_keeps_root() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    std::fs::write(cache_root.join("blob.bin"), b"some bytes").unwrap();
    std::fs::create_dir_all(cache_root.join("sub")).unwrap();
    std::fs::write(cache_root.join("sub/inner"), b"x").unwrap();

    let out = run_with_cache(&cache_root, &["cache", "clean"]);
    assert!(
        out.status.success(),
        "cache clean must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(cache_root.exists(), "root preserved");
    assert!(!cache_root.join("blob.bin").exists(), "blob removed");
    assert!(!cache_root.join("sub").exists(), "sub removed");
    let remaining: Vec<_> = std::fs::read_dir(&cache_root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert!(remaining.is_empty(), "cache root empty after clean");
}

#[test]
fn cache_size_prints_total_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(cache_root.join("artifacts/demo")).unwrap();
    std::fs::write(cache_root.join("artifacts/demo/blob.whl"), b"hello").unwrap();
    std::fs::create_dir_all(cache_root.join("content/ab")).unwrap();
    std::fs::write(cache_root.join("content/ab/hash"), b"xyz").unwrap();

    let out = run_with_cache(&cache_root, &["cache", "size"]);
    assert!(
        out.status.success(),
        "cache size must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "8 bytes");
}

#[test]
fn cache_clean_on_missing_root_is_no_op() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("does-not-exist");
    let out = run_with_cache(&cache_root, &["cache", "clean"]);
    assert!(
        out.status.success(),
        "clean on missing root must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no_op"), "stderr signals no-op: {stderr:?}");
}

#[test]
fn cache_prune_dry_run_keeps_selected_entries() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(cache_root.join("artifacts/demo")).unwrap();
    std::fs::write(cache_root.join("artifacts/demo/blob.whl"), b"some bytes").unwrap();

    let out = run_with_cache(
        &cache_root,
        &["cache", "prune", "--package", "demo", "--dry-run"],
    );
    assert!(
        out.status.success(),
        "cache prune must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        cache_root.join("artifacts/demo/blob.whl").exists(),
        "dry-run must not remove selected file"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("would_prune") && stderr.contains("10 bytes selected"),
        "dry-run stderr reports selected bytes: {stderr:?}"
    );
}

#[test]
fn cache_prune_package_removes_matching_artifact_only() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(cache_root.join("artifacts/demo")).unwrap();
    std::fs::create_dir_all(cache_root.join("artifacts/other")).unwrap();
    std::fs::write(cache_root.join("artifacts/demo/blob.whl"), b"demo").unwrap();
    std::fs::write(cache_root.join("artifacts/other/blob.whl"), b"other").unwrap();

    let out = run_with_cache(&cache_root, &["cache", "prune", "--package", "demo"]);
    assert!(
        out.status.success(),
        "cache prune must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !cache_root.join("artifacts/demo/blob.whl").exists(),
        "matching artifact removed"
    );
    assert!(
        cache_root.join("artifacts/other/blob.whl").exists(),
        "non-matching artifact preserved"
    );
}

#[test]
fn cache_without_subcommand_fails() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    let out = run_with_cache(&cache_root, &["cache"]);
    assert!(
        !out.status.success(),
        "bare `cache` must require subcommand"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("dir")
            && stderr.contains("size")
            && stderr.contains("clean")
            && stderr.contains("prune"),
        "stderr lists subcommands: {stderr:?}"
    );
}
