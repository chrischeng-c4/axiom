// SPEC-MANAGED: apps/defer/external-contracts/behavior/2213.md#defer-cli-interface-conformance
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-cli-interface-conformance
// @capability cli-interface
// @claim defer-cli-convention-and-task-verbs
// @contract standard-domain-cli-offline-spec-and-render-contract
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test cli_contract -- --nocapture
// AW-EC-END

// Contract: `defer --help` exits successfully and exposes the standard `llm`, `upgrade`, and `issue` commands together with `serve`, `spec`, `queue`, `task`, `dispatch`, `backup`, `k8s`, and `dockerfile`; independent `task --help` and `queue --help` observations must include create/status/cancel and get/put/control respectively.
// Contract: `defer llm --topic outline` runs while every HTTP(S)/ALL proxy variable points at a local connection trap, makes zero network connections, advertises workflow/API/delivery/HA/auth topics, and terminates with `next: done`.
// Contract: `defer spec --format openapi` emits the queue-task route and a terminal marker, while TypeScript generation must produce exactly types.ts, runtime.ts, client.ts, hooks.ts, and index.ts; client.ts must contain createDeferClient plus taskCreate/taskStatus/taskCancel route functions, so an arbitrary non-empty placeholder fails.
// Contract: Source/release Dockerfile and CRD/operator/prod-instance render commands each have their exit status asserted before content inspection and emit the expected lifecycle-layer kinds; `defer serve --help` exposes the shared pretty/json log-format policy.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_cli_interface_conformance() {
    let command = "cargo test -p defer --test cli_contract -- --nocapture";
    let id = "defer-cli-interface-conformance";
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
