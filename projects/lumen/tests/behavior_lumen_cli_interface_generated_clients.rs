// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-generated-clients
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-generated-clients
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract spec-gen-generated-clients-public-api-journey
// @category behavior
// @required_for_production true
// @command PATH=$HOME/.pyenv/shims:$PATH cargo test -p lumen --test spec_gen_e2e generated_client_live_h2c_public_api_journey -- --exact --ignored --nocapture
// AW-EC-END

// Contract: lumen spec gen emits Python, TypeScript, and Rust clients from the offline OpenAPI document.
// Contract: generated clients drive health, readiness, version, collection creation, indexing, search, duplicates, stats, and forced drop against a live h2c Lumen service.
// Contract: the generated Python client validates recursive pydantic QueryNode union shapes while using the bundled h2c runtime.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_generated_clients() {
    let command =
        "PATH=$HOME/.pyenv/shims:$PATH cargo test -p lumen --test spec_gen_e2e generated_client_live_h2c_public_api_journey -- --exact --ignored --nocapture";
    let id = "lumen-cli-interface-generated-clients";
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
