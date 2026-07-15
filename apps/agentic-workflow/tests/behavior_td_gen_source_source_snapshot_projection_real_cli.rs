// SPEC-MANAGED: apps/agentic-workflow/tech-design/semantic/td-gen-source-source-snapshot-projection.md#td-gen-source-source-snapshot-projection-real-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec td-gen-source-source-snapshot-projection-real-cli
// @capability existing-project-standardization
// @claim authoritative-source-snapshot-projection
// @contract td-gen-source-source-snapshot-projection-real-cli
// @category behavior
// @required_for_production true
// @command cargo test -p agentic-workflow --test cli_tests test_gen_source_projects_legacy_snapshot_and_runs_generated_test -- --nocapture
// AW-EC-END

// Contract: a const changes from before to after in the exact requested target
// Contract: a uniquely named generated Rust test is present in exact target bytes
// Contract: cargo test with that unique filter reports running 1 test and 1 passed
// Contract: siblings and an unmatched existing target remain byte-identical
// Contract: a second replay reports summary.wrote_files=false
// Contract: the unmatched target error names the snapshot target and runnable --target remediation
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn td_gen_source_source_snapshot_projection_real_cli() {
    let command =
        "cargo test -p agentic-workflow --test cli_tests test_gen_source_projects_legacy_snapshot_and_runs_generated_test -- --nocapture";
    let id = "td-gen-source-source-snapshot-projection-real-cli";
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
