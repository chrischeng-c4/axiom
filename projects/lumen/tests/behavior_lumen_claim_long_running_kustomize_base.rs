// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-long-running-kustomize-base
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-long-running-kustomize-base
// @capability long-running-stability
// @claim kustomize-base-overlays-hpa
// @contract long-running-kustomize-base-overlays
// @category behavior
// @required_for_production true
// @command kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator
// AW-EC-END

// Contract: The base, dev, staging, prod, and operator kustomize surfaces render valid Kubernetes manifests.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_long_running_kustomize_base() {
    let command =
        "kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator";
    let id = "lumen-claim-long-running-kustomize-base";
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
