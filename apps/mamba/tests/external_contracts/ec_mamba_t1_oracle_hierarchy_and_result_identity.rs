// SPEC-MANAGED: apps/mamba/external-contracts/behavior/2010.md#mamba-t1-oracle-hierarchy-and-result-identity
// CODEGEN-BEGIN
// AW-EC-BEGIN
// @ec mamba-t1-oracle-hierarchy-and-result-identity
// @capability mamba-core-semantics
// @claim oracle-hierarchy-and-result-identity
// @contract MAMBA-T1-ORACLE-HIERARCHY-AND-RESULT-IDENTITY
// @category behavior
// @required_for_production true
// @command cargo test -p mamba --release --test mamba_core_semantics_ec -- oracle_hierarchy_and_result_identity --exact
// AW-EC-END

// Contract: This EC is keyed to canonical capability_id = mamba-core-semantics and claim_id = oracle-hierarchy-and-result-identity for WI #2010. CAPABILITIES work-root registration is a separate human-reviewed step; the verifier must not treat the presence of this EC text as implicit capability registration or self-authorization.
// Contract: The verifier must fail closed unless all three pinned artifacts exist and agree: apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/manifest.toml, apps/mamba/tests/governance/gates/t1_oracle_hierarchy_inventory/cases.jsonl, and apps/mamba/external-contracts/evidence/mamba-t1-oracle-hierarchy-lock.json. The manifest is the inventory root, cases.jsonl is the executable row set, and the evidence lock is the independently auditable identity record. Missing files, empty files, or disagreement between them are hard failures; the verifier must never synthesize a zero-row or best-effort inventory.
// Contract: The manifest schema must pin, at minimum, schema_version, inventory_path, inventory_sha256, row_count, required_dimensions = [behavior, stability, efficiency], required_channels = [compile, behavior, concurrency, performance], source_sets, migration_inputs, cpython312_executable, cpython312_version, cpython313t_executable, cpython313t_version, platform, mamba_git_sha, mamba_binary_sha256, and generated_at. The evidence lock must repeat those identities plus the verifier command, source revision, and capture timestamp. If the inventory has not yet been generated, success is impossible; the verifier fails rather than fabricating row_count or digest values.
// Contract: cases.jsonl is not an authority for denominator selection. Before reading it, the verifier must independently discover and digest the authoritative tracked source sets: source_set ordinary_parity_corpus from apps/mamba/tests/harness/cpython/config/manifests/**/*.toml together with apps/mamba/tests/harness/cpython/config/behavior_gaps.txt and apps/mamba/tests/harness/cpython/config/type_divergences.txt; source_set tier1_ec_cases from apps/mamba/external-contracts/{behavior,stability,efficiency}/*.md whose embedded EC YAML has capability_id = mamba-core-semantics; and source_set tier1_gate_denominators from apps/mamba/tests/governance/gates/t1_*_denominator/{manifest.toml,denominator.txt}. For each source set the verifier must compute per-source path inventory, identity count, and sha256 digest before reconciling to cases.jsonl.
// Contract: Exact set equality is required after reconciliation. Every cases.jsonl row must map to one independently discovered source identity, and every independently discovered Tier 1 EC case or ordinary-parity source identity must map to exactly one cases.jsonl row unless the evidence lock records an explicit out_of_scope disposition with reviewer-auditable reason. Deletion, omission, duplicate mapping, stale mapping, source identities that vanish from cases.jsonl, or inventory rows with no authoritative source are hard failures.
// Contract: Each non-comment JSONL row in cases.jsonl must have a unique case_id and must declare: tier1_dimension, channel, sample_role, fixture_or_probe_path, probe_id, oracle_kind, oracle_command, oracle_executable, oracle_version, sut_command, expected_divergence_class, expected_result_channel, expected_terminal_classification, expected_probe_anchor, source_identity, source_set, platform, mamba_git_sha, and mamba_binary_sha256. Duplicate case IDs, duplicate probe identities, blank fields, unknown enum values, rows outside apps/mamba/, or rows whose expected_result_channel disagrees with channel are hard failures.
// Contract: Routing is exact and fail-closed: ordinary parity rows must use oracle_kind = cpython312_identity and must compare Mamba against the exact pinned CPython 3.12 executable/version; free-thread observable parity rows must use oracle_kind = cpython313t_identity and must compare Mamba against the exact pinned CPython 3.13t executable/version; intentionally different scheduling rows must use oracle_kind = property and must name a verifier-owned property oracle command; Force Typed rows must use oracle_kind = force_typed_expected and must declare explicit compile or runtime expected outcomes. Unknown oracle kind, unknown CPython version, unknown divergence class, or a row that routes through Mamba itself as the oracle is a hard failure.
// Contract: The verifier must maintain four separate complete-accounting channels named compile, behavior, concurrency, and performance. Every selected row must terminate in exactly one of those channels, and the final report must give exact selected, executed, completed, green, intentional_red, undeclared_red, undeclared_diverge, oracle_skip, timeout, signal, and unclassified counts for each channel. A row counted twice, counted in no channel, dropped after an early failure, or omitted from terminal accounting is a hard failure.
// Contract: Result identity is part of the contract. For every executed row, the verifier must record the exact oracle_command and sut_command it replayed, the absolute oracle executable path, oracle version string, platform triple, Mamba git SHA, Mamba binary sha256, fixture or probe path, probe anchor, exit status, signal or timeout state, stdout sha256, stderr sha256, and terminal classification. PATH-dependent aliases, patch-version drift, binary drift, or missing identity fields are hard failures.
// Contract: The required Tier 1 dimensions for this WI are behavior, stability, and efficiency. The inventory must contain at least one green sample and at least one intentional_red sample in each of those three dimensions, and every one of the four channels must have a nonzero selected count. Zero-row dimensions, zero-row channels, only-green dimensions, only-intentional-red dimensions, duplicate samples standing in for distinct probes, or stale samples carried over without current execution evidence are hard failures.
// Contract: The verifier must include the named behavior witness source_identity = mamba-t1-to-thread-gather-results from apps/mamba/external-contracts/behavior/mamba-t1-to-thread-gather-results.md and must preserve its exact CPython 3.12 parity rule: the gathered result list contains every expected value exactly once in asyncio.gather input order, with no None, missing, duplicate, stale, or cross-worker value. A Rust-only helper or registry cannot satisfy this witness.
// Contract: The verifier must include the named concurrency witness source_identity = mamba-t1-to-thread-gather-stability from apps/mamba/external-contracts/stability/mamba-t1-to-thread-gather-stability.md. This witness must execute 100 rounds of eight concurrently gathered CPU-bound asyncio.to_thread calls with zero crash, panic, timeout, or deadlock; after a 250 ms quiescence period the OS-visible worker/thread count must return to the pre-soak baseline plus at most one runtime service thread; and peak RSS in soak window two must be no greater than 1.10 times window one plus 8 MiB. Failure to satisfy these exact thresholds is evidence of concurrency instability or retained-state leakage, not a soft warning.
// Contract: The verifier must include the named performance probe source_identity = mamba-t1-to-thread-gather-efficiency from apps/mamba/external-contracts/efficiency/mamba-t1-to-thread-gather-efficiency.md. On hosts exposing at least four logical CPUs, this probe must record logical CPU count, serial wall time, parallel wall time, process CPU time, peak RSS, result digest, and speedup, and must enforce the live Tier 1 thresholds: parallel wall-clock speedup at least 1.50x versus serial, process CPU time divided by parallel wall time at least 1.50, and parallel peak RSS no greater than 1.25 times serial peak RSS plus 16 MiB. Unsupported hosts are explicit non-passing evidence, never silent success.
// Contract: Because apps/mamba/tests/governance/gates/t1_multicore_scaling_denominator/denominator.txt is a tracked Tier 1 source surface but currently lacks a populated manifest-backed runnable denominator, the verifier must record that source identity and fail closed until either a manifest-backed mapping exists or the evidence lock carries an explicit out_of_scope disposition reviewed against WI #2022. It must not invent rows or treat an empty source file as proof.
// Contract: Force Typed rows must declare expected_outcome_kind = compile_reject, runtime_typeerror, or runtime_accept and must prove the exact named probe rather than any earlier failure in the same file. For compile_reject rows the verifier must match the pinned call site or span plus the declared diagnostic class; for runtime_typeerror rows it must prove the named runtime operation raises at the pinned probe; for runtime_accept rows it must prove successful execution with the declared output. A red result at another line or another call in the file is not acceptable evidence.
// Contract: The existing keep_status.py meter may be used only as a prefilter or cross-check. Its process exit code is never the terminal oracle. The final verifier must parse per-row results and exit nonzero on every RED, DIVERGE, ORACLE_SKIP, timeout, signal, or unclassified row unless that row is declared as intentional_red in cases.jsonl and the captured evidence proves the named probe, not an earlier failure. A green meter process with undeclared bad rows is a verifier failure, not a pass.
// Contract: The legacy input apps/mamba/tests/harness/cpython/config/type_divergences.txt is a migration source, not a trusted terminal registry. The verifier must reconcile every legacy exclusion into the new inventory and fail if any exclusion is undocumented, stale, duplicated, or missing a current evidence row. apps/mamba/tests/cpython/errors/std-libs/os/scandir_float_raises_typeerror.py must not survive as an intentional divergence because it is currently green on Mamba; if the producer-owned registry still contains it, the verifier must fail and direct registry repair rather than silently excluding it.
// Contract: apps/mamba/tests/cpython/behavior/builtin-libs/builtins/builtin_test__test_ord.py may be admitted as an intentional divergence only if the inventory pins the ord(42) probe itself and the verifier proves that exact probe is what triggered the declared rejection. Evidence that only shows an earlier ord(bytes) rejection, or any other earlier failure in the file, is insufficient and must be classified as stale or misattributed evidence.
// Contract: Migration and reconciliation are full-surface obligations. The manifest's migration_inputs must include apps/mamba/tests/harness/cpython/config/type_divergences.txt, apps/mamba/tests/harness/cpython/config/behavior_gaps.txt, the authoritative harness manifests under apps/mamba/tests/harness/cpython/config/manifests/, the Tier 1 gate denominator surfaces under apps/mamba/tests/governance/gates/t1_*_denominator/, and the existing Tier 1 EC inventory under apps/mamba/external-contracts/{behavior,stability,efficiency}/ for capability_id = mamba-core-semantics. Every migrated source case must map to at least one inventory row or be explicitly marked out_of_scope with a reviewer-auditable reason in the lock; silent omission is a hard failure.
// Contract: Ordinary parity completeness must be manifest-backed. Any behavior, surface, errors, real_world, compile, concurrency, or performance row that is not declared as intentional_red in the inventory must satisfy its exact routed oracle, and no undocumented exclusion, stale legacy carry-over, early-abort omission, or verifier-owned sample shrink may remove it from the denominator. Zero-row, duplicate-row, stale-row, or unexecuted-row acceptance is forbidden.
// Contract: Success requires exact agreement among the manifest, inventory, evidence lock, authoritative source-set digests, oracle identities, Mamba identities, per-row accounting, per-channel accounting, and required-dimension coverage. Unknown classes, stale exclusions, self-oracle behavior, incomplete migration, missing green or intentional-red samples, or any attempt to let cases.jsonl define its own denominator are hard failures.
#[test]
#[ignore = "AW EC gate: run via `aw health --verify-ec` or `cargo test -- --ignored`"]
fn mamba_t1_oracle_hierarchy_and_result_identity() {
    let command =
        "cargo test -p mamba --release --test mamba_core_semantics_ec -- oracle_hierarchy_and_result_identity --exact";
    let id = "mamba-t1-oracle-hierarchy-and-result-identity";
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
