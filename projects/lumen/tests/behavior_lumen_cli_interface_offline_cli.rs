// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-offline-cli
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-offline-cli
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract offline-cli-agent-onboarding
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test spec_cli -- --nocapture
// AW-EC-END

// Contract: lumen spec emits valid OpenAPI JSON, OpenAPI YAML, and JSON-schema output offline.
// Contract: lumen spec exposes query-shape, field, analyzer, and vector-metric catalogs.
// Contract: lumen llm outline, workflow, integration, quickstart, and recipes preserve the ingest-search-hydrate agent workflow and non-goals.
// Contract: lumen llm integration recommends the Postgres/AlloyDB boundary: database commit/outbox or CDC, external adapter-owned Pub/Sub retry/DLQ, HTTP writes into lumen, and no direct external publishing to lumen's internal broker WAL.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_offline_cli() {
    let command = "cargo test -p lumen --test spec_cli -- --nocapture";
    let id = "lumen-cli-interface-offline-cli";
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
