// SPEC-MANAGED: apps/tape/external-contracts/cli-interface/behavior/cli-interface.md#tape-cli-interface-offline-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-cli-interface-offline-cli
// @capability cli-interface
// @claim tape-cli-replay-admin-contract
// @contract tape-offline-cli-agent-onboarding
// @category behavior
// @required_for_production true
// @command cargo test -p tape --test cli_contract --test behavior_tape_claim_cli_interface -- --nocapture
// AW-EC-END

// Contract: tape append, replay, checkpoint, spec, llm, upgrade, and issue parse through the real Tape binary.
// Contract: tape spec emits deterministic route, OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline.
// Contract: tape llm publishes deterministic, offline guidance for topic replay and checkpoint workflows.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_cli_interface_offline_cli() {
    let command =
        "cargo test -p tape --test cli_contract --test behavior_tape_claim_cli_interface -- --nocapture";
    let id = "tape-cli-interface-offline-cli";
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
