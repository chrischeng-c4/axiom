// SPEC-MANAGED: apps/vat/tech-design/semantic/source/projects-vat-tests-vat_toml_runner-rs.md#rust-source-unit
// CODEGEN-BEGIN
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
#[cfg(unix)]
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
#[cfg(unix)]
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

const SCENARIO_E2E_REQUIRED: &str = "VAT_SCENARIO_E2E_REQUIRED";

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

#[cfg(unix)]
fn write_doctor_executable(path: &Path, source: &str) {
    std::fs::write(path, source).expect("write doctor fake runtime");
    let mut permissions = std::fs::metadata(path)
        .expect("doctor fake runtime metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make doctor fake runtime executable");
}

#[cfg(unix)]
fn write_builder_observation_fake(path: &Path) {
    write_doctor_executable(
        path,
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_BUILDER_LOG"
case "$*" in
  "builder status --format json")
    case "${VAT_BUILDER_MODE:-valid}" in
      valid|stats_hang|df_hang)
        printf '%s\n' '[{"configuration":{"id":"buildkit","resources":{"cpus":2,"memoryInBytes":2147483648}},"status":{"state":"running"}}]'
        ;;
      empty)
        printf '%s\n' '[]'
        ;;
      malformed)
        printf '%s\n' '{not-json'
        ;;
      nonzero)
        printf '%s\n' 'fake builder status failure' >&2
        exit 73
        ;;
      status_hang)
        exec /bin/sleep 5
        ;;
      *)
        printf '%s\n' "unexpected builder mode: $VAT_BUILDER_MODE" >&2
        exit 64
        ;;
    esac
    ;;
  "stats buildkit --no-stream --format json")
    if [ "${VAT_BUILDER_MODE:-valid}" = "stats_hang" ]; then
      exec /bin/sleep 5
    fi
    printf '%s\n' '[{"id":"buildkit","memoryUsageBytes":1578954752,"memoryLimitBytes":2147483648,"cpuUsageUsec":84301766,"numProcesses":21}]'
    ;;
  "system df --format json")
    if [ "${VAT_BUILDER_MODE:-valid}" = "df_hang" ]; then
      exec /bin/sleep 5
    fi
    printf '%s\n' '{"containers":{"total":2,"active":1,"sizeInBytes":1921540096,"reclaimable":4096},"images":{"total":12,"active":2,"sizeInBytes":4731920384,"reclaimable":2327961600}}'
    ;;
  *)
    printf '%s\n' "unexpected fake container argv: $*" >&2
    exit 64
    ;;
esac
"#,
    );
}

#[cfg(unix)]
fn run_capabilities_with_builder_mode(
    fake_bin: &Path,
    log: &Path,
    mode: &str,
) -> std::process::Output {
    Command::new(vat_bin())
        .env("PATH", fake_bin)
        .env("VAT_BUILDER_LOG", log)
        .env("VAT_BUILDER_MODE", mode)
        .args(["capabilities", "--json"])
        .output()
        .expect("run capabilities with fake Apple Container")
}

#[cfg(unix)]
fn assert_builder_probe_never_mutates(calls: &str) {
    for forbidden in [
        "builder start",
        "builder stop",
        "builder delete",
        "system start",
        "system stop",
        "image prune",
        "image rm",
    ] {
        assert!(
            !calls.lines().any(|call| call == forbidden),
            "builder observation invoked forbidden mutation {forbidden:?}: {calls}"
        );
    }
}

