// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-stats-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-stats-fake
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_docker_shim docker_stats -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts only strict non-streaming native-JSON stats, invokes canonical Apple Container argv, preserves valid opaque native JSON and child nonzero exits, and suppresses malformed/oversized stdout.
// Contract: A five-second bounded observation plus isolated process-group cleanup replays stdout only after complete validated capture; an escaped pipe holder fails closed. It does not prove ownership, health, liveness, or a Docker Engine schema.
// Contract: Recorded validation: shared docker_shim library coverage passed 54/54. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; stats targets the temporary owner-labeled nginx container and proves one valid native JSON document only. Fake/unit tests prove byte-preservation and fail-closed details.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_stats_fake() {
    let command = "cargo test -p vat --test vat_docker_shim docker_stats -- --nocapture";
    let id = "vat-docker-stats-fake";
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
