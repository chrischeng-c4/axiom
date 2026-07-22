// SPEC-MANAGED: apps/defer/external-contracts/behavior/2220.md#defer-kubernetes-rendered-stateful-topology
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-kubernetes-rendered-stateful-topology
// @capability kubernetes-native-deployment
// @claim dedicated-task-service-topology
// @contract direct-and-operator-stateful-resource-invariants
// @category behavior
// @required_for_production true
// @command cargo test -p defer --features operator --test direct_k8s_assets --test operator -- --nocapture
// AW-EC-END

// Contract: The direct Kustomize base is exactly one durable voter whose StatefulSet names defer-headless, exposes HTTP 7141 and Raft 7142, wires readiness/liveness/startup probes, mounts the data claim exactly once at /data, requests a named 10Gi ReadWriteOnce PVC, and uses restricted non-root security; its exact headless and client Service selectors/ports and maxUnavailable=0 PDB must match the pod selector.
// Contract: The composed production overlay renders successfully with a 100Gi PVC, token Secret projection, ServiceMonitor, NetworkPolicy, and no voter HorizontalPodAutoscaler; raw-string presence or an unconnected patch cannot pass.
// Contract: With the real operator feature enabled, the exact six-object production graph contains a three-replica StatefulSet tied to the jobs-headless Service, exact peer/client selectors and HTTP/Raft ports, maxUnavailable=1 PDB, probes and /data claim, connected token/signing/peer-TLS volumes, mounts, and env paths, plus a backup CronJob whose token Secret and destination are exact; the structural CRD contains signing and backup fields.
// Contract: Both test binaries must execute non-zero tests; compiling the operator test with its cfg disabled fails this contract.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_kubernetes_rendered_stateful_topology() {
    let command =
        "cargo test -p defer --features operator --test direct_k8s_assets --test operator -- --nocapture";
    let id = "defer-kubernetes-rendered-stateful-topology";
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
