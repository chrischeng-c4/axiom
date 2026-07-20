// SPEC-MANAGED: apps/tape/external-contracts/security-hardening/security/auth-bearer-rbac.md#tape-security-hardening-auth-bearer-rbac
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-security-hardening-auth-bearer-rbac
// @capability security-hardening
// @claim tape-bearer-topic-role-auth
// @contract tape-bearer-token-topic-rbac
// @category security
// @required_for_production false
// @command cargo test -p tape --test service_auth -- --nocapture
// AW-EC-END

// Contract: When TAPE_AUTH=required, missing and unknown bearer tokens are rejected; a reader cannot append to a topic.
// Contract: Topic-scoped write grants authorize append, topic-scoped read grants authorize replay and checkpoint operations, and wildcard administrator grants cover every topic.
// Contract: The required-mode registry fails fast for missing, malformed, empty, or unknown auth configuration; TAPE_AUTH=off remains the explicit local tokenless mode.
// Contract: Auth rejection uses the shared unauthenticated/forbidden envelope and the standard probe, metrics, OpenAPI, and docs routes remain tokenless.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_security_hardening_auth_bearer_rbac() {
    let command = "cargo test -p tape --test service_auth -- --nocapture";
    let id = "tape-security-hardening-auth-bearer-rbac";
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
