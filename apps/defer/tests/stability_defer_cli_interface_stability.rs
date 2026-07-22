// SPEC-MANAGED: apps/defer/external-contracts/behavior/2213.md#defer-cli-interface-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-cli-interface-stability
// @capability cli-interface
// @claim defer-cli-convention-and-task-verbs
// @contract repeated-offline-cli-determinism-and-resource-plateau
// @category stability
// @required_for_production true
// @command cargo test --release -p defer --test cli_stability -- --ignored --nocapture
// AW-EC-END

// Contract: Sixty-four repeated rounds of help, llm outline, OpenAPI emission, and source-Dockerfile rendering complete with zero failures and byte-identical stdout for each operation; a nondeterministic or partial output fails.
// Contract: Sixteen additional TypeScript generation rounds produce byte-identical exact five-file outputs, and every temporary output directory disappears after its round rather than accumulating state.
// Contract: The full repeated gate completes within 60 seconds and reports file-descriptor growth <= 8 when the host exposes an FD inventory; a missing progress count, leaked child/temp state, breached bound, or skipped ignored test fails.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_cli_interface_stability() {
    let command = "cargo test --release -p defer --test cli_stability -- --ignored --nocapture";
    let id = "defer-cli-interface-stability";
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
