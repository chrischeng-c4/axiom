// SPEC-MANAGED: apps/preview/external-contracts/behavior/local-apply-gitops-execution.md#local-apply-gitops-execution-ec
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec local-apply-gitops-execution-ec
// @capability preview-external-contracts
// @claim local-apply-gitops-execution-ec
// @contract local-apply-gitops-execution-ec
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract local_apply_plan_and_gitops_bundle_are_deterministic
// AW-EC-END

// Contract: `preview render` emits plans/manifest-inventory.json with deterministic Kubernetes object order.
// Contract: `preview apply --dir <rendered-dir> --plan-only` prints an MR-comment-friendly ordered apply summary without contacting a cluster.
// Contract: `preview gitops render --dir <rendered-dir> --out <bundle-dir>` writes a deterministic relative-path GitOps bundle with no local absolute paths.
// Contract: The kind lifecycle gate covers `preview apply --dry-run`, direct apply, idempotent re-apply, and rollout against a local cluster.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn local_apply_gitops_execution_ec() {
    let command =
        "cargo test -p preview --test local_cicd_contract local_apply_plan_and_gitops_bundle_are_deterministic";
    let id = "local-apply-gitops-execution-ec";
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
