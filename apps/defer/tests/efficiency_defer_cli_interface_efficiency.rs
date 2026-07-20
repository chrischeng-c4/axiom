// SPEC-MANAGED: apps/defer/external-contracts/behavior/2213.md#defer-cli-interface-efficiency
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-cli-interface-efficiency
// @capability cli-interface
// @claim defer-cli-convention-and-task-verbs
// @contract bounded-offline-cli-and-codegen-latency
// @category efficiency
// @required_for_production true
// @command cargo test --release -p defer --test cli_efficiency -- --ignored --nocapture
// AW-EC-END

// Contract: After one warmup per operation, the release binary completes at least 20 measured offline operations spanning help, llm outline, OpenAPI emission, and TypeScript generation with zero command failures and non-empty contract output.
// Contract: The emitted numeric report includes operation count, median, p95, and p99 latency; median must be <= 250 ms and p99 <= 750 ms on the local gate host, and missing metrics or a zero-operation report fails.
// Contract: Every measured codegen operation validates the exact five-file TypeScript output and Defer task client methods, so faster placeholder or empty generation cannot satisfy the efficiency threshold.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_cli_interface_efficiency() {
    let command = "cargo test --release -p defer --test cli_efficiency -- --ignored --nocapture";
    let id = "defer-cli-interface-efficiency";
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
