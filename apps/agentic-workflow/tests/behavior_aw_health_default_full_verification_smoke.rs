// SPEC-MANAGED: apps/agentic-workflow/tech-design/validate/health-defaults-to-streaming-full-verification.md#aw-health-default-full-verification-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec aw-health-default-full-verification-smoke
// @capability existing-project-standardization
// @claim aw-health-default-full-verification-smoke
// @contract aw-health-default-full-verification-smoke
// @category behavior
// @required_for_production true
// @command ./target/debug/aw health --project agentic-workflow | tail -n 1 | grep -q payload_path
// AW-EC-END

// Contract: stdout includes progress JSONL events before the final result when long gates run
// Contract: the final result includes payload_path
// Contract: the payload file contains complete blocker and command evidence
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn aw_health_default_full_verification_smoke() {
    let command =
        "./target/debug/aw health --project agentic-workflow | tail -n 1 | grep -q payload_path";
    let id = "aw-health-default-full-verification-smoke";
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
