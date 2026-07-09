// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-dev-command-with-native-manager-ui-sidebar-preview-t.md#stories-dev-manager
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec stories-dev-manager
// @capability component-workbench
// @claim stories-dev-manager
// @contract stories-dev-manager
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test manager -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn stories_dev_manager() {
    let command = "cargo test -p jet --test manager -- --nocapture";
    let id = "stories-dev-manager";
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
