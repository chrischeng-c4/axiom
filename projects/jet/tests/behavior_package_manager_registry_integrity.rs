// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3779.md#package-manager-registry-integrity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec package-manager-registry-integrity
// @capability package-manager
// @claim package-manager-registry-integrity
// @contract package-manager-registry-integrity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib pkg_manager -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn package_manager_registry_integrity() {
    let command = "cargo test -p jet --lib pkg_manager -- --nocapture";
    let id = "package-manager-registry-integrity";
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
