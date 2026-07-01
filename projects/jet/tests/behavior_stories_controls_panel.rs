// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-controls-args-panel-derived-from-ts-prop-types-argty.md#stories-controls-panel
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec stories-controls-panel
// @capability component-workbench
// @claim stories-controls-panel
// @contract stories-controls-panel
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test controls -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn stories_controls_panel() {
    let command = "cargo test -p jet --test controls -- --nocapture";
    let id = "stories-controls-panel";
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
