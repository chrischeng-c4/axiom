// SPEC-MANAGED: projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#vat-cluster-runscoped-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cluster-runscoped-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cluster-runscoped-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat vat_cluster_create_exports_kubeconfig -- --nocapture --ignored
// AW-EC-END

// Contract: with a real backend and Docker available, vat creates the cluster, the runner reaches the cluster via KUBECONFIG, and vat state shows services[].cluster.backend.
// Contract: the cluster is deleted at teardown under keep=never; the test skips gracefully when no backend/docker is present.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cluster_runscoped_smoke() {
    let command =
        "cargo test -p vat vat_cluster_create_exports_kubeconfig -- --nocapture --ignored";
    let id = "vat-cluster-runscoped-smoke";
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
