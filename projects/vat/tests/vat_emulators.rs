// SPEC-MANAGED: projects/vat/tech-design/semantic/source/projects-vat-tests-vat_emulators-rs.md#rust-source-unit
// CODEGEN-BEGIN
//! Integration coverage for the GCP / Firebase emulator presets.
//!
//! The unavailable path is deterministic and always runs (empty PATH → no
//! gcloud, no docker). The live emulator tests need their tooling, so they are
//! `#[ignore]` and also skip gracefully when it is absent.

use std::process::{Command, Stdio};

use serde_json::Value;

fn vat_bin() -> &'static str {
    env!("CARGO_BIN_EXE_vat")
}

fn jsonl(stdout: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(stdout)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn result_event(events: &[Value]) -> &Value {
    events
        .iter()
        .find(|event| event["type"] == "result")
        .expect("missing result event")
}

fn on_path(bin: &str) -> bool {
    Command::new(bin)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn gcloud_component_installed(component: &str) -> bool {
    Command::new("gcloud")
        .args([
            "components",
            "list",
            "--only-local-state",
            "--format=value(id)",
        ])
        .stderr(Stdio::null())
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .any(|l| l.trim() == component)
        })
        .unwrap_or(false)
}

fn firestore_native_available() -> bool {
    on_path("gcloud") && on_path("java") && gcloud_component_installed("cloud-firestore-emulator")
}

#[test]
fn gcloud_emulator_unavailable_reports_jsonl_error() {
    // A firestore preset with an empty PATH (no gcloud, no docker) must emit the
    // structured service_runtime_unavailable envelope and fail — never a panic.
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[services]]
id = "fs"
preset = "firestore"

[[runners]]
id = "test"
requires = ["fs"]
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
    assert!(
        events.iter().any(|event| {
            event["type"] == "error"
                && event["code"] == "service_runtime_unavailable"
                && event["preset"] == "firestore"
                && event["service"] == "fs"
        }),
        "missing service_runtime_unavailable event: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn firebase_without_firebase_json_is_rejected() {
    // A firebase preset with no firebase.json must fail validation (no panic).
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[services]]
id = "fb"
preset = "firebase"

[[runners]]
id = "test"
requires = ["fb"]
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
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("firebase.json")
            || String::from_utf8_lossy(&output.stdout).contains("firebase.json"),
        "expected a firebase.json requirement error"
    );
}

#[test]
#[ignore = "requires the native gcloud Firestore emulator (gcloud + java + cloud-firestore-emulator)"]
fn firestore_emulator_exports_host() {
    if !firestore_native_available() {
        return;
    }
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "always"

[[services]]
id = "fs"
preset = "firestore"
runtime = "native"
timeout_s = 120

[[runners]]
id = "probe"
requires = ["fs"]
cmd = ["sh", "-c", "test -n \"$FIRESTORE_EMULATOR_HOST\""]
timeout_s = 120
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .arg("run")
        .output()
        .unwrap();
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(
        result["ok"],
        Value::Bool(true),
        "run failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );

    let id = result["id"].as_str().unwrap().to_string();
    let state_out = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["state", &id, "--compact"])
        .output()
        .unwrap();
    let state: Value = serde_json::from_slice(&state_out.stdout).unwrap();
    let service = &state["test_run"]["services"][0];
    let exported = service["exported_env"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        exported.iter().any(|v| v == "FIRESTORE_EMULATOR_HOST"),
        "FIRESTORE_EMULATOR_HOST not exported: {}",
        String::from_utf8_lossy(&state_out.stdout)
    );
}

#[test]
#[ignore = "requires firebase-tools + a firebase.json"]
fn firebase_bundle_exports_hosts() {
    if !on_path("firebase") {
        return;
    }
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("firebase.json"),
        r#"{ "emulators": { "firestore": { "port": 8080 }, "hub": { "port": 4400 } } }"#,
    )
    .unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[workspace]
keep = "always"

[[services]]
id = "fb"
preset = "firebase"
timeout_s = 120

[[runners]]
id = "probe"
requires = ["fb"]
cmd = ["sh", "-c", "test -n \"$FIRESTORE_EMULATOR_HOST\""]
timeout_s = 120
"#,
    )
    .unwrap();

    let output = Command::new(vat_bin())
        .current_dir(project.path())
        .env("VAT_HOME", vat_home.path())
        .arg("run")
        .output()
        .unwrap();
    let events = jsonl(&output.stdout);
    let result = result_event(&events);
    assert_eq!(
        result["ok"],
        Value::Bool(true),
        "run failed: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}
// CODEGEN-END
