// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/td-generation-target-ownership.md#td-generation-target-ownership-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec td-generation-target-ownership-real-cli
// @capability td-cb-lifecycle-automation
// @claim ambiguous-multi-target-generation-preflight
// @contract td-generation-target-ownership-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests td_gen_ambiguous_schema_plan_fails_before_any_lifecycle_mutation -- --nocapture
// AW-EC-END

// Contract: the public binary emits exactly one stdout JSON error envelope and no second stderr error
// Contract: error_kind, section, sorted targets, completion, and executable next.command are stable
// Contract: HEAD, symbolic branch, index tree, porcelain-z status, issue bytes, and TD branch ref are unchanged
// Contract: the prepared spec and every target blob remain byte-identical
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn td_generation_target_ownership_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests td_gen_ambiguous_schema_plan_fails_before_any_lifecycle_mutation -- --nocapture";
    let id = "td-generation-target-ownership-real-cli";
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
