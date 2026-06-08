//! Schema gate for the stdlib dataclasses/typing behavioral fixture —
//! closes #2633.
//!
//! Acceptance (issue #2633):
//!
//!   1. Fixture fails on wrong dataclass initialization or equality
//!      behavior. `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_{instance_creation, field_defaults,
//!      repr, equality} + distinct exit codes 94/95/96/97 +
//!      must_distinguish_each_failure_kind.
//!   2. Current unsupported typing behavior is marked with a linked
//!      blocker if needed. `[unsupported_typing_blocker_contract]`
//!      pins must_mark_unsupported_typing_with_xfail +
//!      must_link_unsupported_typing_to_blocker_issue +
//!      forbid_silently_skipping + missing-blocker exit_code=98.
//!   3. Fixture is part of the ecosystem summary.
//!      `[ecosystem_summary_inclusion_contract]` pins
//!      must_be_listed_in_ecosystem_summary +
//!      ecosystem_summary_fixture_issue=2814 +
//!      summary_required_fields + missing-summary exit_code=99.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("dataclasses_typing_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("stdlib_dataclasses_typing_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2633));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("stdlib_dataclasses_typing_behavioral"));
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
fn surface_covers_dataclasses_and_typing() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["dataclasses", "typing"] {
        assert!(modules.contains(m), "covered_modules must include {m}");
    }
    for f in &[
        "must_be_importable_via_import_statement",
        "must_cover_dataclass_decorator",
        "must_cover_field_with_default",
        "must_cover_field_with_default_factory",
        "must_cover_instance_creation",
        "must_cover_repr",
        "must_cover_equality",
        "must_cover_get_type_hints_when_supported",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
}

#[test]
fn dataclass_definition_is_pinned_and_sample_covers_every_aspect() {
    let doc = crate::common::load_toml(&manifest_path());
    let dc = doc.get("dataclass_definition").and_then(|v| v.as_table()).expect("[dataclass_definition] missing");
    assert_eq!(dc.get("class_name").and_then(|v| v.as_str()), Some("Point"));
    for k in &[
        "field_a_name", "field_a_type", "field_a_default_python_repr",
        "field_b_name", "field_b_type", "field_b_default_python_repr",
        "field_c_name", "field_c_type", "field_c_default_factory_python_repr",
    ] {
        assert!(dc.get(*k).and_then(|v| v.as_str()).is_some(), "dataclass_definition.{k} missing");
    }

    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let specs: &[(&str, &[&str])] = &[
        ("instance_creation_cases", &["call", "expected_x", "expected_y", "expected_tags_python_repr"]),
        ("repr_cases", &["call", "expected_repr"]),
        ("equality_cases", &["left_call", "right_call", "expected_relation"]),
        ("get_type_hints_cases", &["target", "expected_hints_python_repr", "supported"]),
    ];
    for (key, fields) in specs {
        let arr = doc.get(*key).and_then(|v| v.as_array()).unwrap_or_else(|| panic!("[[{key}]] missing"));
        assert!(!arr.is_empty(), "[[{key}]] must not be empty");
        for c in arr {
            let t = c.as_table().expect("case must be a table");
            for f in *fields {
                assert!(t.get(*f).is_some(), "{key}.{f} missing");
            }
        }
    }
}

// Acceptance: "Fixture fails on wrong dataclass initialization or equality behavior."
#[test]
fn fixture_fails_on_wrong_initialization_or_equality() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc.get("failure_on_incorrect_behavior_contract").and_then(|v| v.as_table()).expect(
        "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong dataclass initialization or equality behavior.\"",
    );
    for k in &[
        "must_fail_on_incorrect_instance_creation",
        "must_fail_on_incorrect_field_defaults",
        "must_fail_on_incorrect_repr",
        "must_fail_on_incorrect_equality",
        "must_distinguish_each_failure_kind",
    ] {
        assert_eq!(f.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exits: Vec<i64> = [
        "initialization_mismatch_exit_code",
        "field_defaults_mismatch_exit_code",
        "repr_mismatch_exit_code",
        "equality_mismatch_exit_code",
    ].iter().map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap()).collect();
    assert_eq!(exits, vec![94, 95, 96, 97]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), exits.len(), "failure exit codes must all differ");
}

// Acceptance: "Current unsupported typing behavior is marked with a linked blocker if needed."
#[test]
fn unsupported_typing_behavior_marked_with_linked_blocker() {
    let doc = crate::common::load_toml(&manifest_path());
    let b = doc.get("unsupported_typing_blocker_contract").and_then(|v| v.as_table()).expect(
        "[unsupported_typing_blocker_contract] missing — acceptance: \
         \"Current unsupported typing behavior is marked with a linked blocker if needed.\"",
    );
    for k in &[
        "must_mark_unsupported_typing_with_xfail",
        "must_link_unsupported_typing_to_blocker_issue",
        "forbid_silently_skipping_unsupported_typing",
        "forbid_falsely_passing_unsupported_typing",
    ] {
        assert_eq!(b.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(b.get("blocker_link_field_name").and_then(|v| v.as_str()), Some("blocker_issue"));
    assert_eq!(b.get("unsupported_typing_outcome_value").and_then(|v| v.as_str()), Some("xfail"));
    let exit = b.get("missing_blocker_link_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 98);
    assert_eq!(b.get("missing_blocker_link_failure_kind").and_then(|v| v.as_str()), Some("unsupported_typing_missing_blocker_link"));
    let allowed: Vec<&str> = b.get("allowed_unsupported_typing_paths").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(!allowed.is_empty(), "allowed_unsupported_typing_paths must enumerate at least one path");
}

// Acceptance: "Fixture is part of the ecosystem summary."
#[test]
fn fixture_is_part_of_ecosystem_summary() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc.get("ecosystem_summary_inclusion_contract").and_then(|v| v.as_table()).expect(
        "[ecosystem_summary_inclusion_contract] missing — acceptance: \
         \"Fixture is part of the ecosystem summary.\"",
    );
    assert_eq!(e.get("must_be_listed_in_ecosystem_summary").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(e.get("ecosystem_summary_fixture_issue").and_then(|v| v.as_integer()), Some(2814));
    assert_eq!(e.get("must_emit_summary_record").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(e.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = e.get("summary_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["fixture", "outcome", "duration_ms", "covered_modules", "case"] {
        assert!(required.contains(f), "summary_required_fields must include {f}");
    }
    let covered: Vec<&str> = e.get("covered_modules_in_summary").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for m in &["dataclasses", "typing"] {
        assert!(covered.contains(m), "covered_modules_in_summary must include {m}");
    }
    let exit = e.get("missing_summary_record_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 99);
    assert_eq!(e.get("missing_summary_record_failure_kind").and_then(|v| v.as_str()), Some("ecosystem_summary_missing_dataclasses_typing"));
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "call", "expected_repr", "expected_relation", "expected_hints_python_repr",
        "blocker_issue", "covered_modules",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_on_wrong_initialization_or_equality",
        "unsupported_typing_behavior_marked_with_linked_blocker",
        "fixture_is_part_of_ecosystem_summary",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
    let outcomes: Vec<&str> = c.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(outcomes.contains(&"xfail"), "runner_contract.outcome_values must include xfail");
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("exhaustive_typing_semantics").and_then(|v| v.as_bool()), Some(true));
}
