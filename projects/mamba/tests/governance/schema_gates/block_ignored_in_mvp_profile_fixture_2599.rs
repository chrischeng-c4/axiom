//! Schema gate for the block-ignored-in-MVP-profile fixture — closes
//! #2599.
//!
//! Acceptance (issue #2599):
//!
//!   1. A required fixture marked xfail fails the MVP profile
//!      validation. `[required_xfail_blocked_contract]` pins
//!      required_id_must_have_no_ignore_marker +
//!      must_validate_required_fixture_ids + must_block_on_any_
//!      ignored_marker_kind + exit_code=42.
//!   2. A non-required exploratory fixture may remain xfail with
//!      reason. `[exploratory_xfail_allowed_contract]` pins
//!      exploratory_id_must_state_reason + reason_field_name =
//!      "ignore_reason" + must_not_block_exploratory_with_nonempty_
//!      reason + empty-reason exit_code=43.
//!   3. Failure output is actionable for the worker.
//!      `[actionable_failure_output_contract]` pins json record +
//!      failure_required_fields including parent_objective and
//!      remediation_hint + missing-parent-objective exit_code=44.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("skip_debt")
        .join("block_ignored_in_mvp_profile")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("block_ignored_in_mvp_profile")
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2599));
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
        Some("block_ignored_in_mvp_profile")
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
fn categorization_cross_references_fixture_2598() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("categorization_cross_reference")
        .and_then(|v| v.as_table())
        .expect("[categorization_cross_reference] missing");
    assert_eq!(
        c.get("fixture_issue").and_then(|v| v.as_integer()),
        Some(2598),
        "must cross-reference categorization fixture #2598",
    );
    assert_eq!(
        c.get("must_share_category_allowed_values")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let values: Vec<&str> = c
        .get("inherited_category_allowed_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["opt_in", "blocker", "flaky", "obsolete"] {
        assert!(
            values.contains(v),
            "inherited_category_allowed_values must include {v}"
        );
    }
}

#[test]
fn ignore_marker_kinds_block_lists_ignore_xfail_skip() {
    let doc = crate::common::load_toml(&manifest_path());
    let i = doc
        .get("ignore_marker_kinds")
        .and_then(|v| v.as_table())
        .expect("[ignore_marker_kinds] missing");
    assert_eq!(
        i.get("must_treat_ignore_as_skip").and_then(|v| v.as_bool()),
        Some(true)
    );
    let kinds: Vec<&str> = i
        .get("ignored_marker_kinds")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["ignore", "xfail", "skip"] {
        assert!(kinds.contains(k), "ignored_marker_kinds must include {k}");
    }
    assert_eq!(
        i.get("must_apply_to_required_regardless_of_category")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "A required fixture marked xfail fails the MVP profile validation."
#[test]
fn required_fixture_marked_xfail_fails_mvp_profile() {
    let doc = crate::common::load_toml(&manifest_path());
    let r = doc
        .get("required_xfail_blocked_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[required_xfail_blocked_contract] missing — acceptance: \
         \"A required fixture marked xfail fails the MVP profile validation.\"",
        );
    assert_eq!(
        r.get("required_id_must_have_no_ignore_marker")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("required_id_with_ignore_marker_failure_kind")
            .and_then(|v| v.as_str()),
        Some("mvp_required_item_carries_ignore_marker")
    );
    let exit = r
        .get("required_id_with_ignore_marker_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 42);
    assert_eq!(
        r.get("must_validate_required_fixture_ids")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("must_validate_required_test_ids")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        r.get("must_block_on_any_ignored_marker_kind")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "A non-required exploratory fixture may remain xfail with reason."
#[test]
fn exploratory_fixture_may_remain_xfail_with_reason() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc
        .get("exploratory_xfail_allowed_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[exploratory_xfail_allowed_contract] missing — acceptance: \
         \"A non-required exploratory fixture may remain xfail with reason.\"",
        );
    assert_eq!(
        e.get("exploratory_id_may_remain_ignored")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("exploratory_id_must_state_reason")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let field = e.get("reason_field_name").and_then(|v| v.as_str()).unwrap();
    assert_eq!(field, "ignore_reason");
    let exit = e
        .get("empty_reason_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 43);
    assert_eq!(
        e.get("must_not_block_exploratory_with_nonempty_reason")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        e.get("must_distinguish_exploratory_from_required_validation_path")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let req_exit = doc
        .get("required_xfail_blocked_contract")
        .and_then(|v| v.get("required_id_with_ignore_marker_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(
        exit, req_exit,
        "exploratory and required exit codes must differ"
    );
}

// Acceptance: "Failure output is actionable for the worker."
#[test]
fn failure_output_is_actionable_for_worker() {
    let doc = crate::common::load_toml(&manifest_path());
    let a = doc
        .get("actionable_failure_output_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[actionable_failure_output_contract] missing — acceptance: \
         \"Failure output is actionable for the worker.\"",
        );
    assert_eq!(
        a.get("must_emit_failure_record").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        a.get("failure_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    let required: Vec<&str> = a
        .get("failure_required_fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for f in &[
        "required_id",
        "marker_kind",
        "category",
        "ignore_reason",
        "parent_objective",
        "remediation_hint",
        "failure_kind",
        "exit_code",
    ] {
        assert!(
            required.contains(f),
            "failure_required_fields must include {f}"
        );
    }
    assert_eq!(
        a.get("must_name_parent_objective_per_violation")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        a.get("must_emit_remediation_hint")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let exit = a
        .get("missing_parent_objective_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_eq!(exit, 44);
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
        "required_id",
        "marker_kind",
        "category",
        "ignore_reason",
        "parent_objective",
        "remediation_hint",
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
        "required_fixture_marked_xfail_fails_mvp_profile",
        "exploratory_fixture_may_remain_xfail_with_reason",
        "failure_output_is_actionable_for_worker",
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
        o.get("defining_every_required_fixture")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}
