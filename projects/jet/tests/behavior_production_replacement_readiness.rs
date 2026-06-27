// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3778.md#production-replacement-readiness
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec production-replacement-readiness
// @capability rust-native-frontend-toolchain
// @claim production-replacement-readiness
// @contract production-replacement-readiness
// @category behavior
// @required_for_production true
// @command projects/jet/scripts/verify-basic-dom-gates.sh --all
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn production_replacement_readiness() {
    let command = "projects/jet/scripts/verify-basic-dom-gates.sh --all";
    let id = "production-replacement-readiness";
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
