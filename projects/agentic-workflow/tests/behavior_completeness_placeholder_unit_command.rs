// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-completeness-placeholder-gate.md#completeness-placeholder-unit-command
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec completeness-placeholder-unit-command
// @capability td-cb-lifecycle-automation
// @claim cb-lifecycle-dispatch
// @contract completeness-placeholder-unit-command
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow completeness_placeholder -- --nocapture
// AW-EC-END

// Contract: placeholder code is rejected
// Contract: omitted prose is rejected
// Contract: future_work_allowed TODO is accepted
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn completeness_placeholder_unit_command() {
    let command = "cargo test -p agentic-workflow completeness_placeholder -- --nocapture";
    let id = "completeness-placeholder-unit-command";
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
