// SPEC-MANAGED: apps/tape/external-contracts/security-hardening/security/access-control.md#tape-security-hardening-access-control
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-security-hardening-access-control
// @capability security-hardening
// @claim tape-topic-subscription-authz-boundary
// @contract tape-topic-security-rbac-and-admission
// @category security
// @required_for_production false
// @command cargo test -p service-auth -- --nocapture && cargo test -p tape --test service_auth --test service_admission -- --nocapture
// AW-EC-END

// Contract: Appending to a topic requires that topic's write grant.
// Contract: Replay and checkpoint operations require that topic's read grant and never expose data from an unauthorized topic.
// Contract: Authentication failures retain the shared service-auth error shape while operational probes remain tokenless.
// Contract: Append is classified as write admission and a configured shared policy returns bounded 429 responses without limiting probes.
// Contract: The projected token registry rotates atomically without restarting Tape, while the shared service-auth suite independently verifies credential-free authorization audit events.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_security_hardening_access_control() {
    let command =
        "cargo test -p service-auth -- --nocapture && cargo test -p tape --test service_auth --test service_admission -- --nocapture";
    let id = "tape-security-hardening-access-control";
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
