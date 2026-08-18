// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-local-k8s-phase0-disposable-k3s
// @capability agent-native-gpu-native-dev-containers
// @claim local-kubernetes-cluster-service-and-vat-cluster
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command VAT_LOCAL_K8S_DISPOSABLE_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_bootstraps_disposable_k3s_via_backing_container_exec -- --ignored --nocapture
// AW-EC-END

// Contract: The explicit opt-in probe creates one auto-booted source-fixture machine, parses only that machine's inspect-returned running `containerId`, and never searches for or touches an ambient container.
// Contract: It requires PID 1 to be systemd and root command execution, installs pinned k3s v1.36.2+k3s1 with a guest-only 0600 admin kubeconfig, first waits within a bounded loop for the Node resource to exist and then waits for `Node Ready`, then creates, waits for, logs, and deletes a BusyBox Job whose marker is `vat-k8s-phase0-workload-ok`.
// Contract: The probe captures node/pod state and k3s journal output in a JSON report, explicitly deletes the Job, then reconciles and proves bounded absence of only its owned machine; Drop cleanup is panic-only fallback. The observed host result is `ephemeral-go`.
// Contract: Without the separate host-API opt-in, this evidence proves a one-machine, one-boot guest substrate only. It does not prove default-add-on readiness, host API access, port exposure, local OCI image delivery, persistent volumes, multi-node networking, or stop/run durability.
// Contract: Because the command explicitly opts into this real-host production gate, a missing or unusable Apple Container CLI is a hard failure rather than a successful skip.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_local_k8s_phase0_disposable_k3s() {
    let command =
        "VAT_LOCAL_K8S_DISPOSABLE_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_bootstraps_disposable_k3s_via_backing_container_exec -- --ignored --nocapture";
    let id = "vat-local-k8s-phase0-disposable-k3s";
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
