// SPEC-MANAGED: projects/mamba/external-contracts/behavior/1942.md#mamba-t1-type-wall-conformance-determinism
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-type-wall-conformance-determinism
// @capability mamba-core-semantics
// @claim deterministic-type-wall-outcomes
// @contract MAMBA-T1-TYPE-WALL-CONFORMANCE-DETERMINISM
// @category stability
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- type_wall_conformance_determinism --exact
// AW-EC-END

// Contract: The external contract verifier verifies fixture inventory against manifest projects/mamba/tests/governance/gates/t1_type_wall_denominator/manifest.toml, asserting exact expected row_count = 7407 and exact expected denominator_sha256 = eb45a673ca92c766c1df6596592aa226fae56ab81f8f27fea47e1168743eae28.
// Contract: The evaluation consists of three fresh isolated subprocess executions of exactly cargo test -p mamba --release --test conformance -- --nocapture executed from the same repository revision, Rust toolchain, environment, and manifest digest, where each subprocess has its own run-local output/temp capture and shares no prior-run in-memory state.
// Contract: Evaluation requires a named baseline evidence artifact at projects/mamba/external-contracts/evidence/mamba-t1-type-wall-conformance-determinism-baseline.json whose required fields include source revision, manifest path (projects/mamba/tests/governance/gates/t1_type_wall_denominator/manifest.toml), manifest digest (denominator_sha256), manifest row count (row_count), full normalized allowed failing-path set, failure count, command (cargo test -p mamba --release --test conformance -- --nocapture), and capture timestamp. Missing baseline artifact or missing required fields constitutes a hard failure.
// Contract: Every one of the three post-fix runs must produce exactly 7407 unique terminal fixture results, with zero skipped, filtered, duplicate, missing, or unparseable rows and a complete terminal summary.
// Contract: The three post-fix normalized failing sets and total failure counts across all three repetitions must be exactly equal.
// Contract: Every post-fix failing path must be a member of the pinned baseline allowed set recorded in projects/mamba/external-contracts/evidence/mamba-t1-type-wall-conformance-determinism-baseline.json, and the post-fix total failure count must not exceed the baseline failure count, closing equal-count replacement.
// Contract: The verifier includes a fail-closed self-test/canary that removes one fixture result and flips one outcome in captured test data, asserting that both simulated mutations are rejected before real evidence can pass.
// Contract: Timeouts, crash or signal termination, output truncation, revision/manifest/environment drift, missing baseline, or parse errors are hard failures.
// Contract: Internal implementation causes, runtime micro-optimizations, and CPython parity comparisons do not constitute proof of external contract compliance and remain explicitly non-oracles.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_type_wall_conformance_determinism() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- type_wall_conformance_determinism --exact";
    let id = "mamba-t1-type-wall-conformance-determinism";
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
// SPEC-MANAGED: projects/mamba/tech-design/validate/conformance-suite-total-failed-count-is-non-deterministic-across.md#unit-test
// CODEGEN-BEGIN

// CODEGEN-END
