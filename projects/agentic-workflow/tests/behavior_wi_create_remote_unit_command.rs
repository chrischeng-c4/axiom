// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-wi-create-remove-remote-flag.md#wi-create-remote-unit-command
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec wi-create-remote-unit-command
// @capability unmapped
// @claim wi-create-remote-unit-command
// @contract wi-create-remote-unit-command
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow wi_create_remote -- --nocapture
// AW-EC-END

// Contract: help hides deprecated remote flag
// Contract: compatibility flag parses
// Contract: backend decision is config-driven
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn wi_create_remote_unit_command() {
    let command = "cargo test -p agentic-workflow wi_create_remote -- --nocapture";
    let id = "wi-create-remote-unit-command";
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
