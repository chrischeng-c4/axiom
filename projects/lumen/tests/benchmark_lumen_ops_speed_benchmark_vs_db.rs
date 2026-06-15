// SPEC-MANAGED: projects/lumen/external-contracts/ops-operability/efficiency/competitive-search-benchmark-vs-db.md#lumen-ops-speed-benchmark-vs-db
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-ops-speed-benchmark-vs-db
// @capability ops-operability
// @claim competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @contract competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @category efficiency
// @required_for_production false
// @command cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1
// AW-EC-END

// Contract: lumen wins the contracted search-latency cells against Postgres (text_bm25 WIN; ratcheted floor holds).
// Contract: floor-dominated cells (pg btree point-lookup) stay EXEMPT, not gated.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_ops_speed_benchmark_vs_db() {
    let command = "cargo test -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1";
    let id = "lumen-ops-speed-benchmark-vs-db";
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
