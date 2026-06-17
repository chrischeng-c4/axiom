//! Schema gate for the third-party SQLAlchemy expression fixture —
//! closes #2644.
//!
//! Acceptance (issue #2644):
//!
//!   1. Fixture fails if SQLAlchemy cannot import.
//!      `[import_failure_contract]` pins must_fail_on_import_error +
//!      must_fail_on_missing_sqlalchemy_module +
//!      forbid_silent_fallback_when_sqlalchemy_missing + exit 151.
//!   2. Fixture does not open a database connection.
//!      `[no_database_connection_contract]` pins
//!      must_not_open_database_connection +
//!      must_not_create_engine_with_real_dsn +
//!      forbid_use_of_create_engine_connect +
//!      forbid_use_of_session_execute_against_engine +
//!      forbid_use_of_dbapi_driver +
//!      must_compile_expression_without_engine_bind + distinct exit
//!      codes 152 (engine connect) / 153 (dbapi driver) +
//!      must_distinguish_engine_connect_from_dbapi_driver_use.
//!   3. Summary records SQLAlchemy as ORM/tooling coverage.
//!      `[orm_tooling_coverage_reporting_contract]` pins
//!      must_emit_orm_tooling_coverage_in_runner_output +
//!      required_orm_tooling_dependencies_covered ⊇ [sqlalchemy] +
//!      must_emit_summary_record_with_orm_tooling_coverage +
//!      forbid_silent_or_implicit_orm_tooling_coverage + exit 154.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("third_party")
        .join("sqlalchemy_expression_behavioral")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("third_party_sqlalchemy_expression_behavioral"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2644));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2529));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("third_party"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("third_party_sqlalchemy_expression_behavioral"));
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
fn surface_covers_sqlalchemy() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("surface").and_then(|v| v.as_table()).expect("[surface] missing");
    let modules: Vec<&str> = s.get("covered_modules").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(modules.contains(&"sqlalchemy"), "covered_modules must include sqlalchemy");
    for f in &[
        "must_be_importable_via_import_statement",
        "must_register_sqlalchemy_in_ecosystem_manifest",
        "must_cover_metadata_construction",
        "must_cover_table_construction",
        "must_cover_column_construction",
        "must_cover_select_expression_construction",
        "must_cover_expression_compilation_or_stringification",
        "must_not_require_database_engine",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    assert_eq!(s.get("import_statement").and_then(|v| v.as_str()), Some("import sqlalchemy"));
}

#[test]
fn expression_definition_pins_canonical_table() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc.get("expression_definition").and_then(|v| v.as_table()).expect("[expression_definition] missing");
    assert_eq!(e.get("metadata_variable").and_then(|v| v.as_str()), Some("metadata_2644"));
    assert_eq!(e.get("table_name").and_then(|v| v.as_str()), Some("mamba_2644"));
    assert_eq!(e.get("column_a_name").and_then(|v| v.as_str()), Some("id"));
    assert_eq!(e.get("column_a_type").and_then(|v| v.as_str()), Some("Integer"));
    assert_eq!(e.get("column_a_primary_key").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(e.get("column_b_name").and_then(|v| v.as_str()), Some("label"));
    assert_eq!(e.get("column_b_type").and_then(|v| v.as_str()), Some("String"));
    assert_eq!(e.get("select_expression_python_repr").and_then(|v| v.as_str()), Some("select(mamba_2644)"));
}

#[test]
fn deterministic_sample_covers_compilation() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("deterministic_sample").and_then(|v| v.as_table()).expect("[deterministic_sample] missing");
    assert_eq!(d.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    let max = d.get("sample_max_records").and_then(|v| v.as_integer()).unwrap();
    let min = d.get("sample_min_records").and_then(|v| v.as_integer()).unwrap();
    assert!(min >= 1 && max >= min && max <= 64, "sample bounds must be sane");

    let arr = doc.get("compilation_cases").and_then(|v| v.as_array()).expect("[[compilation_cases]] missing");
    assert!(!arr.is_empty(), "compilation_cases must not be empty");
    for c in arr {
        let t = c.as_table().expect("case must be a table");
        for f in &[
            "expression_python_repr", "dialect",
            "expected_sql_substring_lower",
            "expected_sql_must_contain_table_name",
            "expected_sql_must_contain_column_id",
            "expected_sql_must_contain_column_label",
        ] {
            assert!(t.get(*f).is_some(), "compilation_cases.{f} missing");
        }
    }
}

