// SPEC-MANAGED: apps/relay/external-contracts/security-hardening/security/security-evidence.md#relay-security-hardening-negative-peer-and-kubernetes-boundaries
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-security-hardening-negative-peer-and-kubernetes-boundaries
// @capability security-hardening
// @claim network-policy-and-peer-mtls-termination
// @contract relay-untrusted-peer-and-restricted-workload-security
// @category security
// @required_for_production true
// @command bash apps/relay/scripts/ec-evidence.sh security-boundaries
// AW-EC-END

// Contract: A client that trusts Relay's legitimate server CA but presents an identity signed by an attacker CA is rejected by the required-mTLS server handshake before HTTP or Raft routing.
// Contract: Peers signed by the trusted CA still elect, replicate, and converge over the authenticated listener.
// Contract: The direct StatefulSet is non-root with a read-only root filesystem and durable PVC; the production overlay projects credentials read-only, enables NetworkPolicy and observability components, and does not apply an unsafe voter HPA.
// Contract: The outer oracle requires both named peer-mTLS tests and both named Kubernetes tests and rejects a zero-test result from either binary.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_security_hardening_negative_peer_and_kubernetes_boundaries() {
    let command = "bash apps/relay/scripts/ec-evidence.sh security-boundaries";
    let id = "relay-security-hardening-negative-peer-and-kubernetes-boundaries";
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
