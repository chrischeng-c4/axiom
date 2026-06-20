// SPEC-MANAGED: projects/lumen/external-contracts/long-running-stability/stability/resilience-survival.md#lumen-long-running-stability-resilience-survival
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-resilience-survival
// @capability long-running-stability
// @claim search-p99-survives-fault-and-recovers
// @contract search-p99-survives-fault-and-recovers
// @category stability
// @required_for_production true
// @command cargo test -p lumen --test drop_drain_e2e --test reindex_stream_e2e -- --nocapture
// AW-EC-END

// Contract: Search p99 stays within 2x baseline under 5% packet loss (toxiproxy timeout toxic; rig resilience scenario).
// Contract: Search survives a full network partition and recovers within budget; post-recovery p99 stays within 2x baseline.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_resilience_survival() {
    let command =
        "cargo test -p lumen --test drop_drain_e2e --test reindex_stream_e2e -- --nocapture";
    let id = "lumen-long-running-stability-resilience-survival";
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
