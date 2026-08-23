// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-ps-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-ps-json-fake
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_ps_json -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts direct docker ps JSON and only the documented container ls/list aliases, normalizes to canonical Apple Container list argv, and byte-for-byte replays one validated opaque native JSON value.
// Contract: Templates/table output, filters, quiet plus JSON, duplicate/unknown flags, positionals, and docker container ps JSON fail before runtime; malformed, oversized, or escaped-pipe stdout fails closed under the five-second bounded isolated cleanup.
// Contract: Recorded validation: cargo check without default features passed; shared docker_shim library passed 54/54; focused direct ps integration passed 4/4. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; ps is a global read-only inventory smoke observation, not a targeted ownership result. It proves one valid native JSON document only; fake/unit tests prove byte-preservation and fail-closed details.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_ps_json_fake() {
    let command =
        "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_ps_json -- --nocapture";
    let id = "vat-docker-ps-json-fake";
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
