// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-compose-bounded-wait-fake-lifecycle
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-compose-bounded-wait-fake-lifecycle
// @category behavior
// @required_for_production true
// @command VAT_DOCKER_COMPOSE_SHIM_LIFECYCLE_REQUIRED=1 RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim compose_wait_ -- --nocapture
// AW-EC-END

// Contract: The focused deterministic fake cases cover ready, timeout, later recovery, and down cleanup for docker compose up -d --wait.
// Contract: They prove one final ready up JSON with topology, timeout runtime/registry retention, later ready observation of the same launch, and no endpoint leakage from a timed-out result.
// Contract: The corresponding opt-in real Apple Container dual-service E2E is passed on this host; the fake suite remains the deterministic coverage for timeout/recovery/replacement races.
// Contract: The production EC command fails closed instead of skipping when the runner forbids the loopback sockets required by the fake lifecycle.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_compose_bounded_wait_fake_lifecycle() {
    let command =
        "VAT_DOCKER_COMPOSE_SHIM_LIFECYCLE_REQUIRED=1 RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim compose_wait_ -- --nocapture";
    let id = "vat-docker-compose-bounded-wait-fake-lifecycle";
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
