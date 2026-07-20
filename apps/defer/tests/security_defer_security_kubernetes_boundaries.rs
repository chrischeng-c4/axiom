// SPEC-MANAGED: apps/defer/external-contracts/behavior/2215.md#defer-security-kubernetes-boundaries
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-security-kubernetes-boundaries
// @capability security-hardening
// @claim delayed-task-security-boundary
// @contract restricted-server-operator-secret-and-network-policy-assets
// @category security
// @required_for_production false
// @command cargo test -p defer --test direct_k8s_assets -- --nocapture
// AW-EC-END

// Contract: The direct StatefulSet and operator Deployment run non-root with RuntimeDefault seccomp, disallow privilege escalation, use read-only root filesystems, and drop all Linux capabilities; the StatefulSet retains its durable 10Gi PVC.
// Contract: kubectl kustomize must successfully render the production overlay; the rendered graph contains the read-only token registry Secret projection, a ServiceMonitor, a 100Gi durable PVC, and no HorizontalPodAutoscaler.
// Contract: The rendered NetworkPolicy selects only Defer server pods, declares ingress and egress policy types, and names the public HTTP 7141 and peer 7142 ports.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_security_kubernetes_boundaries() {
    let command = "cargo test -p defer --test direct_k8s_assets -- --nocapture";
    let id = "defer-security-kubernetes-boundaries";
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
