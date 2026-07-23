// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/specs/aw-capability-claim-closure-ec-inventory.md#aw-core-client-typed-prompt-ir-and-envelope-projection
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec aw-core-client-typed-prompt-ir-and-envelope-projection
// @capability aw-core-client-model-workitem-first-artifact-lifecycle
// @claim typed-prompt-ir-and-envelope-projection
// @contract aw-core-client-typed-prompt-ir-and-envelope-projection
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --lib cli::run::tests::workflow_envelope_serializes_typed_prompt_contract_from_same_ir -- --exact --nocapture
// AW-EC-END

// Contract: a production WorkflowEnvelope pins every typed prompt JSON field and its rendered agent_prompt from the same decoded IR
// Contract: an invalid typed contract makes WorkflowEnvelope serialization fail instead of falling back to prose
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn aw_core_client_typed_prompt_ir_and_envelope_projection() {
    let command =
        "cargo test -p agentic-workflow --lib cli::run::tests::workflow_envelope_serializes_typed_prompt_contract_from_same_ir -- --exact --nocapture";
    let id = "aw-core-client-typed-prompt-ir-and-envelope-projection";
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
