// SPEC-MANAGED: projects/vat/tech-design/logic/production-like-integration-scenarios.md#scenario-failure-keeps-topology-and-logs
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec scenario-failure-keeps-topology-and-logs
// @capability agent-native-gpu-native-dev-containers
// @claim production-like-integration-scenarios
// @contract production-like-integration-scenarios
// @category behavior
// @required_for_production true
// @command cargo test -p vat scenario_failure_keeps_topology_and_logs -- --nocapture
// AW-EC-END

// Contract: failing runner forwards its exit code
// Contract: keep=failed retains the vat directory
// Contract: vat logs exposes runner output
// Contract: vat state exposes scenario topology
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn scenario_failure_keeps_topology_and_logs() {
    let command = "cargo test -p vat scenario_failure_keeps_topology_and_logs -- --nocapture";
    let id = "scenario-failure-keeps-topology-and-logs";
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
