// SPEC-MANAGED: apps/loom/tech-design/semantic/source/apps-loom-tests-cli-contract-rs.md#unit-test
// <HANDWRITE gap="missing-generator:test:loom-bootstrap" tracker="#541" reason="Initial CLI contract test.">
use std::process::Command;

fn loom_bin() -> &'static str {
    env!("CARGO_BIN_EXE_loom")
}

#[test]
fn help_ships_standard_and_control_commands() {
    let output = Command::new(loom_bin())
        .arg("--help")
        .output()
        .expect("run loom --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "controller",
        "worker",
        "spec",
        "llm",
        "upgrade",
        "issue",
    ] {
        assert!(stdout.contains(needle), "help should contain {needle}");
    }
}

#[test]
fn controller_cli_surface_help() {
    let help = Command::new(loom_bin())
        .args(["controller", "--help"])
        .output()
        .expect("run loom controller --help");
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("strongly-consistent DAG state"));
}

#[test]
fn worker_cli_surface_help() {
    let help = Command::new(loom_bin())
        .args(["worker", "--help"])
        .output()
        .expect("run loom worker --help");
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("Resident pull-loop worker harness"));
}

#[test]
fn llm_cli_surface_help() {
    let help = Command::new(loom_bin())
        .args(["llm", "--help"])
        .output()
        .expect("run loom llm --help");
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("outline"));
}

#[test]
fn llm_cli_outline_runs() {
    let output = Command::new(loom_bin())
        .arg("llm")
        .output()
        .expect("run loom llm");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("architecture"));
    assert!(stdout.contains("roles"));
    assert!(stdout.contains("control-api"));
}

#[test]
fn llm_cli_topic_runs() {
    let output = Command::new(loom_bin())
        .args(["llm", "architecture"])
        .output()
        .expect("run loom llm architecture");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("never traverse loom"));
}

#[test]
fn upgrade_cli_surface_help() {
    let help = Command::new(loom_bin())
        .args(["upgrade", "--help"])
        .output()
        .expect("run loom upgrade --help");
    assert!(help.status.success());
}

#[test]
fn upgrade_cli_check_runs() {
    let output = Command::new(loom_bin())
        .args(["upgrade", "--check"])
        .output()
        .expect("run loom upgrade --check");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("current:"));
}

#[test]
fn issue_cli_surface_help() {
    let help = Command::new(loom_bin())
        .args(["issue", "--help"])
        .output()
        .expect("run loom issue --help");
    assert!(help.status.success());
    let stdout = String::from_utf8_lossy(&help.stdout);
    assert!(stdout.contains("search"));
    assert!(stdout.contains("view"));
    assert!(stdout.contains("create"));
}

#[test]
fn spec_cli_openapi_runs_and_emits_json() {
    let output = Command::new(loom_bin())
        .args(["spec", "--format", "openapi"])
        .output()
        .expect("run loom spec --format openapi");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Parse the output as JSON to guarantee it is valid OpenAPI JSON schema
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("spec output should be valid JSON");
    assert_eq!(json["openapi"].as_str().unwrap_or(""), "3.0.3");
    assert!(json["paths"].is_object());
    assert!(json["components"].is_object());
}

#[test]
fn spec_gen_python_client_runs_successfully() {
    let out_dir = std::env::temp_dir().join(format!("loom-spec-gen-py-{}", std::process::id()));
    let output = Command::new(loom_bin())
        .args([
            "spec",
            "gen",
            "--lang",
            "py",
            "--out",
            out_dir.to_str().unwrap(),
        ])
        .output()
        .expect("run loom spec gen --lang py");
    assert!(output.status.success());
    assert!(out_dir.join("models.py").exists());
    assert!(out_dir.join("client.py").exists());
    let _ = std::fs::remove_dir_all(out_dir);
}

#[test]
fn worker_fails_without_env() {
    let output = Command::new(loom_bin())
        .arg("worker")
        .output()
        .expect("run loom worker");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires LOOM_KEEP"));
}

#[test]
fn issue_search_fails_offline() {
    let output = Command::new(loom_bin())
        .args(["issue", "search"])
        .output()
        .expect("run loom issue search");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no `online` feature"));
}

#[test]
fn issue_view_fails_offline() {
    let output = Command::new(loom_bin())
        .args(["issue", "view", "1"])
        .output()
        .expect("run loom issue view 1");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("no `online` feature"));
}

#[test]
fn issue_create_dry_run_offline() {
    let output = Command::new(loom_bin())
        .args(["issue", "create"])
        .output()
        .expect("run loom issue create");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let all = format!("{}\n{}", stdout, stderr);
    assert!(all.contains("cannot file directly"));
    assert!(all.contains("labels=project%3Aloom"));
}
// </HANDWRITE>
