// SPEC-MANAGED: apps/relay/external-contracts/security-hardening/security/security-evidence.md#relay-security-hardening-auth-and-admission-behavior
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-security-hardening-auth-and-admission-behavior
// @capability security-hardening
// @claim request-limit-and-malformed-frame-negative-tests
// @contract relay-auth-rbac-and-admission-behavior
// @category behavior
// @required_for_production true
// @command bash apps/relay/scripts/ec-evidence.sh security-behavior
// AW-EC-END

// Contract: Required auth returns 401 for missing or unknown bearer tokens and 403 when a reader attempts publish or a subject-scoped reader crosses its grant; the streaming consume route enforces the same boundary.
// Contract: Valid subject writers/readers and wildcard administrators retain their intended publish, lease, ack, heartbeat, and batch behavior, while health, readiness, metrics, OpenAPI, and docs remain tokenless.
// Contract: A configured one-write admission budget allows the first publish, rejects the second with 429 and Retry-After: 60, and never rate-limits health probes.
// Contract: The outer oracle requires all eight auth and two admission test names and independently rejects either suite when its executed count is zero.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_security_hardening_auth_and_admission_behavior() {
    let command = "bash apps/relay/scripts/ec-evidence.sh security-behavior";
    let id = "relay-security-hardening-auth-and-admission-behavior";
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
