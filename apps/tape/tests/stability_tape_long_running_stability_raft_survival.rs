// SPEC-MANAGED: apps/tape/external-contracts/long-running-stability/stability/resilience-survival.md#tape-long-running-stability-raft-survival
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-long-running-stability-raft-survival
// @capability long-running-stability
// @claim tape-raft-leader-loss-replay-survival
// @contract tape-raft-failover-and-snapshot-survival
// @category stability
// @required_for_production true
// @command cargo test -p tape --test raft_cluster --test raft_failover -- --test-threads=1
// AW-EC-END

// Contract: A three-node Tape group elects a leader, replicates ordered journal events, forwards follower writes, and continues after leader loss.
// Contract: A new Tape node catches up through snapshot installation without losing committed replay history.
// Contract: SIGKILL of the elected leader leaves surviving Tape nodes able to reelect with no committed-event loss.
// Contract: This contract proves replay survival; it does not claim search latency, packet-loss p99, or a completed live multi-shard soak.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_long_running_stability_raft_survival() {
    let command = "cargo test -p tape --test raft_cluster --test raft_failover -- --test-threads=1";
    let id = "tape-long-running-stability-raft-survival";
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