/// The production timeout is deliberately tight (500ms). Serializing the
/// fake-child checks keeps test-process scheduler contention from being
/// mistaken for an Apple Container timeout while still exercising kill/reap.
#[cfg(unix)]
fn builder_observation_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[test]
fn vat_capabilities_json_reports_effective_backends() {
    #[cfg(unix)]
    let _lock = builder_observation_test_lock();
    let output = Command::new(vat_bin())
        .args(["capabilities", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["host"]["os"], std::env::consts::OS);
    assert_eq!(json["host"]["arch"], std::env::consts::ARCH);
    assert_eq!(json["workspace"]["diff_basis"], "size_mtime_manifest");
    assert_eq!(json["services"]["external_attach"], true);
    assert!(json["isolation"].as_array().unwrap().iter().any(|cap| {
        cap["id"] == "process" && cap["implemented"] == true && cap["available"] == true
    }));
    assert!(json["isolation"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cap| { cap["id"] == "linux-netns" && cap["implemented"] == false }));
    assert!(
        json["docker"]["cli"].is_boolean(),
        "docker capability should be explicit even when Docker is absent"
    );
    assert!(
        json["docker"]["daemon_probe"].is_null(),
        "direct vat capabilities must retain its normal Docker probe behavior"
    );
    assert!(
        matches!(
            json["services"]["docker_services"].as_str(),
            Some("available" | "unavailable")
        ),
        "a full Docker probe must report a conclusive Docker service availability: {}",
        json["services"]
    );
}

#[cfg(unix)]
#[test]
fn vat_capabilities_reports_shared_builder_configuration_and_observed_stats() {
    let _lock = builder_observation_test_lock();
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let log = fake_bin.path().join("container.log");
    write_builder_observation_fake(&fake_bin.path().join("container"));

    let output = run_capabilities_with_builder_mode(fake_bin.path(), &log, "valid");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
    let builder = &json["apple_container"]["builder"];
    assert_eq!(json["apple_container"]["cli"], true);
    assert_eq!(builder["state"], "running");
    assert_eq!(builder["ownership"], "shared_unknown");
    assert_eq!(builder["automatic_cleanup"], false);
    assert_eq!(builder["configuration"]["id"], "buildkit");
    assert_eq!(builder["configuration"]["resources"]["cpus"], 2.0);
    assert_eq!(
        builder["configuration"]["resources"]["memory_bytes"],
        2_147_483_648u64
    );
    assert_eq!(
        builder["observed_stats"]["memory_usage_bytes"],
        1_578_954_752u64
    );
    assert_eq!(
        builder["observed_stats"]["memory_limit_bytes"],
        2_147_483_648u64
    );
    assert_eq!(builder["observed_stats"]["process_count"], 21);
    assert_eq!(builder["global_disk"]["scope"], "global_apple_container");
    assert_eq!(
        builder["global_disk"]["images"]["size_bytes"],
        4_731_920_384u64
    );
    assert_eq!(
        builder["global_disk"]["images"]["reclaimable_bytes"],
        2_327_961_600u64
    );

    let calls = std::fs::read_to_string(&log).expect("read fake container calls");
    assert_eq!(
        calls.lines().collect::<Vec<_>>(),
        vec![
            "builder status --format json",
            "stats buildkit --no-stream --format json",
            "system df --format json",
        ]
    );
    assert_builder_probe_never_mutates(&calls);
}

#[cfg(unix)]
#[test]
fn vat_capabilities_keeps_missing_malformed_and_nonzero_builder_status_advisory() {
    let _lock = builder_observation_test_lock();
    for (mode, state, error_fragment) in [
        ("empty", "not_running", None),
        ("malformed", "unknown", Some("invalid JSON")),
        ("nonzero", "unknown", Some("fake builder status failure")),
    ] {
        let fake_bin = tempfile::tempdir().expect("fake runtime bin");
        let log = fake_bin.path().join("container.log");
        write_builder_observation_fake(&fake_bin.path().join("container"));

        let output = run_capabilities_with_builder_mode(fake_bin.path(), &log, mode);
        assert!(
            output.status.success(),
            "mode={mode} stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json: Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
        let builder = &json["apple_container"]["builder"];
        assert_eq!(builder["state"], state, "mode={mode}");
        assert_eq!(builder["ownership"], "shared_unknown", "mode={mode}");
        assert_eq!(builder["automatic_cleanup"], false, "mode={mode}");
        assert!(builder["configuration"].is_null(), "mode={mode}: {builder}");
        assert!(
            builder["observed_stats"].is_null(),
            "mode={mode}: {builder}"
        );
        assert!(builder["global_disk"].is_null(), "mode={mode}: {builder}");
        match error_fragment {
            Some(error_fragment) => assert!(
                builder["probe_errors"]
                    .as_array()
                    .expect("advisory errors")
                    .iter()
                    .any(|error| error["message"]
                        .as_str()
                        .is_some_and(|message| message.contains(error_fragment))),
                "mode={mode}: {builder}"
            ),
            None => assert!(
                builder["probe_errors"].is_null(),
                "empty builder status is an ordinary not-running observation: {builder}"
            ),
        }

        let calls = std::fs::read_to_string(&log).expect("read fake container calls");
        assert_eq!(calls.trim(), "builder status --format json", "mode={mode}");
        assert_builder_probe_never_mutates(&calls);
    }
}

#[cfg(unix)]
#[test]
fn vat_capabilities_bounds_hung_builder_stats_and_disk_observations() {
    let _lock = builder_observation_test_lock();
    for (mode, missing_field, error_probe) in [
        ("status_hang", "configuration", "builder_status"),
        ("stats_hang", "observed_stats", "stats"),
        ("df_hang", "global_disk", "system_df"),
    ] {
        let fake_bin = tempfile::tempdir().expect("fake runtime bin");
        let log = fake_bin.path().join("container.log");
        write_builder_observation_fake(&fake_bin.path().join("container"));

        let started = std::time::Instant::now();
        let output = run_capabilities_with_builder_mode(fake_bin.path(), &log, mode);
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "mode={mode} exceeded bounded advisory deadline: {:?}",
            started.elapsed()
        );
        assert!(
            output.status.success(),
            "mode={mode} stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let json: Value = serde_json::from_slice(&output.stdout).expect("capabilities JSON");
        let builder = &json["apple_container"]["builder"];
        assert!(builder[missing_field].is_null(), "mode={mode}: {builder}");
        assert!(
            builder["probe_errors"]
                .as_array()
                .expect("advisory error")
                .iter()
                .any(|error| error["probe"] == error_probe
                    && error["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("timed out"))),
            "mode={mode}: {builder}"
        );
        let calls = std::fs::read_to_string(&log).expect("read fake container calls");
        assert_builder_probe_never_mutates(&calls);
    }
}

#[test]
fn vat_gc_dry_run_and_execute_prunes_successful_vats_safely() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("input.txt"), "seed").unwrap();

    let ok = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--json", "--", "sh", "-c", "true"])
        .output()
        .unwrap();
    assert!(
        ok.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&ok.stdout),
        String::from_utf8_lossy(&ok.stderr)
    );
    let ok_state: Value = serde_json::from_slice(&ok.stdout).unwrap();
    let ok_id = ok_state["id"].as_str().unwrap().to_string();

    let failed = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--json", "--", "sh", "-c", "exit 7"])
        .output()
        .unwrap();
    assert_eq!(failed.status.code(), Some(7));
    let failed_state: Value = serde_json::from_slice(&failed.stdout).unwrap();
    let failed_id = failed_state["id"].as_str().unwrap().to_string();

    let dry_run = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["gc", "--keep-last", "0", "--apparent", "--json"])
        .output()
        .unwrap();
    assert!(dry_run.status.success());
    let dry_json: Value = serde_json::from_slice(&dry_run.stdout).unwrap();
    assert_eq!(dry_json["dry_run"], true);
    assert_gc_entry(&dry_json, &ok_id, true, "candidate", false);
    assert_gc_entry(&dry_json, &failed_id, false, "failed_retained", false);
    assert!(vat_home.path().join("vats").join(&ok_id).exists());
    assert!(vat_home.path().join("vats").join(&failed_id).exists());

    let execute = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args([
            "gc",
            "--keep-last",
            "0",
            "--execute",
            "--apparent",
            "--json",
        ])
        .output()
        .unwrap();
    assert!(execute.status.success());
    let execute_json: Value = serde_json::from_slice(&execute.stdout).unwrap();
    assert_eq!(execute_json["dry_run"], false);
    assert_gc_entry(&execute_json, &ok_id, true, "candidate", true);
    assert_gc_entry(&execute_json, &failed_id, false, "failed_retained", false);
    assert!(!vat_home.path().join("vats").join(&ok_id).exists());
    assert!(vat_home.path().join("vats").join(&failed_id).exists());
}

