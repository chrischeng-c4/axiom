//! CLI integration tests for the `mamba run` package-manager
//! preflight — closes the runtime side of
//! tests/governance/gates/pkgmgr/run/manifest.toml (#2684).
//!
//! Pinned acceptance:
//!
//!   1. `mamba run <file>` BEFORE `mamba sync` (with a populated
//!      lockfile) fails exit 1 with stderr "environment is not synced".
//!   2. `mamba run <file>` AFTER `mamba sync` proceeds past the
//!      preflight (it may then succeed or fail for unrelated compile
//!      reasons, but never with "environment is not synced").
//!   3. `mamba run <file>` outside a mamba project (no mamba.toml)
//!      proceeds straight through legacy mode.

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

fn stake_pkg(index: &Path, name: &str, version: &str) {
    let ver_dir = index.join(normalize_pep503(name)).join(version);
    std::fs::create_dir_all(&ver_dir).unwrap();
    std::fs::write(ver_dir.join("metadata.toml"), "requires = []\n").unwrap();
}

fn build_index() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    stake_pkg(dir.path(), "frozen_demo_pkg", "0.1.0");
    dir
}

fn setup_locked_project(proj: &Path, index: &Path) {
    assert!(run(proj, &["init"]).status.success());
    assert!(
        run(
            proj,
            &["add", "frozen_demo_pkg==0.1.0", "--index", index.to_str().unwrap()]
        )
        .status
        .success()
    );
    assert!(
        run(proj, &["lock", "--index", index.to_str().unwrap()])
            .status
            .success()
    );
}

fn write_trivial_script(proj: &Path) -> PathBuf {
    let scripts = proj.join("scripts");
    std::fs::create_dir_all(&scripts).unwrap();
    let path = scripts.join("hello.py");
    std::fs::write(&path, "print('frozen-demo-sentinel:OK')\n").unwrap();
    path
}

#[test]
fn run_before_sync_fails_with_environment_is_not_synced() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());
    write_trivial_script(&proj);

    let out = run(&proj, &["run", "scripts/hello.py"]);
    assert!(
        !out.status.success(),
        "run before sync must fail; stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("environment is not synced"),
        "stderr must signal env-not-synced, got: {stderr:?}"
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        !stdout.contains("frozen-demo-sentinel:OK"),
        "sentinel must NOT print before sync; stdout: {stdout:?}"
    );
    assert!(
        !proj.join(".venv").exists(),
        "preflight failure must not create .venv"
    );
}

#[test]
fn run_after_sync_clears_preflight() {
    let index = build_index();
    let tmp = tempfile::tempdir().unwrap();
    let proj = tmp.path().join("demo");
    std::fs::create_dir(&proj).unwrap();
    setup_locked_project(&proj, index.path());
    write_trivial_script(&proj);

    assert!(run(&proj, &["sync"]).status.success());

    let out = run(&proj, &["run", "scripts/hello.py"]);
    // The compiler stage may or may not succeed depending on what
    // mamba supports for the trivial script today — the contract we
    // own here is that the preflight no longer fires.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("environment is not synced"),
        "post-sync run must not trip the preflight, got: {stderr:?}"
    );
}

#[test]
fn run_outside_mamba_project_is_legacy() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(
        tmp.path().join("solo.py"),
        "print('frozen-demo-sentinel:OK')\n",
    )
    .unwrap();

    let out = run(tmp.path(), &["run", "solo.py"]);
    // No mamba.toml here, so preflight is bypassed entirely. The
    // command must NOT fail with the env-not-synced diagnostic.
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("environment is not synced"),
        "legacy run must skip the preflight, got: {stderr:?}"
    );
}
