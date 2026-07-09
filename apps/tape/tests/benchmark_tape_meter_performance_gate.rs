// SPEC-MANAGED: apps/tape/external-contracts/competitor-performance/efficiency/meter-gate.md#tape-meter-performance-gate
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec tape-meter-performance-gate
// @capability competitor-performance
// @claim topic-replay-competitor-performance-baseline
// @contract tape-meter-throughput-ratchet
// @category efficiency
// @required_for_production true
// @command cd apps/tape && ../../target/debug/vat run meter-perf
// AW-EC-END

// Contract: The local Tape performance regression gate passes for append, replay, and checkpoint operations.
// Contract: Tape's NATS JetStream local backlog replay win is backed by a real-service benchmark gate.
// Contract: The gate is executed by meter inside a vat workspace, not by a direct-cargo-only dispatch path.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn tape_meter_performance_gate() {
    let command = "cd apps/tape && ../../target/debug/vat run meter-perf";
    let id = "tape-meter-performance-gate";
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
