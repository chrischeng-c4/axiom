// SPEC-MANAGED: projects/lumen/external-contracts/competitor-performance/efficiency/competitive-benchmark.md#lumen-competitor-performance-competitive
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-competitor-performance-competitive
// @capability competitor-performance
// @claim competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @contract search-efficiency-filtering-ranking-pagination
// @category efficiency
// @required_for_production true
// @command target/debug/meter test -- -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1
// AW-EC-END

// Contract: FILTERING: filtered_search (AND[BM25+term+range]) beats pg >= 3.73x and OpenSearch(disk) >= 2.4x; filtered_knn beats pg >= 2.4x (OS exempt, no kNN plugin).
// Contract: RANKING: text_bm25 single-term beats pg >= 14.56x; text_and multi-term beats pg >= 1.47x (OS >= 2.4x each).
// Contract: PAGINATION/SORT: pure_sort (scan + sort) beats pg >= 18.32x (OS >= 2.4x); cursor pagination stays within the search_qps pin.
// Contract: Floors are ratcheted (perf-baseline.json, 0.8); btree point-lookup cells stay EXEMPT, not gated. lumen vs pg/OS only.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_competitor_performance_competitive() {
    let command =
        "target/debug/meter test -- -p lumen --release --test perf_gate_vs_db -- --ignored --test-threads=1";
    let id = "lumen-competitor-performance-competitive";
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
