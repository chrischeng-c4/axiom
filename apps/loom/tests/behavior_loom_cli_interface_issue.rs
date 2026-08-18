// SPEC-MANAGED: apps/loom/external-contracts/behavior/541.md#loom-cli-interface-issue
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec loom-cli-interface-issue
// @capability cli-interface
// @claim loom-cli-convention-and-control-verbs
// @contract loom-cli-issue-verbs
// @category behavior
// @required_for_production true
// @command cargo test -p loom --test cli_contract
// AW-EC-END

// Contract: loom --help contains standard subcommands (controller, worker, spec, llm, upgrade, issue).
// Contract: loom controller --help outputs: strongly-consistent DAG state
// Contract: loom worker --help outputs: Resident pull-loop worker harness
// Contract: loom llm outline outputs: architecture, roles, control-api
// Contract: loom llm architecture outputs: never traverse loom
// Contract: loom upgrade --check outputs: current:
// Contract: loom spec --format openapi outputs a valid OpenAPI JSON document
// Contract: loom spec gen --lang py outputs models.py and client.py
// Contract: loom worker fails without environment variables (requires LOOM_KEEP)
// Contract: loom issue search and view fail offline with no online feature error
// Contract: loom issue create runs offline in dry run mode emitting a pre-filled issue URL
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn loom_cli_interface_issue() {
    let command = "cargo test -p loom --test cli_contract";
    let id = "loom-cli-interface-issue";
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
