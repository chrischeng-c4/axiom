// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/common_change_spec/preamble.md#project-capability-define
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec project-capability-define
// @capability capability-control-plane
// @claim project-capability-define
// @contract capability-define-onboarding
// @category product-journey
// @required_for_production false
// @command vat run manual --gpu none
// @evaluator capability-agent-eval tool=codex command=codex exec --json --output e2e-results/agent-eval/project-capability-define.json report=e2e-results/agent-eval/project-capability-define.json
// AW-EC-END

// Contract: user can complete the capability definition journey
// Contract: visual evidence captures the primary UI states
// Contract: agent evaluation report has no blocking violations
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn project_capability_define() {
    let command = "vat run manual --gpu none";
    let id = "project-capability-define";
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
