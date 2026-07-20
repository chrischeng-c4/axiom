// SPEC-MANAGED: apps/defer/external-contracts/behavior/2216.md#defer-competitor-queue-controls-and-dedupe
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-queue-controls-and-dedupe
// @capability competitor-feature-parity
// @claim delayed-task-competitor-feature-matrix
// @contract committed-rate-burst-concurrency-control-and-dedupe
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test rate_limits -- --nocapture
// AW-EC-END

// Contract: Exactly eight deterministic single-scheduler queue-policy tests execute and enforce per-tick budget, token-bucket rate and burst timing, expired-lease reclaim, queue-local policy updates, pause/resume isolation, and disabled-queue create/dispatch rejection.
// Contract: A batch containing an existing task id fails atomically without inserting either surrounding task, and batch settlement rejects the wrong executor and stale epoch before accepting the exact current fence.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_queue_controls_and_dedupe() {
    let command = "cargo test -p defer --test rate_limits -- --nocapture";
    let id = "defer-competitor-queue-controls-and-dedupe";
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
