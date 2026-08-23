// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-headless-leased-k8s-local-image-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim headless-ephemeral-kubernetes-session
// @contract vat-headless-leased-k8s-local-image-real-host
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_imports_local_image_without_registry_pull -- --ignored --nocapture
// AW-EC-END

// Contract: Passed 1/1 (36 filtered) in 49.73s: an already-local Apple alpine:3.20 image is inspected, privately delivered into one active K3s lease, and reported with an OCI descriptor digest.
// Contract: The one fixture pod uses imagePullPolicy Never, completes, and emits its marker log; this proves the imported local image for that pod only, not registry-pull generality.
// Contract: Explicit delete confirms exact Apple machine and private session storage cleanup. This is not persistence, GUI, or Docker Engine/API evidence.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_headless_leased_k8s_local_image_real_host() {
    let command =
        "RUST_TEST_THREADS=1 VAT_K8S_LOCAL_IMAGE_E2E_REQUIRED=1 cargo test -p vat --test vat_k8s_ephemeral apple_container_k3s_lease_imports_local_image_without_registry_pull -- --ignored --nocapture";
    let id = "vat-headless-leased-k8s-local-image-real-host";
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
