// SPEC-MANAGED: projects/vat/tech-design/logic/kind-like-local-kubernetes-clusters.md#vat-cluster-standalone-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cluster-standalone-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim vat-cluster-standalone-smoke
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_cluster -- --nocapture --include-ignored
// AW-EC-END

// Contract: vat cluster create then ls --json lists the cluster, kubeconfig prints a usable path, and delete removes it from the registry and the backend.
// Contract: the test skips gracefully when no backend/docker is present.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cluster_standalone_smoke() {
    let command = "cargo test -p vat --test vat_cluster -- --nocapture --include-ignored";
    let id = "vat-cluster-standalone-smoke";
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
