// SPEC-MANAGED: projects/vat/tech-design/logic/production-like-integration-scenarios.md#scenario-run-starts-app-dependency-and-runner
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec scenario-run-starts-app-dependency-and-runner
// @capability agent-native-gpu-native-dev-containers
// @claim production-like-integration-scenarios
// @contract production-like-integration-scenarios
// @category behavior
// @required_for_production true
// @command cargo test -p vat scenario_run_starts_app_dependency_and_runner -- --nocapture
// AW-EC-END

// Contract: vat run --scenario prod-like succeeds
// Contract: app readiness marker exists before runner marker
// Contract: vat state includes test_run.scenario id/app/runner/services
// Contract: result JSONL includes scenario and app
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn scenario_run_starts_app_dependency_and_runner() {
    let command = "cargo test -p vat scenario_run_starts_app_dependency_and_runner -- --nocapture";
    let id = "scenario-run-starts-app-dependency-and-runner";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .status()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    assert!(
        status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}",
        status.code()
    );
}
// CODEGEN-END
