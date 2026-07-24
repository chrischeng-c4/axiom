// SPEC-MANAGED: apps/agentic-workflow/tech-design/specs/3901.md#authoritative-fixture-blocks-on-regenerability-gap
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec authoritative-fixture-blocks-on-regenerability-gap
// @capability existing-project-standardization
// @claim authoritative-fixture-blocks-on-regenerability-gap
// @contract authoritative-fixture-blocks-on-regenerability-gap
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests authoritative_regenerability_gaps_block_project_health -- --nocapture
// AW-EC-END

// Contract: a non-self fixture configured generator_authoritative reports production_ready false for a tracked regenerability gap
// Contract: the health payload exposes the regenerability production blocker and a runnable remediation command
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn authoritative_fixture_blocks_on_regenerability_gap() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests authoritative_regenerability_gaps_block_project_health -- --nocapture";
    let id = "authoritative-fixture-blocks-on-regenerability-gap";
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
