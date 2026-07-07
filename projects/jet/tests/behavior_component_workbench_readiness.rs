// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-dev-command-with-native-manager-ui-sidebar-preview-t.md#component-workbench-readiness
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec component-workbench-readiness
// @capability component-workbench
// @claim component-workbench-readiness
// @contract component-workbench-readiness
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test stories_build -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn component_workbench_readiness() {
    let command = "cargo test -p jet --test stories_build -- --nocapture";
    let id = "component-workbench-readiness";
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
