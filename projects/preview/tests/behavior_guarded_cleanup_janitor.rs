// SPEC-MANAGED: projects/preview/external-contracts/behavior/guarded-cleanup-janitor.md#guarded-cleanup-janitor
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec guarded-cleanup-janitor
// @capability gke-uat-preview-environment-rendering
// @claim guarded-cleanup-janitor
// @contract guarded-cleanup-janitor
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract local_cleanup_janitor_plan_reports_guarded_actions
// AW-EC-END

// Contract: `preview cleanup plan` emits keep, drain, and delete decisions from MR/TTL/orphan state.
// Contract: Protected base/control namespaces are reported as skipped and are not deleted.
// Contract: `preview cleanup apply --plan <json>` is covered by the kind lifecycle gate and deletes only preview namespaces and route-binding ConfigMaps.
// Contract: Repeated cleanup runs are idempotent.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn guarded_cleanup_janitor() {
    let command =
        "cargo test -p preview --test local_cicd_contract local_cleanup_janitor_plan_reports_guarded_actions";
    let id = "guarded-cleanup-janitor";
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
