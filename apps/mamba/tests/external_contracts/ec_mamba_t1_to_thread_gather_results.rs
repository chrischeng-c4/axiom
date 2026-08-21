// SPEC-MANAGED: apps/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md#mamba-t1-to-thread-gather-results
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-to-thread-gather-results
// @capability mamba-core-semantics
// @claim parallel-to-thread-gather-preserves-every-result
// @contract MAMBA-T1-FT-GATHER-RESULTS
// @category behavior
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_results --exact
// AW-EC-END

// Contract: Two or more concurrently completing asyncio.to_thread calls with distinct inputs are gathered into one result list containing every expected value exactly once and in asyncio.gather input order.
// Contract: No gathered slot may be None, missing, duplicated, stale, or borrowed from another worker, regardless of whether that worker finishes before or after the gather await begins.
// Contract: The contract exercises the public Mamba asyncio surface from a compiled Python program; a Rust-only registry or helper test cannot satisfy this behavior gate by itself.
// Contract: The CPython control program must produce the same ordered values; Mamba's intentional divergence is multicore execution, not gather result semantics.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_to_thread_gather_results() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- to_thread_gather_results --exact";
    let id = "mamba-t1-to-thread-gather-results";
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
