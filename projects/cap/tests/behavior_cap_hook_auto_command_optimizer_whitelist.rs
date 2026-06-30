// SPEC-MANAGED: projects/cap/tech-design/logic/cap-hook-auto-command-optimizer-whitelist.md#cap-hook-auto-command-optimizer-whitelist
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec cap-hook-auto-command-optimizer-whitelist
// @capability agent-hook-installation
// @claim hook-payload-rewrite-adapters
// @contract hook-payload-rewrite-adapters
// @category behavior
// @required_for_production true
// @command cargo test -p cap --lib hook -- --nocapture && cargo test -p cap command_planner -- --nocapture && cargo test -p cap active_replacements_match_success_and_error_behavior -- --nocapture && cargo bench -p cap --bench command_resources
// AW-EC-END

// Contract: the Bash hook rewrites non-recursive commands to cap run original-command-string and does not expose same-name replacement decisions
// Contract: cap run command-string parsing routes shell-free active replacements to the same fast implementation family as cap <cmd>
// Contract: complex shell commands keep shell semantics by falling back internally to bash -c original
// Contract: simple high-entry-count ls, large single-file sort, bounded sed, recursive literal grep, find, du, cat, and uniq satisfy the dual-win cap replacement gate for both cap <cmd> and cap run command-string surfaces
// Contract: active replacements match original-command success output, missing-path error behavior, and quiet nonzero behavior
// Contract: true, false, pwd, basename, dirname, head, tail, mkdir, touch, awk, xargs, and shell pipes remain scout-only or compatibility-fallback candidates until they beat the dual-win gate or a material RSS-fallback gate
// Contract: gated replacements fail the benchmark when their dual-win or RSS-fallback resource policy is not satisfied
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn cap_hook_auto_command_optimizer_whitelist() {
    let command =
        "cargo test -p cap --lib hook -- --nocapture && cargo test -p cap command_planner -- --nocapture && cargo test -p cap active_replacements_match_success_and_error_behavior -- --nocapture && cargo bench -p cap --bench command_resources";
    let id = "cap-hook-auto-command-optimizer-whitelist";
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
