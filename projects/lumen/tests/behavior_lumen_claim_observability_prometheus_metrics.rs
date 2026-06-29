// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-observability-prometheus-metrics
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-observability-prometheus-metrics
// @capability observability
// @claim prometheus-metrics-endpoint
// @contract observability-prometheus-metrics
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e metrics_exposes_prometheus_text -- --exact --nocapture
// AW-EC-END

// Contract: The /metrics endpoint emits Prometheus text with the expected scrape content type.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_observability_prometheus_metrics() {
    let command =
        "cargo test -p lumen --test api_e2e metrics_exposes_prometheus_text -- --exact --nocapture";
    let id = "lumen-claim-observability-prometheus-metrics";
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
