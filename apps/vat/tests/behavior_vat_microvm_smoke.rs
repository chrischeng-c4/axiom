// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-microvm-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim microvm-sandbox-backend-for-vat-run
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_sandbox_microvm -- --nocapture
// AW-EC-END

// Contract: AC1/R2: `vat run --isolation micro_vm --microvm-image <ref> -- <cmd>` resolves and executes a real `container run` invocation, rootfs bind-mounted at /workspace, workdir honored, env vars visible inside the guest, and `--network none` enforced under EgressPolicy::Deny. Skips cleanly (does not fail) when the `container` CLI is not installed — mirrors the existing Docker-gated test pattern.
// Contract: Registered in the generated `apps/vat/aw.toml` EC inventory alongside the fail-closed integration test so `aw ec gen --verify` / `aw health --verify-tests` pick both up as configured EC-gated test commands for this capability.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_microvm_smoke() {
    let command = "cargo test -p vat --test vat_sandbox_microvm -- --nocapture";
    let id = "vat-microvm-smoke";
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
