//! Schema gate for the offline init-add-install-sync-run E2E
//! skeleton fixture — closes #2588.
//!
//! Acceptance (issue #2588):
//!
//!   1. Test names every workflow step and current status.
//!      `[step_naming_contract]` pins must_name_every_workflow_step
//!      + must_record_status_per_step + allowed_status_values =
//!      [supported, blocked, skipped] +
//!      forbid_silent_skip_for_blocked_step +
//!      must_print_step_name_and_status_in_order.
//!   2. Supported steps execute in order in a temp project.
//!      `[step_execution_contract]` pins
//!      must_execute_supported_steps_in_declared_order +
//!      must_fail_if_order_violated + per-step exit/output/diff
//!      capture + supported_step_failure_kind +
//!      exit_code=17.
//!   3. Unsupported steps are linked to follow-up issues or
//!      blockers. `[blocker_contract]` pins
//!      must_link_each_blocked_step_to_followup_issue +
//!      must_emit_blocker_diagnostic +
//!      forbid_silent_skip_of_blocked_step + blocker_step_failure_
//!      kind + exit_code=18 + must_distinguish_blocked_from_
//!      supported_failures.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("package_manager")
        .join("init_add_install_sync_run_e2e")
        .join("manifest.toml")
}

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(
        doc.get("fixture").and_then(|v| v.as_str()),
        Some("init_add_install_sync_run_e2e"),
    );
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(2588));
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
        Some("init_add_install_sync_run_e2e")
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
fn temp_project_is_per_test_and_isolated() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc
        .get("temp_project")
        .and_then(|v| v.as_table())
        .expect("[temp_project] missing");
    for f in &[
        "must_use_per_test_temp_dir",
        "forbid_writes_outside_temp_dir",
        "must_clean_up_on_success",
        "must_preserve_on_failure_for_diagnostics",
        "must_capture_command_output_per_step",
        "must_capture_project_file_changes_per_step",
    ] {
        assert_eq!(
            t.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
}

#[test]
fn steps_declare_canonical_uv_like_workflow_in_order() {
    let doc = crate::common::load_toml(&manifest_path());
    let steps = doc
        .get("steps")
        .and_then(|v| v.as_array())
        .expect("[[steps]] missing");
    assert_eq!(
        steps.len(),
        5,
        "must declare exactly 5 canonical workflow steps"
    );

    let mut prev_order = 0i64;
    let mut blocked_count = 0;
    let mut names = Vec::new();
    let allowed_statuses = ["supported", "blocked", "skipped"];

    for s in steps {
        let t = s.as_table().unwrap();
        let order = t.get("order").and_then(|v| v.as_integer()).unwrap();
        assert!(
            order > prev_order,
            "steps must be declared in strictly increasing order"
        );
        prev_order = order;
        let name = t.get("name").and_then(|v| v.as_str()).unwrap();
        let status = t.get("status").and_then(|v| v.as_str()).unwrap();
        assert!(
            allowed_statuses.contains(&status),
            "status {status} not allowed"
        );
        if status == "blocked" {
            blocked_count += 1;
            let blocker = t.get("blocker_issue").and_then(|v| v.as_integer());
            assert!(
                blocker.is_some(),
                "blocked step {name} must declare blocker_issue"
            );
            let reason = t.get("blocker_reason").and_then(|v| v.as_str());
            assert!(
                reason.is_some(),
                "blocked step {name} must declare blocker_reason"
            );
        }
        names.push(name);
    }

    for required in &["init", "add", "install", "sync", "run"] {
        assert!(
            names.contains(required),
            "steps must include canonical step {required}"
        );
    }
    assert!(
        blocked_count >= 1,
        "fixture must declare at least one blocked step with follow-up issue"
    );
}

// Acceptance: "Test names every workflow step and current status."
#[test]
fn test_names_every_workflow_step_and_status() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("step_naming_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[step_naming_contract] missing — acceptance: \
         \"Test names every workflow step and current status.\"",
        );
    assert_eq!(
        s.get("must_name_every_workflow_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_record_status_per_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let allowed: Vec<&str> = s
        .get("allowed_status_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for v in &["supported", "blocked", "skipped"] {
        assert!(
            allowed.contains(v),
            "allowed_status_values must include {v}"
        );
    }
    assert_eq!(
        s.get("forbid_silent_skip_for_blocked_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("forbid_silent_skip_for_unknown_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_print_step_name_and_status_in_order")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Supported steps execute in order in a temp project."
#[test]
fn supported_steps_execute_in_order_in_temp_project() {
    let doc = crate::common::load_toml(&manifest_path());
    let e = doc
        .get("step_execution_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[step_execution_contract] missing — acceptance: \
         \"Supported steps execute in order in a temp project.\"",
        );
    for f in &[
        "must_execute_supported_steps_in_declared_order",
        "must_fail_if_order_violated",
        "must_record_per_step_exit_status",
        "must_record_per_step_captured_output",
        "must_record_per_step_project_file_diff",
    ] {
        assert_eq!(
            e.get(*f).and_then(|v| v.as_bool()),
            Some(true),
            "{f} must be true"
        );
    }
    assert_eq!(
        e.get("expected_outcome_when_all_supported_steps_pass")
            .and_then(|v| v.as_str()),
        Some("pass")
    );
    assert_eq!(
        e.get("expected_outcome_when_any_supported_step_fails")
            .and_then(|v| v.as_str()),
        Some("fail")
    );
    assert_eq!(
        e.get("supported_step_failure_kind")
            .and_then(|v| v.as_str()),
        Some("supported_step_failed")
    );
    let exit = e
        .get("supported_step_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 17);
}

// Acceptance: "Unsupported steps are linked to follow-up issues or blockers."
#[test]
fn unsupported_steps_link_to_followup_issues_or_blockers() {
    let doc = crate::common::load_toml(&manifest_path());
    let b = doc
        .get("blocker_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[blocker_contract] missing — acceptance: \
         \"Unsupported steps are linked to follow-up issues or blockers.\"",
        );
    assert_eq!(
        b.get("must_link_each_blocked_step_to_followup_issue")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("must_emit_blocker_diagnostic_for_each_blocked_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("forbid_silent_skip_of_blocked_step")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        b.get("blocker_step_failure_kind").and_then(|v| v.as_str()),
        Some("blocked_step_without_followup")
    );
    let exit = b
        .get("blocker_step_failure_exit_code")
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(exit, 0);
    assert_eq!(exit, 18);
    assert_eq!(
        b.get("must_distinguish_blocked_from_supported_failures")
            .and_then(|v| v.as_bool()),
        Some(true)
    );

    let supported_exit = doc
        .get("step_execution_contract")
        .and_then(|v| v.get("supported_step_failure_exit_code"))
        .and_then(|v| v.as_integer())
        .unwrap();
    assert_ne!(
        exit, supported_exit,
        "blocker exit code must differ from supported-step-failure exit code"
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
        "step_order",
        "step_name",
        "step_status",
        "blocker_issue",
        "captured_output",
        "project_file_diff",
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
        "workflow_steps_named_and_statuses_recorded",
        "supported_steps_execute_in_declared_order",
        "blocked_steps_link_to_followup_issues",
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
        o.get("making_every_step_fully_implemented_in_this_issue")
            .and_then(|v| v.as_bool()),
        Some(true),
    );
}
