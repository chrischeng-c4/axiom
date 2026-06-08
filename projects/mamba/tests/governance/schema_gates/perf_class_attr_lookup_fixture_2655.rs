//! Schema gate for the class attribute lookup benchmark fixture —
//! closes #2655.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("performance")
        .join("class_attr_lookup")
        .join("manifest.toml")
}

const BENCH_ID: &str = "class_attr_lookup";
const ISSUE: i64 = 2655;
const FIXTURE: &str = "perf_class_attr_lookup";
const OOS_FLAG: &str = "object_layout_optimization";

#[test]
fn header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());
    assert_eq!(doc.get("fixture").and_then(|v| v.as_str()), Some(FIXTURE));
    assert_eq!(doc.get("issue").and_then(|v| v.as_integer()), Some(ISSUE));
    assert_eq!(doc.get("benchmark_id").and_then(|v| v.as_str()), Some(BENCH_ID));
    assert_eq!(doc.get("profile").and_then(|v| v.as_str()), Some("performance"));
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
fn workload_uses_single_small_class() {
    let doc = crate::common::load_toml(&manifest_path());
    let w = doc.get("workload").and_then(|v| v.as_table()).unwrap();
    assert!(w.get("class_attribute_count").and_then(|v| v.as_integer()).unwrap() > 0);
    assert!(w.get("n_iterations").and_then(|v| v.as_integer()).unwrap() > 0);
    assert_eq!(w.get("must_use_single_instance").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(w.get("deterministic").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Wrong lookup result fails the benchmark."
#[test]
fn wrong_lookup_result_fails_benchmark() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("checksum_contract").and_then(|v| v.as_table()).expect(
        "[checksum_contract] missing — acceptance: \
         \"Wrong lookup result fails the benchmark.\"",
    );
    assert_eq!(c.get("must_validate_before_accepting_timing").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("on_mismatch").and_then(|v| v.as_str()), Some("fail"));
    assert_eq!(c.get("on_mismatch_exit_code").and_then(|v| v.as_integer()), Some(1));
    assert_eq!(c.get("on_mismatch_must_block_speedup_record").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(c.get("diagnostic_must_name_benchmark").and_then(|v| v.as_str()), Some(BENCH_ID));
}

// Acceptance: "Benchmark has tier metadata and internal timing marker."
#[test]
fn timing_contract_has_tier_and_internal_marker() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc.get("timing_contract").and_then(|v| v.as_table()).expect(
        "[timing_contract] missing — acceptance: \
         \"Benchmark has tier metadata and internal timing marker.\"",
    );
    assert!(t.get("tier").and_then(|v| v.as_str()).is_some(), "tier must be set");
    assert_eq!(t.get("must_emit_internal_timing_marker").and_then(|v| v.as_bool()), Some(true));
    assert!(
        t.get("internal_timing_marker_field_name").and_then(|v| v.as_str()).is_some(),
        "internal_timing_marker_field_name must be declared",
    );
    let mode = t.get("timing_mode").and_then(|v| v.as_str()).unwrap();
    let allowed: Vec<&str> = t.get("allowed_timing_modes").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    assert!(allowed.contains(&mode));
}

// Acceptance: "Summary reports speedup separately from other runtime
// benchmarks."
#[test]
fn summary_reports_speedup_separately() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc.get("performance_summary_contract").and_then(|v| v.as_table()).expect(
        "[performance_summary_contract] missing — acceptance: \
         \"Summary reports speedup separately from other runtime benchmarks.\"",
    );
    assert_eq!(s.get("must_appear_in_machine_readable_summary").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("must_be_keyed_by_benchmark_id").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(s.get("must_not_pool_with_other_benchmarks").and_then(|v| v.as_bool()), Some(true));
    let req: Vec<&str> = s.get("required_record_keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &["benchmark_id", "tier", "timing_mode", "internal_elapsed_seconds", "speedup", "outcome"] {
        assert!(req.contains(k), "required_record_keys must include {k}");
    }
}

#[test]
fn runner_contract_declares_keys_and_outcomes() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc.get("runner_contract").and_then(|v| v.as_table()).unwrap();
    let keys: Vec<&str> = c.get("keys").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for k in &[
        "benchmark_id", "tier", "timing_mode", "checksum",
        "expected_checksum", "internal_elapsed_seconds", "speedup", "outcome", "exit_code",
    ] { assert!(keys.contains(k), "runner_contract.keys must include {k}"); }
    let outcomes: Vec<&str> = c.get("outcome_values").and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect()).unwrap_or_default();
    for o in &["pass", "fail"] { assert!(outcomes.contains(o)); }
}

#[test]
fn pins_out_of_scope_per_issue() {
    let doc = crate::common::load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get(OOS_FLAG).and_then(|v| v.as_bool()), Some(true));
}
