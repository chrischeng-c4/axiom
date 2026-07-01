// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-discover-and-parse-csf-stories-tsx-into-a-story-inde.md#csf-story-discovery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec csf-story-discovery
// @capability component-workbench
// @claim csf-story-discovery
// @contract csf-story-discovery
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test csf_discovery -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn csf_story_discovery() {
    let command = "cargo test -p jet --test csf_discovery -- --nocapture";
    let id = "csf-story-discovery";
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
