// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-compose-detached-ownership-and-cleanup
// @capability agent-native-gpu-native-dev-containers
// @claim microvm-sandbox-backend-for-vat-run
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_compose -- --nocapture
// AW-EC-END

// Contract: Foreground and detached paths use the same project/token handoff. Only the token owner creates and synchronously publishes the durable VAT id; the parent only rereads the token-owned registry and never polls global VAT-store name/time evidence. handoff_protocol: 1 survives transient PID/token clearing; internal parent/child claim reacquisition waits at most ten seconds while external lifecycle commands remain non-blocking.
// Contract: A token without a launcher PID expires after its bounded grace period; a stale child cannot overwrite a newer project binding.
// Contract: down writes the VAT-owned stop request, projects stopping while VAT remains Running with runner/service teardown pending, and treats every current handoff_protocol: 1 metadata load/read/malformed/missing error as EvidenceUnavailable rather than terminal reset. Only a protocol-absent historic record with a separately confirmed metadata NotFound may recover. It waits for Status::Exited plus terminal runner/service state and rejects a concurrent up until cleanup acknowledgement completes.
// Contract: A persisted cleanup_error retains the VAT, binding, and published-port ownership; retrying down removes the recorded Docker or MicroVM resource and releases only after rm success or successful exact-name list absence proof. The runner or scenario result is nonzero even when keep is never or failed.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_compose_detached_ownership_and_cleanup() {
    let command = "cargo test -p vat --test vat_compose -- --nocapture";
    let id = "vat-compose-detached-ownership-and-cleanup";
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
