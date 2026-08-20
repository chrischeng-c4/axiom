// SPEC-MANAGED: apps/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-developer-llm-v2-task-navigation
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-developer-llm-v2-task-navigation
// @capability api-cli-agent-integration
// @claim standard-cli-and-agent-interface
// @contract developer-llm-v2-task-navigation
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli dx_llm_v2_json_and_markdown_share_one_typed_contract -- --exact --nocapture
// AW-EC-END

// Contract: Agent task navigation exposes one deterministic typed contract in Markdown and JSON.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_developer_llm_v2_task_navigation() {
    let command =
        "cargo test -p lumen --test spec_cli dx_llm_v2_json_and_markdown_share_one_typed_contract -- --exact --nocapture";
    let id = "lumen-claim-developer-llm-v2-task-navigation";
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
