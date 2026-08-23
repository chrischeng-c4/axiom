// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-local-k8s-phase0-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim local-kubernetes-cluster-service-and-vat-cluster
// @contract local-agent-test-runner-protocol
// @category stability
// @required_for_production false
// @command VAT_LOCAL_K8S_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_exec_control_is_usable_before_k3s -- --ignored --nocapture
// AW-EC-END

// Contract: The ignored, explicit opt-in test preflights `local/vat-k8s-systemd:phase0` (or `VAT_LOCAL_K8S_MACHINE_IMAGE`) and skips cleanly when the Apple Container CLI is absent. It never builds or publishes an image itself.
// Contract: Using the source-controlled systemd fixture, the test creates one unique no-boot machine with no home mount, 2 CPUs, and 4G memory. It requires PID 1 to be `systemd`, waits for systemd, stops the machine, and requires `container machine run` to restart it and return the control marker.
// Contract: The test records exact argv, output, inspect/logs, bounded exact-name cleanup attempts, and a `go` or `no-go` verdict in a JSON report under `VAT_LOCAL_K8S_EVIDENCE_DIR` (default `/private/tmp`). It never deletes an ambient machine: after a failed or timed-out create it continues exact-name reconciliation through a stabilization window and requires a structured exact-name absence result; Drop is panic-only fallback.
// Contract: Recorded durability evidence: Apple Container 1.1.0 on macOS 26.5.1 boots the systemd fixture, but `container machine run` returns `Operation not supported by device` after a restart retry. A separate host-API probe also saw `machine create` report a bootMachine XPC timeout while its uniquely named machine was running, so failed create is treated as delayed allocation rather than proof of absence. `container exec` can diagnose an already-running backing container only; it cannot restart a stopped machine and is not a substitute for this control. The verdict remains NO-GO.
// Contract: This test does not treat a disposable `container exec` session as a durable pass. The durable failure blocks host kubeconfig, local image loading, networking, storage, stop/start reconciliation, and Phase 1 backend implementation.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_local_k8s_phase0_real_host() {
    let command =
        "VAT_LOCAL_K8S_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_exec_control_is_usable_before_k3s -- --ignored --nocapture";
    let id = "vat-local-k8s-phase0-real-host";
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
