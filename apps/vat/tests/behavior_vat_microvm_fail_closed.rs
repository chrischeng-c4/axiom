// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-microvm-fail-closed
// @capability agent-native-gpu-native-dev-containers
// @claim microvm-sandbox-backend-for-vat-run
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_sandbox_microvm_fail_closed -- --nocapture
// AW-EC-END

// Contract: AC3: `sandbox::pick(spec)` returns a hard Err (never a silently-degraded backend) for isolation=MicroVm when gpu=GpuRequest::Required, when spec.microvm_image is None, when egress=EgressPolicy::LocalhostOnly, and when microvm::available() is false; the LocalhostOnly error text carries the Phase-0-confirmed gateway-IP reasoning, not a generic 'no bridge exists' message.
// Contract: AC4: a dedicated case exercises the run.rs `gpu_satisfied()` preflight helper directly (not just pick()) rejecting `--isolation micro_vm --gpu required` before any workspace clone begins — proving the dual fail-closed layers are both wired, independently.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_microvm_fail_closed() {
    let command = "cargo test -p vat --test vat_sandbox_microvm_fail_closed -- --nocapture";
    let id = "vat-microvm-fail-closed";
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
