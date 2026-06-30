// SPEC-MANAGED: projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#vat-cluster-backend-unavailable-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cluster-backend-unavailable-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim local-kubernetes-cluster-service-and-vat-cluster
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_cluster -- --nocapture
// AW-EC-END

// Contract: a cluster service with no backend on PATH emits a cluster_backend_unavailable JSONL error and a non-zero exit.
// Contract: vat never panics on the unavailable path.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cluster_backend_unavailable_smoke() {
    let command = "cargo test -p vat --test vat_cluster -- --nocapture";
    let id = "vat-cluster-backend-unavailable-smoke";
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
