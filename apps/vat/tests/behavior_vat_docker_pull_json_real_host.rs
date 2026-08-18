// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-pull-json-real-host
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-pull-json-real-host
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_pull_json_receipt_contract -- --ignored --nocapture
// AW-EC-END

// Contract: Passed 1/1 (73 filtered) in 27.14 seconds. The opt-in probe proves one strict Docker-pull receipt and records public Apple `container image pull alpine:3.20` argv with JSON/deadline selectors stripped.
// Contract: The E2E deliberately uses a shared/cacheable alpine image but still runs the real pull client: it neither deletes that image nor asserts ownership on success or failure. It can contact a registry or alter shared image state, so it is bounded receipt evidence rather than transfer, image-state, registry-auth, or cleanup proof.
// Contract: The receipt remains not_owned_no_auto_cleanup; this host proof does not establish Docker Engine/API, registry management/auth lifecycle, provenance, digest, platform, freshness, security, secret redaction, cancellation, download completion, or rollback.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_pull_json_real_host() {
    let command =
        "RUST_TEST_THREADS=1 VAT_DOCKER_PULL_JSON_E2E_REQUIRED=1 cargo test -p vat --test vat_docker_shim apple_container_docker_pull_json_receipt_contract -- --ignored --nocapture";
    let id = "vat-docker-pull-json-real-host";
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
