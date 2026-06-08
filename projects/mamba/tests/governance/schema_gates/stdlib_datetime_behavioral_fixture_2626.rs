//! Schema gate for the stdlib datetime behavioral fixture —
//! closes #2626.
//!
//! Acceptance (issue #2626):
//!
//!   1. Fixture fails on incorrect arithmetic or formatting.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_arithmetic +
//!      must_fail_on_incorrect_comparison +
//!      must_fail_on_incorrect_isoformat_parse +
//!      must_fail_on_incorrect_isoformat_format + distinct exit
//!      codes 62/63/64 + must_distinguish_arithmetic_from_
//!      comparison_from_isoformat.
//!   2. Fixture is part of the required ecosystem gate.
//!      `[required_ecosystem_gate_contract]` pins
//!      must_be_listed_in_required_module_manifest +
//!      required_module_manifest_fixture_issue=2624 +
//!      manifest_module_name="datetime" +
//!      must_run_in_default_ecosystem_profile +
//!      skipped-in-required exit_code=65.
//!   3. Timezone or locale-sensitive behavior is avoided unless
//!      deterministic. `[timezone_locale_safety_contract]` pins
//!      forbid_naive_local_now_comparison +
//!      forbid_locale_sensitive_string_formatting +
//!      forbid_implicit_system_timezone_dependency +
//!      allowed_timezone_modes / forbidden_timezone_modes +
//!      nondeterministic-path exit_code=66.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("datetime_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_datetime_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2626));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_datetime_behavioral"));
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
    let p = doc.get("python_target").and_then(|v| v.as_table()).expect("[python_target] missing");
    assert_eq!(p.get("python_major").and_then(|v| v.as_integer()), Some(3));
    assert_eq!(p.get("python_minor").and_then(|v| v.as_integer()), Some(12));
    assert_eq!(p.get("must_be_python_3_12").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn surface_covers_date_datetime_timedelta_isoformat() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    assert_eq!(s.get("module_name").and_then(|v| v.as_str()), Some("datetime"));
    assert_eq!(s.get("must_be_importable_via_import_statement").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import datetime"));
    for f in &[
        "must_cover_date",
        "must_cover_datetime",
        "must_cover_timedelta",
        "must_cover_isoformat_parse",
        "must_cover_isoformat_format",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn deterministic_sample_arrays_are_bounded_and_consistent() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("must_be_small_enough_for_per_run_check").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min, "sample bounds must be sane: min={min} max={max}");
    assert!(max <= 64, "sample_max_records must stay small enough for per-run check, got {max}");

    let arithmetic = doc.get("arithmetic_cases").and_then(|v| v.as_array()).expect("[[arithmetic_cases]] missing");
    assert!(!arithmetic.is_empty(), "arithmetic_cases must not be empty");
    for c in arithmetic {
        let t = c.as_table().expect("arithmetic case must be a table");
        assert!(t.get("left").and_then(|v| v.as_str()).is_some(), "arithmetic.left missing");
        assert!(t.get("delta_days").and_then(|v| v.as_integer()).is_some(), "arithmetic.delta_days missing");
        assert!(t.get("expected_right").and_then(|v| v.as_str()).is_some(), "arithmetic.expected_right missing");
        assert_eq!(t.get("operator").and_then(|v| v.as_str()), Some("add_timedelta_days"));
    }

    let comparison = doc.get("comparison_cases").and_then(|v| v.as_array()).expect("[[comparison_cases]] missing");
    assert!(!comparison.is_empty(), "comparison_cases must not be empty");
    for c in comparison {
        let t = c.as_table().expect("comparison case must be a table");
        assert!(t.get("left").and_then(|v| v.as_str()).is_some(), "comparison.left missing");
        assert!(t.get("right").and_then(|v| v.as_str()).is_some(), "comparison.right missing");
        let rel = t.get("expected_relation").and_then(|v| v.as_str()).expect("comparison.expected_relation missing");
        assert!(
            ["less_than", "equal", "greater_than"].contains(&rel),
            "expected_relation must be one of less_than|equal|greater_than, got {rel}",
        );
    }

    let iso = doc.get("isoformat_cases").and_then(|v| v.as_array()).expect("[[isoformat_cases]] missing");
    assert!(!iso.is_empty(), "isoformat_cases must not be empty");
    for c in iso {
        let t = c.as_table().expect("isoformat case must be a table");
        assert!(t.get("input").and_then(|v| v.as_str()).is_some(), "isoformat.input missing");
        for f in &["expected_year", "expected_month", "expected_day", "expected_hour", "expected_minute", "expected_second"] {
            assert!(t.get(*f).and_then(|v| v.as_integer()).is_some(), "isoformat.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails on incorrect arithmetic or formatting."
#[test]
fn fixture_fails_on_incorrect_arithmetic_or_formatting() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_incorrect_behavior_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on incorrect arithmetic or formatting.\"",
    );
    for k in &[
        "must_fail_on_incorrect_arithmetic",
        "must_fail_on_incorrect_comparison",
        "must_fail_on_incorrect_isoformat_parse",
        "must_fail_on_incorrect_isoformat_format",
        "must_distinguish_arithmetic_from_comparison_from_isoformat",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let arith = f.get("arithmetic_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let comp = f.get("comparison_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    let iso = f.get("isoformat_mismatch_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(arith, 62);
    assert_eq!(comp, 63);
    assert_eq!(iso, 64);
    assert_ne!(arith, comp, "arithmetic and comparison exit codes must differ");
    assert_ne!(arith, iso, "arithmetic and isoformat exit codes must differ");
    assert_ne!(comp, iso, "comparison and isoformat exit codes must differ");
    assert_eq!(f.get("arithmetic_mismatch_failure_kind").and_then(|v| v.as_str()), Some("datetime_arithmetic_mismatch"));
    assert_eq!(f.get("comparison_mismatch_failure_kind").and_then(|v| v.as_str()), Some("datetime_comparison_mismatch"));
    assert_eq!(f.get("isoformat_mismatch_failure_kind").and_then(|v| v.as_str()), Some("datetime_isoformat_mismatch"));
}

// Acceptance: "Fixture is part of the required ecosystem gate."
#[test]
fn fixture_is_part_of_required_ecosystem_gate() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc.get("required_ecosystem_gate_contract").and_then(|v| v.as_table()).expect(
        "[required_ecosystem_gate_contract] missing — acceptance: \
         \"Fixture is part of the required ecosystem gate.\"",
    );
    for k in &[
        "must_be_listed_in_required_module_manifest",
        "must_be_required_in_manifest",
        "must_run_in_default_ecosystem_profile",
        "must_fail_default_profile_if_skipped",
    ] {
        assert_eq!(r.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(r.get("required_module_manifest_fixture_issue").and_then(|v| v.as_integer()), Some(2624));
    assert_eq!(r.get("manifest_module_name").and_then(|v| v.as_str()), Some("datetime"));
    let exit = r.get("skipped_in_required_profile_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 65);
    assert_eq!(r.get("skipped_in_required_profile_failure_kind").and_then(|v| v.as_str()), Some("required_stdlib_fixture_skipped"));
}

// Acceptance: "Timezone or locale-sensitive behavior is avoided unless deterministic."
#[test]
fn timezone_and_locale_behavior_is_deterministic_or_avoided() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("timezone_locale_safety_contract").and_then(|v| v.as_table()).expect(
        "[timezone_locale_safety_contract] missing — acceptance: \
         \"Timezone or locale-sensitive behavior is avoided unless deterministic.\"",
    );
    for k in &[
        "forbid_naive_local_now_comparison",
        "forbid_locale_sensitive_string_formatting",
        "forbid_implicit_system_timezone_dependency",
    ] {
        assert_eq!(t.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let allowed: Vec<&str> = t.get("allowed_timezone_modes").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["utc_only", "fixed_offset", "naive_in_utc_for_arithmetic"] {
        assert!(allowed.contains(m), "allowed_timezone_modes must include {m}");
    }
    let forbidden: Vec<&str> = t.get("forbidden_timezone_modes").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["system_local_timezone", "implicit_dst_dependent", "locale_dependent_strftime"] {
        assert!(forbidden.contains(m), "forbidden_timezone_modes must include {m}");
    }
    for m in &allowed {
        assert!(!forbidden.contains(m), "{m} cannot be both allowed and forbidden");
    }
    let exit = t.get("nondeterministic_path_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 66);
    assert_eq!(t.get("nondeterministic_path_failure_kind").and_then(|v| v.as_str()), Some("datetime_nondeterministic_path_used"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name", "operator",
        "left", "right", "expected_right", "expected_relation",
        "isoformat_input", "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_on_incorrect_arithmetic_or_formatting",
        "fixture_is_part_of_required_ecosystem_gate",
        "timezone_and_locale_behavior_is_deterministic_or_avoided",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("full_timezone_database_compatibility").and_then(|v| v.as_bool()), Some(true));
}
