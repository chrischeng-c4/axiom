// SPEC-MANAGED: projects/preview/external-contracts/behavior/cleanup-dry-run-planning.md#cleanup-dry-run-planning
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cleanup-dry-run-planning
// @capability gke-uat-preview-environment-rendering
// @claim cleanup-dry-run-planning
// @contract cleanup-dry-run-planning
// @category behavior
// @required_for_production true
// @command cargo test -p preview cleanup_plan_marks_closed_mr_for_namespace_delete
// AW-EC-END

// Contract: Closed MR cleanup plans delete both the preview namespace and route binding.
// Contract: Cleanup output keeps the route target and namespace explicit for SRE review.
// Contract: Cleanup output lists the base namespace and control namespace as protected namespaces.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cleanup_dry_run_planning() {
    let command = "cargo test -p preview cleanup_plan_marks_closed_mr_for_namespace_delete";
    let id = "cleanup-dry-run-planning";
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
