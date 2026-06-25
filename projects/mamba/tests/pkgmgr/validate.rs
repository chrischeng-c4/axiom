//! Integration test for `mamba pkgmgr-validate` — drives the
//! release-blocking package-manager workflow families end-to-end.
//!
//! Closes Tick 10 of the uv-style package-management ramp:
//! validation/profiles/package_manager.toml [families.*] is the
//! contract; this test pins that all required families pass
//! against the in-repo CLI.

use std::path::PathBuf;
use std::process::Command;

fn mamba_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_mamba"))
}

#[test]
fn pkgmgr_validate_human_reports_all_families_pass() {
    let out = Command::new(mamba_bin())
        .args(["pkgmgr-validate"])
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "validator must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    for fam in [
        "init", "index", "add", "lock", "export", "tree", "version", "pip", "sync", "run",
        "install", "hash", "cache",
    ] {
        assert!(
            stderr.contains(&format!("[pass] {fam}")),
            "stderr must include `[pass] {fam}`: {stderr}"
        );
    }
    assert!(
        stderr.contains("13 passed, 0 failed"),
        "summary must report 13 passed: {stderr}"
    );
}

#[test]
fn pkgmgr_validate_json_has_runner_contract_shape() {
    let out = Command::new(mamba_bin())
        .args(["pkgmgr-validate", "--json"])
        .output()
        .expect("spawn mamba");
    assert!(
        out.status.success(),
        "validator must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    // The runner_contract per family pins keys: passed / failed /
    // missing / fixtures (per validation/profiles/package_manager.toml).
    // The summary block also names project_path / lockfile_path /
    // environment_path. Probe presence rather than structural equality
    // so the test stays portable.
    for key in [
        "\"include_live_network\"",
        "\"families\"",
        "\"outcome\"",
        "\"passed\"",
        "\"failed\"",
        "\"missing\"",
        "\"fixtures\"",
        "\"project_path\"",
        "\"lockfile_path\"",
        "\"environment_path\"",
    ] {
        assert!(stdout.contains(key), "json missing {key}: {stdout}");
    }
    for fam in [
        "init", "index", "add", "lock", "export", "tree", "version", "pip", "sync", "run",
        "install", "hash", "cache",
    ] {
        assert!(
            stdout.contains(&format!("\"{fam}\":")),
            "json missing family {fam}: {stdout}"
        );
    }
}

#[test]
fn pkgmgr_validate_include_live_network_flag_is_propagated() {
    let out = Command::new(mamba_bin())
        .args(["pkgmgr-validate", "--include-live-network", "--json"])
        .output()
        .expect("spawn mamba");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"include_live_network\": true"),
        "live-network flag must surface in JSON: {stdout}"
    );
}
