// SPEC-MANAGED: apps/mamba/external-contracts/stability/build-globals-dict-key-leak-free.md#build-globals-dict-key-leak-free
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec build-globals-dict-key-leak-free
// @capability mamba-core-semantics
// @claim build-globals-dict-leaks-no-key-references
// @contract MAMBA-T1-BUILD-GLOBALS-DICT-KEY-LEAK-FREE
// @category stability
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- build_globals_dict_key_leak_free --exact
// AW-EC-END

// Contract: Comparing two mamba runs that call globals() 100 times versus 50,000 times (42 exposed names per call: 20 id_ns values, 20 func_info functions, plus the __name__ and total module globals), peak process RSS for the 50,000-call run is no greater than the 100-call run's peak RSS plus 24 MiB; unreleased per-call key allocations would instead grow roughly linearly with call count.
// Contract: Equal id_ns and func_info name counts (20 each) ensure a leak isolated to either loop alone still produces per-call growth far in excess of the 24 MiB slack, so the gate cannot pass by exercising only one of the two loops build_globals_dict populates.
// Contract: Each run's script accumulates and prints the sum of len(globals()) across every call; the gate asserts this sum exactly equals iterations times the expected 42-key count, so a degenerate or empty build_globals_dict cannot pass vacuously by having nothing left to leak.
// Contract: Peak RSS is sampled via the OS process resource-usage counter (wait4/getrusage-equivalent) on the real compiled mamba binary, not a runtime-internal self-reported counter.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn build_globals_dict_key_leak_free() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- build_globals_dict_key_leak_free --exact";
    let id = "build-globals-dict-key-leak-free";
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
