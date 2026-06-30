// SPEC-MANAGED: projects/vat/tech-design/interfaces/cli/vat-upgrade-and-report-issue-subcommands-for-the-mandatory-cli-c.md#vat-cli-convention-help-lists-all-three
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-cli-convention-help-lists-all-three
// @capability agent-native-gpu-native-dev-containers
// @claim vat-llm-vat-upgrade-vat-issue-mandatory-cli-convention
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_cli_convention -- --nocapture
// AW-EC-END

// Contract: `vat --help` output contains `llm`, `upgrade`, and `issue`.
// Contract: `vat upgrade --check` exits 0 and reports current vs latest without writing the binary (network-permitting; offline it errors cleanly, never panics).
// Contract: `vat issue create --title X --dry-run` prints a body containing the vat version and OS/arch and submits nothing.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_cli_convention_help_lists_all_three() {
    let command = "cargo test -p vat --test vat_cli_convention -- --nocapture";
    let id = "vat-cli-convention-help-lists-all-three";
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
