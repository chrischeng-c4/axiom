// SPEC-MANAGED: apps/defer/external-contracts/behavior/2214.md#defer-delayed-task-kind-pvc-recovery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-delayed-task-kind-pvc-recovery
// @capability long-running-stability
// @claim delayed-task-soak-and-recovery
// @contract operator-pvc-pod-replacement-and-post-recovery-mutation
// @category stability
// @required_for_production true
// @command bash apps/defer/scripts/kind-e2e.sh
// AW-EC-END

// Contract: The gate requires Docker, Kind, kubectl, curl, and jq; it creates a disposable cluster, builds and loads the real source image, installs the Defer CRD/operator, reconciles a single-shard StatefulSet, and waits until its PVC is Bound with an exact 1Gi request and capacity before exercising the API.
// Contract: Two future-scheduled tasks committed through the public batch API are visible before replacement; deleting the serving pod must produce a different pod UID, regain readiness, and recover the same queue inventory and task identity from PVC-backed Raft state.
// Contract: After recovery, queue pause and task cancellation both commit, the canceled task becomes terminal Canceled, queue accounting remains two records with one terminal record, and a successful gate requires cluster deletion plus an explicit absence check unless preservation was requested.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_delayed_task_kind_pvc_recovery() {
    let command = "bash apps/defer/scripts/kind-e2e.sh";
    let id = "defer-delayed-task-kind-pvc-recovery";
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
