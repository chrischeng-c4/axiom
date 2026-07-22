// SPEC-MANAGED: apps/tape/external-contracts/topology/behavior/shard-topology.md#tape-topology-existing-raft-replica-sync
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-topology-existing-raft-replica-sync
// @capability primary-replicas
// @claim tape-raft-log-replica-sync-existing-pvc
// @contract tape-topology-existing-raft-replica-sync
// @category stability
// @required_for_production true
// @command cargo test -p tape --test raft_cluster --test raft_failover --test raft_persistence -- --test-threads=1
// AW-EC-END

// Contract: A Tape Raft group elects, replicates committed journal appends, forwards follower appends to its leader, and retains the durable applied floor across restart.
// Contract: A fresh Tape node catches up by InstallSnapshot and a killed leader is replaced by a surviving elected group without committed-event loss.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_topology_existing_raft_replica_sync() {
    let command =
        "cargo test -p tape --test raft_cluster --test raft_failover --test raft_persistence -- --test-threads=1";
    let id = "tape-topology-existing-raft-replica-sync";
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
