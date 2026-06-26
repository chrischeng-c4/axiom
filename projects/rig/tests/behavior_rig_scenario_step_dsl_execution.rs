// SPEC-MANAGED: projects/rig/tech-design/logic/external-contracts.md#rig-scenario-step-dsl-execution
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec rig-scenario-step-dsl-execution
// @capability scenario-engine
// @claim scenario-step-dsl-execution
// @contract scenario-step-dsl-execution
// @category behavior
// @required_for_production true
// @command cargo test -p rig
// AW-EC-END

// Contract: scenario engine tests pass
// Contract: step DSL execution remains covered by rig's unit and e2e tests
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn rig_scenario_step_dsl_execution() {
    let command = "cargo test -p rig";
    let id = "rig-scenario-step-dsl-execution";
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