fn assert_gc_entry(json: &Value, id: &str, candidate: bool, reason: &str, deleted: bool) {
    let entry = json["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("missing gc entry for {id}: {json}"));
    assert_eq!(entry["candidate"], candidate);
    assert_eq!(entry["reason"], reason);
    assert_eq!(entry["deleted"], deleted);
    assert!(entry["apparent_size_bytes"].as_u64().is_some());
}

#[test]
fn scenario_run_starts_app_dependency_and_runner() {
    if !python3_available() {
        assert_ne!(
            std::env::var(SCENARIO_E2E_REQUIRED).as_deref(),
            Ok("1"),
            "{SCENARIO_E2E_REQUIRED}=1 requires a working python3 interpreter"
        );
        eprintln!("Skipping scenario runner test: python3 is unavailable");
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
fn occupied_native_service_endpoint_fails_closed_without_starting_any_owned_process() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind unrelated HTTP listener");
    listener.set_nonblocking(true).unwrap();
    let endpoint = listener.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_server = Arc::clone(&stop);
    let server = std::thread::spawn(move || {
        while !stop_server.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut request = [0u8; 1024];
                    let _ = stream.read(&mut request);
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok",
                    );
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(_) => break,
            }
        }
    });
    let service_marker = project.path().join("owned-service-started");
    let runner_marker = project.path().join("dependent-runner-started");
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
default_runner = "e2e"

[workspace]
keep = "always"

[[services]]
id = "api"
port = {port}
cmd = ["sh", "-c", "touch {service_marker}; while :; do sleep 1; done"]
ready_http = "http://127.0.0.1:{port}/readyz"
timeout_s = 2

[[runners]]
id = "e2e"
requires = ["api"]
cmd = ["sh", "-c", "touch {runner_marker}"]
"#,
            port = endpoint.port(),
            service_marker = service_marker.display(),
            runner_marker = runner_marker.display(),
        ),
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
    assert!(events.iter().any(|event| {
        event["type"] == "error"
            && event["code"] == "native_service_endpoint_conflict"
            && event["service"] == "api"
            && event["endpoint"] == endpoint.to_string()
    }));
    assert!(
        !service_marker.exists(),
        "VAT must not spawn the owned service"
    );
    assert!(
        !runner_marker.exists(),
        "VAT must not start a dependent runner"
    );

    let mut stale = TcpStream::connect(endpoint).expect("unrelated listener must stay alive");
    stale
        .write_all(b"GET /readyz HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .unwrap();
    let mut response = String::new();
    stale.read_to_string(&mut response).unwrap();
    assert!(response.starts_with("HTTP/1.1 200 OK"));
    stop.store(true, Ordering::Relaxed);
    server.join().unwrap();
}

