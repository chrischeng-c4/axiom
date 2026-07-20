// SPEC-MANAGED: apps/defer/external-contracts/behavior/766.md#defer-http-dispatch-and-retries
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http-dispatch-and-retries
// @capability http-dispatch-and-retries
// @claim http-target-attempt-contract
// @contract manual-dispatch-route-terminal-state-and-recovery
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_api -- --nocapture
// AW-EC-END

// Contract: POST /v1/queues/{queue}/dispatch on a due task returns a JSON dispatch report with target_status 204 for the real HTTP target, and the public task GET route then reports status Succeeded for that exact task id.
// Contract: The manual dispatch route increments the public metrics surface so /metrics contains defer_dispatch_acked_total 1 after the committed success.
// Contract: A backup taken after the manual dispatch can seed a fresh raft store, and the recovered public task state still reports the same terminal Succeeded outcome instead of replaying or dropping the committed dispatch result.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http_dispatch_and_retries() {
    let command = "cargo test -p defer --test http_api -- --nocapture";
    let id = "defer-http-dispatch-and-retries";
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
