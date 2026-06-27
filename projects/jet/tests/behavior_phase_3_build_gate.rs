// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/remove-optional-esbuild-from-phase-3-production-build-gate.md#phase-3-build-gate
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec phase-3-build-gate
// @capability bundler-production-build
// @claim bundler-production-readiness
// @contract phase-3-build-gate
// @category behavior
// @required_for_production true
// @command projects/jet/scripts/verify-basic-dom-gates.sh --phase build
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn phase_3_build_gate() {
    let command = "projects/jet/scripts/verify-basic-dom-gates.sh --phase build";
    let id = "phase-3-build-gate";
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
