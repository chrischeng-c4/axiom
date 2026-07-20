// SPEC-MANAGED: apps/defer/external-contracts/behavior/2216.md#defer-competitor-durable-ha
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-durable-ha
// @capability competitor-feature-parity
// @claim delayed-task-competitor-feature-matrix
// @contract replicated-scheduler-state-and-repeated-failover
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test raft_scheduler -- --nocapture
// AW-EC-END

// Contract: A real three-node Raft cluster commits queue and task state, preserves the live lease across leader loss, rejects wrong-replica and stale-epoch settlement, and only reassigns after committed expiry with a higher epoch and fresh attempt id.
// Contract: The failed node restarts from the same durable directory, catches up terminal state, then a second leader loss elects another primary that commits and completes a new task, proving repeated durable HA rather than a one-shot memory replay.
// Contract: Rate, burst, and max-in-flight limits are consumed by proposals through different followers as one committed aggregate; the next leader preserves both the sub-second rate denial and active in-flight denial, then admits work only after the committed one-second refill or exact fenced acknowledgement.
// Contract: A two-attempt task is nacked through a follower from Retried to DeadLettered, every live replica converges, and a killed node restarted from the same durable directory recovers one terminal task with no scheduled or in-flight residue.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_durable_ha() {
    let command = "cargo test -p defer --test raft_scheduler -- --nocapture";
    let id = "defer-competitor-durable-ha";
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
