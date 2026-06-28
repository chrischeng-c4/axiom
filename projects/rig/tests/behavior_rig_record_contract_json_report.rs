// SPEC-MANAGED: projects/rig/tech-design/logic/external-contracts.md#rig-record-contract-json-report
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec rig-record-contract-json-report
// @capability scenario-engine
// @claim record-contract-check-and-json-report
// @contract record-contract-check-and-json-report
// @category behavior
// @required_for_production true
// @command cargo test -p rig
// AW-EC-END

// Contract: rig record-contract and report tests pass
// Contract: rig.report/1 remains the single agent-readable output contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn rig_record_contract_json_report() {
    let command = "cargo test -p rig";
    let id = "rig-record-contract-json-report";
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
