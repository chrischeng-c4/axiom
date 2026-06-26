// SPEC-MANAGED: projects/rig/tech-design/logic/external-contracts.md#rig-vat-delegated-scenario-execution
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec rig-vat-delegated-scenario-execution
// @capability vat-wrapped-runs
// @claim vat-delegated-scenario-execution
// @contract vat-delegated-scenario-execution
// @category stability
// @required_for_production true
// @command cargo test -p rig
// AW-EC-END

// Contract: vat delegation tests pass
// Contract: rig keeps environment setup delegated to vat
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn rig_vat_delegated_scenario_execution() {
    let command = "cargo test -p rig";
    let id = "rig-vat-delegated-scenario-execution";
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
