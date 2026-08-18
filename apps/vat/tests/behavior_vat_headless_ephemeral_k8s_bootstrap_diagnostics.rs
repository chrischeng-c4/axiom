// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-headless-ephemeral-k8s-bootstrap-diagnostics
// @capability agent-native-gpu-native-dev-containers
// @claim headless-ephemeral-kubernetes-session
// @contract vat-headless-ephemeral-k8s-bootstrap-diagnostics
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_k8s_ephemeral -- --nocapture
// AW-EC-END

// Contract: The passed deterministic fake regression keeps the bootstrap root error primary, then emits staged non-sensitive installer/guest/machine diagnostics with exactly guest_install_log, guest_k3s_system, backing_container_logs, machine_boot_log, machine_inspect, and container_system_status.
// Contract: The diagnostics are fixed read-only probes under a six-second total and one-second-per-probe budget; they exclude private kubeconfig/cache and host credentials, do not change the existing 300-second bootstrap behavior, do not rerun k3s --version or add a wrapper/recovery command, and exact cleanup still runs.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_headless_ephemeral_k8s_bootstrap_diagnostics() {
    let command = "cargo test -p vat --test vat_k8s_ephemeral -- --nocapture";
    let id = "vat-headless-ephemeral-k8s-bootstrap-diagnostics";
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
