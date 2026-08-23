// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-direct-json-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-direct-json-real-host
// @category behavior
// @required_for_production true
// @command VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_run_published_port_contract -- --ignored --nocapture
// AW-EC-END

// Contract: Passed 1/1 (50 filtered) on Apple Container 1.1.0. The gate uses a high-entropy nonce+PID temporary nginx container and verifies io.cclab.vat.e2e-owner=<token> by inspect before cleanup. Exact-label rechecks are conservative best-effort precautions, and the emergency guard retains the container on uncertainty. Apple Container has no atomic conditional delete, so this does not claim race-free cleanup or that a misdelete is impossible; the shared/cacheable nginx image is not cleaned up.
// Contract: Ps/images are global read-only inventory smoke observations; inspect/stats/VAT logs and direct exec target the temporary owner-labeled nginx container. The host smoke proves one valid native JSON document or VAT wrapper only, including an exec wrapper with both stdout and stderr markers.
// Contract: Fake/unit tests prove byte-preservation and fail-closed details. This remains bounded direct-command evidence only: it does not establish guest-timeout termination, Docker Engine parity, generic runtime behavior, Docker multiplex/demux, broader Compose semantics, immutable image identity, or image cleanup.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_direct_json_real_host() {
    let command =
        "VAT_DOCKER_SHIM_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_run_published_port_contract -- --ignored --nocapture";
    let id = "vat-docker-direct-json-real-host";
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
