// SPEC-MANAGED: apps/defer/external-contracts/behavior/2215.md#defer-security-credential-rotation-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-security-credential-rotation-stability
// @capability security-hardening
// @claim delayed-task-security-boundary
// @contract live-rotation-and-last-known-good-role-map
// @category stability
// @required_for_production true
// @command cargo test -p defer --test service_auth -- --nocapture
// AW-EC-END

// Contract: A shipped defer serve process with required auth observes an atomic registry-file replacement through the production 15-second watcher cadence, invalidates the old bearer, and activates the new queue reader without restart.
// Contract: Malformed replacement JSON is rejected while the last-known-good reader remains authenticated for read and denied for write.
// Contract: The process-level rotation and denial emit credential_registry_reload and authorization_decision events through Defer's configured JSON tracing sink, and captured logs contain neither the old nor new bearer bytes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_security_credential_rotation_stability() {
    let command = "cargo test -p defer --test service_auth -- --nocapture";
    let id = "defer-security-credential-rotation-stability";
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
