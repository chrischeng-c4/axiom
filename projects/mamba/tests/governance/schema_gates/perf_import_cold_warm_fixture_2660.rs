//! Schema gate for the import cold+warm benchmark fixture — closes
//! #2660.

use std::path::{Path, PathBuf};

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("gates")
        .join("performance")
        .join("import_cold_warm")
        .join("manifest.toml")
}

fn load_toml(path: &Path) -> toml::Value {
    let raw = std::fs::read_to_string(path).unwrap();
    raw.parse().unwrap()
}

const BENCH_ID: &str = "import_cold_warm";
const ISSUE: i64 = 2660;
const FIXTURE: &str = "perf_import_cold_warm";
const OOS_FLAG: &str = "import_system_optimization";

#[test]
fn header_is_well_formed() {
    let doc = load_toml(&manifest_path());
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
fn workload_imports_local_and_stdlib_modules() {
    let doc = load_toml(&manifest_path());
    let w = doc.get("workload").and_then(|v| v.as_table()).unwrap();
    assert!(w
        .get("local_module_name")
        .and_then(|v| v.as_str())
        .is_some());
    assert!(w
        .get("stdlib_module_name")
        .and_then(|v| v.as_str())
        .is_some());
    assert!(w
        .get("local_module_sentinel_name")
        .and_then(|v| v.as_str())
        .is_some());
    assert!(w
        .get("stdlib_sentinel_attribute")
        .and_then(|v| v.as_str())
        .is_some());
    assert_eq!(
        w.get("must_invalidate_module_cache_between_cold_runs")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(w.get("deterministic").and_then(|v| v.as_bool()), Some(true));
}

// Acceptance: "Benchmark output validates imported sentinel values."
#[test]
fn sentinel_values_are_validated() {
    let doc = load_toml(&manifest_path());
    let c = doc
        .get("checksum_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[checksum_contract] missing — acceptance: \
         \"Benchmark output validates imported sentinel values.\"",
        );
    assert_eq!(
        c.get("must_validate_before_accepting_timing")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_validate_local_sentinel_value")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        c.get("must_validate_stdlib_sentinel_callable")
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
        c.get("diagnostic_must_name_benchmark")
            .and_then(|v| v.as_str()),
        Some(BENCH_ID)
    );
}

// Acceptance: "Fixture declares whether startup cost is included."
#[test]
fn timing_contract_declares_startup_cost_inclusion() {
    let doc = load_toml(&manifest_path());
    let t = doc
        .get("timing_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[timing_contract] missing — acceptance: \
         \"Fixture declares whether startup cost is included.\"",
        );
    assert!(t.get("tier").and_then(|v| v.as_str()).is_some());
    assert_eq!(
        t.get("must_declare_startup_cost_included")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert!(
        t.get("startup_cost_included")
            .and_then(|v| v.as_bool())
            .is_some(),
        "startup_cost_included must be explicitly declared as a bool",
    );
    assert_eq!(
        t.get("must_distinguish_cold_and_warm_timings")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

// Acceptance: "Summary distinguishes cold import from warm import."
#[test]
fn summary_distinguishes_cold_from_warm() {
    let doc = load_toml(&manifest_path());
    let s = doc
        .get("performance_summary_contract")
        .and_then(|v| v.as_table())
        .expect(
            "[performance_summary_contract] missing — acceptance: \
         \"Summary distinguishes cold import from warm import.\"",
        );
    assert_eq!(
        s.get("must_appear_in_machine_readable_summary")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_emit_cold_and_warm_speedups_separately")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        s.get("must_be_keyed_by_benchmark_id_and_phase")
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    let phases: Vec<&str> = s
        .get("allowed_phase_labels")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for p in &["cold", "warm"] {
        assert!(phases.contains(p), "allowed_phase_labels must include {p}");
    }
    let req: Vec<&str> = s
        .get("required_record_keys")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    for k in &["benchmark_id", "phase", "speedup", "outcome"] {
        assert!(req.contains(k), "required_record_keys must include {k}");
    }
}

#[test]
fn runner_contract_declares_keys_and_outcomes() {
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
    for k in &[
        "benchmark_id",
        "phase",
        "tier",
        "timing_mode",
        "startup_cost_included",
        "checksum",
        "expected_checksum",
        "cold_speedup",
        "warm_speedup",
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
    let doc = load_toml(&manifest_path());
    let o = doc.get("out_of_scope").and_then(|v| v.as_table()).unwrap();
    assert_eq!(o.get(OOS_FLAG).and_then(|v| v.as_bool()), Some(true));
}
