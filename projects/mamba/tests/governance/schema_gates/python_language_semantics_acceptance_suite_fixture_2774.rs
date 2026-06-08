// Locks the shape of the MVP Python language semantics acceptance
// suite fixture pinned by tests/governance/gates/mvp/
// python_language_semantics_acceptance_suite/manifest.toml.
// Closes #2774. Parent: #2526.

use std::fs;
use std::path::PathBuf;

use toml::Value;

fn manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/governance/gates/mvp/python_language_semantics_acceptance_suite/manifest.toml")
}

fn manifest() -> Value {
    let raw = fs::read_to_string(manifest_path()).expect("read manifest");
    raw.parse::<Value>().expect("parse manifest toml")
}

#[test]
fn header_is_well_formed() {
    let m = manifest();
    assert_eq!(m["version"].as_integer(), Some(1));
    assert_eq!(
        m["fixture"].as_str(),
        Some("python_language_semantics_acceptance_suite")
    );
    assert_eq!(m["issue"].as_integer(), Some(2774));
    assert_eq!(m["parent_issue"].as_integer(), Some(2526));
    assert_eq!(m["profile"].as_str(), Some("mvp"));
    assert_eq!(
        m["family"].as_str(),
        Some("python_language_semantics_acceptance_suite")
    );
    assert_eq!(m["network"].as_str(), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let iso = &manifest()["isolation"];
    for key in [
        "forbid_writes_outside_project",
        "forbid_user_home_reads",
        "forbid_global_cache_reads",
        "forbid_global_cache_writes",
    ] {
        assert_eq!(iso[key].as_bool(), Some(true), "isolation.{key}");
    }
}

#[test]
fn python_target_is_pinned_to_3_12() {
    let py = &manifest()["python_target"];
    assert_eq!(py["python_major"].as_integer(), Some(3));
    assert_eq!(py["python_minor"].as_integer(), Some(12));
    assert_eq!(py["must_be_python_3_12"].as_bool(), Some(true));
}

#[test]
fn surface_pins_grouping_and_no_skip_and_failure_reporting() {
    let s = &manifest()["surface"];
    for key in [
        "must_cover_grouping_by_semantic_area",
        "must_cover_no_skip_no_import_only_for_required_fixtures",
        "must_cover_failure_reporting_language_feature_and_fixture_id",
        "must_be_offline",
        "must_be_deterministic",
        "must_be_runnable_without_network",
    ] {
        assert_eq!(s[key].as_bool(), Some(true), "surface.{key}");
    }
}

#[test]
fn child_leaf_issues_are_grouped_by_semantic_area() {
    let c = &manifest()["semantic_areas_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("child_leaf_issues_are_grouped_by_semantic_area")
    );
    let areas: Vec<_> = c["required_semantic_areas"]
        .as_array()
        .expect("required_semantic_areas")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        areas,
        vec![
            "scope",
            "functions",
            "classes",
            "descriptors",
            "exceptions",
            "generators",
            "async",
            "pattern_matching",
            "comprehensions",
            "imports",
            "protocol_behavior",
        ]
    );
    assert_eq!(
        c["semantic_area_field_name"].as_str(),
        Some("semantic_area")
    );
    for key in [
        "forbid_unknown_semantic_area",
        "forbid_silently_dropping_required_semantic_area",
        "forbid_collapsed_or_implicit_semantic_area",
        "must_distinguish_unknown_area_from_missing_required_area",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["unknown_semantic_area_failure_kind"].as_str(),
        Some("py_language_semantics_unknown_area")
    );
    assert_eq!(c["unknown_semantic_area_exit_code"].as_integer(), Some(241));
    assert_eq!(
        c["missing_required_semantic_area_failure_kind"].as_str(),
        Some("py_language_semantics_required_area_missing")
    );
    assert_eq!(
        c["missing_required_semantic_area_exit_code"].as_integer(),
        Some(242)
    );
}

