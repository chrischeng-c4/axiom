// SPEC-MANAGED: projects/preview/external-contracts/behavior/kind-gke-lifecycle-ec.md#kind-gke-lifecycle-ec
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec kind-gke-lifecycle-ec
// @capability preview-external-contracts
// @claim kind-gke-lifecycle-ec
// @contract kind-gke-lifecycle-ec
// @category behavior
// @required_for_production true
// @command PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture
// AW-EC-END

// Contract: When a kind/GKE kubectl context is configured, preview apply performs direct apply, server-side dry-run after namespace creation, idempotent re-apply, rollout, endpoint checks, and /readyz port-forward smoke.
// Contract: The kind/GKE gate creates a base Deployment/Service fixture and runs preview discover-base before rendering the preview namespace.
// Contract: The kind/GKE gate validates namespace-local workload RBAC and rejects an oversized pod through ResourceQuota/LimitRange admission.
// Contract: The kind/GKE gate cleans temporary preview/control namespaces after success or failure.
// Contract: Without a configured kubectl context, the test reports an explicit skip instead of falsely applying to an unknown cluster.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn kind_gke_lifecycle_ec() {
    let command = "PREVIEW_KIND_E2E=1 cargo test -p preview --test kind_lifecycle -- --nocapture";
    let id = "kind-gke-lifecycle-ec";
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