#[test]
fn native_service_child_exit_before_endpoint_transition_never_starts_runner() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let runner_marker = project.path().join("dependent-runner-started");
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
default_runner = "e2e"

[workspace]
keep = "always"

[[services]]
id = "api"
cmd = ["sh", "-c", "exit 23", "--", "{{port}}"]
timeout_s = 2

[[runners]]
id = "e2e"
requires = ["api"]
cmd = ["sh", "-c", "touch {runner_marker}"]
"#,
            runner_marker = runner_marker.display(),
        ),
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
    let child_exit = events
        .iter()
        .find(|event| event["code"] == "owned_service_exited_before_readiness")
        .expect("owned child exit must be structured");
    assert_eq!(child_exit["service"], "api");
    assert_eq!(child_exit["exit_code"], 23);
    assert!(child_exit["endpoint"]
        .as_str()
        .unwrap()
        .starts_with("127.0.0.1:"));
    assert!(
        !runner_marker.exists(),
        "VAT must not start a dependent runner"
    );

    let result = result_event(&events);
    let id = result["id"].as_str().unwrap();
    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let state: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    let service = &state["test_run"]["services"][0];
    assert_eq!(service["status"], "failed");
    assert_eq!(service["exit_code"], 23);
    assert!(service["readiness_error"]
        .as_str()
        .unwrap()
        .contains("before endpoint"));
}

#[test]
fn vat_plan_json_reports_runner_topology_without_creating_vat() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "failed"

[[services]]
id = "web"
cmd = ["sh", "-c", "while :; do sleep 1; done"]
ready_http = "http://127.0.0.1:{port}/"

[[runners]]
id = "e2e"
requires = ["web"]
cmd = ["sh", "-c", "true"]
artifacts = ["test-results/**"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .args(["plan", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["selection"]["kind"], "runner");
    assert_eq!(json["selection"]["runner_id"], "e2e");
    assert_eq!(json["selection"]["reason"], "default_runner");
    assert_eq!(json["services"][0]["id"], "web");
    assert_eq!(json["services"][0]["backing"], "cmd");
    assert!(json["env"]["services"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "VAT_SERVICE_WEB_URL"));
    assert_eq!(json["artifacts"][0], "test-results/**");
    assert!(
        !project.path().join(".vat").exists(),
        "vat plan must not create a vat store"
    );
}

#[test]
fn vat_doctor_json_reports_unreachable_external_service() {
    let project = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
default_runner = "e2e"

[[services]]
id = "postgres"
external = {{ host = "127.0.0.1", port = {port} }}

[[runners]]
id = "e2e"
requires = ["postgres"]
cmd = ["sh", "-c", "true"]
"#
        ),
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], false);
    assert!(json["checks"].as_array().unwrap().iter().any(|check| {
        check["component"] == "external"
            && check["id"] == "postgres"
            && check["ok"] == false
            && check["code"] == "external_tcp"
    }));
}

#[test]
fn vat_doctor_host_only_needs_no_vat_toml() {
    let empty_project = tempfile::tempdir().unwrap();
    let output = Command::new(vat_bin())
        .current_dir(empty_project.path())
        .args(["doctor", "--host-only", "--json"])
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    let json: Value = serde_json::from_slice(&output.stdout).expect("host-only doctor JSON");
    assert_eq!(json["ok"], true);
    assert_eq!(json["mode"], "host_only");
    assert_eq!(json["next"], "vat capabilities --json");
    assert!(json["capabilities"]["workspace"]["primary_clone_method"].is_string());
    assert!(json["gpu"]["accessible"].is_boolean());
    let checks = json["checks"].as_array().expect("host-only checks");
    for id in ["copy_on_write", "host", "cli", "daemon", "kubectl"] {
        assert!(
            checks.iter().any(|check| check["id"] == id),
            "host-only doctor is missing `{id}` evidence: {checks:?}"
        );
    }
}

#[test]
fn vat_doctor_json_includes_capabilities_and_egress_check() {
    let project = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "localhost-only"

[[runners]]
id = "e2e"
cmd = ["sh", "-c", "true"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .args(["doctor", "--json"])
        .output()
        .unwrap();

    assert!(
        output.status.code().is_some(),
        "doctor should exit cleanly, got {:?}",
        output.status
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["capabilities"]["host"]["os"], std::env::consts::OS);
    let enforceable = json["capabilities"]["isolation"]
        .as_array()
        .unwrap()
        .iter()
        .any(|cap| {
            (cap["id"] == "macos-seatbelt" || cap["id"] == "linux-netns")
                && cap["available"] == true
        });
    let check = json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|check| {
            check["component"] == "isolation"
                && check["id"] == "egress"
                && check["code"] == "egress_enforcement"
        })
        .expect("doctor should report egress enforcement capability");
    assert_eq!(check["ok"], enforceable);
}

