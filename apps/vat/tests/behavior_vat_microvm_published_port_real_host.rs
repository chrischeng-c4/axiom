// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-microvm-published-port-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim microvm-sandbox-backend-for-vat-run
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command VAT_MICROVM_E2E_REQUIRED=1 cargo test -p vat --test vat_microvm_published_port -- --ignored --nocapture
// AW-EC-END

// Contract: On an explicit opt-in host with Apple's container CLI, a VAT-owned nginx MicroVM has its guest and published host endpoint checked separately.
// Contract: A host endpoint that resets or cannot complete the configured HTTP contract fails nonzero with service, endpoint, runtime, inspect, and logs remediation rather than Ready.
// Contract: The test removes only its uniquely named VAT-owned MicroVM and records the observed Apple container evidence for tracker review. A nonzero rm -f is accepted only when the successful bounded JSON list has no matching id; query failure, timeout, malformed JSON, or a matching id leaves durable cleanup_error, keeps the VAT and compose binding unavailable for reuse until retry, and returns nonzero.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_microvm_published_port_real_host() {
    let command =
        "VAT_MICROVM_E2E_REQUIRED=1 cargo test -p vat --test vat_microvm_published_port -- --ignored --nocapture";
    let id = "vat-microvm-published-port-real-host";
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
