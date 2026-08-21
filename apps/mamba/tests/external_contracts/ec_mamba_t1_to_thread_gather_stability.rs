// SPEC-MANAGED: apps/mamba/external-contracts/stability/mamba-t1-to-thread-gather-stability.md#mamba-t1-to-thread-gather-stability
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-to-thread-gather-stability
// @capability mamba-core-semantics
// @claim parallel-to-thread-gather-preserves-every-result
// @contract MAMBA-T1-FT-GATHER-STABILITY
// @category stability
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_stability --exact
// AW-EC-END

// Contract: Across 100 rounds of eight concurrently gathered CPU-bound asyncio.to_thread calls, every round returns all eight distinct expected values exactly once with zero crash, panic, timeout, or deadlock.
// Contract: The stability gate varies worker completion order and must fail on any None, missing, duplicate, stale, cross-worker, or wrong result; aggregate pass counts are insufficient evidence.
// Contract: Using an OS-visible process-thread count, the worker/thread count after a 250 ms quiescence period following the final round returns to the pre-soak baseline plus at most one runtime service thread; private runtime registries are not an EC oracle.
// Contract: Peak RSS is sampled in two equal soak windows; window two must be no greater than 1.10 times window one plus 8 MiB, so monotonic retained-result leaks fail the required gate.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_to_thread_gather_stability() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_stability --exact";
    let id = "mamba-t1-to-thread-gather-stability";
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
