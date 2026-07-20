// SPEC-MANAGED: apps/defer/external-contracts/behavior/2216.md#defer-competitor-matrix-scope
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-competitor-matrix-scope
// @capability competitor-feature-parity
// @claim delayed-task-competitor-feature-matrix
// @contract managed-http-push-scope-and-explicit-exclusions
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test competitor_feature_matrix -- --nocapture
// AW-EC-END

// Contract: A parsed machine-readable contract names Google Cloud Tasks as the sole managed-HTTP-push semantic competitor, classifies Celery and Sidekiq exactly as category mismatches, and requires one unique four-field row for each of the exact 15 capabilities.
// Contract: The exact DLQ row records that Cloud Tasks deletes retry-exhausted tasks while Defer intentionally retains an explicit replicated DeadLettered terminal; official RetryConfig and Cloud-Tasks-versus-Pub/Sub references remain named in the artifact.
// Contract: Force-run bypass, arbitrary worker execution, cron composition, and workflows retain exact exclusion rationales; complete assertions keep real Cloud Tasks performance unproven, VAT protocol-only, and Relay a non-substitute local overhead ceiling capped at 20%.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_competitor_matrix_scope() {
    let command = "cargo test -p defer --test competitor_feature_matrix -- --nocapture";
    let id = "defer-competitor-matrix-scope";
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
