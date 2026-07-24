// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/specs/aw-self-hosting-runner-policy.md#self-hosting-wi-admission
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec self-hosting-wi-admission
// @capability workflow-root-runner
// @claim self-hosting-root-runner-policy
// @contract self-hosting-wi-admission
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_work_item_root_is_rejected_before_loop_state_or_dispatch -- --nocapture
// AW-EC-END

// Contract: the WI root emits action self_hosting_policy before loop state or dispatch
// Contract: the envelope exposes no invoke command and both the repository tree and resolved runtime workspace remain byte-identical
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn self_hosting_wi_admission() {
    let command =
        "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test self_hosting_work_item_root_is_rejected_before_loop_state_or_dispatch -- --nocapture";
    let id = "self-hosting-wi-admission";
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
