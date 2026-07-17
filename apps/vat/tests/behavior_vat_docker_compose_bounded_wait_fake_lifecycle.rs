// SPEC-MANAGED: apps/vat/tech-design/logic/vat-headless-docker-command-shim.md#vat-docker-compose-bounded-wait-fake-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-compose-bounded-wait-fake-lifecycle
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-compose-bounded-wait-fake-lifecycle
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_docker_shim -- --nocapture
// AW-EC-END

// Contract: The passed deterministic fake suite covers ready, timeout, later recovery/down cleanup, and down/re-import/relaunch replacement races for docker compose up -d --wait.
// Contract: It proves one final ready up JSON with topology, timeout runtime/registry retention, target-pinned safe ps handoff only after current observation, and no unsafe next for terminal/replaced/bare deadlines; degraded exposes no endpoint.
// Contract: The corresponding opt-in real Apple Container dual-service E2E is passed on this host; the fake suite remains the deterministic coverage for timeout/recovery/replacement races.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_compose_bounded_wait_fake_lifecycle() {
    let command = "cargo test -p vat --test vat_docker_shim -- --nocapture";
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
