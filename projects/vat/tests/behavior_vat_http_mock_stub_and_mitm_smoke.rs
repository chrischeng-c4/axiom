// SPEC-MANAGED: projects/vat/tech-design/logic/built-in-http-mock-record-replay-proxy.md#vat-http-mock-stub-and-mitm-smoke
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec vat-http-mock-stub-and-mitm-smoke
// @capability agent-native-gpu-native-dev-containers
// @claim built-in-http-mock-record-replay-proxy-https-mitm
// @contract local-agent-test-runner-protocol
// @category behavior
// @required_for_production true
// @command cargo test -p vat --test vat_emulator_httpmock -- --nocapture
// AW-EC-END

// Contract: a stubbed GET http://api.test/v1/x through the proxy returns the stub body with no network.
// Contract: a client trusting the minted CA and using the proxy GETs a stubbed https://api.test/v1/y via CONNECT+MITM and receives the stub (no real upstream).
// Contract: with no stub, a first GET to a local upstream records+forwards and a second GET replays from the cassette with the upstream down.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn vat_http_mock_stub_and_mitm_smoke() {
    let command = "cargo test -p vat --test vat_emulator_httpmock -- --nocapture";
    let id = "vat-http-mock-stub-and-mitm-smoke";
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
