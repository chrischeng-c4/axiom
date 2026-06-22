//! Schema gate for the transitive dependency resolution fixture —
//! closes #2592.
//!
//! Acceptance (issue #2592):
//!
//!   1. Lockfile contains both direct and transitive dependency
//!      entries. `[lockfile_contract]` pins must_record_direct +
//!      must_record_transitive + must_mark_transitive_as_dependency_
//!      of_direct + exact-version pins + missing_transitive_entry_
//!      exit_code=26.
//!   2. Runtime import proves transitive dependency activation.
//!      `[runtime_import_contract]` pins must_run_import_script +
//!      direct + transitive import + distinct failure kinds for
//!      transitive (28) vs direct (29) import failure.
//!   3. Resolver output names the dependency edge.
//!      `[resolver_diagnostic_contract]` pins json + names direct +
//!      transitive + edge from/to/constraint fields + exit_code=30
//!      when the edge is missing.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("transitive_dependency_resolution")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("transitive_dependency_resolution"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2592));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2532)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("package_manager")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("transitive_dependency_resolution")
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
fn index_cross_references_frozen_local_simple_index() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc
        .get("index")
        .and_then(|v| v.as_table())
        .expect("[index] missing");
    assert_eq!(
        i.get("kind").and_then(|v| v.as_str()),
        Some("frozen_local_simple_index")
    );
    assert_eq!(
        i.get("local_simple_index_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2585),
        "must cross-reference frozen local simple-index fixture #2585",
    );
    assert_eq!(
        i.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn dependency_edge_is_single_and_consistent_with_direct_and_transitive() {
    let doc = crate::common::load_toml(&manifest_path());
    let direct = doc
        .get("direct_dependency")
        .and_then(|v| v.as_table())
        .expect("[direct_dependency] missing");
    let transitive = doc
        .get("transitive_dependency")
        .and_then(|v| v.as_table())
        .expect("[transitive_dependency] missing");
    let edge = doc
        .get("dependency_edge")
        .and_then(|v| v.as_table())
        .expect("[dependency_edge] missing");

    let direct_name = direct.get("package_name").and_then(|v| v.as_str()).unwrap();
    let transitive_name = transitive
        .get("package_name")
        .and_then(|v| v.as_str())
        .unwrap();
    assert_ne!(
        direct_name, transitive_name,
        "direct and transitive must be distinct packages"
    );

    let from = edge.get("from_package").and_then(|v| v.as_str()).unwrap();
    let to = edge.get("to_package").and_then(|v| v.as_str()).unwrap();
    assert_eq!(from, direct_name, "edge.from must equal direct package");
    assert_eq!(to, transitive_name, "edge.to must equal transitive package");

    let constraint = edge.get("constraint").and_then(|v| v.as_str()).unwrap();
    assert!(!constraint.is_empty());
    assert_eq!(
        edge.get("must_be_single_edge").and_then(|v| v.as_bool()),
        Some(true)
    );

    for tbl in &[direct, transitive] {
        let stmt = tbl
            .get("import_statement")
            .and_then(|v| v.as_str())
            .unwrap();
        assert!(stmt.starts_with("import "));
        let name = tbl.get("package_name").and_then(|v| v.as_str()).unwrap();
        assert!(
            stmt.contains(name),
            "import statement must reference package name {name}"
        );
    }
}

// Acceptance: "Lockfile contains both direct and transitive dependency entries."
#[test]
fn lockfile_contains_direct_and_transitive_entries() {
    let doc = crate::common::load_toml(&manifest_path());
    let l = doc
        .get("lockfile_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[lockfile_contract] missing — acceptance: \
         \"Lockfile contains both direct and transitive dependency entries.\"",
        );
    assert_eq!(
        l.get("must_emit_lockfile").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("lockfile_format").and_then(|v| v.as_str()),
        Some("toml")
    );
    assert_eq!(
        l.get("must_record_direct_dependency_entry")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_record_transitive_dependency_entry")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_mark_direct_as_root_requirement")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_mark_transitive_as_dependency_of_direct")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_pin_exact_version_for_direct")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_pin_exact_version_for_transitive")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("missing_transitive_entry_failure_kind")
            .and_then(|v| v.as_str()),
        Some("transitive_dependency_missing_from_lockfile")
    );
    let miss = l
        .get("missing_transitive_entry_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(miss, 0);
    assert_eq!(miss, 26);
    let unpinned = l
        .get("unpinned_lockfile_entry_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(unpinned, 27);
    assert_ne!(miss, unpinned);
}

// Acceptance: "Runtime import proves transitive dependency activation."
#[test]
fn runtime_import_proves_transitive_activation() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc
        .get("runtime_import_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[runtime_import_contract] missing — acceptance: \
         \"Runtime import proves transitive dependency activation.\"",
        );
    assert_eq!(
        r.get("must_run_import_script_after_install")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("import_script_must_import_direct_dependency")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("import_script_must_import_transitive_dependency")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("expected_import_exit_code")
            .and_then(|v| v.as_integer()),
        Some(0)
    );
    assert_eq!(
        r.get("transitive_not_importable_failure_kind")
            .and_then(|v| v.as_str()),
        Some("transitive_dependency_not_importable")
    );
    let trans_exit = r
        .get("transitive_not_importable_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(trans_exit, 28);
    let direct_exit = r
        .get("direct_not_importable_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(direct_exit, 29);
    assert_ne!(
        trans_exit, direct_exit,
        "transitive and direct import failure exits must differ"
    );
    assert_eq!(
        r.get("must_distinguish_transitive_import_failure_from_direct_import_failure")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Resolver output names the dependency edge."
#[test]
fn resolver_output_names_dependency_edge() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc
        .get("resolver_diagnostic_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[resolver_diagnostic_contract] missing — acceptance: \
         \"Resolver output names the dependency edge.\"",
        );
    assert_eq!(
        d.get("must_emit_resolver_diagnostic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("diagnostic_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    assert_eq!(
        d.get("must_name_direct_dependency_in_diagnostic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_name_transitive_dependency_in_diagnostic")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_name_edge_from_field").and_then(|v| v.as_str()),
        Some("from")
    );
    assert_eq!(
        d.get("must_name_edge_to_field").and_then(|v| v.as_str()),
        Some("to")
    );
    assert_eq!(
        d.get("must_name_constraint_field").and_then(|v| v.as_str()),
        Some("constraint")
    );
    assert_eq!(
        d.get("edge_missing_from_diagnostic_failure_kind")
            .and_then(|v| v.as_str()),
        Some("edge_missing_from_diagnostic")
    );
    let exit = d
        .get("edge_missing_from_diagnostic_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 30);
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
        "direct_package",
        "direct_version",
        "transitive_package",
        "transitive_version",
        "edge_from",
        "edge_to",
        "edge_constraint",
        "lockfile_path",
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
        "lockfile_contains_direct_and_transitive_entries",
        "runtime_import_proves_transitive_activation",
        "resolver_output_names_dependency_edge",
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
        o.get("complex_backtracking_beyond_one_edge")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
