// SPEC-MANAGED: apps/defer/external-contracts/behavior/2219.md#defer-http2-offline-spec-and-three-client-codegen
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec defer-http2-offline-spec-and-three-client-codegen
// @capability http2-api-list
// @claim h2c-openapi-route-list
// @contract offline-openapi-route-twin-and-typed-client-generation
// @category behavior
// @required_for_production true
// @command cargo test -p defer --test cli_contract offline_spec_and_typed_client_generation_use_one_contract -- --nocapture
// AW-EC-END

// Contract: defer spec --format openapi exits successfully, emits parseable chainable JSON followed by exactly next: done, equals the canonical OpenAPI IR semantically, and contains the exact same nine domain operations as the live contract.
// Contract: defer spec --format routes emits exactly all nine method/path strings including task batch creation and admin backup plus a terminal marker; a stale hand-maintained subset or extra route fails equality.
// Contract: TypeScript, Python, and Rust generation each exits successfully with an exact language-specific file inventory and terminal marker; every generated client contains all nine typed operations and the batch path, while Python includes sync and async h2c clients.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn defer_http2_offline_spec_and_three_client_codegen() {
    let command =
        "cargo test -p defer --test cli_contract offline_spec_and_typed_client_generation_use_one_contract -- --nocapture";
    let id = "defer-http2-offline-spec-and-three-client-codegen";
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
