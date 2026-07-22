// SPEC-MANAGED: apps/relay/external-contracts/competitor-performance/efficiency/perf-gate.md#relay-competitor-performance-behavior
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-competitor-performance-behavior
// @capability competitor-performance
// @claim normalized-win-ratchet-decision-model
// @contract relay-work-queue-performance-workload-behavior
// @category behavior
// @required_for_production true
// @command bash apps/relay/scripts/ec-evidence.sh performance-behavior
// AW-EC-END

// Contract: The publish, ordered lease, batch acknowledgement, committed-watermark, redelivery, and per-subject isolation workloads complete with every message acknowledged exactly once.
// Contract: The normalized decision model fails both a pinned-ratio regression and loss of a must-beat cell; this behavior case does not claim that its hard-coded inputs are measured performance.
// Contract: The outer oracle requires all eleven named behavior tests and a non-zero execution count in each test binary, so a renamed or removed test cannot pass as a zero-match Cargo filter.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_competitor_performance_behavior() {
    let command = "bash apps/relay/scripts/ec-evidence.sh performance-behavior";
    let id = "relay-competitor-performance-behavior";
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
