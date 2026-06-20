//! Schema gate for the regex findall benchmark fixture — closes #2663.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("governance")
        .join("gates")
        .join("performance")
        .join("regex_findall")
        .join("manifest.toml")
}

const BENCH_ID: &str = "regex_findall";
const ISSUE: i64 = 2663;
const FIXTURE: &str = "perf_regex_findall";
const OOS_FLAG: &str = "regex_engine_optimization";

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
fn workload_exercises_single_and_multi_group_patterns() {
    let doc = crate::common::load_toml(&manifest_path());
    let w = doc.get("workload").and_then(|v| v.as_table()).unwrap();
    assert_eq!(
        w.get("must_exercise_single_group")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        w.get("must_exercise_multi_group").and_then(|v| v.as_bool()),
        Some(true)
    );
    let pats = w.get("patterns").and_then(|v| v.as_array()).unwrap();
    assert!(
        pats.len() >= 2,
        "must declare at least one single-group and one multi-group pattern"
    );
    let groups: Vec<i64> = pats
        .iter()
        .filter_map(|p| {
            p.as_table()
                .and_then(|t| t.get("group_count"))
                .and_then(|v| v.as_integer())
        })
        .collect();
    assert!(
        groups.iter().any(|&g| g == 1),
        "must include a single-group pattern"
    );
    assert!(
        groups.iter().any(|&g| g >= 2),
        "must include a multi-group pattern"
    );
    assert_eq!(w.get("deterministic").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Wrong match result fails before speedup is accepted."
#[test]
fn wrong_match_fails_before_speedup_accepted() {
    let doc = crate::common::load_toml(&manifest_path());
    let c = doc
        .get("checksum_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[checksum_contract] missing — acceptance: \
         \"Wrong match result fails before speedup is accepted.\"",
        );
    assert_eq!(
        c.get("must_validate_before_accepting_timing")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_validate_match_count").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_validate_match_checksum")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(c.get("on_mismatch").and_then(|v| v.as_str()), Some("fail"));
    assert_eq!(
        c.get("on_mismatch_exit_code").and_then(|v| v.as_integer()),
        Some(1)
    );
    assert_eq!(
        c.get("on_mismatch_must_block_speedup_record")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("on_mismatch_must_fail_before_speedup_accepted")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("diagnostic_must_name_benchmark")
            .and_then(|v| v.as_str()),
        Some(BENCH_ID)
    );
}

// Acceptance: "Fixture declares tier and timing mode."
#[test]
fn timing_contract_declares_tier_and_mode() {
    let doc = crate::common::load_toml(&manifest_path());
    let t = doc
        .get("timing_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[timing_contract] missing — acceptance: \
         \"Fixture declares tier and timing mode.\"",
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
}

// Acceptance: "Benchmark appears in machine-readable summary."
#[test]
fn appears_in_machine_readable_summary() {
    let doc = crate::common::load_toml(&manifest_path());
    let s = doc
        .get("performance_summary_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[performance_summary_contract] missing — acceptance: \
         \"Benchmark appears in machine-readable summary.\"",
        );
    assert_eq!(
        s.get("must_appear_in_machine_readable_summary")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_be_machine_readable").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("summary_record_format").and_then(|v| v.as_str()),
        Some("json")
    );
    assert_eq!(
        s.get("must_be_keyed_by_benchmark_id")
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
        "match_count",
        "match_checksum",
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
        "tier",
        "timing_mode",
        "n_iterations",
        "match_count",
        "expected_match_count",
        "match_checksum",
        "expected_match_checksum",
        "speedup",
        "outcome",
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
