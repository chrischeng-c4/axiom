// SPEC-MANAGED: apps/workbench/external-contracts/stability/folder-agent-artifact-stability.md#folder-agent-artifact-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec folder-agent-artifact-stability
// @capability terminal-first-agent-workbench
// @claim folder-agent-artifact-production-stability
// @contract folder-agent-artifact-stability
// @category stability
// @required_for_production true
// @command cargo test -p workbench --test production_journey -- --nocapture
// AW-EC-END

// Contract: Twelve consecutive production Tauri IPC sessions each complete real PTY launch, input, resize, and one of interrupt, terminate, or normal-exit lifecycle modes.
// Contract: Every child process id observed at launch is reaped and no longer alive after the cycle; the selected canonical folder remains unchanged.
// Contract: Unavailable-agent recovery precedes a successful real launch, and Git, Markdown, and AW context plus source navigation are rendered and asserted after every completed lifecycle cycle.
// Contract: Every session transcript remains at or below the 524288-byte production bound, with measured peak, twelve context cycles, and lifecycle modes retained in ipc-journey.json.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn folder_agent_artifact_stability() {
    let command = "cargo test -p workbench --test production_journey -- --nocapture";
    let id = "folder-agent-artifact-stability";
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
