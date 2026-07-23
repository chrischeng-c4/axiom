// SPEC-MANAGED: apps/lumen/external-contracts/long-running-stability/stability/query-resilience.md#lumen-long-running-stability-query-resilience
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-query-resilience
// @capability long-running-stability
// @claim search-p99-survives-fault-and-recovers
// @contract search-stability-fault-resilience
// @category stability
// @required_for_production true
// @command cd apps/lumen && ../../target/debug/vat run rig-resilience
// AW-EC-END

// Contract: packet_loss_p99 routes baseline and fault search samples through toxiproxy with downstream timeout toxicity 0.05, requires 0 < loss_fail <= 30 and loss_p99 <= 2 * baseline_p99 + 20ms, then removes the toxic, records loss_recovered_recovered_secs <= 10, requires loss_recovery_fail == 0, and requires loss_recovery_p99 <= 2 * baseline_p99 + 1ms.
// Contract: partition_recovery applies a full downstream partition, requires partition_fail > 0, records recovered_recovered_secs <= 10, and requires recovery_p99 <= 2 * baseline_p99 + 1ms after the toxic is removed.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_query_resilience() {
    let command = "cd apps/lumen && ../../target/debug/vat run rig-resilience";
    let id = "lumen-long-running-stability-query-resilience";
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
