// SPEC-MANAGED: apps/workbench/external-contracts/behavior/folder-agent-artifact-journey.md#folder-agent-artifact-journey
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec folder-agent-artifact-journey
// @capability terminal-first-agent-workbench
// @claim folder-agent-artifact-production-journey
// @contract folder-agent-artifact-journey
// @category behavior
// @required_for_production true
// @command cargo test -p workbench --test production_journey -- --nocapture
// AW-EC-END

// Contract: The production configure_builder Tauri IPC handler resolves a canonical registered folder and launches a deterministic agent executable through the same real PTY command boundary used by Claude Code, Codex, and AGY; only the executable is substituted.
// Contract: The composed IPC journey sends input, resizes, interrupts, terminates, observes OSC 7 cwd, and renders Git, Markdown, and configured AW context with canonical source navigation.
// Contract: Unavailable-agent errors cross the production IPC boundary without losing the selected folder, and a subsequent available agent launch succeeds.
// Contract: Jet rejects invalid production bridge arguments and asserts recorded launch agent and canonical cwd, terminal input, context root, and context target values before accepting keyboard, desktop, constrained, and placeholder-free evidence.
// Contract: apps/workbench/evidence/production-journey/v1/manifest.json binds these assertions to ipc-journey.json, the PTY transcript, context summary, and screenshots.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn folder_agent_artifact_journey() {
    let command = "cargo test -p workbench --test production_journey -- --nocapture";
    let id = "folder-agent-artifact-journey";
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
