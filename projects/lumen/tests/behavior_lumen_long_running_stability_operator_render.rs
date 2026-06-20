// SPEC-MANAGED: projects/lumen/external-contracts/long-running-stability/behavior/devops-render.md#lumen-long-running-stability-operator-render
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-operator-render
// @capability long-running-stability
// @claim lumen-crd-reconcile-loop-kube-rs-operator
// @contract devops-operator-render-golden
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --features operator --test operator_render -- --nocapture
// AW-EC-END

// Contract: render(Lumen) emits the managed serving Deployment/Service/HPA/PDB plus the NATS JetStream ConfigMap/StatefulSet/Service when NATS is managed.
// Contract: BYO-NATS (nats.externalUrl) omits the managed NATS objects and wires the external URL into the serving env.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_operator_render() {
    let command = "cargo test -p lumen --features operator --test operator_render -- --nocapture";
    let id = "lumen-long-running-stability-operator-render";
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
