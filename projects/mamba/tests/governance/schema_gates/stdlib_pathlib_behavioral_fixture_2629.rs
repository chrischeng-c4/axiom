//! Schema gate for the stdlib pathlib behavioral fixture — closes #2629.
//!
//! Acceptance (issue #2629):
//!
//!   1. Fixture fails on wrong path component behavior.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_{joining, suffix, name, parent,
//!      relative_to} + must_distinguish_each_component_mismatch +
//!      distinct exit codes 74/75/76/77/78.
//!   2. Fixture avoids machine-specific absolute path expectations.
//!      `[machine_agnostic_path_contract]` pins
//!      forbid_assertions_on_absolute_paths +
//!      must_use_purepath_or_relative_paths_only + allowed/forbidden
//!      lists + machine-specific exit_code=79.
//!   3. Runner records pathlib under required stdlib coverage.
//!      `[required_ecosystem_gate_contract]` pins
//!      required_module_manifest_fixture_issue=2624 +
//!      manifest_module_name="pathlib" +
//!      runner_must_record_module_name="pathlib" + skipped exit_code=80.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("stdlib")
        .join("pathlib_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("stdlib_pathlib_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2629));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_pathlib_behavioral")
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
fn surface_covers_purepath_join_suffix_name_parent_relative() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("surface")
        .and_then(|v| v.as_table())
        .expect("[surface] missing");
    assert_eq!(
        s.get("module_name").and_then(|v| v.as_str()),
        Some("pathlib")
    );
    assert_eq!(
        s.get("must_be_importable_via_import_statement")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import pathlib")
    );
    for f in &[
        "must_cover_purepath",
        "must_cover_path_joining",
        "must_cover_suffix",
        "must_cover_name",
        "must_cover_parent",
        "must_cover_relative_to",
        "must_allow_tempdir_backed_path",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn deterministic_sample_covers_every_component_kind() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("deterministic_sample")
        .and_then(|v| v.as_table())
        .expect("[deterministic_sample] missing");
    assert_eq!(
        d.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_be_small_enough_for_per_run_check")
            .and_then(|v| v.as_bool()),
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
    assert!(
        min >= 1 && max >= min,
        "sample bounds must be sane: min={min} max={max}"
    );
    assert!(
        max <= 64,
        "sample_max_records must stay small enough for per-run check, got {max}"
    );
    assert_eq!(
        d.get("purepath_class_name").and_then(|v| v.as_str()),
        Some("PurePosixPath")
    );

    let specs: &[(&str, &[&str])] = &[
        ("joining_cases", &["parts", "expected_joined_str"]),
        ("suffix_cases", &["input_path", "expected_suffix"]),
        ("name_cases", &["input_path", "expected_name"]),
        ("parent_cases", &["input_path", "expected_parent_str"]),
        (
            "relative_to_cases",
            &["input_path", "base_path", "expected_relative_str"],
        ),
    ];
    for (key, fields) in specs {
        let arr = doc
            .get(*key)
            .and_then(|v| v.as_array())
            .unwrap_or_else(|| panic!("[[{key}]] missing"));
        assert!(!arr.is_empty(), "[[{key}]] must not be empty");
        for c in arr {
            let t = c.as_table().expect("case must be a table");
            for f in *fields {
                assert!(t.get(*f).is_some(), "{key}.{f} missing");
            }
        }
    }
}

// Acceptance: "Fixture fails on wrong path component behavior."
#[test]
fn fixture_fails_on_wrong_path_component_behavior() {
    let doc = crate::common::load_toml(&manifest_path());
    let f = doc
        .get("failure_on_incorrect_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong path component behavior.\"",
        );
    for k in &[
        "must_fail_on_incorrect_joining",
        "must_fail_on_incorrect_suffix",
        "must_fail_on_incorrect_name",
        "must_fail_on_incorrect_parent",
        "must_fail_on_incorrect_relative_to",
        "must_distinguish_each_component_mismatch",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let exits: Vec<i64> = [
        "joining_mismatch_exit_code",
        "suffix_mismatch_exit_code",
        "name_mismatch_exit_code",
        "parent_mismatch_exit_code",
        "relative_to_mismatch_exit_code",
    ]
    .iter()
    .map(|k| f.get(*k).and_then(|v| v.as_integer()).unwrap())
    .collect();
    assert_eq!(exits, vec![74, 75, 76, 77, 78]);
    let mut sorted = exits.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        exits.len(),
        "all component-mismatch exit codes must differ"
    );
}

// Acceptance: "Fixture avoids machine-specific absolute path expectations."
#[test]
fn fixture_avoids_machine_specific_absolute_path_expectations() {
    let doc = crate::common::load_toml(&manifest_path());
    let m = doc
        .get("machine_agnostic_path_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[machine_agnostic_path_contract] missing — acceptance: \
         \"Fixture avoids machine-specific absolute path expectations.\"",
        );
    for k in &[
        "forbid_assertions_on_absolute_paths",
        "forbid_assertions_on_user_home_paths",
        "forbid_assertions_on_current_working_directory_paths",
        "must_use_purepath_or_relative_paths_only",
    ] {
        assert_eq!(
            m.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let allowed: Vec<&str> = m
        .get("allowed_path_classes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for c in &["PurePosixPath", "PurePath", "Path_under_tempdir"] {
        assert!(allowed.contains(c), "allowed_path_classes must include {c}");
    }
    let forbidden: Vec<&str> = m
        .get("forbidden_assertion_patterns")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for p in &[
        "starts_with_slash",
        "starts_with_drive_letter",
        "contains_home_dir",
        "contains_user_name",
        "contains_temp_dir_absolute_prefix",
    ] {
        assert!(
            forbidden.contains(p),
            "forbidden_assertion_patterns must include {p}"
        );
    }
    let exit = m
        .get("machine_specific_assertion_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 79);
    assert_eq!(
        m.get("machine_specific_assertion_failure_kind")
            .and_then(|v| v.as_str()),
        Some("pathlib_machine_specific_absolute_path")
    );
}

// Acceptance: "Runner records pathlib under required stdlib coverage."
#[test]
fn runner_records_pathlib_under_required_stdlib_coverage() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc
        .get("required_ecosystem_gate_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[required_ecosystem_gate_contract] missing — acceptance: \
         \"Runner records pathlib under required stdlib coverage.\"",
        );
    for k in &[
        "must_be_listed_in_required_module_manifest",
        "must_be_required_in_manifest",
        "must_run_in_default_ecosystem_profile",
        "must_fail_default_profile_if_skipped",
    ] {
        assert_eq!(
            r.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    assert_eq!(
        r.get("required_module_manifest_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2624)
    );
    assert_eq!(
        r.get("manifest_module_name").and_then(|v| v.as_str()),
        Some("pathlib")
    );
    assert_eq!(
        r.get("runner_must_record_module_name")
            .and_then(|v| v.as_str()),
        Some("pathlib")
    );
    assert_eq!(
        r.get("runner_coverage_field_name").and_then(|v| v.as_str()),
        Some("stdlib_required_module_covered")
    );
    let exit = r
        .get("skipped_in_required_profile_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 80);
    assert_eq!(
        r.get("skipped_in_required_profile_failure_kind")
            .and_then(|v| v.as_str()),
        Some("required_stdlib_fixture_skipped")
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
        "input_path",
        "expected_joined_str",
        "expected_suffix",
        "expected_name",
        "expected_parent_str",
        "expected_relative_str",
        "stdlib_required_module_covered",
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
        "fixture_fails_on_wrong_path_component_behavior",
        "fixture_avoids_machine_specific_absolute_path_expectations",
        "runner_records_pathlib_under_required_stdlib_coverage",
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
        o.get("platform_specific_filesystem_edge_cases")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
