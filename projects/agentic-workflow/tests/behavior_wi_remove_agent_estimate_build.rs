// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-wi-remove-agent-estimate.md#wi-remove-agent-estimate-build
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec wi-remove-agent-estimate-build
// @capability unmapped
// @claim wi-remove-agent-estimate-build
// @contract wi-remove-agent-estimate-build
// @category behavior
// @required_for_production true
// @command cargo build -p agentic-workflow --bin aw
// AW-EC-END

// Contract: the aw binary builds after removing estimate helpers
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn wi_remove_agent_estimate_build() {
    let command = "cargo build -p agentic-workflow --bin aw";
    let id = "wi-remove-agent-estimate-build";
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
