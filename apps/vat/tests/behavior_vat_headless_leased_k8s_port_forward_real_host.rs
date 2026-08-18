// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-headless-leased-k8s-port-forward-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim headless-ephemeral-kubernetes-session
// @contract vat-headless-leased-k8s-port-forward-real-host
// @category behavior
// @required_for_production true
// @command VAT_K8S_PORT_FORWARD_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_port_forwards_local_service_to_one_credential_free_host_child -- --ignored --nocapture
// AW-EC-END

// Contract: Independent-kubectl real-host E2E passed 1/1 (36 filtered) in 49.57s. A local alpine fixture was loaded into the active K3s lease; because BusyBox lacks `httpd`, an in-pod HTTP probe verified the fixture before its Service endpoint responded through one VAT-owned 127.0.0.1 text forward and the strict one-document JSON tunnel.
// Contract: One credential-free host curl child proves VAT supplied endpoint metadata but did not inject kubeconfig/cache/API variables or VAT_HOME. Its terminal record begins on a new line after child output.
// Contract: The terminal result confirms forward cleanup, the selected loopback port is closed afterward, and the lease remains active only until explicit delete confirms exact machine cleanup. This bounded result is not a same-UID OS-isolation claim, daemonized-child claim, persistence, ingress, public bind, background-proxy, or general Kubernetes claim.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_headless_leased_k8s_port_forward_real_host() {
    let command =
        "VAT_K8S_PORT_FORWARD_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_port_forwards_local_service_to_one_credential_free_host_child -- --ignored --nocapture";
    let id = "vat-headless-leased-k8s-port-forward-real-host";
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
