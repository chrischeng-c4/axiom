// SPEC-MANAGED: projects/cap/tech-design/logic/external-contracts.md#cap-standard-agent-cli-operations
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-standard-agent-cli-operations
// @capability standard-agent-cli-operations
// @claim shared-standard-cli-commands
// @contract standard-agent-cli-operations
// @category behavior
// @required_for_production true
// @command cargo test -p cap --lib cli_std_convention -- --nocapture && cargo test -p cap installed_frontend_exposes_standard_agent_commands -- --nocapture && cargo build -p cap --features release
// AW-EC-END

// Contract: cap help lists llm, upgrade, issue, and report-issue compatibility commands
// Contract: installed cap frontend delegates standard commands through the cap-full sibling
// Contract: installed cap frontend preserves the caller environment for cap-full passthrough commands
// Contract: cap llm renders cap-specific offline docs through cli-std
// Contract: cap issue create and report-issue dry-run payloads carry project:cap diagnostics
// Contract: release-feature builds enable cli-std online paths
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_standard_agent_cli_operations() {
    let command =
        "cargo test -p cap --lib cli_std_convention -- --nocapture && cargo test -p cap installed_frontend_exposes_standard_agent_commands -- --nocapture && cargo build -p cap --features release";
    let id = "cap-standard-agent-cli-operations";
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
