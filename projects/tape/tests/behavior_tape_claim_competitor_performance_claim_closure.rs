// SPEC-MANAGED: projects/tape/external-contracts/claim-closure/production-claims.md#tape-competitor-performance-claim-closure
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-competitor-performance-claim-closure
// @capability competitor-performance
// @claim topic-replay-competitor-performance-baseline
// @contract topic-replay-performance-local-and-nats-win
// @category efficiency
// @required_for_production true
// @command cargo test -p tape --test tape_perf_gate -- --nocapture && cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture
// AW-EC-END

// Contract: The local Tape performance regression gate passes for append, replay, and checkpoint operations.
// Contract: Tape's NATS JetStream local backlog replay win is backed by a real-service benchmark gate.
// Contract: Other replay-log peer performance wins remain unclaimed until calibrated real-service benchmark runs exist.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_competitor_performance_claim_closure() {
    let command =
        "cargo test -p tape --test tape_perf_gate -- --nocapture && cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture";
    let id = "tape-competitor-performance-claim-closure";
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
