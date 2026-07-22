// SPEC-MANAGED: apps/defer/external-contracts/behavior/2217.md#defer-competitor-performance-durable-lifecycle-ceiling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-performance-durable-lifecycle-ceiling
// @capability competitor-performance
// @claim delayed-task-competitor-performance-baseline
// @contract same-host-durable-lifecycle-throughput-overhead-ceiling
// @category efficiency
// @required_for_production true
// @command cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
// AW-EC-END

// Contract: The explicitly unignored release-mode oracle completes exactly 1,000 items for both Defer and Relay in ten 100-item batches with the same exactly asserted 128-byte serialized JSON payload, one Raft voter, fsync-always durability, and durable enqueue -> committed lease -> committed ack lifecycle.
// Contract: Every batch hard-asserts created/published and leased counts; Defer additionally requires exactly 100 settlement outcomes and every outcome Acked(true), Relay requires exactly 100 acknowledgements, and accumulated completed and latency-sample counts must each equal 1,000 before throughput is calculated.
// Contract: Before emitting JSON, both sides independently require finite positive throughput, CPU, and disk amplification plus non-zero p50/p95/p99 batch-lifecycle latency, process-shared RSS, and durable disk bytes; any command error, failed count or settlement, missing sample, zero metric, NaN, infinity, or zero-operation run fails closed.
// Contract: The measured defer_to_relay_ratio must be at least the emitted minimum_ratio of 0.80, bounding Defer's additional ETA, queue-control, permit, retry, DLQ, and terminal-state overhead to no more than 20% under this identical local workload.
// Contract: The command parses an exact machine-readable benchmark contract and emits matching scope fields: Relay is the performance comparator, scope is same-host-sibling-implementation-ceiling, RSS is process-shared-not-component-isolated, the dated observation is non-authoritative, and both Cloud Tasks performance and universal superiority claims are false.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_performance_durable_lifecycle_ceiling() {
    let command =
        "cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture";
    let id = "defer-competitor-performance-durable-lifecycle-ceiling";
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
