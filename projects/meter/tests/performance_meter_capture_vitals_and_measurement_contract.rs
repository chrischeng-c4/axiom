// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-capture-vitals-and-measurement-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-capture-vitals-and-measurement-contract
// @capability runtime-resource-attribution
// @claim capture-vitals-and-measurement-contract
// @contract capture-vitals-and-measurement-contract
// @category performance
// @required_for_production false
// @command cargo test -p meter capture::vitals
// AW-EC-END

// Contract: capture vitals tests pass
// Contract: cpu time, wall time, and peak RSS remain available as the L1 measurement contract
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_capture_vitals_and_measurement_contract() {
    let command = "cargo test -p meter capture::vitals";
    let id = "meter-capture-vitals-and-measurement-contract";
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