#[cfg(unix)]
#[test]
fn vat_doctor_routes_explicit_microvm_services_to_apple_container() {
    let _lock = builder_observation_test_lock();
    let project = tempfile::tempdir().expect("project");
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let container_log = fake_bin.path().join("container.log");
    let docker_log = fake_bin.path().join("docker.log");
    write_doctor_executable(
        &fake_bin.path().join("container"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_CONTAINER_LOG"
case "$*" in
  "builder status --format json")
    printf '%s\n' '[{"configuration":{"id":"buildkit","resources":{"cpus":2,"memoryInBytes":2147483648}},"status":{"state":"running"}}]'
    ;;
  "stats buildkit --no-stream --format json")
    printf '%s\n' '[{"id":"buildkit","memoryUsageBytes":1578954752,"memoryLimitBytes":2147483648,"numProcesses":21}]'
    ;;
  "system df --format json")
    printf '%s\n' '{"images":{"total":12,"active":2,"sizeInBytes":4731920384,"reclaimable":2327961600}}'
    ;;
  "system status")
    ;;
  *)
    exit 64
    ;;
esac
"#,
    );
    write_doctor_executable(
        &fake_bin.path().join("docker"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_DOCKER_LOG"
exit 91
"#,
    );
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "open"

[[services]]
id = "image-microvm"
image = "example.test/image:latest"
container_port = 8080
runtime = "micro_vm"

[[services]]
id = "preset-microvm"
preset = "postgres"
runtime = "micro_vm"

[[services]]
id = "unselected-docker"
image = "example.test/unselected:latest"
container_port = 8080

[[runners]]
id = "e2e"
requires = ["image-microvm", "preset-microvm"]
cmd = ["true"]

[[runners]]
id = "docker-runner"
requires = ["unselected-docker"]
cmd = ["true"]
"#,
    )
    .expect("write MicroVM doctor config");

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("PATH", fake_bin.path())
        .env("VAT_DOCTOR_CONTAINER_LOG", &container_log)
        .env("VAT_DOCTOR_DOCKER_LOG", &docker_log)
        .args(["doctor", "--json"])
        .output()
        .expect("run MicroVM doctor");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    let checks = json["checks"].as_array().expect("doctor checks");
    for id in ["image-microvm", "preset-microvm"] {
        let check = checks
            .iter()
            .find(|check| check["id"] == id && check["code"] == "apple_container_system")
            .unwrap_or_else(|| panic!("missing Apple Container doctor check for {id}: {checks:?}"));
        assert_eq!(check["component"], "apple_container");
        assert_eq!(check["ok"], true);
        assert!(check["message"]
            .as_str()
            .expect("Apple Container message")
            .contains("container system status"));
        let builder_advisory = checks
            .iter()
            .find(|check| check["id"] == id && check["code"] == "apple_container_builder_shared")
            .unwrap_or_else(|| panic!("missing shared builder advisory for {id}: {checks:?}"));
        assert_eq!(builder_advisory["component"], "apple_container");
        assert_eq!(builder_advisory["ok"], true);
        let message = builder_advisory["message"]
            .as_str()
            .expect("shared builder advisory message");
        assert!(message.contains("ownership=shared_unknown"));
        assert!(message.contains("automatic_cleanup=false"));
        assert!(message.contains("state=running"));
        assert!(message.contains("observed_memory_bytes=1578954752"));
    }
    assert!(
        !checks.iter().any(|check| check["component"] == "docker"),
        "explicit MicroVM doctor checks must not fall back to Docker: {checks:?}"
    );
    assert_eq!(json["capabilities"]["docker"]["cli"], true);
    assert_eq!(
        json["capabilities"]["docker"]["daemon_probe"]["state"],
        "skipped"
    );
    assert_eq!(
        json["capabilities"]["docker"]["daemon_probe"]["reason"],
        "Docker daemon probe skipped for Apple-Container-only selected plan"
    );
    assert_eq!(
        json["capabilities"]["docker"]["error"],
        "Docker daemon probe skipped for Apple-Container-only selected plan"
    );
    assert_eq!(
        json["capabilities"]["services"]["docker_services"], "not_probed",
        "a deliberately skipped Docker daemon probe must not be reported as unavailable"
    );
    assert!(
        !docker_log.exists(),
        "selected Apple-Container-only plan must not execute Docker even when an unselected Docker service and Docker binary exist: {}",
        docker_log.display()
    );
    let calls_text = std::fs::read_to_string(&container_log).expect("read container status calls");
    let calls: Vec<_> = calls_text.lines().collect();
    assert_eq!(
        calls,
        vec![
            "builder status --format json",
            "stats buildkit --no-stream --format json",
            "system df --format json",
            "system status",
        ]
    );
    assert_builder_probe_never_mutates(&calls_text);
}

