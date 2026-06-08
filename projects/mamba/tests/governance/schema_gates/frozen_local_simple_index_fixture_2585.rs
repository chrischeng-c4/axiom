//! Schema gate for the frozen local simple-index fixture — closes
//! #2585.
//!
//! Acceptance (issue #2585):
//!
//!   1. Default package-manager tests do not hit PyPI.
//!      `[default_offline_gate]` pins
//!      must_be_default_for_package_manager_tests +
//!      forbid_network_access_in_default_tests + forbid_pypi_origin
//!      + forbid_dns_lookups + forbid_outbound_sockets +
//!      default_index_url_must_point_to_local_fixture.
//!   2. Fixture can serve or load package metadata deterministically.
//!      `[determinism_contract]` pins load+serve modes,
//!      deterministic across runs, sha256 checksum per artifact in
//!      json record format.
//!   3. Missing fixture files fail validation.
//!      `[validation_contract]` pins file_missing failure_kind +
//!      exit_code=12 + diagnostic_must_name_path +
//!      validation_must_run_before_any_resolver_use.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("package_manager")
        .join("frozen_local_simple_index")
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
        Some("frozen_local_simple_index"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2585));
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
        Some("frozen_local_simple_index")
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
fn index_kind_is_frozen_local_simple_pep503() {
    let doc = load_toml(&manifest_path());
    let i = doc
        .get("index")
        .and_then(|v| v.as_table())
        .expect("[index] missing");
    assert_eq!(
        i.get("kind").and_then(|v| v.as_str()),
        Some("frozen_local_simple_index")
    );
    assert_eq!(i.get("layout_pep").and_then(|v| v.as_integer()), Some(503));
    assert_eq!(
        i.get("must_be_offline").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_be_deterministic").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("must_serve_from_local_disk")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(i.get("must_be_small").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn index_layout_declares_required_files() {
    let doc = load_toml(&manifest_path());
    let l = doc
        .get("index_layout")
        .and_then(|v| v.as_table())
        .expect("[index_layout] missing");
    assert_eq!(
        l.get("root_relative_path").and_then(|v| v.as_str()),
        Some("simple/")
    );
    assert_eq!(
        l.get("per_package_subdir_kind").and_then(|v| v.as_str()),
        Some("normalized_project_name")
    );
    assert_eq!(
        l.get("must_include_root_index_html")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_include_per_package_index_html_for_each_package")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        l.get("must_include_at_least_one_wheel_per_package")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn graph_declares_two_packages_and_one_dependency_edge() {
    let doc = load_toml(&manifest_path());
    let pkgs = doc
        .get("packages")
        .and_then(|v| v.as_array())
        .expect("[[packages]] missing");
    assert!(pkgs.len() >= 2, "must declare at least two packages");

    let mut names = Vec::new();
    for p in pkgs {
        let t = p.as_table().unwrap();
        let name = t.get("name").and_then(|v| v.as_str()).unwrap();
        let ver = t.get("version").and_then(|v| v.as_str()).unwrap();
        let filename = t.get("filename").and_then(|v| v.as_str()).unwrap();
        assert!(!name.is_empty());
        assert!(!ver.is_empty());
        assert!(filename.contains(name) && filename.contains(ver));
        names.push(name);
    }
    for w in names.windows(2) {
        assert_ne!(w[0], w[1], "package names must be distinct");
    }

    let g = doc
        .get("graph")
        .and_then(|v| v.as_table())
        .expect("[graph] missing");
    assert_eq!(
        g.get("must_declare_at_least_two_packages")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        g.get("must_declare_at_least_one_dependency_edge")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let edges = g
        .get("edges")
        .and_then(|v| v.as_array())
        .expect("graph.edges missing");
    assert!(
        !edges.is_empty(),
        "must declare at least one dependency edge"
    );
    for e in edges {
        let t = e.as_table().unwrap();
        let from = t.get("from").and_then(|v| v.as_str()).unwrap();
        let to = t.get("to").and_then(|v| v.as_str()).unwrap();
        assert!(
            names.contains(&from),
            "edge.from must be a declared package: {from}"
        );
        assert!(
            names.contains(&to),
            "edge.to must be a declared package: {to}"
        );
        assert_ne!(from, to, "edge cannot be a self-loop");
    }
}

// Acceptance: "Default package-manager tests do not hit PyPI."
#[test]
fn default_package_manager_tests_do_not_hit_pypi() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("default_offline_gate")
        .and_then(|v| v.as_table())
        .expect(
            "[default_offline_gate] missing — acceptance: \
         \"Default package-manager tests do not hit PyPI.\"",
        );
    for f in &[
        "must_be_default_for_package_manager_tests",
        "forbid_network_access_in_default_tests",
        "forbid_pypi_origin",
        "forbid_dns_lookups",
        "forbid_outbound_sockets",
        "default_index_url_must_point_to_local_fixture",
    ] {
        assert_eq!(
            d.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

// Acceptance: "Fixture can serve or load package metadata deterministically."
#[test]
fn fixture_serves_or_loads_metadata_deterministically() {
    let doc = load_toml(&manifest_path());
    let d = doc
        .get("determinism_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[determinism_contract] missing — acceptance: \
         \"Fixture can serve or load package metadata deterministically.\"",
        );
    assert_eq!(
        d.get("must_load_metadata_deterministically_across_runs")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("must_serve_metadata_deterministically_when_run_as_server")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("load_mode_allowed").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("serve_mode_allowed").and_then(|v| v.as_bool()),
        Some(true)
    );
    let modes: Vec<&str> = d
        .get("allowed_modes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for m in &["load", "serve"] {
        assert!(modes.contains(m), "allowed_modes must include {m}");
    }
    assert_eq!(
        d.get("checksum_must_be_recorded_for_each_artifact")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        d.get("checksum_algorithm").and_then(|v| v.as_str()),
        Some("sha256")
    );
    assert_eq!(
        d.get("checksum_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
}

// Acceptance: "Missing fixture files fail validation."
#[test]
fn missing_fixture_files_fail_validation() {
    let doc = load_toml(&manifest_path());
    let v = doc
        .get("validation_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[validation_contract] missing — acceptance: \
         \"Missing fixture files fail validation.\"",
        );
    assert_eq!(
        v.get("must_validate_all_declared_files_exist")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        v.get("must_validate_per_package_index_html_exists")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        v.get("must_validate_root_index_html_exists")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        v.get("missing_file_failure_kind").and_then(|v| v.as_str()),
        Some("fixture_file_missing")
    );
    let exit = v
        .get("missing_file_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 12);
    assert_eq!(
        v.get("missing_file_diagnostic_must_name_path")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        v.get("validation_must_run_before_any_resolver_use")
            .and_then(|v| v.as_bool()),
        Some(true)
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
        "package",
        "version",
        "filename",
        "checksum",
        "missing_file_path",
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
        "default_tests_resolve_against_local_index",
        "metadata_loads_deterministically",
        "missing_fixture_file_fails_validation",
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
        o.get("live_pypi_integration").and_then(|v| v.as_bool()),
        Some(true)
    );
}
