// SPEC-MANAGED: apps/defer/external-contracts/behavior/2215.md#defer-security-auth-http-boundary
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-security-auth-http-boundary
// @capability security-hardening
// @claim delayed-task-security-boundary
// @contract required-bearer-queue-rbac-and-tokenless-probes
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_api -- --nocapture
// AW-EC-END

// Contract: With required auth, a missing bearer receives 401 on the queue surface, a jobs-only reader can read jobs but receives 403 on both jobs mutation and another tenant's queue, and a wildcard administrator can mutate both queues.
// Contract: On that same required-auth h2c process, health, readiness, docs, OpenAPI, and metrics remain live without a token while unauthenticated task creation and admin backup receive 401 and a queue reader receives 403 on admin backup.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_security_auth_http_boundary() {
    let command = "cargo test -p defer --test http_api -- --nocapture";
    let id = "defer-security-auth-http-boundary";
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
