// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-discovery-csf2-template-bind-re-exported-stories-and.md#csf2-template-bind-re-exports
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec csf2-template-bind-re-exports
// @capability component-workbench
// @claim csf2-template-bind-re-exports
// @contract csf2-template-bind-re-exports
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test csf_discovery -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn csf2_template_bind_re_exports() {
    let command = "cargo test -p jet --test csf_discovery -- --nocapture";
    let id = "csf2-template-bind-re-exports";
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
