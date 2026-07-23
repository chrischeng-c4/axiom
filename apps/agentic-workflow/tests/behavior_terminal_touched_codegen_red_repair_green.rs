// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/td-code-check-touched-codegen-drift.md#terminal-touched-codegen-red-repair-green
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec terminal-touched-codegen-red-repair-green
// @capability td-cb-lifecycle-automation
// @claim terminal-touched-codegen-drift-gate
// @contract terminal-touched-codegen-red-repair-green
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests test_code_check_terminal_touched_codegen_red_repair_green_unrelated_and_retry -- --nocapture
// AW-EC-END

// Contract: committed accepted CODEGEN drift refuses before EC and leaves phase, state, issue bytes, HEAD, index tree, cached diff, status, and target bytes unchanged
// Contract: the finding names only the accepted target and exact spec section while a second unaccepted generated target remains drifted
// Contract: the emitted aw cb gen slug command regenerates and commits only the accepted target, preserves terminal phase, and emits the exact retry command
// Contract: restored parity runs EC once, closes the WI, and a td_merged retry neither reruns EC nor duplicates the terminal commit
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn terminal_touched_codegen_red_repair_green() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests test_code_check_terminal_touched_codegen_red_repair_green_unrelated_and_retry -- --nocapture";
    let id = "terminal-touched-codegen-red-repair-green";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("aw.toml").is_file() {
        assert!(
            root.pop(),
            "AW EC {id}: no aw.toml repository root above {}",
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
