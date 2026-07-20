// SPEC-MANAGED: apps/jet/external-contracts/behavior/counter-dom-wasm-parity-after-click.md#counter-dom-wasm-parity-after-click
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec counter-dom-wasm-parity-after-click
// @capability browser-trace-parity
// @claim parity-corpus-gates
// @contract counter-dom-wasm-parity-after-click
// @category behavior
// @required_for_production true
// @command cargo test -p jet --test react_dom_oracle_conformance counter_demo_matches_react_dom_oracle_initial_and_after_click -- --nocapture
// AW-EC-END

// Contract: initial DOM tree equals normalized Jet WASM element tree
// Contract: post-click DOM tree equals normalized Jet WASM element tree
// Contract: WASM observation includes hook value 1 after click
// Contract: mismatch output includes concrete DOM and WASM JSON
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn counter_dom_wasm_parity_after_click() {
    let command =
        "cargo test -p jet --test react_dom_oracle_conformance counter_demo_matches_react_dom_oracle_initial_and_after_click -- --nocapture";
    let id = "counter-dom-wasm-parity-after-click";
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
