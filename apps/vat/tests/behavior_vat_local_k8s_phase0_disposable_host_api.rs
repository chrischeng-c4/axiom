// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-local-k8s-phase0-disposable-host-api
// @capability agent-native-gpu-native-dev-containers
// @claim local-kubernetes-cluster-service-and-vat-cluster
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command VAT_LOCAL_K8S_DISPOSABLE_E2E=1 VAT_LOCAL_K8S_HOST_API_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_bootstraps_disposable_k3s_via_backing_container_exec -- --ignored --nocapture
// AW-EC-END

// Contract: The second explicit opt-in re-inspects the exact owned running machine and requires its backing container ID and IP address to be unchanged before it exports any credential. It installs k3s with that inspected IP as a TLS SAN.
// Contract: It copies the guest admin kubeconfig only into a private 0700 temporary directory, rewrites exactly one loopback server endpoint to the inspected IP, restricts the copied file to 0600, removes ambient KUBECONFIG and proxy environment variables, and makes kubectl use a private discovery cache inside the same owned directory. No credential contents are emitted in JSON evidence.
// Contract: The host command is `kubectl --kubeconfig <owned-path> --cache-dir <owned-path>/kubectl-cache --request-timeout=20s get nodes -o json`; the observed result is `ephemeral-host-api-go`, then the credential directory and exact owned machine are both confirmed absent.
// Contract: This is a one-boot reachability diagnostic only. It does not export a durable user kubeconfig or unlock a persistent microvm-k3s backend, port publication, local image delivery, storage, multi-node networking, or stop/run durability.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_local_k8s_phase0_disposable_host_api() {
    let command =
        "VAT_LOCAL_K8S_DISPOSABLE_E2E=1 VAT_LOCAL_K8S_HOST_API_E2E=1 cargo test -p vat --test vat_local_k8s_phase0 apple_machine_bootstraps_disposable_k3s_via_backing_container_exec -- --ignored --nocapture";
    let id = "vat-local-k8s-phase0-disposable-host-api";
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
