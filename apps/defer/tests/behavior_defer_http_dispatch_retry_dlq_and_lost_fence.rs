// SPEC-MANAGED: apps/defer/external-contracts/behavior/766.md#defer-http-dispatch-retry-dlq-and-lost-fence
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http-dispatch-retry-dlq-and-lost-fence
// @capability http-dispatch-and-retries
// @claim http-target-attempt-contract
// @contract retry-dlq-lost-fence-and-stable-idempotency
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_dispatch -- --nocapture && cargo test -p defer --test task_lifecycle nack_reschedules_then_dead_letters_after_max_attempts -- --nocapture
// AW-EC-END

// Contract: A real HTTP target 503 causes a committed retry, the next delivery reuses the stable idempotency key queue/task_id, and each delivery carries a fresh attempt id before the eventual committed Acked disposition.
// Contract: If a target may have accepted the effect but the executor loses its committed fence before settlement, Defer reports LostOwnership and only a later fenced retry may commit the terminal success.
// Contract: When max_attempts is exhausted, the second nack commits DeadLettered and the public scheduler state for that queue/task becomes terminal DeadLettered rather than remaining leased or requeueing forever.
// Contract: Shared-executor dispatch stays bounded: the dispatcher pass does not exceed the configured hard concurrency cap while still making non-zero progress.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http_dispatch_retry_dlq_and_lost_fence() {
    let command =
        "cargo test -p defer --test http_dispatch -- --nocapture && cargo test -p defer --test task_lifecycle nack_reschedules_then_dead_letters_after_max_attempts -- --nocapture";
    let id = "defer-http-dispatch-retry-dlq-and-lost-fence";
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
