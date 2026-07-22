// SPEC-MANAGED: apps/defer/external-contracts/behavior/766.md#defer-http-dispatch-efficiency-ceiling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http-dispatch-efficiency-ceiling
// @capability http-dispatch-and-retries
// @claim http-target-attempt-contract
// @contract same-host-durable-dispatch-overhead-ceiling
// @category efficiency
// @required_for_production true
// @command cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture
// AW-EC-END

// Contract: Under the declared single-voter durable lifecycle workload (durable enqueue -> committed lease -> committed ack), Defer throughput is non-zero and reported together with p50/p95/p99, CPU, RSS, disk bytes, amplification, and error counts.
// Contract: The measured report includes errors = 0 for both sides and enforces defer_to_relay_ratio >= 0.80 as the bounded scheduler overhead ceiling for Defer relative to Relay on the same host.
// Contract: The efficiency oracle is the emitted numeric report and hard threshold, not a qualitative statement about Defer being fast enough.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http_dispatch_efficiency_ceiling() {
    let command =
        "cargo test --release -p defer --test relay_performance_ceiling -- --ignored --nocapture";
    let id = "defer-http-dispatch-efficiency-ceiling";
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
