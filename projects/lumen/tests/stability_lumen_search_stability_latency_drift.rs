// SPEC-MANAGED: projects/lumen/external-contracts/search/stability/query-resilience.md#lumen-search-stability-latency-drift
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-stability-latency-drift
// @capability search
// @claim no-latency-drift-over-soak
// @contract search-stability-latency-drift
// @category stability
// @required_for_production false
// @command cargo test -p lumen --test stability_lumen_search_stability_latency_drift -- --ignored
// AW-EC-END

// Contract: GAP (f): search p99 per window over a long soak drifts <= ~10% (analogous to the RSS plateau gate). Scenario being added.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_stability_latency_drift() {
    let command = "cargo test -p lumen --test stability_lumen_search_stability_latency_drift -- --ignored";
    let id = "lumen-search-stability-latency-drift";
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
