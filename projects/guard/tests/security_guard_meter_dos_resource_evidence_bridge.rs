// SPEC-MANAGED: projects/guard/tech-design/semantic/guard-ec-static-security-smoke.md#guard-meter-dos-resource-evidence-bridge
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guard-meter-dos-resource-evidence-bridge
// @capability dynamic-security-evidence
// @claim meter-dos-resource-evidence-bridge
// @contract meter-dos-resource-evidence-bridge
// @category security
// @required_for_production true
// @command target/debug/guard scan projects/guard --compact --no-persist --meter-target projects/guard
// AW-EC-END

// Contract: guard can fold meter resource evidence into its report
// Contract: resource-abuse evidence remains visible in guard.report/1
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guard_meter_dos_resource_evidence_bridge() {
    let command = "target/debug/guard scan projects/guard --compact --no-persist --meter-target projects/guard";
    let id = "guard-meter-dos-resource-evidence-bridge";
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
