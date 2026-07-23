// SPEC-MANAGED: apps/agentic-workflow/tech-design/surface/specs/aw-self-hosting-runner-policy.md#self-hosting-health-policy
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec self-hosting-health-policy
// @capability workflow-root-runner
// @claim python-first-self-hosting-admission
// @contract self-hosting-health-policy
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test python_first_self_hosting_health_reports_bounded_fallback -- --nocapture
// AW-EC-END

// Contract: health emits policy_mode python_first_lifecycle and fallback_mode bounded_direct_repair
// Contract: health pins fallback_trigger selected_worker_verb_is_broken, fallback_scope current_change_only, fallback_required_trailer Refs #<issue>, and direct_repair_default false
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn self_hosting_health_policy() {
    let command =
        "cargo test -p agentic-workflow --test self_hosting_runner_policy_cli_test python_first_self_hosting_health_reports_bounded_fallback -- --nocapture";
    let id = "self-hosting-health-policy";
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
