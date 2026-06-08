//! Schema gate for the MVP performance profile manifest — closes #2815.
//!
//! Acceptance (issue #2815):
//!
//!   1. Profile fails if CPython runtime is not CPython 3.12.
//!      `[runtime_identity].required_cpython` must equal `"3.12"`.
//!   2. Profile fails if required benchmarks miss 1x floor or 10x
//!      average. `[policy].per_benchmark_floor` ≥ 1.0 and
//!      `[policy].suite_average_floor` ≥ 10.0.
//!   3. Summary names slowest blockers.
//!      `[summary].slowest_blocker_count` is positive and
//!      `[summary].fields` includes `name` plus the speedup field.
//!
//! Also asserts that the referenced baseline.json exists and uses the
//! field names declared in `[summary.source]` — that pins the runner's
//! input contract.
//!
//! Cheap test — two TOML reads + one JSON read + field walk. Stays in
//! the default `cargo test -p mamba` set.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

const EXPECTED_BUCKETS: &[&str] = &["required", "optional", "xfail", "blocker"];
const PER_BENCH_FLOOR: f64 = 1.0;
const SUITE_AVERAGE_FLOOR: f64 = 10.0;

fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("profiles")
        .join("performance.toml")
}

fn umbrella_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("validation")
        .join("mvp.toml")
}

#[test]
fn performance_profile_manifest_header_is_well_formed() {
    let doc = crate::common::load_toml(&manifest_path());

    assert_eq!(
        doc.get("profile").and_then(|v| v.as_str()),
        Some("performance"),
        "performance.toml `profile` must be \"performance\""
    );
    assert_eq!(
        doc.get("issue").and_then(|v| v.as_integer()),
        Some(2815),
        "performance.toml `issue` must record #2815 as the owning issue"
    );

    let buckets = doc
        .get("buckets")
        .and_then(|v| v.as_array())
        .expect("performance.toml missing required `buckets` array");
    let buckets: BTreeSet<&str> = buckets.iter().filter_map(|v| v.as_str()).collect();
    let expected: BTreeSet<&str> = EXPECTED_BUCKETS.iter().copied().collect();
    assert_eq!(
        buckets, expected,
        "performance.toml `buckets` must list exactly {EXPECTED_BUCKETS:?}"
    );
}

#[test]
fn performance_profile_requires_cpython_3_12() {
    let doc = crate::common::load_toml(&manifest_path());

    let identity = doc
        .get("runtime_identity")
        .and_then(|v| v.as_table())
        .expect("performance.toml missing `[runtime_identity]` block");

    let cpython = identity
        .get("required_cpython")
        .and_then(|v| v.as_str())
        .expect("performance.toml `[runtime_identity].required_cpython` must be set");
    assert_eq!(
        cpython, "3.12",
        "performance.toml `[runtime_identity].required_cpython` must be \"3.12\" \
         (acceptance: \"Profile fails if CPython runtime is not CPython 3.12.\")"
    );

    let mamba_edition = identity
        .get("required_mamba_edition")
        .and_then(|v| v.as_str())
        .expect("performance.toml `[runtime_identity].required_mamba_edition` must be set");
    assert!(
        !mamba_edition.is_empty(),
        "required_mamba_edition must be non-empty"
    );
}

#[test]
fn performance_profile_policy_pins_floors() {
    let doc = crate::common::load_toml(&manifest_path());
    let policy = doc
        .get("policy")
        .and_then(|v| v.as_table())
        .expect("performance.toml missing `[policy]` block");

    let per_bench = policy
        .get("per_benchmark_floor")
        .and_then(|v| v.as_float())
        .or_else(|| {
            policy
                .get("per_benchmark_floor")
                .and_then(|v| v.as_integer())
                .map(|i| i as f64)
        })
        .expect("performance.toml `[policy].per_benchmark_floor` must be numeric");
    assert!(
        per_bench >= PER_BENCH_FLOOR,
        "performance.toml `[policy].per_benchmark_floor` = {per_bench} must be \
         ≥ {PER_BENCH_FLOOR} (acceptance: \"required benchmarks miss 1x floor\")"
    );

    let suite_avg = policy
        .get("suite_average_floor")
        .and_then(|v| v.as_float())
        .or_else(|| {
            policy
                .get("suite_average_floor")
                .and_then(|v| v.as_integer())
                .map(|i| i as f64)
        })
        .expect("performance.toml `[policy].suite_average_floor` must be numeric");
    assert!(
        suite_avg >= SUITE_AVERAGE_FLOOR,
        "performance.toml `[policy].suite_average_floor` = {suite_avg} must be \
         ≥ {SUITE_AVERAGE_FLOOR} (acceptance: \"or 10x average\")"
    );

    let method = policy
        .get("suite_average_method")
        .and_then(|v| v.as_str())
        .expect("performance.toml `[policy].suite_average_method` must be set");
    assert!(
        matches!(method, "geometric_mean" | "harmonic_mean"),
        "performance.toml `[policy].suite_average_method` = {method:?} must be \
         a ratio-safe mean (`geometric_mean` or `harmonic_mean`); arithmetic \
         mean over-weights outliers and is not acceptable for speedup ratios"
    );

    let required: BTreeSet<&str> = policy
        .get("release_required_buckets")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        required.contains("required"),
        "performance.toml `[policy].release_required_buckets` must include \
         `required`; got {required:?}"
    );

    let avg_includes: BTreeSet<&str> = policy
        .get("average_includes")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        avg_includes.contains("required") && avg_includes.len() == 1,
        "performance.toml `[policy].average_includes` must be exactly \
         {{\"required\"}}; xfail/blocker outliers must not skew the suite \
         average. got {avg_includes:?}"
    );
}

