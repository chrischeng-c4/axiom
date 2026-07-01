// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-controls-generic-cross-file-and-intersection-prop-ty.md#generic-cross-file-prop-controls
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec generic-cross-file-prop-controls
// @capability component-workbench
// @claim generic-cross-file-prop-controls
// @contract generic-cross-file-prop-controls
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test controls -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn generic_cross_file_prop_controls() {
    let command = "cargo test -p jet --test controls -- --nocapture";
    let id = "generic-cross-file-prop-controls";
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
