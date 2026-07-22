// SPEC-MANAGED: apps/defer/external-contracts/behavior/2219.md#defer-http2-live-one-port-route-contract
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http2-live-one-port-route-contract
// @capability http2-api-list
// @claim h2c-openapi-route-list
// @contract one-port-http1-h2c-standard-and-domain-routes
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_api -- --nocapture
// AW-EC-END

// Contract: Independent HTTP/1.1 and prior-knowledge h2c clients use the same bound service URL and each receive 200 from /healthz, /readyz, /docs, /openapi.json, and /metrics, while an HTTP/1.1 queue GET succeeds after an h2c queue PUT.
// Contract: The served OpenAPI JSON is parsed and must equal the canonical Defer OpenAPI IR; its exact sorted inventory is nine method/path operations covering queue GET/PUT/control, task create/batch-create/status/cancel, dispatch, and admin backup, so an added, omitted, or wrong-method operation fails.
// Contract: Real live requests exercise every advertised domain operation: queue configure/read/control, individual and batch task creation, task status and cancellation, dispatch to a real target, and backup whose bytes recover the committed terminal task into a fresh Raft store.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http2_live_one_port_route_contract() {
    let command = "cargo test -p defer --test http_api -- --nocapture";
    let id = "defer-http2-live-one-port-route-contract";
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