#[cfg(unix)]
#[test]
fn vat_doctor_rejects_unsupported_microvm_preset_without_docker_fallback() {
    let _lock = builder_observation_test_lock();
    let project = tempfile::tempdir().expect("project");
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let container_log = fake_bin.path().join("container.log");
    let docker_log = fake_bin.path().join("docker.log");
    write_doctor_executable(
        &fake_bin.path().join("container"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_CONTAINER_LOG"
case "$*" in
  "builder status --format json")
    printf '%s\n' '[]'
    ;;
  "system status")
    ;;
  *)
    exit 64
    ;;
esac
"#,
    );
    write_doctor_executable(
        &fake_bin.path().join("docker"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_DOCKER_LOG"
exit 91
"#,
    );
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "open"

[[services]]
id = "firebase-microvm"
preset = "firebase"
runtime = "micro_vm"

[[runners]]
id = "e2e"
requires = ["firebase-microvm"]
cmd = ["true"]
"#,
    )
    .expect("write unsupported MicroVM preset config");
    std::fs::write(project.path().join("firebase.json"), "{}")
        .expect("write required Firebase workspace config");

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("PATH", fake_bin.path())
        .env("VAT_DOCTOR_CONTAINER_LOG", &container_log)
        .env("VAT_DOCTOR_DOCKER_LOG", &docker_log)
        .args(["doctor", "--json"])
        .output()
        .expect("run unsupported MicroVM preset doctor");

    assert_eq!(output.status.code(), Some(1));
    assert!(
        !output.stdout.is_empty(),
        "unsupported MicroVM preset doctor emitted no JSON\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    let checks = json["checks"].as_array().expect("doctor checks");
    let preset = checks
        .iter()
        .find(|check| {
            check["id"] == "firebase-microvm" && check["code"] == "microvm_preset_unsupported"
        })
        .expect("unsupported MicroVM preset check");
    assert_eq!(preset["component"], "preset");
    assert_eq!(preset["ok"], false);
    let message = preset["message"]
        .as_str()
        .expect("unsupported preset message");
    assert!(message.contains("no declared Apple Container OCI image route"));
    assert!(message.contains("will not fall back to Docker"));
    assert!(
        !checks.iter().any(|check| check["component"] == "docker"),
        "unsupported MicroVM preset must not fall back to Docker: {checks:?}"
    );
    assert!(
        !docker_log.exists(),
        "unsupported MicroVM preset doctor must not invoke Docker"
    );
    let calls = std::fs::read_to_string(&container_log).expect("read fake container calls");
    assert_eq!(
        calls.lines().collect::<Vec<_>>(),
        vec!["builder status --format json", "system status"]
    );
    assert_builder_probe_never_mutates(&calls);
}

#[cfg(unix)]
#[test]
fn vat_doctor_selected_cluster_forces_a_docker_probe() {
    let _lock = builder_observation_test_lock();
    let project = tempfile::tempdir().expect("project");
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let docker_log = fake_bin.path().join("docker.log");
    write_doctor_executable(
        &fake_bin.path().join("container"),
        r#"#!/bin/sh
set -eu
[ "$#" -eq 2 ]
[ "$1" = "system" ]
[ "$2" = "status" ]
"#,
    );
    write_doctor_executable(
        &fake_bin.path().join("docker"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_DOCKER_LOG"
case "$1" in
  context)
    [ "${2:-}" = "show" ]
    printf '%s\n' fake-context
    ;;
  version)
    [ "${2:-}" = "--format" ]
    printf '%s\n' 1.0
    ;;
  info)
    ;;
  *)
    exit 64
    ;;
esac
"#,
    );
    write_doctor_executable(&fake_bin.path().join("kind"), "#!/bin/sh\nexit 0\n");
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "open"

[[services]]
id = "image-microvm"
image = "example.test/image:latest"
container_port = 8080
runtime = "micro_vm"

[[services]]
id = "local-cluster"
cluster = "kind"

[[runners]]
id = "e2e"
requires = ["image-microvm", "local-cluster"]
cmd = ["true"]
"#,
    )
    .expect("write cluster doctor config");

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("PATH", fake_bin.path())
        .env("VAT_DOCTOR_DOCKER_LOG", &docker_log)
        .args(["doctor", "--json"])
        .output()
        .expect("run cluster doctor");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    assert_eq!(json["capabilities"]["docker"]["daemon"], true);
    assert!(
        json["capabilities"]["docker"]["daemon_probe"].is_null(),
        "selected cluster must use the normal Docker capability probe: {}",
        json["capabilities"]["docker"]
    );
    assert_eq!(
        json["capabilities"]["services"]["docker_services"], "available",
        "the fake selected cluster has a reachable Docker daemon"
    );
    let calls = std::fs::read_to_string(&docker_log).expect("read Docker calls");
    for expected in ["version --format {{.Server.Version}}", "info"] {
        assert!(
            calls.lines().any(|call| call == expected),
            "selected cluster omitted required Docker probe {expected:?}: {calls}"
        );
    }
}