// Acceptance: "Fixture fails if SQLAlchemy cannot import."
#[test]
fn fixture_fails_if_sqlalchemy_cannot_import() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("import_failure_contract").and_then(|v| v.as_table()).expect(
        "[import_failure_contract] missing — acceptance: \
         \"Fixture fails if SQLAlchemy cannot import.\"",
    );
    for k in &[
        "must_fail_on_import_error",
        "must_fail_on_missing_sqlalchemy_module",
        "must_emit_import_failure_kind_when_sqlalchemy_missing",
        "forbid_silent_fallback_when_sqlalchemy_missing",
    ] {
        assert_eq!(i.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let exit = i.get("sqlalchemy_import_failure_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 151);
    assert_eq!(
        i.get("sqlalchemy_import_failure_kind").and_then(|v| v.as_str()),
        Some("third_party_sqlalchemy_import_failed"),
    );
}

// Acceptance: "Fixture does not open a database connection."
#[test]
fn fixture_does_not_open_a_database_connection() {
    let doc = crate::common::load_toml(&manifest_path());
    let n = doc.get("no_database_connection_contract").and_then(|v| v.as_table()).expect(
        "[no_database_connection_contract] missing — acceptance: \
         \"Fixture does not open a database connection.\"",
    );
    for k in &[
        "must_not_open_database_connection",
        "must_not_create_engine_with_real_dsn",
        "must_not_perform_network_io",
        "forbid_use_of_create_engine_connect",
        "forbid_use_of_session_execute_against_engine",
        "forbid_use_of_dbapi_driver",
        "must_compile_expression_without_engine_bind",
        "must_distinguish_engine_connect_from_dbapi_driver_use",
    ] {
        assert_eq!(n.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    let conn = n.get("engine_connect_exit_code").and_then(|v| v.as_integer()).unwrap();
    let dbapi = n.get("dbapi_driver_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(conn, 152);
    assert_eq!(dbapi, 153);
    assert_ne!(conn, dbapi, "engine-connect and dbapi-driver exit codes must differ");
    assert_eq!(
        n.get("engine_connect_failure_kind").and_then(|v| v.as_str()),
        Some("sqlalchemy_engine_connect_used"),
    );
    assert_eq!(
        n.get("dbapi_driver_failure_kind").and_then(|v| v.as_str()),
        Some("sqlalchemy_dbapi_driver_used"),
    );
}

// Acceptance: "Summary records SQLAlchemy as ORM/tooling coverage."
#[test]
fn summary_records_sqlalchemy_as_orm_tooling_coverage() {
    let doc = crate::common::load_toml(&manifest_path());
    let h = doc.get("orm_tooling_coverage_reporting_contract").and_then(|v| v.as_table()).expect(
        "[orm_tooling_coverage_reporting_contract] missing — acceptance: \
         \"Summary records SQLAlchemy as ORM/tooling coverage.\"",
    );
    for k in &[
        "must_emit_orm_tooling_coverage_in_runner_output",
        "must_emit_summary_record_with_orm_tooling_coverage",
        "forbid_silent_or_implicit_orm_tooling_coverage",
        "must_distinguish_orm_tooling_coverage_from_overall_outcome",
    ] {
        assert_eq!(h.get(*k).and_then(|v| v.as_bool()), Some(true), "{k} must be true");
    }
    assert_eq!(
        h.get("orm_tooling_coverage_field_name").and_then(|v| v.as_str()),
        Some("orm_tooling_dependencies_covered"),
    );
    let req: Vec<&str> = h.get("required_orm_tooling_dependencies_covered").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(req.contains(&"sqlalchemy"), "required_orm_tooling_dependencies_covered must include sqlalchemy");
    assert_eq!(h.get("summary_record_format").and_then(|v| v.as_str()), Some("json"));
    let exit = h.get("missing_orm_tooling_coverage_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 154);
    assert_eq!(
        h.get("missing_orm_tooling_coverage_failure_kind").and_then(|v| v.as_str()),
        Some("sqlalchemy_orm_tooling_coverage_missing"),
    );
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "module_name",
        "expression_python_repr", "dialect",
        "expected_sql_substring_lower",
        "expected_sql_must_contain_table_name",
        "expected_sql_must_contain_column_id",
        "expected_sql_must_contain_column_label",
        "orm_tooling_dependencies_covered",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "fixture_fails_if_sqlalchemy_cannot_import",
        "fixture_does_not_open_a_database_connection",
        "summary_records_sqlalchemy_as_orm_tooling_coverage",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("db_driver_integration").and_then(|v| v.as_bool()), Some(true));
}
