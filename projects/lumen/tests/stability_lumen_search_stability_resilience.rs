// SPEC-MANAGED: projects/lumen/external-contracts/search/stability/query-resilience.md#lumen-search-stability-resilience
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-search-stability-resilience
// @capability search
// @claim search-p99-survives-fault-and-recovers
// @contract search-stability-fault-resilience
// @category stability
// @required_for_production true
// @command rig run --dir projects/lumen/tests/rig/cases/resilience
// AW-EC-END

// Contract: FILTERING/RANKING: under 5% packet loss (toxiproxy timeout toxic) search p99 stays <= 2x baseline_p99 + 20ms.
// Contract: ALL: after a full network partition, search recovers within 10s and post-recovery p99 stays <= 2x baseline_p99 + 1ms.
// Contract: RSS plateau: over the bounded-keyspace soak, window-2 RSS <= 1.10x window-1 RSS (no leak).
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_search_stability_resilience() {
    let command = "rig run --dir projects/lumen/tests/rig/cases/resilience";
    let id = "lumen-search-stability-resilience";
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
