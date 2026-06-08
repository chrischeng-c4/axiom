// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-tests.md#schema
// CODEGEN-BEGIN
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

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn result_event(events: &[Value]) -> &Value {
    events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("missing result event")
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
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "always"

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
        .arg("run")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    assert_eq!(events[0]["type"], "select");
    assert_eq!(events[0]["runner"], "e2e");
    assert_eq!(events[0]["reason"], "default_runner");
    assert!(events.iter().any(|event| event["type"] == "ready"));
    let result = result_event(&events);
    assert_eq!(result["ok"], true);
    assert_eq!(result["state"], "kept");
    let id = result["id"].as_str().unwrap();

    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let json: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    assert_eq!(json["test_run"]["runner_id"], "e2e");
    assert_eq!(json["test_run"]["runner"]["exit_code"], 0);
    assert_eq!(json["test_run"]["services"][0]["status"], "exited");
    assert_eq!(
        json["test_run"]["artifacts"][0]["path"],
        "runner-artifact.txt"
    );
    assert!(
        vat_home.path().join("vats").join(id).exists(),
        "always-retained run should stay inspectable"
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
        .args(["run", "fail"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], false);
    assert_eq!(result["exit_code"], 7);
    assert_eq!(result["state"], "kept");
    let id = result["id"].as_str().unwrap();
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
fn ambiguous_vat_run_requires_default_runner() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[runners]]
id = "unit"
cmd = ["sh", "-c", "true"]

[[runners]]
id = "e2e"
cmd = ["sh", "-c", "true"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .arg("run")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let events = jsonl(&output.stdout);
    assert_eq!(events[0]["type"], "error");
    assert_eq!(events[0]["code"], "runner_required");
    assert_eq!(events[0]["runners"][0], "unit");
    assert_eq!(events[0]["runners"][1], "e2e");
}

#[test]
fn missing_preset_binary_reports_jsonl_error() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[services]]
id = "redis"
preset = "redis"

[[runners]]
id = "test"
requires = ["redis"]
cmd = ["sh", "-c", "true"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .env("PATH", project.path())
        .arg("run")
        .output()
        .unwrap();

    assert!(!output.status.success());
    let events = jsonl(&output.stdout);
    assert!(events.iter().any(|event| {
        event["type"] == "error"
            && event["code"] == "missing_service_binary"
            && event["service"] == "redis"
    }));
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
        "vat run",
        "vat run <runner-id>",
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
// CODEGEN-END
