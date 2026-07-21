// SPEC-MANAGED: apps/defer/external-contracts/behavior/2220.md#defer-kubernetes-cli-layered-artifacts
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-kubernetes-cli-layered-artifacts
// @capability kubernetes-native-deployment
// @claim dedicated-task-service-topology
// @contract cli-owned-image-crd-operator-and-instance-layers
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test cli_contract deploy_artifacts_render_by_lifecycle_layer -- --nocapture
// AW-EC-END

// Contract: The shipped Defer CLI independently renders source and release Dockerfiles with executable entrypoints, a structural Defer CRD, an operator Deployment in the requested namespace with downward-API pod identity, and a production Defer instance with three replicas per shard and scheduled backup configuration; every command exit status is checked before content.
// Contract: The release Dockerfile is version-bound to the current Defer package, so a stale image artifact or a renderer that emits only Kubernetes YAML cannot satisfy the lifecycle-layer contract.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_kubernetes_cli_layered_artifacts() {
    let command =
        "cargo test -p defer --test cli_contract deploy_artifacts_render_by_lifecycle_layer -- --nocapture";
    let id = "defer-kubernetes-cli-layered-artifacts";
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
