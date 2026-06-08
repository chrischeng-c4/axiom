//! Schema gate for the skip-debt counts in smoke summary fixture —
//! closes #2600.
//!
//! Acceptance (issue #2600):
//!
//!   1. Smoke summary includes ignore, xfail, and skip totals.
//!      `[totals_in_summary_contract]` pins must_emit_ignore_total +
//!      must_emit_xfail_total + must_emit_skip_total +
//!      must_emit_combined_skip_debt_total + json record +
//!      totals_required_fields + missing-total exit_code=45.
//!   2. Counts link back to the inventory command or output file.
//!      `[inventory_link_contract]` pins must_link_back_to_inventory
//!      + inventory_link_field_name + allowed_inventory_link_kinds
//!      [command, output_file] + missing-link exit_code=46 +
//!      unknown-kind exit_code=47.
//!   3. A test covers the summary fields.
//!      `[summary_fields_coverage_contract]` pins must_have_test_
//!      covering_summary_fields + must_assert_every_required_field_
//!      present + must_keep_summary_compact + summary_max_bytes +
//!      oversize-summary exit_code=48.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("skip_debt")
        .join("skip_debt_counts_in_smoke_summary")
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
        Some("skip_debt_counts_in_smoke_summary")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2600));
    assert_eq!(
        doc.get("parent_issue").and_then(|v| v.as_integer()),
        Some(2533)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("skip_debt")
    );
    assert_eq!(
        doc.get("family").and_then(|v| v.as_str()),
        Some("skip_debt_counts_in_smoke_summary")
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
fn smoke_gate_and_inventory_cross_references_are_pinned() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("smoke_gate_cross_reference")
        .and_then(|v| v.as_table())
        .expect("[smoke_gate_cross_reference] missing");
    assert_eq!(
        s.get("fixture_issue").and_then(|v| v.as_integer()),
        Some(2527),
        "must extend the smoke gate fixture #2527"
    );
    assert_eq!(
        s.get("must_extend_existing_smoke_summary")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_not_break_existing_summary_consumers")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let inv = doc
        .get("inventory_cross_reference")
        .and_then(|v| v.as_table())
        .expect("[inventory_cross_reference] missing");
    assert_eq!(
        inv.get("categorization_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2598)
    );
    assert_eq!(
        inv.get("mvp_required_guard_fixture_issue")
            .and_then(|v| v.as_integer()),
        Some(2599)
    );
    let kinds: Vec<&str> = inv
        .get("shared_marker_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["ignore", "xfail", "skip"] {
        assert!(kinds.contains(k), "shared_marker_kinds must include {k}");
    }
}

// Acceptance: "Smoke summary includes ignore, xfail, and skip totals."
#[test]
fn smoke_summary_includes_ignore_xfail_skip_totals() {
    let doc = load_toml(&manifest_path());
    let t = doc
        .get("totals_in_summary_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[totals_in_summary_contract] missing — acceptance: \
         \"Smoke summary includes ignore, xfail, and skip totals.\"",
        );
    for f in &[
        "must_emit_ignore_total",
        "must_emit_xfail_total",
        "must_emit_skip_total",
        "must_emit_combined_skip_debt_total",
        "must_emit_counts_by_mvp_objective_when_available",
        "nonzero_count_must_not_be_omitted",
    ] {
        assert_eq!(
            t.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        t.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let required: Vec<&str> = t
        .get("totals_required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &[
        "ignore_total",
        "xfail_total",
        "skip_total",
        "skip_debt_total",
        "counts_by_mvp_objective",
    ] {
        assert!(
            required.contains(f),
            "totals_required_fields must include {f}"
        );
    }
    let exit = t
        .get("missing_total_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 45);
}

// Acceptance: "Counts link back to the inventory command or output file."
#[test]
fn counts_link_back_to_inventory() {
    let doc = load_toml(&manifest_path());
    let i = doc
        .get("inventory_link_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[inventory_link_contract] missing — acceptance: \
         \"Counts link back to the inventory command or output file.\"",
        );
    assert_eq!(
        i.get("must_link_back_to_inventory")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("inventory_link_field_name").and_then(|v| v.as_str()),
        Some("inventory_link")
    );
    let kinds: Vec<&str> = i
        .get("allowed_inventory_link_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["command", "output_file"] {
        assert!(
            kinds.contains(k),
            "allowed_inventory_link_kinds must include {k}"
        );
    }
    assert_eq!(
        i.get("must_state_inventory_link_kind")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        i.get("link_field_name_for_kind").and_then(|v| v.as_str()),
        Some("inventory_link_kind")
    );
    let missing = i
        .get("missing_inventory_link_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(missing, 46);
    let unknown = i
        .get("unknown_inventory_link_kind_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(unknown, 47);
    assert_ne!(missing, unknown);
}

// Acceptance: "A test covers the summary fields."
#[test]
fn summary_fields_are_tested() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("summary_fields_coverage_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[summary_fields_coverage_contract] missing — acceptance: \
         \"A test covers the summary fields.\"",
        );
    assert_eq!(
        s.get("must_have_test_covering_summary_fields")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_assert_every_required_field_present")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_assert_field_types_are_integers_for_counts")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_keep_summary_compact").and_then(|v| v.as_bool()),
        Some(true)
    );
    let budget = s
        .get("summary_max_bytes")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert!(budget > 0);
    assert!(
        budget <= 65536,
        "summary_max_bytes must be a compact budget (≤ 64KiB), got {budget}"
    );
    let exit = s
        .get("oversize_summary_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 48);
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
        "ignore_total",
        "xfail_total",
        "skip_total",
        "skip_debt_total",
        "counts_by_mvp_objective",
        "inventory_link",
        "inventory_link_kind",
        "summary_size_bytes",
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
        "smoke_summary_includes_ignore_xfail_skip_totals",
        "counts_link_back_to_inventory",
        "summary_fields_are_tested",
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
        o.get("changing_individual_skip_behavior")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
