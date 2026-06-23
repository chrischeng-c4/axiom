// SPEC-MANAGED: projects/lumen/external-contracts/long-running-stability/behavior/meta-api.md#lumen-long-running-stability-meta-api
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-meta-api
// @capability long-running-stability
// @claim meta-api-health-ready-metrics-version
// @contract ops-meta-api-surface
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e -- --nocapture
// AW-EC-END

// Contract: GET /healthz (liveness) returns 200 always; GET /readyz returns 200 normally and 503 while draining; both bypass auth.
// Contract: GET /metrics returns Prometheus text v0.0.4 with the scrape content-type; bypasses auth.
// Contract: GET /version returns 200 with the build version (and git SHA / build time when available); bypasses auth.
// Contract: The k8s livenessProbe/startupProbe (/healthz) and readinessProbe (/readyz) paths match the actual endpoints; the Prometheus scrape path (/metrics) matches.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_meta_api() {
    let command = "cargo test -p lumen --test api_e2e -- --nocapture";
    let id = "lumen-long-running-stability-meta-api";
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
