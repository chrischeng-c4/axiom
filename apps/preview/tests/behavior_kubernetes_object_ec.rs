// SPEC-MANAGED: apps/preview/external-contracts/behavior/kubernetes-object-ec.md#kubernetes-object-ec
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec kubernetes-object-ec
// @capability preview-external-contracts
// @claim kubernetes-object-ec
// @contract kubernetes-object-ec
// @category behavior
// @required_for_production true
// @command cargo test -p preview --test k8s_object_contract
// AW-EC-END

// Contract: Rendered Kubernetes YAML parses as Namespace, ServiceAccount, ResourceQuota, LimitRange, Role, RoleBinding, Deployment, Service, and ConfigMap objects.
// Contract: Service selectors match Deployment pod labels.
// Contract: Rendered Deployment includes readiness/liveness probes, explicit Workload Identity service account, bounded resources, and base workload clone annotations.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn kubernetes_object_ec() {
    let command = "cargo test -p preview --test k8s_object_contract";
    let id = "kubernetes-object-ec";
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
