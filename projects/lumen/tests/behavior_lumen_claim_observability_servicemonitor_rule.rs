// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-observability-servicemonitor-rule
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-observability-servicemonitor-rule
// @capability observability
// @claim servicemonitor-prometheusrule-bundle
// @contract observability-servicemonitor-rule
// @category behavior
// @required_for_production true
// @command kustomize build projects/lumen/k8s/overlays/prod
// AW-EC-END

// Contract: The production overlay renders the ServiceMonitor and PrometheusRule bundle.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_observability_servicemonitor_rule() {
    let command = "kustomize build projects/lumen/k8s/overlays/prod";
    let id = "lumen-claim-observability-servicemonitor-rule";
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
