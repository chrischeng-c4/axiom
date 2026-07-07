// SPEC-MANAGED: projects/preview/external-contracts/behavior/local-fake-gcp-data-lifecycle.md#local-fake-gcp-data-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec local-fake-gcp-data-lifecycle
// @capability gke-uat-preview-environment-rendering
// @claim local-fake-gcp-data-lifecycle
// @contract local-fake-gcp-data-lifecycle
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test local_cicd_contract local_data_plan_fake_provider_and_secret_rewrite_are_deterministic
// AW-EC-END

// Contract: `preview render --data-*` emits plans/data-plan.json and k8s/data-secret.yaml only when a data contract is supplied.
// Contract: The data plan models a fake GCP Cloud SQL preview resource with read-only source, preview-* target naming, TTL, and ownership guardrails.
// Contract: The rendered Deployment rewrites DATABASE_URL to the namespace-local preview database Secret.
// Contract: `preview data apply` and `preview data cleanup` mutate only fake provider state and are idempotent for local CI.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn local_fake_gcp_data_lifecycle() {
    let command =
        "cargo test -p preview --test local_cicd_contract local_data_plan_fake_provider_and_secret_rewrite_are_deterministic";
    let id = "local-fake-gcp-data-lifecycle";
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
