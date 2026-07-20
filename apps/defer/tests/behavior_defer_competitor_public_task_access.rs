// SPEC-MANAGED: apps/defer/external-contracts/behavior/2216.md#defer-competitor-public-task-access
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-public-task-access
// @capability competitor-feature-parity
// @claim delayed-task-competitor-feature-matrix
// @contract public-h2c-create-cancel-inspect-and-live-credentials
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_api --test service_auth -- --nocapture
// AW-EC-END

// Contract: The public h2c API creates a future task with an authorized administrator, cancels it through DELETE, and lets a queue reader inspect the same id as terminal Canceled while unauthenticated task/admin requests and cross-queue access are rejected.
// Contract: The shipped required-auth process adopts an atomic registry replacement at the production watcher cadence, rejects the retired bearer, activates the replacement reader without restart, and emits credential-free structured audit events.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_public_task_access() {
    let command = "cargo test -p defer --test http_api --test service_auth -- --nocapture";
    let id = "defer-competitor-public-task-access";
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
