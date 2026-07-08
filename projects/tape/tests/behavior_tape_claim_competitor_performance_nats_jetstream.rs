// SPEC-MANAGED: projects/tape/external-contracts/competitor-performance/efficiency/competitive-benchmark.md#tape-competitor-performance-nats-jetstream-replay-win
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-competitor-performance-nats-jetstream-replay-win
// @capability competitor-performance
// @claim topic-replay-competitor-performance-baseline
// @contract topic-replay-nats-jetstream-local-backlog-win
// @category efficiency
// @required_for_production true
// @command cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture
// AW-EC-END

// Contract: The test starts a real local nats-server with JetStream enabled.
// Contract: Tape and JetStream replay the same 20,000-event, 128-byte-payload backlog workload from the beginning.
// Contract: Tape's zero-copy full-replay latency is at least 1.5x faster than NATS JetStream for the local backlog replay workload.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_competitor_performance_nats_jetstream_replay_win() {
    let command = "cargo test -p tape --test tape_vs_nats_jetstream -- --nocapture";
    let id = "tape-competitor-performance-nats-jetstream-replay-win";
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
