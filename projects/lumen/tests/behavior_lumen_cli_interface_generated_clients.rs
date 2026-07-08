// SPEC-MANAGED: projects/lumen/external-contracts/cli-interface/behavior/cli-interface.md#lumen-cli-interface-generated-clients
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-cli-interface-generated-clients
// @capability cli-interface
// @claim lumen-spec-schema-openapi-json-yaml-json-schema-offline
// @contract spec-gen-generated-clients-public-api-journey
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test generated_clients_crud_e2e -- --nocapture
// AW-EC-END

// Contract: lumen spec gen emits Python, TypeScript, and Rust clients from the offline OpenAPI document.
// Contract: generated Python, TypeScript, and Rust clients compile or import as real downstream consumers.
// Contract: each generated client drives create collection, index, search, stats, delete indexed id, and forced drop against a real Lumen service.
// Contract: the generated Python client uses the bundled h2c runtime; the TypeScript and Rust clients exercise the same public API over their native HTTP runtimes.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_cli_interface_generated_clients() {
    let command = "cargo test -p lumen --test generated_clients_crud_e2e -- --nocapture";
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
