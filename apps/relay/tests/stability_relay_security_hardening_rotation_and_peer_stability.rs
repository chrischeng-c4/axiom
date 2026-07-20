// SPEC-MANAGED: apps/relay/external-contracts/security-hardening/security/security-evidence.md#relay-security-hardening-rotation-and-peer-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-security-hardening-rotation-and-peer-stability
// @capability security-hardening
// @claim bearer-auth-token-registry
// @contract relay-security-last-known-good-stability
// @category stability
// @required_for_production true
// @command bash apps/relay/scripts/ec-evidence.sh security-stability
// AW-EC-END

// Contract: A valid registry rotation becomes visible without restarting Relay, invalid JSON/empty/read-failed rotations retain the last-known-good registry, and failure audit classes remain credential-free.
// Contract: The trusted three-peer required-mTLS group remains able to elect, replicate, converge, and shut down after the negative certificate-rejection case is present.
// Contract: The outer oracle requires all five reload tests plus the exact Relay rotation and trusted-peer tests, and each focused invocation must execute at least one test.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_security_hardening_rotation_and_peer_stability() {
    let command = "bash apps/relay/scripts/ec-evidence.sh security-stability";
    let id = "relay-security-hardening-rotation-and-peer-stability";
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
