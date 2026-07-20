// SPEC-MANAGED: apps/defer/external-contracts/behavior/2216.md#defer-competitor-http-target-delivery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-http-target-delivery
// @capability competitor-feature-parity
// @claim delayed-task-competitor-feature-matrix
// @contract real-method-header-body-retry-idempotency-and-terminal-success
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_dispatch -- --nocapture
// AW-EC-END

// Contract: A real target observes PATCH, the per-task x-defer-tenant header, and the exact JSON body on both a 503 attempt and its 204 retry; the stable queue/task idempotency key is unchanged, attempt ids are fresh, and committed status becomes Succeeded.
// Contract: If the target accepts HTTP 204 after the committed lease fence is lost, the stale executor reports LostOwnership and a fresh fenced attempt retries with the same idempotency key before committing success.
// Contract: Eight real HTTP effects complete with non-zero concurrency while observed peak in-flight work equals the configured hard bound of three.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_http_target_delivery() {
    let command = "cargo test -p defer --test http_dispatch -- --nocapture";
    let id = "defer-competitor-http-target-delivery";
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
