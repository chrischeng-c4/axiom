// SPEC-MANAGED: projects/lumen/external-contracts/competitor-feature-parity/behavior/serve-functional.md#lumen-competitor-feature-parity-api-and-search
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec lumen-competitor-feature-parity-api-and-search
// @capability competitor-feature-parity
// @claim query-planner-boolean-eval-roaring-postings
// @contract serve-functional-api-and-search-correctness
// @category behavior
// @required_for_production true
// @command cargo test -p lumen --test api_e2e --test vector_e2e --test planner_diff -- --nocapture
// AW-EC-END

// Contract: The HTTP API end-to-end (create -> index -> search -> hydrate ids) returns correct ranked external_ids and never documents.
// Contract: Vector kNN and filtered kNN return the nearest within the filter without recall collapse.
// Contract: The query planner produces byte-identical plans (planner_diff) across the search-flavor sub-capabilities.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn lumen_competitor_feature_parity_api_and_search() {
    let command =
        "cargo test -p lumen --test api_e2e --test vector_e2e --test planner_diff -- --nocapture";
    let id = "lumen-competitor-feature-parity-api-and-search";
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