#[cfg(unix)]
#[test]
fn vat_doctor_microvm_status_failure_names_apple_container_remediation() {
    let _lock = builder_observation_test_lock();
    let project = tempfile::tempdir().expect("project");
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let container_log = fake_bin.path().join("container.log");
    write_doctor_executable(
        &fake_bin.path().join("container"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_CONTAINER_LOG"
exit 75
"#,
    );
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "open"

[[services]]
id = "image-microvm"
image = "example.test/image:latest"
container_port = 8080
runtime = "micro_vm"

[[runners]]
id = "e2e"
requires = ["image-microvm"]
cmd = ["true"]
"#,
    )
    .expect("write failing MicroVM doctor config");

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("PATH", fake_bin.path())
        .env("VAT_DOCTOR_CONTAINER_LOG", &container_log)
        .args(["doctor", "--json"])
        .output()
        .expect("run failing MicroVM doctor");

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    let checks = json["checks"].as_array().expect("doctor checks");
    let check = checks
        .iter()
        .find(|check| check["id"] == "image-microvm")
        .expect("MicroVM doctor check");
    assert_eq!(check["component"], "apple_container");
    assert_eq!(check["code"], "apple_container_system");
    assert_eq!(check["ok"], false);
    let message = check["message"].as_str().expect("Apple Container message");
    assert!(message.contains("Apple Container"));
    assert!(message.contains("container system status"));
    assert!(
        !message.contains("Docker"),
        "MicroVM remediation must not suggest Docker fallback: {message}"
    );
    assert!(
        !checks.iter().any(|check| check["component"] == "docker"),
        "failing MicroVM doctor checks must not fall back to Docker: {checks:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&container_log)
            .expect("read container status calls")
            .lines()
            .collect::<Vec<_>>(),
        vec!["builder status --format json", "system status"]
    );
}

#[cfg(unix)]
#[test]
fn vat_doctor_retains_docker_checks_for_docker_runtime_and_auto_image() {
    let project = tempfile::tempdir().expect("project");
    let fake_bin = tempfile::tempdir().expect("fake runtime bin");
    let docker_log = fake_bin.path().join("docker.log");
    write_doctor_executable(
        &fake_bin.path().join("docker"),
        r#"#!/bin/sh
set -eu
printf '%s\n' "$*" >> "$VAT_DOCTOR_DOCKER_LOG"
case "$1" in
  context)
    [ "${2:-}" = "show" ]
    printf '%s\n' fake-context
    ;;
  version)
    [ "${2:-}" = "--format" ]
    printf '%s\n' 1.0
    ;;
  *)
    exit 64
    ;;
esac
"#,
    );
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1
default_runner = "e2e"

[network]
egress = "open"

[[services]]
id = "image-auto"
image = "example.test/image:latest"
container_port = 8080

[[services]]
id = "preset-docker"
preset = "postgres"
runtime = "docker"

[[services]]
id = "preset-auto"
preset = "postgres"

[[runners]]
id = "e2e"
requires = ["image-auto", "preset-docker", "preset-auto"]
cmd = ["true"]
"#,
    )
    .expect("write Docker doctor config");

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("PATH", fake_bin.path())
        .env("VAT_DOCTOR_DOCKER_LOG", &docker_log)
        .args(["doctor", "--json"])
        .output()
        .expect("run Docker doctor");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).expect("doctor JSON");
    let checks = json["checks"].as_array().expect("doctor checks");
    for id in ["image-auto", "preset-docker"] {
        let check = checks
            .iter()
            .find(|check| check["id"] == id && check["code"] == "docker_daemon")
            .unwrap_or_else(|| panic!("missing Docker doctor check for {id}: {checks:?}"));
        assert_eq!(check["component"], "docker");
        assert_eq!(check["ok"], true);
    }
    let fallback = checks
        .iter()
        .find(|check| check["id"] == "preset-auto" && check["code"] == "preset_runtime")
        .expect("auto preset fallback doctor check");
    assert_eq!(fallback["component"], "preset");
    assert_eq!(fallback["ok"], true);
    assert!(fallback["message"]
        .as_str()
        .expect("auto preset fallback message")
        .contains("Docker fallback"));
    let calls = std::fs::read_to_string(&docker_log).expect("read Docker calls");
    assert!(calls.lines().any(|call| call == "context show"));
    assert!(
        calls
            .lines()
            .any(|call| call.starts_with("version --format")),
        "Docker daemon capability probe was not preserved: {calls}"
    );
}

#[test]
fn vat_run_plan_records_plan_evidence_and_injects_env() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("impact.json"), r#"{"tests":["unit"]}"#).unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "always"

[[runners]]
id = "impacted"
cmd = ["sh", "-c", "test -f \"$VAT_PLAN_PATH\" && grep -q unit \"$VAT_PLAN_PATH\" && test -n \"$VAT_PLAN_DIGEST\" && printf '%s' \"$VAT_PLAN_DIGEST\" > digest.txt"]
artifacts = ["digest.txt"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--plan", "impact.json", "impacted"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    let id = result_event(&events)["id"].as_str().unwrap();
    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let json: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    assert_eq!(
        json["plan"]["digest"], json["test_run"]["plan"]["digest"],
        "top-level and test-run plan evidence should match"
    );
    assert!(json["test_run"]["plan"]["rootfs_path"]
        .as_str()
        .unwrap()
        .contains(".vat-plan/impact.json"));
    assert_eq!(json["test_run"]["topology"]["runners"][0], "impacted");
    assert_eq!(
        json["spec"]["env"]["VAT_PLAN_DIGEST"],
        json["plan"]["digest"]
    );
    assert_eq!(json["test_run"]["artifacts"][0]["path"], "digest.txt");
}

