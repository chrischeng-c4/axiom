// SPEC-MANAGED: apps/mamba/external-contracts/efficiency/mamba-t1-to-thread-gather-efficiency.md#mamba-t1-to-thread-gather-efficiency
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-to-thread-gather-efficiency
// @capability mamba-core-semantics
// @claim parallel-to-thread-gather-preserves-every-result
// @contract MAMBA-T1-FT-GATHER-EFFICIENCY
// @category efficiency
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_efficiency --exact
// AW-EC-END

// Contract: On a host exposing at least four logical CPUs, four equal CPU-bound asyncio.to_thread jobs complete with wall-clock speedup at least 1.50x versus the same jobs run serially, while returning the exact same ordered results.
// Contract: During the parallel phase, measured process CPU time divided by wall time is at least 1.50, proving concurrent work used more than one core rather than cooperative single-loop scheduling.
// Contract: Parallel peak RSS is no greater than 1.25 times serial peak RSS plus 16 MiB for the same workload; CPU scaling cannot be purchased with unbounded retained worker/future state.
// Contract: The gate records logical CPU count, serial and parallel wall time, process CPU time, peak RSS, result digest, and speedup; unsupported hosts are explicit evidence, never a silent pass.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_to_thread_gather_efficiency() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_efficiency --exact";
    let id = "mamba-t1-to-thread-gather-efficiency";
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
