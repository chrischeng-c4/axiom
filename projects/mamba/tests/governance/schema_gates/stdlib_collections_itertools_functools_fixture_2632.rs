//! Schema gate for the stdlib collections/itertools/functools behavioral
//! fixture — closes #2632.
//!
//! Acceptance (issue #2632):
//!
//!   1. Fixture fails when any helper behavior is wrong.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_{counter, defaultdict, chain, islice,
//!      lru_cache, partial} + distinct exit codes 89/90/91 +
//!      must_distinguish_each_helper_family.
//!   2. Failure output identifies the helper family.
//!      `[failure_naming_contract]` pins
//!      must_emit_helper_family_in_failure_output +
//!      allowed_helper_family_values + must_emit_helper_name +
//!      forbid_generic_unnamed_helper_failure + exit_code=92.
//!   3. Fixture remains fast enough for the ecosystem gate.
//!      `[performance_budget_contract]` pins max_total_runtime_ms +
//!      max_per_helper_runtime_ms + must_not_perform_network_io +
//!      must_not_perform_disk_io + forbid_sleep_calls +
//!      ecosystem_profile_fixture_issue=2814 + exit_code=93.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("collections_itertools_functools_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("stdlib_collections_itertools_functools_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2632));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_collections_itertools_functools_behavioral")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("isolation").and_then(|v| v.as_table()).unwrap();
    for f in &[
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(i.get(*f).and_then(|v| v.as_bool()), Some(true));
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc
        .get("python_target")
        .and_then(|v| v.as_table())
        .expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(
        p.get("must_be_python_3_12").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn surface_covers_collections_itertools_functools() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("surface")
        .and_then(|v| v.as_table())
        .expect("[surface] missing");
    let modules: Vec<&str> = s
        .get("covered_modules")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for m in &["collections", "itertools", "functools"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    let families: Vec<&str> = s
        .get("helper_families")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &["collections", "itertools", "functools"] {
        assert!(families.contains(f), "helper_families must include {f}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_cover_counter",
        "must_cover_defaultdict",
        "must_cover_chain",
        "must_cover_islice",
        "must_cover_lru_cache",
        "must_cover_partial",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn deterministic_sample_covers_every_helper() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("deterministic_sample")
        .and_then(|v| v.as_table())
        .expect("[deterministic_sample] missing");
    assert_eq!(
        d.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    let max = d
        .get("sample_max_records")
        .and_then(|v| v.as_integer())
        .unwrap();
    let min = d
        .get("sample_min_records")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(min >= 1 && max >= min, "sample bounds must be sane");
    assert!(
        max <= 64,
        "sample_max_records must stay small enough for per-run check"
    );

    let helpers = doc
        .get("helper_cases")
        .and_then(|v| v.as_array())
        .expect("[[helper_cases]] missing");
    let mut seen: Vec<&str> = Vec::new();
    for c in helpers {
        let t = c.as_table().expect("helper case must be a table");
        let family = t
            .get("family")
            .and_then(|v| v.as_str())
            .expect("helper.family missing");
        let helper = t
            .get("helper")
            .and_then(|v| v.as_str())
            .expect("helper.helper missing");
        assert!(
            t.get("input_python_repr")
                .and_then(|v| v.as_str())
                .is_some(),
            "helper.input_python_repr missing"
        );
        assert!(
            t.get("expected_python_repr")
                .and_then(|v| v.as_str())
                .is_some(),
            "helper.expected_python_repr missing"
        );
        assert!(
            ["collections", "itertools", "functools"].contains(&family),
            "unexpected family={family}"
        );
        seen.push(helper);
    }
    for h in &[
        "Counter",
        "defaultdict_list",
        "chain",
        "islice",
        "lru_cache",
        "partial",
    ] {
        assert!(seen.contains(h), "helper_cases must include helper={h}");
    }
}

// Acceptance: "Fixture fails when any helper behavior is wrong."
#[test]
fn fixture_fails_on_any_helper_behavior_mismatch() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc
        .get("failure_on_incorrect_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails when any helper behavior is wrong.\"",
        );
    for k in &[
        "must_fail_on_incorrect_counter",
        "must_fail_on_incorrect_defaultdict",
        "must_fail_on_incorrect_chain",
        "must_fail_on_incorrect_islice",
        "must_fail_on_incorrect_lru_cache",
        "must_fail_on_incorrect_partial",
        "must_distinguish_each_helper_family",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exits: Vec<i64> = [
        "collections_helper_mismatch_exit_code",
        "itertools_helper_mismatch_exit_code",
        "functools_helper_mismatch_exit_code",
    ]
    .iter()
    .map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap())
    .collect();
    assert_eq!(exits, vec![89, 90, 91]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        exits.len(),
        "helper family exit codes must differ"
    );
}

// Acceptance: "Failure output identifies the helper family."
#[test]
fn failure_output_identifies_helper_family() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc
        .get("failure_naming_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_naming_contract] missing — acceptance: \
         \"Failure output identifies the helper family.\"",
        );
    assert_eq!(
        n.get("must_emit_helper_family_in_failure_output")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        n.get("helper_family_field_name").and_then(|v| v.as_str()),
        Some("helper_family")
    );
    let allowed: Vec<&str> = n
        .get("allowed_helper_family_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for fam in &["collections", "itertools", "functools"] {
        assert!(
            allowed.contains(fam),
            "allowed_helper_family_values must include {fam}"
        );
    }
    assert_eq!(
        n.get("must_emit_helper_name_in_failure_output")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        n.get("helper_name_field_name").and_then(|v| v.as_str()),
        Some("helper")
    );
    assert_eq!(
        n.get("forbid_generic_unnamed_helper_failure")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let exit = n
        .get("unnamed_helper_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 92);
    assert_eq!(
        n.get("unnamed_helper_failure_kind")
            .and_then(|v| v.as_str()),
        Some("helper_failure_missing_family")
    );
}

// Acceptance: "Fixture remains fast enough for the ecosystem gate."
#[test]
fn fixture_remains_fast_enough_for_ecosystem_gate() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc
        .get("performance_budget_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[performance_budget_contract] missing — acceptance: \
         \"Fixture remains fast enough for the ecosystem gate.\"",
        );
    for k in &[
        "must_run_in_ecosystem_profile",
        "must_not_perform_network_io",
        "must_not_perform_disk_io",
        "forbid_sleep_calls",
    ] {
        assert_eq!(
            p.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        p.get("ecosystem_profile_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2814)
    );
    let total = p
        .get("max_total_runtime_ms")
        .and_then(|v| v.as_integer())
        .unwrap();
    let per = p
        .get("max_per_helper_runtime_ms")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(total > 0 && per > 0 && per <= total, "runtime bounds sane");
    assert!(
        total <= 5000,
        "total runtime must stay light for ecosystem gate, got {total}ms"
    );
    let recur = p
        .get("forbid_long_running_recursion_above_iteration_count")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(
        recur > 0 && recur <= 100_000,
        "recursion bound must be sane, got {recur}"
    );
    let exit = p
        .get("performance_budget_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 93);
    assert_eq!(
        p.get("performance_budget_failure_kind")
            .and_then(|v| v.as_str()),
        Some("helper_fixture_over_budget")
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("runner_contract")
        .and_then(|v| v.as_table())
        .unwrap();
    let keys: Vec<&str> = c
        .get("keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "outcome",
        "case",
        "module_name",
        "helper_family",
        "helper",
        "input_python_repr",
        "expected_python_repr",
        "actual_python_repr",
        "duration_ms",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            keys.contains(required),
            "runner_contract.keys must include {required}"
        );
    }
    let cases: Vec<&str> = c
        .get("case_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for required in &[
        "fixture_fails_on_any_helper_behavior_mismatch",
        "failure_output_identifies_helper_family",
        "fixture_remains_fast_enough_for_ecosystem_gate",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("full_iterator_algebra_compatibility")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
