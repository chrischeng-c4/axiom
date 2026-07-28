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
// Contract: Before repeated evaluation, the verifier proves the denominator-scoped executable surface by invoking the current release cpython_ported_integration test binary twice with --list: first without a libtest filter, which must yield exactly 13767 unique terminal test names for the whole target, and then with the exact libtest filter cpython_ported::gen::_type, which must yield exactly 7407 unique terminal test names that are set-equal to projects/mamba/tests/governance/gates/t1_type_wall_denominator/denominator.txt.
// Contract: The evaluation consists of three fresh isolated subprocess executions of exactly cargo test -p mamba --release --test cpython_ported_integration cpython_ported::gen::_type -- --nocapture. Before run 1, the verifier must observe and record the current run revision and current-run environment fingerprint, then assert that the same current run revision, exact rustup toolchain surface (rustc 1.96.1 (31fca3adb 2026-06-26) and cargo 1.96.1 (356927216 2026-06-26)), current-run environment fingerprint, and manifest digest remain unchanged through run 3. Each subprocess has its own run-local output/temp capture and shares no prior-run in-memory state.
// Contract: Evaluation requires a named baseline evidence artifact at projects/mamba/external-contracts/evidence/mamba-t1-type-wall-conformance-determinism-baseline.json whose required fields include source_revision as immutable provenance of the historical baseline capture, manifest path (projects/mamba/tests/governance/gates/t1_type_wall_denominator/manifest.toml), manifest digest (denominator_sha256), manifest row count (row_count), exact rustup_toolchain string, exact cargo_version string, baseline environment_fingerprint, full normalized allowed failing-path set, failure count, command (cargo test -p mamba --release --test cpython_ported_integration cpython_ported::gen::_type -- --nocapture), and capture timestamp. Missing baseline artifact or missing required fields constitutes a hard failure.
// Contract: The verifier must treat baseline source_revision and baseline environment_fingerprint as immutable capture provenance only. It must separately record and verify the three-run current run revision and current-run environment fingerprint, and it must not require current HEAD to equal the historical baseline source_revision. Lifecycle commits after baseline capture are therefore allowed only if the three post-fix executions themselves stay on one unchanged current run revision and one unchanged current-run environment.
// Contract: Every one of the three post-fix runs must produce an executed terminal-name set that is exactly set-equal to the denominator's 7407 cpython_ported::gen::_type names, with no ignored or skipped executed rows, duplicates, missing rows, cross-namespace rows, or unparseable rows.
// Contract: For every one of the three post-fix runs, passed plus failed executed rows must equal exactly 7407.
// Contract: For every one of the three post-fix runs, the verifier must reconcile whole-target libtest filtering by proving exactly 6360 filtered-out rows, equal to the pinned whole-target preflight count 13767 minus the executed denominator count 7407.
// Contract: The three post-fix normalized failing sets and total failure counts across all three repetitions must be exactly equal.
// Contract: Every post-fix failing path must be a member of the pinned baseline allowed set recorded in projects/mamba/external-contracts/evidence/mamba-t1-type-wall-conformance-determinism-baseline.json, and the post-fix total failure count must be less than or equal to the baseline failure count, closing both subset and equal-count replacement.
// Contract: The verifier includes fail-closed self-tests/canaries that remove one fixture result and flip one outcome in captured test data, asserting that both simulated mutations are rejected before real evidence can pass.
// Contract: Timeouts, crash or signal termination, nonzero exit without complete terminal accounting, output truncation, missing baseline, parse errors, or manifest/toolchain/current-run revision/current-run environment drift across the three post-fix executions are hard failures.
// Contract: Internal implementation causes, runtime micro-optimizations, CPython parity comparisons, and any inferred bijection to unrelated run_conformance::<path> cases do not constitute proof of external contract compliance and remain explicitly non-oracles.
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
