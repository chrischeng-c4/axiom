//! Schema gate for the stdlib json behavioral fixture — closes #2628.
//!
//! Acceptance (issue #2628):
//!
//!   1. Fixture fails on wrong roundtrip or escaping behavior.
//!      `[failure_on_incorrect_behavior_contract]` pins
//!      must_fail_on_incorrect_roundtrip +
//!      must_fail_on_incorrect_escaping +
//!      must_distinguish_roundtrip_from_escaping + distinct exit
//!      codes 70/71.
//!   2. Fixture is part of the required ecosystem gate.
//!      `[required_ecosystem_gate_contract]` pins
//!      required_module_manifest_fixture_issue=2624 +
//!      manifest_module_name="json" + skipped exit_code=72.
//!   3. Failure output names json as the module.
//!      `[failure_naming_contract]` pins must_emit_module_name_in_
//!      failure_output + module_name_value="json" +
//!      forbid_generic_unnamed_json_failure + unnamed exit_code=73.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("stdlib")
        .join("json_behavioral")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("stdlib_json_behavioral")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2628));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2529)
    );
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("stdlib"));
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("stdlib_json_behavioral")
    );
    assert_eq!(doc.get("network").and_then(|v| v.as_str()), Some("offline"));
}

#[test]
fn isolation_pins_no_global_state() {
    let doc = load_toml(&manifest_path());
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
    let doc = load_toml(&manifest_path());
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
fn surface_covers_dumps_loads_and_all_value_kinds() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("surface")
        .and_then(|v| v.as_table())
        .expect("[surface] missing");
    assert_eq!(s.get("module_name").and_then(|v| v.as_str()), Some("json"));
    assert_eq!(
        s.get("must_be_importable_via_import_statement")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("import_statement").and_then(|v| v.as_str()),
        Some("import json")
    );
    for f in &[
        "must_cover_dumps",
        "must_cover_loads",
        "must_cover_dict",
        "must_cover_list",
        "must_cover_string_escaping",
        "must_cover_numeric",
        "must_cover_boolean",
        "must_cover_null",
    ] {
        assert_eq!(
            s.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn deterministic_sample_covers_all_value_kinds_and_escaping() {
    let doc = load_toml(&manifest_path());
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

    let roundtrips = doc
        .get("roundtrip_cases")
        .and_then(|v| v.as_array())
        .expect("[[roundtrip_cases]] missing");
    let mut kinds: Vec<&str> = Vec::new();
    for c in roundtrips {
        let t = c.as_table().expect("roundtrip case must be a table");
        let k = t
            .get("kind")
            .and_then(|v| v.as_str())
            .expect("roundtrip.kind missing");
        assert!(
            t.get("input_python_repr")
                .and_then(|v| v.as_str())
                .is_some(),
            "roundtrip.input_python_repr missing"
        );
        assert!(
            t.get("expected_dumps").and_then(|v| v.as_str()).is_some(),
            "roundtrip.expected_dumps missing"
        );
        kinds.push(k);
    }
    for k in &["dict", "list", "numeric", "boolean", "null"] {
        assert!(kinds.contains(k), "roundtrip_cases must cover kind={k}");
    }

    let escapings = doc
        .get("escaping_cases")
        .and_then(|v| v.as_array())
        .expect("[[escaping_cases]] missing");
    assert!(!escapings.is_empty(), "escaping_cases must not be empty");
    for c in escapings {
        let t = c.as_table().expect("escaping case must be a table");
        assert!(
            t.get("input_python_string")
                .and_then(|v| v.as_str())
                .is_some(),
            "escaping.input_python_string missing"
        );
        assert!(
            t.get("ensure_ascii").and_then(|v| v.as_bool()).is_some(),
            "escaping.ensure_ascii missing"
        );
        assert!(
            t.get("expected_dumps").and_then(|v| v.as_str()).is_some(),
            "escaping.expected_dumps missing"
        );
    }
}

// Acceptance: "Fixture fails on wrong roundtrip or escaping behavior."
#[test]
fn fixture_fails_on_wrong_roundtrip_or_escaping() {
    let doc = load_toml(&manifest_path());
    let f = doc
        .get("failure_on_incorrect_behavior_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_on_incorrect_behavior_contract] missing — acceptance: \
         \"Fixture fails on wrong roundtrip or escaping behavior.\"",
        );
    for k in &[
        "must_fail_on_incorrect_roundtrip",
        "must_fail_on_incorrect_escaping",
        "must_distinguish_roundtrip_from_escaping",
    ] {
        assert_eq!(
            f.get(*k).and_then(|v| v.as_bool()),
            Some(true),
            "{k} must be true"
        );
    }
    let rt = f
        .get("roundtrip_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    let esc = f
        .get("escaping_mismatch_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(rt, 70);
    assert_eq!(esc, 71);
    assert_ne!(rt, esc, "roundtrip and escaping exit codes must differ");
    assert_eq!(
        f.get("roundtrip_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("json_roundtrip_mismatch")
    );
    assert_eq!(
        f.get("escaping_mismatch_failure_kind")
            .and_then(|v| v.as_str()),
        Some("json_escaping_mismatch")
    );
}

// Acceptance: "Fixture is part of the required ecosystem gate."
#[test]
fn fixture_is_part_of_required_ecosystem_gate() {
    let doc = load_toml(&manifest_path());
    let r = doc
        .get("required_ecosystem_gate_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[required_ecosystem_gate_contract] missing — acceptance: \
         \"Fixture is part of the required ecosystem gate.\"",
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
        Some("json")
    );
    let exit = r
        .get("skipped_in_required_profile_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 72);
    assert_eq!(
        r.get("skipped_in_required_profile_failure_kind")
            .and_then(|v| v.as_str()),
        Some("required_stdlib_fixture_skipped")
    );
}

// Acceptance: "Failure output names json as the module."
#[test]
fn failure_output_names_json_as_the_module() {
    let doc = load_toml(&manifest_path());
    let n = doc
        .get("failure_naming_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[failure_naming_contract] missing — acceptance: \
         \"Failure output names json as the module.\"",
        );
    assert_eq!(
        n.get("must_emit_module_name_in_failure_output")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        n.get("module_name_field_name").and_then(|v| v.as_str()),
        Some("module_name")
    );
    assert_eq!(
        n.get("module_name_value").and_then(|v| v.as_str()),
        Some("json")
    );
    assert_eq!(
        n.get("forbid_generic_unnamed_json_failure")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let exit = n
        .get("unnamed_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 73);
    assert_eq!(
        n.get("unnamed_failure_failure_kind")
            .and_then(|v| v.as_str()),
        Some("json_failure_missing_module_name")
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = load_toml(&manifest_path());
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
        "roundtrip_kind",
        "input_python_repr",
        "expected_dumps",
        "actual_dumps",
        "ensure_ascii",
        "input_python_string",
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
        "fixture_fails_on_wrong_roundtrip_or_escaping",
        "fixture_is_part_of_required_ecosystem_gate",
        "failure_output_names_json_as_the_module",
    ] {
        assert!(
            cases.contains(required),
            "runner_contract.case_values must include {required}"
        );
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        o.get("full_json_compliance_suite")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
