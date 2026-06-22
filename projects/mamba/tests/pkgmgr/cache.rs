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
//!   4. `cache prune` is currently equivalent to `clean`.

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
fn cache_prune_behaves_like_clean_for_now() {
    let tmp = tempfile::tempdir().unwrap();
    let cache_root = tmp.path().join("mamba-cache");
    std::fs::create_dir_all(&cache_root).unwrap();
    std::fs::write(cache_root.join("blob.bin"), b"some bytes").unwrap();

    let out = run_with_cache(&cache_root, &["cache", "prune"]);
    assert!(
        out.status.success(),
        "cache prune must succeed; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(cache_root.exists(), "root preserved");
    assert!(!cache_root.join("blob.bin").exists(), "blob removed");
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
        stderr.contains("dir") && stderr.contains("clean") && stderr.contains("prune"),
        "stderr lists subcommands: {stderr:?}"
    );
}
