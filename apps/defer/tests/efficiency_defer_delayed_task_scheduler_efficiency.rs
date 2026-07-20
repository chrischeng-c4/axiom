// SPEC-MANAGED: apps/defer/external-contracts/behavior/2214.md#defer-delayed-task-scheduler-efficiency
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-delayed-task-scheduler-efficiency
// @capability long-running-stability
// @claim delayed-task-soak-and-recovery
// @contract durable-lifecycle-throughput-overhead-ceiling
// @category efficiency
// @required_for_production true
// @command cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
// AW-EC-END

// Contract: The release-mode oracle completes exactly 1,000 durable Defer operations in ten 100-item batches, where every item traverses committed enqueue, lease, and ack with zero failed settlements under single-voter Raft with fsync enabled.
// Contract: The emitted numeric report includes non-zero throughput, p50/p95/p99 latency, CPU time, RSS, durable disk bytes, disk amplification, and errors = 0 for both Defer and the same-host Relay control workload.
// Contract: Defer's measured throughput must be at least 80% of Relay's under the identical payload, batching, voter, fsync, and lifecycle shape; a zero-operation run, missing metric, error, or ratio below 0.80 fails.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_delayed_task_scheduler_efficiency() {
    let command =
        "cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture";
    let id = "defer-delayed-task-scheduler-efficiency";
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
