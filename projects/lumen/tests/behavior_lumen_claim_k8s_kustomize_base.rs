// SPEC-MANAGED: projects/lumen/external-contracts/claim-closure/production-claims.md#lumen-claim-k8s-kustomize-base
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-claim-k8s-kustomize-base
// @capability kubernetes-native-deployment
// @claim kustomize-base-overlays-hpa
// @contract k8s-kustomize-base-overlays
// @category behavior
// @required_for_production true
// @command kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator
// AW-EC-END

// Contract: The Kubernetes base, overlays, HPA/PDB, and operator manifests render successfully.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_claim_k8s_kustomize_base() {
    let command =
        "kustomize build projects/lumen/k8s/base && kustomize build projects/lumen/k8s/overlays/dev && kustomize build projects/lumen/k8s/overlays/staging && kustomize build projects/lumen/k8s/overlays/prod && kustomize build projects/lumen/k8s/operator";
    let id = "lumen-claim-k8s-kustomize-base";
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
