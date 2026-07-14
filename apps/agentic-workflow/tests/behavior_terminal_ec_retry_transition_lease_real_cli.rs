// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/aw-terminal-vat-ec-process-lifecycle.md#terminal-ec-retry-transition-lease-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec terminal-ec-retry-transition-lease-real-cli
// @capability td-cb-lifecycle-automation
// @claim terminal-ec-process-liveness
// @contract terminal-ec-retry-transition-lease-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests test_code_check_retry_contends_while_terminal_transition_holds_lease -- --nocapture
// AW-EC-END

// Contract: a bounded debug-only barrier pauses the owner after td_merged is written while its lease remains held
// Contract: the second process reads retry phase and promptly receives terminal_ec_single_flight
// Contract: the refusal points to the exact same-slug aw td code-check retry
// Contract: after releasing the owner there is one EC launch and one Cb-CodeCheck terminal commit
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn terminal_ec_retry_transition_lease_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests test_code_check_retry_contends_while_terminal_transition_holds_lease -- --nocapture";
    let id = "terminal-ec-retry-transition-lease-real-cli";
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
