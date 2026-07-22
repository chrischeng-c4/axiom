// SPEC-MANAGED: apps/defer/external-contracts/behavior/2214.md#defer-delayed-task-state-recovery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-delayed-task-state-recovery
// @capability long-running-stability
// @claim delayed-task-soak-and-recovery
// @contract committed-lifecycle-rate-and-raft-failover-recovery
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test task_lifecycle --test rate_limits --test raft_scheduler -- --nocapture
// AW-EC-END

// Contract: The complete lifecycle and queue-policy suites make non-zero progress and preserve ETA-before-priority ordering, FIFO within equal priority, bounded dispatch/concurrency, retry-to-DLQ, cancellation, queue-local rate buckets, pause/resume/disable control, atomic batch rejection, and stale-executor settlement rejection.
// Contract: A real three-node h2c Raft cluster commits queue and task state, rejects settlement by a replica that does not own the lease, survives leader loss with the live lease preserved, rejects the abandoned stale fence, and only reassigns after committed expiry with a higher epoch and fresh attempt id.
// Contract: After the original node restarts from durable state it converges on both terminal tasks; a second leader loss elects a different leader that commits and completes another task, proving repeated failover recovery rather than a one-shot in-memory replay.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_delayed_task_state_recovery() {
    let command =
        "cargo test -p defer --test task_lifecycle --test rate_limits --test raft_scheduler -- --nocapture";
    let id = "defer-delayed-task-state-recovery";
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
