// SPEC-MANAGED: apps/defer/external-contracts/behavior/766.md#defer-http-dispatch-signing-oracle-and-negatives
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http-dispatch-signing-oracle-and-negatives
// @capability http-dispatch-and-retries
// @claim http-target-attempt-contract
// @contract exact-hmac-oracle-attempt-binding-and-negative-cases
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test http_dispatch_signing -- --nocapture
// AW-EC-END

// Contract: The target-side oracle recomputes HMAC-SHA256 over the exact length-delimited field sequence [idempotency_key, attempt_id, target_url, timestamp_ms, exact JSON body bytes] and the received x-defer-signature must match exactly, not merely start with v1=.
// Contract: A retry preserves the same idempotency key but changes attempt_id, so the recomputed signature changes with the fresh attempt identity while still validating under the shared secret.
// Contract: Tampering with any signed field or the exact received body bytes causes the oracle to reject the signature instead of accepting a stale or partially matching header.
// Contract: Using the wrong key id or wrong secret is a negative case that must fail verification; Defer may not claim signed-target integrity until this executable oracle passes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http_dispatch_signing_oracle_and_negatives() {
    let command = "cargo test -p defer --test http_dispatch_signing -- --nocapture";
    let id = "defer-http-dispatch-signing-oracle-and-negatives";
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
