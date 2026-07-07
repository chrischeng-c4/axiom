// SPEC-MANAGED: projects/preview/external-contracts/behavior/ci-template-lifecycle.md#ci-template-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec ci-template-lifecycle
// @capability preview-external-contracts
// @claim ci-template-lifecycle
// @contract ci-template-lifecycle
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order
// AW-EC-END

// Contract: GitHub Actions, GitLab CI, and local kind templates define all required PREVIEW_* variables.
// Contract: Templates preserve open/update/rerun command order from discover-base through render, apply plan, dry-run, apply, rollout, router resolve, and comment.
// Contract: Templates preserve close/merge command order from cleanup plan to cleanup apply.
// Contract: The kind lifecycle gate validates the documented local path stays runnable.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn ci_template_lifecycle() {
    let command =
        "cargo test -p preview --test local_cicd_contract ci_templates_document_required_variables_and_command_order";
    let id = "ci-template-lifecycle";
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
