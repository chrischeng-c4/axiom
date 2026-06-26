// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md#wi-remove-agent-estimate-spec-check
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec wi-remove-agent-estimate-spec-check
// @capability work-item-planning
// @claim capability-to-epic-planning
// @contract wi-remove-agent-estimate-spec-check
// @category behavior
// @required_for_production true
// @command ./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md
// AW-EC-END

// Contract: the canonical contract remains parseable by td check
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn wi_remove_agent_estimate_spec_check() {
    let command =
        "./target/debug/aw td check projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md";
    let id = "wi-remove-agent-estimate-spec-check";
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
