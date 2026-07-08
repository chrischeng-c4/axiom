// SPEC-MANAGED: projects/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md#tape-competitor-performance-local-regression-and-calibration-ledger
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-competitor-performance-local-regression-and-calibration-ledger
// @capability competitor-performance
// @claim topic-replay-competitor-performance-baseline
// @contract topic-replay-local-performance-and-peer-calibration
// @category efficiency
// @required_for_production true
// @command cargo test -p tape --test tape_perf_gate -- --nocapture
// AW-EC-END

// Contract: Tape's local append/replay/checkpoint benchmark stays inside conservative regression budgets.
// Contract: Kafka, Redpanda, Pulsar, and RabbitMQ Streams performance wins are not claimed without calibrated real-service peer runs.
// Contract: RabbitMQ topic exchange remains routing-only and is not treated as a replay performance baseline.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_competitor_performance_local_regression_and_calibration_ledger() {
    let command = "cargo test -p tape --test tape_perf_gate -- --nocapture";
    let id = "tape-competitor-performance-local-regression-and-calibration-ledger";
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
