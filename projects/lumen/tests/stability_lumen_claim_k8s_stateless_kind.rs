// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-k8s-stateless-kind
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-k8s-stateless-kind
// @capability kubernetes-native-deployment
// @claim stateless-serving-rebuild-from-log-no-pvc
// @contract k8s-stateless-kind-dogfood
// @category stability
// @required_for_production true
// @command projects/lumen/scripts/kind-e2e.sh
// AW-EC-END

// Contract: The live kind dogfood path proves stateless serving rebuilds from log without a serving PVC.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_k8s_stateless_kind() {
    let command = "projects/lumen/scripts/kind-e2e.sh";
    let id = "lumen-claim-k8s-stateless-kind";
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
