// SPEC-MANAGED: libs/service-http/external-contracts/behavior/2420.md#shared-w3c-request-completion-event
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec shared-w3c-request-completion-event
// @capability shared-http-service-scaffold
// @claim shared-http-service-scaffold-contract
// @contract service-http-request-completion-event
// @category behavior
// @required_for_production true
// @command cargo test -p service-http --test request_completion_event
// AW-EC-END

// Contract: One completed HTTP request produces exactly one decoded axiom.service.log.v1 INFO record whose event is http_request_complete and whose attributes contain method, uri, status, and non-negative latency_ms.
// Contract: The same decoded completion record preserves a valid inbound W3C trace_id, parent_span_id, and trace_flags while using a distinct non-zero local span_id.
// Contract: A missing or malformed traceparent produces a fresh valid trace_id and span_id with no parent_span_id, while still emitting one completion record.
// Contract: The public trace_layer API takes no collector endpoint, credential, routing, storage, or Sift-specific configuration; collector ownership remains outside service-http.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn shared_w3c_request_completion_event() {
    let command = "cargo test -p service-http --test request_completion_event";
    let id = "shared-w3c-request-completion-event";
    let mut root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    while !root.join("aw.toml").is_file() {
        assert!(
            root.pop(),
            "AW EC {id}: no aw.toml repository root above {}",
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
