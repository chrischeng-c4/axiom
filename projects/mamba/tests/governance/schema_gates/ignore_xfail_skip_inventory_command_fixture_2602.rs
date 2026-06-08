//! Schema gate for the ignore/xfail/skip inventory command fixture —
//! closes #2602.
//!
//! Acceptance (issue #2602):
//!
//!   1. Inventory command reports current debt counts.
//!      `[debt_counts_contract]` pins must_emit_counts_grouped_by_
//!      path + must_emit_counts_grouped_by_category + json record +
//!      counts_required_fields + missing-counts exit_code=52.
//!   2. Command exits nonzero on parse errors or unknown marker
//!      formats. `[parse_error_contract]` pins parse_error_must_exit_
//!      nonzero + unknown_marker_format_must_exit_nonzero +
//!      distinct exit_code=53 (parse error) and 54 (unknown marker
//!      format).
//!   3. Summary can be linked from the MVP smoke report.
//!      `[smoke_link_contract]` pins must_emit_link_artifact + json
//!      + link_artifact_required_fields + schema_version + checksum
//!      + missing-link exit_code=55.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("skip_debt")
        .join("ignore_xfail_skip_inventory_command")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some("ignore_xfail_skip_inventory_command"));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2602));
    assert_eq!(doc.get("parent_issue").and_then(|v| v.as_integer()), Some(2533));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("skip_debt"));
    assert_eq!(doc.get("family").and_then(|v| v.as_str()), Some("ignore_xfail_skip_inventory_command"));
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
fn invocation_pins_single_canonical_entry_point() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc.get("invocation").and_then(|v| v.as_table()).expect("[invocation] missing");
    assert_eq!(i.get("must_have_single_canonical_entry_point").and_then(|v| v.as_bool()), Some(true));
    let tmpl = i.get("canonical_command_template").and_then(|v| v.as_str()).unwrap();
    assert!(tmpl.starts_with("mamba "));
    assert!(tmpl.contains("inventory"));
    assert!(tmpl.contains("--debt"));
    assert_eq!(i.get("must_be_runnable_offline").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_be_deterministic").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(i.get("must_not_require_cargo_build").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn scan_scope_pins_three_marker_kinds_within_mamba_tests() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("scan_scope").and_then(|v| v.as_table()).expect("[scan_scope] missing");
    for f in &[
        "must_scan_rust_test_ignores",
        "must_scan_fixture_xfail_markers",
        "must_scan_runtime_skip_annotations",
        "must_scan_only_within_projects_mamba_tests",
    ] {
        assert_eq!(s.get(*f).and_then(|v| v.as_bool()), Some(true), "{f} must be true");
    }
    let kinds: Vec<&str> = s.get("scanned_marker_kinds").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &["rust_ignore_attribute", "fixture_xfail_marker", "runtime_skip_annotation"] {
        assert!(kinds.contains(k), "scanned_marker_kinds must include {k}");
    }
    assert_eq!(s.get("scan_root_relative_path").and_then(|v| v.as_str()), Some("projects/mamba/tests"));

    let x = doc.get("summary_cross_reference").and_then(|v| v.as_table())
        .expect("[summary_cross_reference] missing");
    assert_eq!(x.get("smoke_summary_fixture_issue").and_then(|v| v.as_integer()), Some(2600));
    assert_eq!(x.get("must_be_consumable_by_smoke_summary").and_then(|v| v.as_bool()), Some(true));
    let shared: Vec<&str> = x.get("shared_marker_kinds").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &["ignore", "xfail", "skip"] {
        assert!(shared.contains(k), "shared_marker_kinds must include {k}");
    }
}

// Acceptance: "Inventory command reports current debt counts."
#[test]
fn inventory_reports_debt_counts() {
    let doc = crate::common::load_toml(&manifest_path());
    let d = doc.get("debt_counts_contract").and_then(|v| v.as_table()).expect(
        "[debt_counts_contract] missing — acceptance: \
         \"Inventory command reports current debt counts.\"",
    );
    assert_eq!(d.get("must_emit_counts_grouped_by_path").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("must_emit_counts_grouped_by_category").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(d.get("counts_record_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = d.get("counts_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &["path", "category", "marker_kind", "count", "total"] {
        assert!(required.contains(f), "counts_required_fields must include {f}");
    }
    assert_eq!(d.get("must_emit_counts_for_every_scanned_marker_kind").and_then(|v| v.as_bool()), Some(true));
    let exit = d.get("missing_counts_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 52);
}

// Acceptance: "Command exits nonzero on parse errors or unknown marker formats."
#[test]
fn command_exits_nonzero_on_parse_or_unknown_marker() {
    let doc = crate::common::load_toml(&manifest_path());
    let p = doc.get("parse_error_contract").and_then(|v| v.as_table()).expect(
        "[parse_error_contract] missing — acceptance: \
         \"Command exits nonzero on parse errors or unknown marker formats.\"",
    );
    assert_eq!(p.get("parse_error_must_exit_nonzero").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(p.get("unknown_marker_format_must_exit_nonzero").and_then(|v| v.as_bool()), Some(true));
    let parse = p.get("parse_error_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(parse, 0);
    assert_eq!(parse, 53);
    let unknown = p.get("unknown_marker_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_ne!(unknown, 0);
    assert_eq!(unknown, 54);
    assert_ne!(parse, unknown, "parse-error and unknown-marker exits must differ");
    assert_eq!(p.get("must_distinguish_parse_error_from_unknown_marker").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(p.get("must_emit_exact_file_and_line_for_parse_error").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Summary can be linked from the MVP smoke report."
#[test]
fn summary_linkable_from_smoke_report() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("smoke_link_contract").and_then(|v| v.as_table()).expect(
        "[smoke_link_contract] missing — acceptance: \
         \"Summary can be linked from the MVP smoke report.\"",
    );
    assert_eq!(s.get("must_emit_link_artifact").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("link_artifact_format").and_then(|v| v.as_str()), Some("json"));
    let required: Vec<&str> = s.get("link_artifact_required_fields").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for f in &[
        "command", "command_template",
        "output_file_path", "schema_version", "checksum",
    ] {
        assert!(required.contains(f), "link_artifact_required_fields must include {f}");
    }
    let path = s.get("link_artifact_relative_path_within_repo").and_then(|v| v.as_str()).unwrap();
    assert!(path.starts_with("projects/mamba/tests/"));
    assert!(path.ends_with(".json"));
    assert_eq!(s.get("must_be_stable_across_runs_on_identical_input").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("must_emit_schema_version").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("schema_version_field_name").and_then(|v| v.as_str()), Some("schema_version"));
    assert_eq!(s.get("checksum_field_name").and_then(|v| v.as_str()), Some("checksum"));
    let exit = s.get("missing_link_artifact_exit_code").and_then(|v| v.as_integer()).unwrap();
    assert_eq!(exit, 55);
}

#[test]
fn runner_contract_declares_keys_and_cases() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "outcome", "case", "command",
        "path", "category", "marker_kind", "count", "total",
        "schema_version", "checksum",
        "failure_kind", "exit_code",
    ] {
        assert!(keys.contains(required), "runner_contract.keys must include {required}");
    }
    let cases: Vec<&str> = c.get("case_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for required in &[
        "inventory_reports_debt_counts",
        "command_exits_nonzero_on_parse_or_unknown_marker",
        "summary_linkable_from_smoke_report",
    ] {
        assert!(cases.contains(required), "runner_contract.case_values must include {required}");
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get("removing_existing_debt").and_then(|v| v.as_bool()), Some(true));
}
