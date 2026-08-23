// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-headless-ephemeral-k8s-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim headless-ephemeral-kubernetes-session
// @contract vat-headless-ephemeral-k8s-real-host
// @category behavior
// @required_for_production true
// @command VAT_K8S_EPHEMERAL_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_session_exposes_host_api_then_cleans_up -- --ignored --nocapture
// AW-EC-END

// Contract: An independently installed non-OrbStack kubectl is first on PATH; an OrbStack-provided binary is rejected before the K3s command runs.
// Contract: The public command's systemd image is present in Apple Container.
// Contract: Host kubectl reads exactly one Ready K3s node through the temporary 0600 kubeconfig.
// Contract: The terminal result's exact machine name returns Apple Container's not-found result after return.
// Contract: A pass is only a one-boot agent-session proof, never a durable microvm-k3s claim.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_headless_ephemeral_k8s_real_host() {
    let command =
        "VAT_K8S_EPHEMERAL_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_session_exposes_host_api_then_cleans_up -- --ignored --nocapture";
    let id = "vat-headless-ephemeral-k8s-real-host";
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
