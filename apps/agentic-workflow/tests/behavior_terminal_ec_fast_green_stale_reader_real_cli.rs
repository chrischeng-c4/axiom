// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/aw-terminal-vat-ec-process-lifecycle.md#terminal-ec-fast-green-stale-reader-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec terminal-ec-fast-green-stale-reader-real-cli
// @capability td-cb-lifecycle-automation
// @claim terminal-ec-process-liveness
// @contract terminal-ec-fast-green-stale-reader-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests test_code_check_fast_green_stale_reader_rechecks_phase_before_ec -- --nocapture
// AW-EC-END

// Contract: a debug-only bounded barrier proves process B read cb_filled before process A completes
// Contract: process A executes the fast-green inventory and completes the terminal transition
// Contract: process B acquires afterward, re-reads td_merged, and reports terminal retry without EC
// Contract: the EC launch marker contains one line and git contains one Cb-CodeCheck terminal commit
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn terminal_ec_fast_green_stale_reader_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests test_code_check_fast_green_stale_reader_rechecks_phase_before_ec -- --nocapture";
    let id = "terminal-ec-fast-green-stale-reader-real-cli";
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
