// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-logs-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-logs-json-fake
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_logs_json -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts direct/container logs only with exact format plus bounded tail selectors before one final safe id, strips the selector, invokes canonical Apple logs argv, and emits one VAT vat.docker.logs.v1/vat_json wrapper rather than a sixth native JSON or Docker multiplex/demux schema.
// Contract: The wrapper carries untrusted Apple stdio, bounded diagnostic stderr, truncation/lossy flags, backend/container/requested tail/runtime/child outcome, and safe inspect next. Ordinary child failure retains wrapper+exit; follow/boot/timestamps/since/until/templates and all other modifiers reject before runtime; timeout/setup/escaped-pipe paths emit no partial wrapper after five-second plus one-second bounded cleanup with dual-stream suffix/serialized caps.
// Contract: Recorded validation: cargo check without default features passed; canonical cargo test -p vat --lib docker_shim -- --nocapture passed 54/54; focused docker_logs_json integration passed 6/6. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; VAT logs targets the high-entropy nonce+PID owner-labeled temporary nginx container and proves one VAT wrapper only. Exact-label rechecks are conservative best-effort precautions, the emergency guard retains on uncertainty, and Apple Container has no atomic conditional delete; this is not a race-free or impossible-to-misdelete cleanup guarantee. No shared nginx image cleanup is claimed. Fake/unit tests prove byte-preservation and fail-closed details.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_logs_json_fake() {
    let command =
        "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_logs_json -- --nocapture";
    let id = "vat-docker-logs-json-fake";
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
