// SPEC-MANAGED: apps/mamba/external-contracts/behavior/2005.md#mamba-t1-pep695-generic-slice-semantics
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-pep695-generic-slice-semantics
// @capability mamba-core-semantics
// @claim pep-695-generic-metadata-and-base-rejection-slice
// @contract MAMBA-T1-PEP695-GENERIC-SLICE-SEMANTICS
// @category behavior
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- pep695_generic_slice_semantics --exact
// AW-EC-END

// Contract: The external contract verifier verifies fixture inventory against manifest apps/mamba/tests/governance/gates/t1_generic_binding_denominator/manifest.toml, asserting exact expected row_count = 11 and exact expected denominator_sha256 = 7f31655cff76e349217304c482fede11d07442b28aa9f9dc82fa1292315d9f1c. An empty denominator, manifest drift, or missing denominator artifact is a hard failure.
// Contract: Before execution, the verifier proves the runnable selector surface against the current cpython_ported_integration target by invoking --list once without a libtest filter, which must yield exactly 13767 unique terminal test names for the whole target, and once with the exact libtest filter pep::_695, which must yield exactly the 11-name set in apps/mamba/tests/governance/gates/t1_generic_binding_denominator/denominator.txt. A selector that resolves to zero, fewer, more, duplicate, cross-namespace, missing, or unparseable rows is a hard failure.
// Contract: The executable proof is one fresh subprocess execution of exactly python3 apps/mamba/tools/run_denominator.py t1_generic_binding from one unchanged repository revision, the repo-required rustup toolchain surface, and the pinned manifest digest. The verifier must fail closed if the denominator runner reports hash drift, an empty list, a renamed or omitted test that libtest would otherwise silently filter out, or any incomplete terminal accounting.
// Contract: Success requires complete terminal accounting for all 11 denominator rows, with exactly 11 passed, 0 failed, and 0 ignored rows. A run that executes fewer than 11 named rows, even with exit code 0, is a failure.
// Contract: The 8 behavior rows in the denominator are the visible positive oracle for this bounded EC slice only: they must keep generic explicit-base retention, generic class value storage, polymorphic generic function behavior, generic method zero-arg super behavior, star-unpacked generic base lists, empty __type_params__ on non-generic declarations, writable __type_params__ metadata, and ordinary function argument defaults on generic functions externally correct as asserted by the ported CPython fixtures.
// Contract: The row test_gen_behavior_pep_695_typeparam_default_args is not a PEP 696 type-parameter-default witness. It proves only that a generic function annotated by T preserves ordinary Python function argument defaults at runtime; it does not prove __default__, type-argument substitution defaults, or invalid type-parameter default ordering/bounds.
// Contract: The 3 error rows in the denominator are the visible negative oracle for this bounded EC slice only: invalid double-generic bases, invalid object-base-plus-generic MRO combinations, and undeclared type parameters in generic bases must still be rejected exactly as asserted by the ported CPython error fixtures. A widening that turns any of those rows green by accepting the invalid form is a contract failure.
// Contract: This EC is intentionally narrower than WI #2005's full roadmap claim. It does not approve or imply coverage for type-parameter scope/order beyond the named rows, bounds, constraints, variance, PEP 696 type-parameter defaults, TypeVarTuple or variadic rejection, traditional TypeVar forms, nested/class/method/type-alias scope completeness, stable runtime metadata outside the named __type_params__ witnesses, or specialization subtype/wall behavior. Those surfaces remain unverified by this 11-row denominator and require separate manifest-backed executable witnesses before they can be claimed complete.
// Contract: This EC is bounded to the accepted 11-row regression-prevention denominator for work root #1505. It does not treat implementation-source claims, internal runtime registries, or Rust-only helper assertions as its oracle; the oracle is the externally observable Python behavior and rejection semantics encoded in the named denominator fixtures plus the manifest-backed denominator inventory.
// Contract: Timeouts, crash or signal termination, output truncation, missing terminal summaries, manifest drift, selector drift, parse errors, or environment/toolchain drift during the measured run are hard failures.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_pep695_generic_slice_semantics() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- pep695_generic_slice_semantics --exact";
    let id = "mamba-t1-pep695-generic-slice-semantics";
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
