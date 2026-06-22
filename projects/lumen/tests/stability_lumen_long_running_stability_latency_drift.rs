// SPEC-MANAGED: projects/lumen/external-contracts/long-running-stability/stability/query-resilience.md#lumen-long-running-stability-latency-drift
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-latency-drift
// @capability long-running-stability
// @claim no-latency-drift-over-soak
// @contract search-stability-latency-drift
// @category stability
// @required_for_production true
// @command cd projects/lumen && ../../target/debug/vat run rig-endurance
// AW-EC-END

// Contract: (f) search p99 per window over the soak drifts <= 1.10x + 6ms (rig endurance/soak_p99_drift.toml). Env-dependent (vat-provisioned lumen).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_latency_drift() {
    let command = "cd projects/lumen && ../../target/debug/vat run rig-endurance";
    let id = "lumen-long-running-stability-latency-drift";
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
