// SPEC-MANAGED: projects/meter/tech-design/logic/external-contracts.md#meter-profile-phase-boundary-cost-report
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec meter-profile-phase-boundary-cost-report
// @capability runtime-resource-attribution
// @claim profile-phase-boundary-cost-report
// @contract profile-phase-boundary-cost-report
// @category performance
// @required_for_production false
// @command cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json
// AW-EC-END

// Contract: meter profile emits ranked boundary-cost findings from a serialized PhaseBreakdown
// Contract: the deterministic profile path remains agent-readable JSON without requiring sampler privileges
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn meter_profile_phase_boundary_cost_report() {
    let command =
        "cargo run -p meter-cli --bin meter -- profile --phases projects/meter/tests/fixtures/profile_phase_breakdown.json";
    let id = "meter-profile-phase-boundary-cost-report";
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
