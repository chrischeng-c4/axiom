---
id: vat-source-projects-vat-tests-vat-toml-runner-rs
summary: Source replay payload for projects/vat/tests/vat_toml_runner.rs
fill_sections: [overview, source, changes]
capability_refs:
  - id: agent-native-gpu-native-dev-containers
    role: primary
    gap: agent-legible-state-and-diff-surface
    claim: agent-legible-state-and-diff-surface
    coverage: full
    rationale: "This source replay TD preserves vat.toml runner evidence, local service orchestration, and agent-legible run state."
---

# Source TD: projects/vat/tests/vat_toml_runner.rs

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/vat/tests/vat_toml_runner.rs` generated from AST during Score force-regeneration standardization.

### Symbols

No public AST symbols.
## Source
<!-- type: source lang: rust -->

`````rust
use std::net::TcpListener;
use std::process::Command;

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn python3_available() -> bool {
    Command::new("python3")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn free_port() -> Option<u16> {
    let listener = TcpListener::bind("127.0.0.1:0").ok()?;
    Some(listener.local_addr().ok()?.port())
}

#[test]
fn vat_toml_runner_starts_service_and_returns_json_evidence() {
    if !python3_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let Some(port) = free_port() else {
        return;
    };
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
name = "smoke"

[workspace]
base = "."
workdir = "."
keep = "failed"

[env]
VAT_TEST_MODE = "runner"

[[services]]
id = "web"
cmd = ["python3", "-m", "http.server", "{port}", "--bind", "127.0.0.1"]
ready_http = "http://127.0.0.1:{port}/"
timeout_s = 10

[[runners]]
id = "e2e"
requires = ["web"]
cmd = ["sh", "-c", "echo ok > runner-artifact.txt"]
artifacts = ["runner-artifact.txt"]
"#
        ),
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "e2e", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["test_run"]["runner_id"], "e2e");
    assert_eq!(json["test_run"]["runner"]["exit_code"], 0);
    assert_eq!(json["test_run"]["services"][0]["status"], "exited");
    assert_eq!(
        json["test_run"]["artifacts"][0]["path"],
        "runner-artifact.txt"
    );
    let id = json["id"].as_str().unwrap();
    assert!(
        !vat_home.path().join("vats").join(id).exists(),
        "successful failed-only runs should be cleaned after JSON is emitted"
    );
}

#[test]
fn failed_vat_toml_runner_keeps_logs_for_inspection() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "failed"

[[runners]]
id = "fail"
cmd = ["sh", "-c", "echo before-fail; exit 7"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "fail", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let id = json["id"].as_str().unwrap();
    assert!(vat_home.path().join("vats").join(id).exists());

    let logs = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["logs", id, "runner"])
        .output()
        .unwrap();
    assert!(logs.status.success());
    assert!(String::from_utf8_lossy(&logs.stdout).contains("before-fail"));
}

#[test]
fn direct_run_mode_still_forwards_exit_code() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--", "sh", "-c", "exit 3"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
}

#[test]
fn llm_guide_mentions_core_agent_contract() {
    let output = Command::new(vat_bin()).arg("llm").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);

    for expected in [
        "vat run <runner-id> --json",
        "vat run -- <command>",
        "vat state <id>",
        "vat diff <id>",
        "vat logs <id>",
        "vat.toml",
        "not Docker",
        "not Docker, OCI, Compose, a Linux runtime, a VM, a daemon",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?} in:\n{stdout}"
        );
    }
}

`````

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: source
changes:
  - path: "projects/vat/tests/vat_toml_runner.rs"
    action: modify
    section: source
    description: |
      Historical source replay payload retained as semantic context. Active
      codegen ownership moved to projects/vat/tech-design/semantic/vat-tests.md#schema.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:projects-vat-tests-vat_toml_runner-rs-source-replay-superseded>"
```
