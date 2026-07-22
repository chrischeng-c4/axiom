// SPEC-MANAGED: apps/defer/external-contracts/behavior/2214.md#defer-delayed-task-live-soak
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-delayed-task-live-soak
// @capability long-running-stability
// @claim delayed-task-soak-and-recovery
// @contract fixed-keyspace-retry-soak-resource-and-latency-plateau
// @category stability
// @required_for_production true
// @command DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh
// AW-EC-END

// Contract: The gate autostarts an isolated real Defer process, creates exactly one terminal success and one continuously retrying HTTP-404 task in a fixed queue, crosses the 1,024-entry proposal-cache and snapshot cadence during warmup, then reports a non-zero measured operation count with errors = 0.
// Contract: The committed retry counter increases during both steady windows while the successful task stays Succeeded and the fault task remains Scheduled or actively Leased, so forward progress cannot be inferred from setup or from unbounded key growth.
// Contract: Across two 30-second steady windows, RSS drift is <= 10%, file-descriptor growth <= 8, thread/task growth <= 4, task-read p99 is <= 250 ms with <= 100% window growth, and any missing numeric measurement or breached bound fails closed.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_delayed_task_live_soak() {
    let command = "DEFER_SOAK_AUTOSTART=1 bash apps/defer/scripts/soak.sh";
    let id = "defer-delayed-task-live-soak";
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
