// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-http2-ops-route-list
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-http2-ops-route-list
// @capability http2-api-list
// @claim ops-metadata-probe-and-metrics-route-list
// @contract http2-ops-route-list
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e -- --nocapture
// AW-EC-END

// Contract: Health, readiness, OpenAPI, metrics, and version routes are exposed and exercised.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_http2_ops_route_list() {
    let command = "cargo test -p lumen --test api_e2e -- --nocapture";
    let id = "lumen-claim-http2-ops-route-list";
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
