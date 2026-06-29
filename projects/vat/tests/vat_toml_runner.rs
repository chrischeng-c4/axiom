// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_toml_runner-rs.md#rust-source-unit
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
fn scenario_run_starts_app_dependency_and_runner() {
    if !python3_available() {
        return;
    }

    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
name = "scenario-smoke"

[workspace]
base = "."
workdir = "."
keep = "always"

[[services]]
id = "api"
cmd = ["python3", "-m", "http.server", "{port}", "--bind", "127.0.0.1"]
ready_http = "http://127.0.0.1:{port}/"
export = { APP_URL = "APP_URL" }
timeout_s = 10

[[services]]
id = "deps"
cmd = ["sh", "-c", "while :; do sleep 1; done"]

[[runners]]
id = "e2e"
requires = ["deps"]
cmd = ["sh", "-c", "case \"$APP_URL\" in http://127.0.0.1:*) echo scenario-ok > scenario-artifact.txt;; *) exit 9;; esac"]
artifacts = ["scenario-artifact.txt"]

[[scenarios]]
id = "prod-like"
app = "api"
requires = ["deps"]
runner = "e2e"
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--scenario", "prod-like"])
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
    assert_eq!(events[0]["scenario"], "prod-like");
    assert_eq!(events[0]["app"], "api");
    assert_eq!(events[0]["runner"], "e2e");
    assert!(events
        .iter()
        .any(|event| event["type"] == "ready" && event["service"] == "api"));
    let result = result_event(&events);
    assert_eq!(result["scenario"], "prod-like");
    assert_eq!(result["app"], "api");
    assert_eq!(result["ok"], true);
    let id = result["id"].as_str().unwrap();

    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let json: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    assert_eq!(json["test_run"]["scenario"]["id"], "prod-like");
    assert_eq!(json["test_run"]["scenario"]["app"], "api");
    assert_eq!(json["test_run"]["scenario"]["runner"], "e2e");
    assert!(json["test_run"]["scenario"]["services"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "api"));
    assert_eq!(
        json["test_run"]["artifacts"][0]["path"],
        "scenario-artifact.txt"
    );
}

#[test]
fn scenario_failure_keeps_topology_and_logs() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "failed"

[[services]]
id = "api"
cmd = ["sh", "-c", "while :; do sleep 1; done"]

[[runners]]
id = "fail"
cmd = ["sh", "-c", "echo scenario-before-fail; exit 7"]

[[scenarios]]
id = "prod-like"
app = "api"
runner = "fail"
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--scenario", "prod-like"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(7));
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], false);
    assert_eq!(result["state"], "kept");
    let id = result["id"].as_str().unwrap();

    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    let json: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    assert_eq!(json["test_run"]["scenario"]["id"], "prod-like");
    assert_eq!(json["test_run"]["scenario"]["app"], "api");

    let logs = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["logs", id, "runner"])
        .output()
        .unwrap();
    assert!(logs.status.success());
    assert!(String::from_utf8_lossy(&logs.stdout).contains("scenario-before-fail"));
}

#[test]
fn scenario_hermetic_requires_http_mock_service() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let marker = project.path().join("runner-started");
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1

[[services]]
id = "api"
cmd = ["sh", "-c", "while :; do sleep 1; done"]

[[runners]]
id = "e2e"
cmd = ["sh", "-c", "touch {}"]

[[scenarios]]
id = "prod-like"
app = "api"
runner = "e2e"
network = "hermetic"
"#,
            marker.display()
        ),
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--scenario", "prod-like"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let events = jsonl(&output.stdout);
    assert!(events.iter().any(|event| {
        event["type"] == "error" && event["code"] == "scenario_hermetic_proxy_required"
    }));
    assert!(
        !marker.exists(),
        "runner should not execute when hermetic proxy is missing"
    );
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
    // `runtime = "native"` forbids the Docker fallback, so a missing binary is a
    // hard error — the structured `missing_service_binary` envelope, not a panic.
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[services]]
id = "redis"
preset = "redis"
runtime = "native"

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
fn auto_runtime_without_native_or_docker_reports_unavailable() {
    // Default `runtime = "auto"` prefers the native binary and falls back to
    // Docker. With an empty PATH neither is present, so vat must emit the
    // structured `service_runtime_unavailable` envelope and fail (no panic).
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
            && event["code"] == "service_runtime_unavailable"
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
    let output = Command::new(vat_bin())
        .args(["llm", "--topic", "guide"])
        .output()
        .unwrap();
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
        // Boundaries: vat is not a Docker replacement and never containerizes
        // the runner, even though dependency services may be containers.
        "not a Docker/OCI/Compose replacement",
        "never containerized",
        // Native-or-Docker service contract is discoverable.
        "native or Docker",
        "runtime = \"docker\"",
        // Cloud Tasks / Cloud Scheduler clients need an explicit REST/factory
        // override (the SDKs don't auto-read the host var and default to gRPC).
        "Pointing a client at",
        "default to gRPC, while vat serves REST",
        "forces the REST transport",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected:?} in:\n{stdout}"
        );
    }
}
// CODEGEN-END
