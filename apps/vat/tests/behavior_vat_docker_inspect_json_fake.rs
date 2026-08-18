// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-docker-inspect-json-fake
// @capability agent-native-gpu-native-dev-containers
// @claim headless-docker-command-shim
// @contract vat-docker-inspect-json-fake
// @category behavior
// @required_for_production true
// @command RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_inspect -- --nocapture
// AW-EC-END

// Contract: The deterministic fake contract accepts direct docker inspect JSON and only the documented container inspect alias, strips the VAT-only selector, invokes canonical Apple Container inspect argv, and byte-for-byte replays one validated opaque native JSON value.
// Contract: --type, --size, templates/table/YAML/TOML, filters, a second id, --, and unknown flags fail before runtime; unformatted inspect remains inherited, valid JSON plus a nonzero child exit preserves status, and malformed, oversized, or flood output suppresses raw stdout under five-second bounded isolated cleanup.
// Contract: It is not Docker Engine inspect schema, ownership/provenance/security/image/registry/build-status, health/readiness/liveness/port-reachability evidence, or a secret-redaction guarantee. Recorded validation: cargo check without default features passed; shared docker_shim library passed 54/54; focused docker_inspect integration passed 5/5. The full serial fake-shim aggregate is intentionally not recorded because an independent serial run exposed a nondeterministic pre-existing Compose JSON logs timing race. Direct real-host observation passed 1/1 (50 filtered) on Apple Container 1.1.0; inspect targets the temporary owner-labeled nginx container and proves one valid native JSON document only. Fake/unit tests prove byte-preservation and fail-closed details.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_docker_inspect_json_fake() {
    let command =
        "RUST_TEST_THREADS=1 cargo test -p vat --test vat_docker_shim docker_inspect -- --nocapture";
    let id = "vat-docker-inspect-json-fake";
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
