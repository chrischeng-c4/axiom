//! Schema gate for the async scheduling benchmark fixture — closes
//! #2664.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("performance")
        .join("async_scheduling")
        .join("manifest.toml")
}

const BENCH_ID: &str = "async_scheduling";
const ISSUE: i64 = 2664;
const FIXTURE: &str = "perf_async_scheduling";
const OOS_FLAG: &str = "async_runtime_optimization";

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some(FIXTURE));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(ISSUE));
    assert_eq!(
        doc.get("benchmark_id").and_then(|v| v.as_str()),
        Some(BENCH_ID)
    );
    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("performance")
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
fn workload_avoids_real_time_sleeps_and_awaits_many_tiny_coroutines() {
    let doc = crate::common::load_toml(&manifest_path());
    let w = doc.get("workload").and_then(|v| v.as_table()).unwrap();
    assert!(w.get("n_coroutines").and_then(|v| v.as_integer()).unwrap() > 0);
    assert_eq!(
        w.get("must_use_event_loop").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        w.get("forbid_asyncio_sleep_with_nonzero_delay")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        w.get("forbid_time_sleep").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        w.get("forbid_real_time_dependency")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(w.get("deterministic").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Fixture does not depend on real-time sleep duration."
#[test]
fn timing_contract_does_not_depend_on_real_time_sleep() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc
        .get("timing_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[timing_contract] missing — acceptance: \
         \"Fixture does not depend on real-time sleep duration.\"",
        );
    assert!(
        t.get("tier").and_then(|v| v.as_str()).is_some(),
        "tier must be set"
    );
    let mode = t.get("timing_mode").and_then(|v| v.as_str()).unwrap();
    let allowed: Vec<&str> = t
        .get("allowed_timing_modes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(allowed.contains(&mode));
    assert_eq!(
        t.get("must_not_depend_on_real_time_sleep_duration")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        t.get("forbid_wall_clock_sleep_in_workload")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Unsupported async behavior is visible and linked to a
// blocker."
#[test]
fn status_contract_links_unsupported_to_blocker() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("status_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[status_contract] missing — acceptance: \
         \"Unsupported async behavior is visible and linked to a blocker.\"",
        );
    let states: Vec<&str> = s
        .get("allowed_status_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for st in &["pass", "xfail", "blocker"] {
        assert!(
            states.contains(st),
            "allowed_status_values must include {st}"
        );
    }
    assert_eq!(
        s.get("must_link_blocker_to_issue_when_status_is_blocker")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("unsupported_async_behavior_must_be_visible")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_appear_in_summary").and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Passing path reports speedup against CPython 3.12."
#[test]
fn pass_path_reports_speedup_against_cpython_3_12() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("performance_summary_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[performance_summary_contract] missing — acceptance: \
         \"Passing path reports speedup against CPython 3.12.\"",
        );
    assert_eq!(
        s.get("must_appear_in_machine_readable_summary")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("baseline_label").and_then(|v| v.as_str()),
        Some("cpython")
    );
    assert_eq!(
        s.get("baseline_version").and_then(|v| v.as_str()),
        Some("3.12")
    );
    assert_eq!(
        s.get("must_pin_baseline_version").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_only_record_speedup_on_pass")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let req: Vec<&str> = s
        .get("required_record_keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &[
        "benchmark_id",
        "status",
        "baseline_label",
        "baseline_version",
        "speedup",
        "outcome",
    ] {
        assert!(req.contains(k), "required_record_keys must include {k}");
    }
}

#[test]
fn runner_contract_declares_keys_and_outcomes() {
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
    for k in &[
        "benchmark_id",
        "status",
        "tier",
        "timing_mode",
        "n_coroutines",
        "result_checksum",
        "expected_result_checksum",
        "completion_count",
        "expected_completion_count",
        "baseline_label",
        "baseline_version",
        "speedup",
        "outcome",
        "blocker_issue",
        "exit_code",
    ] {
        assert!(keys.contains(k), "runner_contract.keys must include {k}");
    }
    let outcomes: Vec<&str> = c
        .get("outcome_values")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for o in &["pass", "fail"] {
        assert!(outcomes.contains(o));
    }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get(OOS_FLAG).and_then(|v| v.as_bool()), Some(true));
}
