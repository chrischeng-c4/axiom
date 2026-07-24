// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/td-generation-target-ownership.md#td-generation-target-generator-gap-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec td-generation-target-generator-gap-real-cli
// @capability td-cb-lifecycle-automation
// @claim exact-generated-unit-target-ownership
// @contract td-generation-target-generator-gap-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests td_gen_unsupported_owned_unit_fails_before_lifecycle_mutation -- --nocapture
// AW-EC-END

// Contract: the public binary emits a typed owned_generated_unit_unsupported HITL envelope
// Contract: the stable unit ID, target, remediation command, and generator_gap reason are explicit
// Contract: HEAD, branch, index, status, issue, and target bytes remain unchanged
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn td_generation_target_generator_gap_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests td_gen_unsupported_owned_unit_fails_before_lifecycle_mutation -- --nocapture";
    let id = "td-generation-target-generator-gap-real-cli";
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