#[test]
fn performance_profile_summary_names_slowest_blockers() {
    let doc = crate::common::load_toml(&manifest_path());

    let summary = doc
        .get("summary")
        .and_then(|v| v.as_table())
        .expect("performance.toml missing `[summary]` block");

    let n = summary
        .get("slowest_blocker_count")
        .and_then(|v| v.as_integer())
        .expect("performance.toml `[summary].slowest_blocker_count` must be set");
    assert!(
        n > 0,
        "performance.toml `[summary].slowest_blocker_count` = {n} must be > 0 \
         (acceptance: \"Summary names slowest blockers.\")"
    );

    let fields: BTreeSet<&str> = summary
        .get("fields")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    assert!(
        fields.contains("name"),
        "performance.toml `[summary].fields` must include `name` so the \
         summary names each blocker; got {fields:?}"
    );
    assert!(
        fields.contains("speedup_vs_cpython"),
        "performance.toml `[summary].fields` must include \
         `speedup_vs_cpython` so the summary surfaces the ratio that \
         tripped the floor; got {fields:?}"
    );

    let sort = summary
        .get("sort")
        .and_then(|v| v.as_str())
        .expect("performance.toml `[summary].sort` must be set");
    assert_eq!(
        sort, "speedup_asc",
        "performance.toml `[summary].sort` must be `speedup_asc` so the \
         slowest blockers come first; got {sort:?}"
    );
}

#[test]
fn performance_profile_baseline_exists_and_matches_summary_contract() {
    let doc = crate::common::load_toml(&manifest_path());

    let baseline_rel = doc
        .get("baseline")
        .and_then(|v| v.as_str())
        .expect("performance.toml missing `baseline` field");
    let baseline_path = manifest_path().parent().unwrap().join(baseline_rel);
    let canonical = baseline_path.canonicalize().unwrap_or_else(|e| {
        panic!(
            "baseline path {} does not resolve: {e}",
            baseline_path.display()
        )
    });
    assert!(
        canonical.exists(),
        "performance.toml `baseline` ({}) must resolve to an existing file",
        canonical.display()
    );

    let summary_source = doc
        .get("summary")
        .and_then(|v| v.get("source"))
        .and_then(|v| v.as_table())
        .expect("performance.toml missing `[summary.source]` block");

    let benchmarks_field = summary_source
        .get("field")
        .and_then(|v| v.as_str())
        .expect("`[summary.source].field` must be set");
    let speedup_key = summary_source
        .get("speedup_key")
        .and_then(|v| v.as_str())
        .expect("`[summary.source].speedup_key` must be set");
    let name_key = summary_source
        .get("name_key")
        .and_then(|v| v.as_str())
        .expect("`[summary.source].name_key` must be set");

    let raw = std::fs::read_to_string(&canonical)
        .unwrap_or_else(|e| panic!("baseline read failed: {e}"));
    let parsed: serde_json::Value =
        serde_json::from_str(&raw).unwrap_or_else(|e| panic!("baseline parse failed: {e}"));

    let benchmarks = parsed
        .get(benchmarks_field)
        .and_then(|v| v.as_array())
        .unwrap_or_else(|| panic!("baseline missing `{benchmarks_field}` array"));
    assert!(
        !benchmarks.is_empty(),
        "baseline `{benchmarks_field}` must be non-empty; the suite-average \
         floor cannot be computed against zero benchmarks"
    );

    let first = benchmarks
        .first()
        .and_then(|v| v.as_object())
        .expect("first baseline entry must be a JSON object");
    assert!(
        first.contains_key(name_key),
        "baseline entries must carry `{name_key}` (summary contract \
         declared in performance.toml [summary.source]); first entry: \
         {first:?}"
    );
    assert!(
        first.contains_key(speedup_key),
        "baseline entries must carry `{speedup_key}` (summary contract \
         declared in performance.toml [summary.source]); first entry: \
         {first:?}"
    );
}

#[test]
fn mvp_umbrella_links_to_performance_manifest() {
    let doc = crate::common::load_toml(&umbrella_path());
    let entry = doc
        .get("profiles")
        .and_then(|v| v.get("performance"))
        .and_then(|v| v.as_table())
        .expect("validation/mvp.toml missing `[profiles.performance]`");

    let manifest = entry
        .get("manifest")
        .and_then(|v| v.as_str())
        .expect("`[profiles.performance].manifest` must be set");
    assert_eq!(
        manifest, "profiles/performance.toml",
        "umbrella must point at profiles/performance.toml; got {manifest:?}"
    );

    let issue = entry
        .get("issue")
        .and_then(|v| v.as_integer())
        .expect("`[profiles.performance].issue` must record the issue id");
    assert_eq!(issue, 2815, "performance profile owner issue must be #2815");
}