#[test]
fn required_semantic_fixtures_cannot_be_skipped_or_counted_as_import_only() {
    let c = &manifest()["no_skip_no_import_only_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("required_semantic_fixtures_cannot_be_skipped_or_counted_as_import_only")
    );
    for key in [
        "must_fail_required_fixture_when_skipped",
        "must_fail_required_fixture_when_xfailed",
        "must_fail_required_fixture_when_ignored",
        "must_fail_required_fixture_when_classified_as_import_only",
        "forbid_silently_passing_required_fixture_via_skip",
        "forbid_silently_passing_required_fixture_via_xfail",
        "forbid_silently_passing_required_fixture_via_ignore",
        "forbid_silently_passing_required_fixture_via_import_only",
        "forbid_required_fixture_being_classified_as_import_only",
        "must_distinguish_required_fixture_skipped_from_required_fixture_import_only",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    assert_eq!(
        c["required_fixture_classification_field_name"].as_str(),
        Some("fixture_classification")
    );
    let allowed: Vec<_> = c["required_classification_allowed_values"]
        .as_array()
        .expect("required_classification_allowed_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(allowed, vec!["executed_assertion"]);
    let disallowed: Vec<_> = c["disallowed_classification_values_for_required_fixtures"]
        .as_array()
        .expect("disallowed_classification_values_for_required_fixtures")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        disallowed,
        vec!["skipped", "xfailed", "ignored", "import_only"]
    );
    assert_eq!(
        c["required_fixture_skipped_failure_kind"].as_str(),
        Some("py_language_semantics_required_fixture_skipped")
    );
    assert_eq!(
        c["required_fixture_skipped_exit_code"].as_integer(),
        Some(243)
    );
    assert_eq!(
        c["required_fixture_import_only_failure_kind"].as_str(),
        Some("py_language_semantics_required_fixture_import_only")
    );
    assert_eq!(
        c["required_fixture_import_only_exit_code"].as_integer(),
        Some(244)
    );
}

#[test]
fn failures_report_the_language_feature_and_fixture_id() {
    let c = &manifest()["failure_reporting_contract"];
    assert_eq!(
        c["case"].as_str(),
        Some("failures_report_the_language_feature_and_fixture_id")
    );
    for key in [
        "must_report_language_feature_on_failure",
        "must_report_fixture_id_on_failure",
        "forbid_omitting_language_feature_on_failure",
        "forbid_omitting_fixture_id_on_failure",
        "forbid_collapsed_or_implicit_failure_report",
        "must_distinguish_language_feature_missing_from_fixture_id_missing",
    ] {
        assert_eq!(c[key].as_bool(), Some(true), "{key}");
    }
    let fields: Vec<_> = c["required_failure_report_fields"]
        .as_array()
        .expect("required_failure_report_fields")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        fields,
        vec!["language_feature", "fixture_id", "semantic_area", "outcome"]
    );
    assert_eq!(
        c["failure_report_field_name"].as_str(),
        Some("failure_report")
    );
    assert_eq!(
        c["language_feature_missing_failure_kind"].as_str(),
        Some("py_language_semantics_failure_report_language_feature_missing")
    );
    assert_eq!(
        c["language_feature_missing_exit_code"].as_integer(),
        Some(245)
    );
    assert_eq!(
        c["fixture_id_missing_failure_kind"].as_str(),
        Some("py_language_semantics_failure_report_fixture_id_missing")
    );
    assert_eq!(c["fixture_id_missing_exit_code"].as_integer(), Some(246));
}

#[test]
fn child_leaf_size_policy_pins_5_to_15_minutes() {
    let c = &manifest()["child_leaf_size_policy"];
    assert_eq!(
        c["must_keep_each_child_leaf_size_agent_sized"].as_bool(),
        Some(true)
    );
    assert_eq!(c["agent_sized_lower_bound_minutes"].as_integer(), Some(5));
    assert_eq!(c["agent_sized_upper_bound_minutes"].as_integer(), Some(15));
    assert_eq!(
        c["forbid_child_leaf_exceeding_upper_bound"].as_bool(),
        Some(true)
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let r = &manifest()["runner_contract"];
    let keys: Vec<_> = r["keys"]
        .as_array()
        .expect("keys")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        keys,
        vec![
            "outcome",
            "case",
            "semantic_area",
            "language_feature",
            "fixture_id",
            "fixture_classification",
            "failure_report",
            "failure_kind",
            "exit_code",
        ]
    );
    let outcomes: Vec<_> = r["outcome_values"]
        .as_array()
        .expect("outcome_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(outcomes, vec!["pass", "fail", "missing", "skip"]);
    let cases: Vec<_> = r["case_values"]
        .as_array()
        .expect("case_values")
        .iter()
        .map(|v| v.as_str().expect("string"))
        .collect();
    assert_eq!(
        cases,
        vec![
            "child_leaf_issues_are_grouped_by_semantic_area",
            "required_semantic_fixtures_cannot_be_skipped_or_counted_as_import_only",
            "failures_report_the_language_feature_and_fixture_id",
        ]
    );
}

#[test]
fn pins_out_of_scope_per_issue() {
    let o = &manifest()["out_of_scope"];
    assert_eq!(o["runtime_optimization"].as_bool(), Some(true));
}
