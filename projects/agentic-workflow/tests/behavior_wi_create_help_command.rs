// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-wi-create-remove-remote-flag.md#wi-create-help-command
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec wi-create-help-command
// @capability unmapped
// @claim wi-create-help-command
// @contract wi-create-help-command
// @category behavior
// @required_for_production true
// @command ./target/debug/aw wi create --help
// AW-EC-END

// Contract: help output does not list --remote
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn wi_create_help_command() {
    let command = "./target/debug/aw wi create --help";
    let id = "wi-create-help-command";
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
