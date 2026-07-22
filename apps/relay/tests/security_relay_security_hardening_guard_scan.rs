// SPEC-MANAGED: apps/relay/external-contracts/security-hardening/security/security-evidence.md#relay-security-hardening-guard-scan
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-security-hardening-guard-scan
// @capability security-hardening
// @claim guard-static-runtime-evidence
// @contract relay-guard-security-report
// @category security
// @required_for_production true
// @command cd apps/relay && ../../target/debug/vat run guard-security
// AW-EC-END

// Contract: guard scan over apps/relay reports no untriaged Docker, Kubernetes, or static security findings.
// Contract: guard runs the fail-closed evidence driver before attaching Meter evidence from auth, admission, peer-mTLS, direct-Kubernetes, and service-auth reload suites; missing required names, zero execution, a failed control, or an outer-oracle self-test regression makes the runner fail.
// Contract: The security evidence runs inside vat so generated reports and transient files do not mutate the host checkout.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_security_hardening_guard_scan() {
    let command = "cd apps/relay && ../../target/debug/vat run guard-security";
    let id = "relay-security-hardening-guard-scan";
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
