// SPEC-MANAGED: apps/defer/external-contracts/behavior/766.md#defer-http-dispatch-retry-soak-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http-dispatch-retry-soak-stability
// @capability http-dispatch-and-retries
// @claim http-target-attempt-contract
// @contract bounded-retry-soak-fixed-state-plateau
// @category stability
// @required_for_production true
// @command DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh
// AW-EC-END

// Contract: The fixed-keyspace soak must make non-zero progress, report errors = 0, keep the committed success task terminally Succeeded, and keep a second real HTTP-404 task retryable rather than silently dropping or dead-lettering it early.
// Contract: With the soak queue's explicit zero-delay retry policy, the committed retry counter must increase in each measured window, proving that lease, HTTP fault, nack, and rescheduling continue to make progress throughout the soak instead of being inferred from a one-shot setup attempt; non-zero backoff timing remains covered by the lifecycle test.
// Contract: Across the two steady windows the soak enforces measurable stability thresholds: RSS drift <= 10%, file-descriptor growth <= 8, thread/task growth <= 4, and p99 <= 250ms with no more than 100% growth between windows.
// Contract: The stability oracle is bounded fault/retry-soak evidence from the script output, not an unmeasured claim that retries are stable over time.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http_dispatch_retry_soak_stability() {
    let command = "DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh";
    let id = "defer-http-dispatch-retry-soak-stability";
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
