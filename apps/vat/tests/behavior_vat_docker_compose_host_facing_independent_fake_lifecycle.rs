// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-compose-host-facing-independent-fake-lifecycle
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-compose-host-facing-independent-fake-lifecycle
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_docker_shim compose_host_facing_independent_profile_runs_two_services_through_the_shim -- --nocapture
// AW-EC-END

// Contract: A deterministic fake runtime starts two literal-image services selected by the exact host-facing-independent-v1 marker, with two distinct loopback host ports.
// Contract: The successful up JSON exposes profile=host-facing-independent-v1, service_name_dns=false, and host_loopback_only=true; exact no-argument ps preserves that known profile and adds ready topology in registered docs/inspector order with canonical loopback endpoints.
// Contract: A typed degraded ps omits every endpoint rather than null-filling or leaking a partial topology; ps --format is unsupported and topology is not an app-healthcheck.
// Contract: This deterministic fixture complements the opt-in real Apple Container dual-service E2E; it does not widen that passed host evidence to service-name DNS, general Compose, Docker Engine API, or Kubernetes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_compose_host_facing_independent_fake_lifecycle() {
    let command =
        "cargo test -p vat --test vat_docker_shim compose_host_facing_independent_profile_runs_two_services_through_the_shim -- --nocapture";
    let id = "vat-docker-compose-host-facing-independent-fake-lifecycle";
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
