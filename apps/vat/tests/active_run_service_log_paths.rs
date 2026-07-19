// HANDWRITE-BEGIN gap="missing-generator:unit-test:84e02adb" tracker="1872" reason="Prove the live service log-path handoff and fail-closed service id normalization."
#![cfg(unix)]

use std::process::Command;

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("VAT stdout JSONL"))
        .collect()
}

#[test]
fn runner_follows_live_service_through_advertised_log_paths() {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "always"

[[services]]
id = "log-producer"
cmd = ["sh", "-c", "printf 'service-live-marker\\n'; printf 'service-stderr-marker\\n' >&2; trap 'exit 0' TERM; while :; do sleep 1; done"]

[[runners]]
id = "collector"
requires = ["log-producer"]
cmd = ["sh", "-c", "set -eu; test -d \"$VAT_LOGS_DIR\"; test -f \"$VAT_SERVICE_LOG_PRODUCER_STDOUT_LOG\"; test -f \"$VAT_SERVICE_LOG_PRODUCER_STDERR_LOG\"; case \"$VAT_SERVICE_LOG_PRODUCER_STDOUT_LOG\" in \"$VAT_LOGS_DIR\"/*) ;; *) exit 41 ;; esac; case \"$VAT_SERVICE_LOG_PRODUCER_STDERR_LOG\" in \"$VAT_LOGS_DIR\"/*) ;; *) exit 42 ;; esac; i=0; while ! grep -q service-live-marker \"$VAT_SERVICE_LOG_PRODUCER_STDOUT_LOG\"; do i=$((i+1)); test $i -lt 100; sleep 0.02; done; grep -q service-stderr-marker \"$VAT_SERVICE_LOG_PRODUCER_STDERR_LOG\""]
timeout_s = 10
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .args(["run", "collector"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("service-live-marker") && !stdout.contains("service-stderr-marker"),
        "VAT must not replay child logs into its JSONL stdout: {stdout}"
    );
    let events = jsonl(&output.stdout);
    let result = events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("result event");
    assert_eq!(result["ok"], true);
    assert_eq!(result["state"], "kept");
}

fn run_invalid_config(config: &str) -> (tempfile::TempDir, std::process::Output) {
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("vat.toml"), config).unwrap();
    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .arg("run")
        .output()
        .unwrap();
    (project, output)
}

#[test]
fn unsafe_or_colliding_service_log_environment_ids_are_rejected() {
    let (project, unsafe_output) = run_invalid_config(
        r#"
version = 1

[[services]]
id = "../escape"
cmd = ["sh", "-c", "printf unsafe-started > unsafe-started; while :; do sleep 1; done"]

[[runners]]
id = "test"
requires = ["../escape"]
cmd = ["sh", "-c", "printf runner-started > runner-started"]
"#,
    );
    assert!(!unsafe_output.status.success());
    assert!(!project.path().join("unsafe-started").exists());
    assert!(!project.path().join("runner-started").exists());
    assert!(String::from_utf8_lossy(&unsafe_output.stdout).contains("unsafe service id"));

    let (_project, collision_output) = run_invalid_config(
        r#"
version = 1

[[services]]
id = "api-v1"
cmd = ["sh", "-c", "while :; do sleep 1; done"]

[[services]]
id = "api.v1"
cmd = ["sh", "-c", "while :; do sleep 1; done"]

[[runners]]
id = "test"
requires = ["api-v1", "api.v1"]
cmd = ["true"]
"#,
    );
    assert!(!collision_output.status.success());
    assert!(String::from_utf8_lossy(&collision_output.stdout)
        .contains("service log environment token collision"));
}
// HANDWRITE-END
