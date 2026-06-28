// SPEC-MANAGED: projects/agentic-workflow/tech-design/surface/specs/aw-repo-view-desktop-app.md#aw-view-repo-snapshot
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec aw-view-repo-snapshot
// @capability repo-view-desktop-app
// @claim repo-desktop-reader
// @contract aw-view-repo-snapshot
// @category behavior
// @required_for_production true
// @command ./target/debug/aw view --snapshot
// AW-EC-END

// Contract: snapshot root is the repository, not a single project
// Contract: snapshot includes the projects/libs catalog
// Contract: snapshot includes terminal status for watching agent-side changes
// Contract: snapshot includes selected README detail and capability rows
// Contract: snapshot includes selected EC inventory and TD summary
// Contract: snapshot includes a renderer-neutral surface tree for renderer-independent tests
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn aw_view_repo_snapshot() {
    let command = "./target/debug/aw view --snapshot";
    let id = "aw-view-repo-snapshot";
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
