// SPEC-MANAGED: .aw/tech-design/projects/jet/validate/complete-package-management-replacement-gate-before-build.md#package-phase-gate
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec package-phase-gate
// @capability package-manager
// @claim package-manager-readiness
// @contract package-phase-gate
// @category behavior
// @required_for_production true
// @command projects/jet/scripts/verify-basic-dom-gates.sh --phase package
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn package_phase_gate() {
    let command = "projects/jet/scripts/verify-basic-dom-gates.sh --phase package";
    let id = "package-phase-gate";
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
