// SPEC-MANAGED: apps/defer/external-contracts/behavior/2220.md#defer-kubernetes-kind-pvc-recovery
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-kubernetes-kind-pvc-recovery
// @capability kubernetes-native-deployment
// @claim dedicated-task-service-topology
// @contract operator-reconcile-pvc-replacement-and-domain-recovery
// @category stability
// @required_for_production true
// @command bash apps/defer/scripts/kind-e2e.sh
// AW-EC-END

// Contract: The gate requires Docker, Kind, kubectl, curl, and jq; it creates a disposable cluster, builds and loads the real source image, installs the Defer CRD and operator, and observes the reconciled single-shard StatefulSet probes and /data mount, exact headless peer and client Service selectors/ports, maxUnavailable=1 PDB, and a Bound PVC with exact 1Gi requested and provisioned capacity before domain mutation.
// Contract: Two future-scheduled tasks committed through the public batch API are visible before pod deletion; replacement must produce a different pod UID, regain readiness, and recover the same queue inventory and task identity from PVC-backed Raft state.
// Contract: After replacement, queue pause and task cancellation must commit and remain readable with exact terminal accounting; success also requires deleting the disposable Kind cluster and proving its name absent unless explicit preservation is requested.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_kubernetes_kind_pvc_recovery() {
    let command = "bash apps/defer/scripts/kind-e2e.sh";
    let id = "defer-kubernetes-kind-pvc-recovery";
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
