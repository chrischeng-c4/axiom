// SPEC-MANAGED: apps/defer/external-contracts/behavior/2215.md#defer-security-target-signing
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-security-target-signing
// @capability security-hardening
// @claim delayed-task-security-boundary
// @contract exact-length-delimited-hmac-and-negative-oracle
// @category security
// @required_for_production false
// @command cargo test -p defer --test http_dispatch_signing -- --nocapture
// AW-EC-END

// Contract: An independent target-side oracle recomputes HMAC-SHA256 over the exact length-delimited idempotency key, fresh attempt id, target URL, timestamp, and received body bytes for both a retry and its success.
// Contract: Changing any signed field or body byte, using the wrong key id, or using the wrong secret causes rejection; retry preserves the idempotency key while changing both attempt id and signature.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_security_target_signing() {
    let command = "cargo test -p defer --test http_dispatch_signing -- --nocapture";
    let id = "defer-security-target-signing";
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
