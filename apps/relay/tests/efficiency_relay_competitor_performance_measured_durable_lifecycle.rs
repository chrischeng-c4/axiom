// SPEC-MANAGED: apps/relay/external-contracts/competitor-performance/efficiency/perf-gate.md#relay-competitor-performance-measured-durable-lifecycle
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec relay-competitor-performance-measured-durable-lifecycle
// @capability competitor-performance
// @claim normalized-win-ratchet-decision-model
// @contract relay-measured-durable-lifecycle-envelope
// @category efficiency
// @required_for_production true
// @command bash apps/relay/scripts/ec-evidence.sh performance-efficiency
// AW-EC-END

// Contract: A report-only child process uses temporary disk storage with FsyncPolicy::Always to publish and then lease/ack exactly 2,000 128-byte payloads in 100-message batches.
// Contract: The child emits one machine-readable report containing non-zero elapsed time, at least 20 samples per phase, throughput, batch p95, acknowledgement counts, and error count; missing, malformed, zero-sample, incomplete, or error reports fail closed.
// Contract: An independent parent parser requires publish and lease/ack throughput >= 500 messages/second and batch p95 <= 500,000 microseconds without calling Relay's perf_gate verdict helper.
// Contract: A test-owned outer oracle first proves its own zero-test and missing-marker rejection, requires both ignored test names, then accepts only exactly one executed gate and exactly one relay_perf_gate report marker before Meter records the same release invocation.
// Contract: RabbitMQ, NATS JetStream, Redis Streams, and Dragonfly results remain advisory calibration; this local envelope does not assert an external-broker win.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn relay_competitor_performance_measured_durable_lifecycle() {
    let command = "bash apps/relay/scripts/ec-evidence.sh performance-efficiency";
    let id = "relay-competitor-performance-measured-durable-lifecycle";
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
