// SPEC-MANAGED: apps/beam/external-contracts/competitor-performance/efficiency/beam-high-throughput.md#beam-competitor-performance-gpu-batch-scaling
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec beam-competitor-performance-gpu-batch-scaling
// @capability competitor-performance
// @claim competitive-throughput-gpu-scaling
// @contract search-efficiency-gpu-scaling
// @category efficiency
// @required_for_production true
// @command cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario gpu-batching
// AW-EC-END

// Contract: System throughput (QPS) scales linearly as `QueryBatch` size increases from 1 to 512.
// Contract: Peak batched throughput demonstrates at least 10x QPS advantage over the `HnswCpuIndex` baseline from Lumen.
// Contract: Exact refinement parity: The final Top-K results emitted by the GPU exactly match the CPU oracle for the identical candidate set.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn beam_competitor_performance_gpu_batch_scaling() {
    let command =
        "cd apps/beam && ../../target/debug/vat run ec-efficiency-meter --scenario gpu-batching";
    let id = "beam-competitor-performance-gpu-batch-scaling";
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
