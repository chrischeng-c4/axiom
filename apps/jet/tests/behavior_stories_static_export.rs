// SPEC-MANAGED: .aw/tech-design/projects/jet/logic/jet-stories-build-static-export-of-the-component-workbench-for-h.md#stories-static-export
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec stories-static-export
// @capability component-workbench
// @claim stories-static-export
// @contract stories-static-export
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test stories_build -- --nocapture
// AW-EC-END

#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn stories_static_export() {
    let command = "cargo test -p jet --test stories_build -- --nocapture";
    let id = "stories-static-export";
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
