// SPEC-MANAGED: apps/preview/external-contracts/behavior/base-workload-discovery.md#base-workload-discovery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec base-workload-discovery
// @capability gke-uat-preview-environment-rendering
// @claim base-workload-discovery
// @contract base-workload-discovery
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test base_discovery_contract
// AW-EC-END

// Contract: Base discovery normalizes Kubernetes Deployment and Service JSON fixtures into a BaseWorkloadContract.
// Contract: Discovery preserves cloneable selector, port, env, probe, and resource fields while excluding runtime identity and cluster-assigned fields.
// Contract: Discovery refuses ambiguous multi-container Deployments without a container matching the requested app.
// Contract: Render can consume a discovered base contract and embed it in plans/workload-clone.json without a live cluster.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn base_workload_discovery() {
    let command = "cargo test -p preview --test base_discovery_contract";
    let id = "base-workload-discovery";
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
