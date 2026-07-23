// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/specs/aw-artifact-skeleton-fill-protocol.md#td-artifact-producer-cli-fixture
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec td-artifact-producer-cli-fixture
// @capability aw-core-client-model-workitem-first-artifact-lifecycle
// @claim shared-artifact-producer-contract
// @contract td-artifact-producer-cli-fixture
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test artifact_producer_cli_test td_create_emits_cli_owned_skeleton_structured_slots_and_ownership -- --nocapture
// AW-EC-END

// Contract: aw td create creates the durable TD skeleton and one JSON payload for the current queued section
// Contract: the TD contract exposes validation, generation, evidence, and a runnable next transition
// Contract: CODEGEN-BEGIN/END and HANDWRITE-BEGIN/END ownership outputs are explicit
// Contract: HANDWRITE requires gap, tracker, and reason fields
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn td_artifact_producer_cli_fixture() {
    let command =
        "cargo test -p agentic-workflow --test artifact_producer_cli_test td_create_emits_cli_owned_skeleton_structured_slots_and_ownership -- --nocapture";
    let id = "td-artifact-producer-cli-fixture";
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
