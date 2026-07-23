// SPEC-MANAGED: apps/lumen/external-contracts/long-running-stability/stability/query-resilience.md#lumen-long-running-stability-resource-leak
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-long-running-stability-resource-leak
// @capability long-running-stability
// @claim no-fd-socket-thread-leak
// @contract search-stability-resource-leak
// @category stability
// @required_for_production true
// @command cd apps/lumen && ../../target/debug/vat run rig-endurance
// AW-EC-END

// Contract: fd_leak resolves the unique Lumen listener PID, runs sustained bounded-keyspace index and search work, records independent fd/socket/thread counts before and after, requires zero request failures, and gates every after count at <= 1.20 * before + 16.
// Contract: soak_rss_plateau warms a bounded keyspace, runs two mixed workload windows with zero request failures, measures rss_w1/rss_w2 from the live Lumen PID, and requires rss_w2 <= 1.10 * rss_w1.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_long_running_stability_resource_leak() {
    let command = "cd apps/lumen && ../../target/debug/vat run rig-endurance";
    let id = "lumen-long-running-stability-resource-leak";
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
