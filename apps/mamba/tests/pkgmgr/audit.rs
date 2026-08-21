//! CLI integration tests for `mamba audit`.

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

fn write_lock(project: &Path, name: &str, version: &str) {
    let lock = format!(
        "format_version = 1\ninput_hash = \"x\"\n\n[[package]]\nname = \"{name}\"\nversion = \"{version}\"\nsha256 = \"\"\nurl = \"\"\nsource = \"pypi://{name}/{version}\"\ndirect = true\ndependencies = []\n"
    );
    std::fs::write(project.join("mamba.lock"), lock).unwrap();
}

#[test]
fn audit_passes_when_advisory_db_has_no_matching_vulnerabilities() {
    let tmp = tempfile::tempdir().unwrap();
    write_lock(tmp.path(), "safe-pkg", "1.0.0");
    let db = tmp.path().join("advisories.json");
    std::fs::write(
        &db,
        r#"{"advisories":[{"id":"GHSA-demo","package":"other","affected":["<9"],"severity":"low"}]}"#,
    )
    .unwrap();

    let out = run(
        tmp.path(),
        &["audit", "--advisory-db", db.to_str().unwrap()],
    );
    assert!(
        out.status.success(),
        "audit must pass; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("No vulnerabilities found"),
        "stdout names clean audit: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

#[test]
fn audit_reports_vulnerability_as_json_and_exits_nonzero() {
    let tmp = tempfile::tempdir().unwrap();
    write_lock(tmp.path(), "demo_pkg", "1.2.0");
    let db = tmp.path().join("advisories.json");
    std::fs::write(
        &db,
        r#"{"advisories":[{"id":"GHSA-demo-1234","package":"demo-pkg","affected":[">=1.0,<1.3"],"severity":"high","summary":"demo vulnerable range","url":"https://example.invalid/GHSA-demo-1234"}]}"#,
    )
    .unwrap();

    let out = run(
        tmp.path(),
        &["audit", "--json", "--advisory-db", db.to_str().unwrap()],
    );
    assert!(!out.status.success(), "vulnerable audit must fail");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("GHSA-demo-1234"),
        "json names advisory: {stdout}"
    );
    assert!(
        stdout.contains("\"package\": \"demo_pkg\""),
        "json names package: {stdout}"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("vulnerable package"),
        "stderr summarizes failure: {stderr:?}"
    );
}
