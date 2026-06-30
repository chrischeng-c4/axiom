// SPEC-MANAGED: projects/lumen/external-contracts/competitor-performance/efficiency/competitive-benchmark.md#lumen-competitor-performance-competitive
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-competitor-performance-competitive
// @capability competitor-performance
// @claim competitive-regression-gate-beat-pg-os-per-cell-ratcheting
// @contract search-efficiency-filtering-ranking-pagination
// @category efficiency
// @required_for_production true
// @command cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter
// AW-EC-END

// Contract: FILTERING: filtered_search (AND[BM25+term+range]) beats pg >= 3.73x and OpenSearch(disk) >= 2.4x; filtered_knn beats pg >= 2.4x (OS exempt, no kNN plugin).
// Contract: RANKING: text_bm25 single-term beats pg >= 14.56x; text_and multi-term beats pg >= 1.47x (OS >= 2.4x each).
// Contract: PAGINATION/SORT: pure_sort (scan + sort) beats pg >= 18.32x (OS >= 2.4x); cursor pagination stays within the search_qps pin.
// Contract: Floors are ratcheted (perf-baseline.json, 0.8); btree point-lookup cells stay EXEMPT, not gated. lumen vs pg/OS only.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_competitor_performance_competitive() {
    let command = "cd projects/lumen && ../../target/debug/vat run ec-efficiency-meter";
    let id = "lumen-competitor-performance-competitive";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join(".aw").is_dir() {
        assert!(
            root.pop(),
            "AW EC {id}: no .aw/ project root above {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(&root)
        .output()
        .unwrap_or_else(|e| panic!("AW EC {id}: failed to spawn `{command}`: {e}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if output.status.success()
        && aw_ec_cargo_test_executed_count(command, &stdout, &stderr) == Some(0)
    {
        panic!("AW EC {id} FAILED: cargo test command passed but executed 0 tests: {command}\nstdout:\n{stdout}\nstderr:\n{stderr}");
    }
    assert!(
        output.status.success(),
        "AW EC {id} FAILED (exit {:?}): {command}\nstdout:\n{stdout}\nstderr:\n{stderr}",
        output.status.code()
    );
}

fn aw_ec_cargo_test_executed_count(command: &str, stdout: &str, stderr: &str) -> Option<usize> {
    if !command.contains("cargo test") {
        return None;
    }
    let mut total = 0usize;
    let mut saw_count = false;
    for line in stdout.lines().chain(stderr.lines()) {
        let Some(count) = aw_ec_parse_cargo_running_test_count(line) else {
            continue;
        };
        total = total.saturating_add(count);
        saw_count = true;
    }
    saw_count.then_some(total)
}

fn aw_ec_parse_cargo_running_test_count(line: &str) -> Option<usize> {
    let rest = line.trim().strip_prefix("running ")?;
    let number = rest
        .strip_suffix(" tests")
        .or_else(|| rest.strip_suffix(" test"))?;
    number.trim().parse().ok()
}
// CODEGEN-END
