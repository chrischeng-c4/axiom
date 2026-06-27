// SPEC-MANAGED: .aw/tech-design/projects/jet/specs/3779.md#package-manager-workspace-parity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec package-manager-workspace-parity
// @capability package-manager
// @claim package-manager-workspace-parity
// @contract package-manager-workspace-parity
// @category behavior
// @required_for_production true
// @command cargo test -p jet --lib pkg_manager::workspace -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn package_manager_workspace_parity() {
    let command = "cargo test -p jet --lib pkg_manager::workspace -- --nocapture";
    let id = "package-manager-workspace-parity";
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
