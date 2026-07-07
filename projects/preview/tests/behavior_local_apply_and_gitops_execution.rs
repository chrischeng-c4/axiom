// SPEC-MANAGED: projects/preview/external-contracts/behavior/local-apply-and-gitops-execution.md#local-apply-and-gitops-execution
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec local-apply-and-gitops-execution
// @capability gke-uat-preview-environment-rendering
// @claim local-apply-and-gitops-execution
// @contract local-apply-and-gitops-execution
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract
// AW-EC-END

// Contract: `preview render` emits plans/manifest-inventory.json with deterministic Kubernetes object order.
// Contract: `preview apply --dir <rendered-dir> --plan-only` prints an ordered summary for MR comments and CI logs.
// Contract: `preview gitops render --dir <rendered-dir> --out <bundle-dir>` writes deterministic relative-path bundle artifacts.
// Contract: `preview apply` is covered against a local cluster by the kind lifecycle gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn local_apply_and_gitops_execution() {
    let command = "cargo test -p preview --test local_cicd_contract";
    let id = "local-apply-and-gitops-execution";
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
