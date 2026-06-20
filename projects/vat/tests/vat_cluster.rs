// SPEC-MANAGED: projects/vat/tech-design/semantic/vat-tests.md#schema
// CODEGEN-BEGIN
//! Integration coverage for kind-like local Kubernetes clusters.
//!
//! The unavailable path is deterministic and always runs (empty PATH → no
//! backend, no Docker). The run-scoped and standalone lifecycle tests need a
//! real backend + Docker, so they are `#[ignore]` and also skip gracefully when
//! none is present.

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

fn any_cluster_backend() -> bool {
    let backend = ["kind", "k3d", "minikube"].iter().any(|bin| {
        Command::new(bin)
            .arg("version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    });
    let docker = Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    backend && docker
}

/// Delete a run-scoped cluster left behind by a `keep = "always"` run.
fn delete_cluster(backend: &str, name: &str) {
    let mut cmd = Command::new(backend);
    match backend {
        "kind" => {
            cmd.args(["delete", "cluster", "--name", name]);
        }
        "k3d" => {
            cmd.args(["cluster", "delete", name]);
        }
        "minikube" => {
            cmd.args(["delete", "-p", name]);
        }
        _ => return,
    }
    let _ = cmd.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[test]
fn cluster_backend_unavailable_reports_jsonl_error() {
    // A `cluster = "auto"` service with an empty PATH (no kind/k3d/minikube and
    // no docker) must emit the structured `cluster_backend_unavailable`
    // envelope and fail — never a panic.
    let project = tempfile::tempdir().unwrap();
    let vat_home = tempfile::tempdir().unwrap();
    std::fs::write(
        project.path().join("vat.toml"),
        r#"
version = 1

[[services]]
id = "k8s"
cluster = "auto"

[[runners]]
id = "test"
requires = ["k8s"]
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
                && event["code"] == "cluster_backend_unavailable"
                && event["service"] == "k8s"
        }),
        "missing cluster_backend_unavailable event: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

#[test]
fn llm_guide_mentions_cluster() {
    let output = Command::new(vat_bin()).arg("llm").output().unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in ["vat cluster", "cluster = \"auto\"", "KUBECONFIG"] {
        assert!(stdout.contains(needle), "llm guide missing `{needle}`");
    }
}

#[test]
#[ignore = "requires a local kubernetes backend (kind/k3d/minikube) + docker"]
fn vat_cluster_create_exports_kubeconfig() {
    if !any_cluster_backend() {
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
id = "k8s"
cluster = "auto"
timeout_s = 600
export = { KUBECONFIG = "{kubeconfig}" }

[[runners]]
id = "probe"
requires = ["k8s"]
cmd = ["sh", "-c", "test -n \"$KUBECONFIG\" && kubectl --kubeconfig \"$KUBECONFIG\" get nodes"]
timeout_s = 600
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
    let backend = service["cluster"]["backend"].as_str().unwrap_or_default();
    assert!(
        ["kind", "k3d", "minikube"].contains(&backend),
        "state missing cluster backend: {}",
        String::from_utf8_lossy(&state_out.stdout)
    );
    let name = service["cluster"]["name"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    let exported = service["exported_env"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(
        exported.iter().any(|v| v == "KUBECONFIG"),
        "KUBECONFIG not exported to the runner"
    );

    // keep = "always" retains the cluster; clean it up so nothing leaks.
    delete_cluster(backend, &name);
}

#[test]
#[ignore = "requires a local kubernetes backend (kind/k3d/minikube) + docker"]
fn vat_cluster_standalone_lifecycle() {
    if !any_cluster_backend() {
        return;
    }
    let vat_home = tempfile::tempdir().unwrap();
    let name = format!("vat-it-{}", std::process::id());

    let create = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["cluster", "create", "--name", &name, "--json"])
        .output()
        .unwrap();
    assert!(
        create.status.success(),
        "create failed: {}",
        String::from_utf8_lossy(&create.stdout)
    );

    let ls = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["cluster", "ls", "--json"])
        .output()
        .unwrap();
    let list: Value = serde_json::from_slice(&ls.stdout).unwrap();
    assert!(
        list.as_array()
            .unwrap()
            .iter()
            .any(|c| c["name"] == name.as_str()),
        "cluster not listed: {}",
        String::from_utf8_lossy(&ls.stdout)
    );

    let kubeconfig = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["cluster", "kubeconfig", &name])
        .output()
        .unwrap();
    assert!(kubeconfig.status.success());
    assert!(String::from_utf8_lossy(&kubeconfig.stdout)
        .trim()
        .ends_with("kubeconfig"));

    let delete = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["cluster", "delete", &name])
        .output()
        .unwrap();
    assert!(delete.status.success());

    let ls_after = Command::new(vat_bin())
        .env("VAT_HOME", vat_home.path())
        .args(["cluster", "ls", "--json"])
        .output()
        .unwrap();
    let list_after: Value = serde_json::from_slice(&ls_after.stdout).unwrap();
    assert!(!list_after
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["name"] == name.as_str()));
}
// CODEGEN-END