#[test]
fn external_service_attaches_to_ci_sidecar() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let _accept_thread = std::thread::spawn(move || while listener.accept().is_ok() {});

    std::fs::write(
        project.path().join("vat.toml"),
        format!(
            r#"
version = 1
name = "external-service-smoke"
default_runner = "e2e"

[workspace]
base = "."
workdir = "."
keep = "always"

[[services]]
id = "postgres"
external = {{ host = "127.0.0.1", port = {port} }}
export = {{ DATABASE_URL = "postgres://postgres@{{host}}:{{port}}/app" }}
timeout_s = 5

[[runners]]
id = "e2e"
requires = ["postgres"]
cmd = ["sh", "-c", "test \"$DATABASE_URL\" = \"postgres://postgres@127.0.0.1:{port}/app\" && test \"$VAT_SERVICE_POSTGRES_HOST\" = \"127.0.0.1\" && test \"$VAT_SERVICE_POSTGRES_PORT\" = \"{port}\" && echo external-ok > external.txt"]
artifacts = ["external.txt"]
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
    assert!(events.iter().any(|event| {
        event["type"] == "prepare"
            && event["service"] == "postgres"
            && event["runtime"] == "external"
            && event["owned_by_vat"] == false
    }));
    assert!(events
        .iter()
        .any(|event| event["type"] == "ready" && event["service"] == "postgres"));
    let result = result_event(&events);
    assert_eq!(result["ok"], true);
    let id = result["id"].as_str().unwrap();

    let state_output = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", id, "--compact"])
        .output()
        .unwrap();
    assert!(state_output.status.success());
    let json: Value = serde_json::from_slice(&state_output.stdout).unwrap();
    let service = &json["test_run"]["services"][0];
    assert_eq!(service["id"], "postgres");
    assert_eq!(service["command"].as_array().unwrap().len(), 0);
    assert_eq!(service["status"], "ready");
    assert_eq!(service["prepare_mode"], "external_attach");
    assert_eq!(service["host"], "127.0.0.1");
    assert_eq!(service["port"].as_u64(), Some(u64::from(port)));
    assert_eq!(service["owned_by_vat"], false);
    assert!(service.get("pid").is_none());
    let exported = service["exported_env"].as_array().unwrap();
    for expected in [
        "DATABASE_URL",
        "VAT_SERVICE_POSTGRES_HOST",
        "VAT_SERVICE_POSTGRES_PORT",
    ] {
        assert!(
            exported.iter().any(|value| value == expected),
            "missing exported env {expected}"
        );
    }
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
fn keep_override_retains_successful_run_logs() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "failed"

[[runners]]
id = "pass"
cmd = ["sh", "-c", "echo kept-success"]
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "--keep", "always", "pass"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(result["ok"], true);
    assert_eq!(result["state"], "kept");
    let id = result["id"].as_str().unwrap();

    let logs = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["logs", id, "runner"])
        .output()
        .unwrap();
    assert!(logs.status.success());
    assert!(String::from_utf8_lossy(&logs.stdout).contains("kept-success"));
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
        // Boundaries: vat has a narrow opt-in Docker command shim, but is not
        // a Docker Engine/general-Compose replacement, is permanently
        // headless, and never containerizes the runner even though dependency
        // services may be containers.
        "not a Docker Engine/API or general-Compose replacement",
        "It is permanently headless",
        "does not expose a Docker Engine socket/API",
        "vat docker install-shim",
        "explicit host port",
        "up -d --build",
        "exec -T SERVICE -- COMMAND",
        "child_exit_code",
        "cleanup_next",
        "VAT-owned `images`",
        "vat k8s ephemeral image build",
        "VAT_K8S_CACHE_DIR",
        "vat_k8s_ephemeral_result",
        "does not use Docker",
        "never containerized",
        // Native/Docker/explicit-Apple-Container service contract is discoverable.
        "native, Docker, or explicit Apple Container",
        "runtime = \"docker\"",
        "external = { host",
        "owned_by_vat = false",
        "Env export contract",
        "VAT_WORKSPACE_BASE",
        "STORAGE_EMULATOR_HOST` includes `http://",
        "vat run --keep always",
        "kern.ipc.somaxconn",
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
    for obsolete in [
        "The shim has one strict Compose profile only",
        "It rejects build, multiple services",
    ] {
        assert!(
            !stdout.contains(obsolete),
            "obsolete Compose guidance {obsolete:?} remains in:\n{stdout}"
        );
    }
}
// CODEGEN-END
