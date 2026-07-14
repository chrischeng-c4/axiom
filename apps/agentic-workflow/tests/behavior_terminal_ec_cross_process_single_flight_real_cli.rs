// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/aw-terminal-vat-ec-process-lifecycle.md#terminal-ec-cross-process-single-flight-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec terminal-ec-cross-process-single-flight-real-cli
// @capability td-cb-lifecycle-automation
// @claim terminal-ec-process-liveness
// @contract terminal-ec-cross-process-single-flight-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests test_code_check_cross_process_single_flight_prevents_duplicate_ec_launch -- --nocapture
// AW-EC-END

// Contract: the first aw process owns the project lock while its EC command runs
// Contract: the second same-slug aw process returns terminal_ec_single_flight promptly
// Contract: both refusal envelopes point to exact aw td code-check slug retry commands
// Contract: the append-only EC launch marker contains exactly one line
// Contract: the work item remains open in cb_filled and no terminal commit is created
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn terminal_ec_cross_process_single_flight_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests test_code_check_cross_process_single_flight_prevents_duplicate_ec_launch -- --nocapture";
    let id = "terminal-ec-cross-process-single-flight-real-cli";
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
