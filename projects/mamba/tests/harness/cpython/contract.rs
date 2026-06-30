//! CPython replacement test-contract gate.
//!
//! This is a meta-test over `tests/cpython/`: it does not execute Python. It
//! fails when the fixture tree stops covering one of the product-level axes that
//! make mamba a credible CPython replacement: positive compatibility, negative
//! compatibility, strict typing, speed, memory, and adversarial safety.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use toml::Value;

#[path = "harness_common.rs"]
mod common;
use common::collect_files;

const MIN_SECURITY_FIXTURES: usize = 30;
const MIN_TYPE_STRICT_FIXTURES: usize = 15;
const MIN_PERF_PINS: usize = 100;
const MIN_COMPILER_RESILIENCE_FIXTURES: usize = 5;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cpython_dir() -> PathBuf {
    manifest_dir().join("tests/cpython")
}

fn cpython_harness_dir() -> PathBuf {
    manifest_dir().join("tests/harness/cpython")
}

fn fixture_files() -> Vec<PathBuf> {
    collect_files(&cpython_dir(), ".py")
}

fn extract_script_toml(text: &str) -> Option<String> {
    let start = text.find("# /// script")?;
    let rest = &text[start + "# /// script".len()..];
    let end = rest.find("\n# ///")?;
    let block = &rest[..end];
    let mut lines = Vec::new();
    for raw in block.lines() {
        if let Some(stripped) = raw.strip_prefix("# ") {
            lines.push(stripped);
        } else if raw == "#" || raw.is_empty() {
            lines.push("");
        } else {
            return None;
        }
    }
    Some(lines.join("\n"))
}

fn tool_mamba(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    let toml_src = extract_script_toml(&text)?;
    let parsed: Value = toml::from_str(&toml_src).ok()?;
    parsed
        .get("tool")
        .and_then(|tool| tool.get("mamba"))
        .cloned()
}

fn meta_str<'a>(meta: &'a Value, key: &str) -> &'a str {
    meta.get(key)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("[tool.mamba].{key} missing or not a string"))
}

fn fixture_rel(path: &Path) -> PathBuf {
    path.strip_prefix(cpython_dir())
        .expect("fixture path under fixtures root")
        .to_path_buf()
}

fn cpython312_surface_subjects() -> BTreeSet<String> {
    let path = manifest_dir().join("data/cpython312_surface.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
    let parsed: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|err| panic!("cannot parse {}: {err}", path.display()));
    let modules = parsed
        .get("modules")
        .and_then(serde_json::Value::as_object)
        .unwrap_or_else(|| panic!("{} missing object `modules`", path.display()));

    let mut subjects = BTreeSet::new();
    for (module, record) in modules {
        let names = record
            .get("names")
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("{} module `{module}` missing names", path.display()));
        for name in names {
            let name = name.as_str().unwrap_or_else(|| {
                panic!("{} module `{module}` has non-string name", path.display())
            });
            subjects.insert(format!("{module}.{name}"));
        }
    }

    let expected_total = parsed
        .get("total_name_count")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or_else(|| panic!("{} missing numeric total_name_count", path.display()))
        as usize;
    assert_eq!(
        subjects.len(),
        expected_total,
        "{} total_name_count does not match unique module.name subjects",
        path.display()
    );
    subjects
}

fn migrated_surface_subjects() -> BTreeSet<String> {
    let mut subjects = BTreeSet::new();
    for path in fixture_files() {
        let Some(meta) = tool_mamba(&path) else {
            continue;
        };
        if meta_str(&meta, "dimension") == "surface" {
            subjects.insert(meta_str(&meta, "subject").to_string());
        }
    }
    subjects
}

#[test]
fn fixture_tree_covers_all_replacement_axes() {
    // Dimension-first layout (STRUCTURE.md): the `[tool.mamba]` record is the
    // source of truth for a fixture's dimension, NOT its path index. The tree is
    // now `{facet}/{bucket}/{lib}/{case}.py`, so the old `path.components()[2]`
    // bucket-first index is gone; every axis assertion reads the record.
    let mut migrated_by_dimension: BTreeMap<String, usize> = BTreeMap::new();
    let mut buckets_by_dimension: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    // Per-lib security (record bucket != the flat `security-matrix` wall). The
    // speed/memory guard compares the perf wall against this apples-to-apples
    // per-lib adversarial count, not the 228-cell matrix wall.
    let mut per_lib_security: usize = 0;

    for path in fixture_files() {
        if let Some(meta) = tool_mamba(&path) {
            let bucket = meta_str(&meta, "bucket").to_string();
            let dimension = meta_str(&meta, "dimension").to_string();
            *migrated_by_dimension.entry(dimension.clone()).or_default() += 1;
            if dimension == "security" && bucket != "security-matrix" {
                per_lib_security += 1;
            }
            buckets_by_dimension
                .entry(dimension)
                .or_default()
                .insert(bucket);

            assert_ne!(
                meta.get("status").and_then(Value::as_str),
                Some("generated"),
                "{} is still an unfilled generated skeleton",
                path.display()
            );
            assert!(
                meta.get("xfail").and_then(Value::as_str).is_some(),
                "{} lacks explicit xfail policy",
                path.display()
            );
        }
    }

    for dimension in ["surface", "behavior", "errors", "real_world", "security"] {
        assert!(
            migrated_by_dimension.get(dimension).copied().unwrap_or(0) > 0,
            "missing migrated CPython replacement dimension `{dimension}`"
        );
    }
    // Speed/memory gate: the perf wall (`dimension=perf`) is the per-record
    // speed/memory facet; drive the assertion off the record, not a path index.
    assert!(
        migrated_by_dimension.get("perf").copied().unwrap_or(0) > 0,
        "missing perf fixtures for speed/memory gates"
    );

    assert!(
        buckets_by_dimension
            .get("surface")
            .is_some_and(|buckets| buckets.contains("std-libs") && buckets.contains("pep")),
        "positive API compatibility must cover std-libs and PEP fixtures"
    );
    assert!(
        buckets_by_dimension
            .get("errors")
            .is_some_and(|buckets| buckets.contains("std-libs") && buckets.contains("pep")),
        "negative compatibility must cover std-libs and PEP fixtures"
    );
    assert!(
        migrated_by_dimension.get("perf").copied().unwrap_or(0) >= per_lib_security,
        "speed/memory coverage should not be smaller than per-lib security coverage"
    );
}

#[test]
fn cpython312_public_api_surface_is_fully_fixture_backed() {
    let expected = cpython312_surface_subjects();
    let covered = migrated_surface_subjects();
    let missing: Vec<_> = expected.difference(&covered).cloned().collect();
    let sample = missing
        .iter()
        .take(40)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");

    assert!(
        missing.is_empty(),
        "CPython 3.12 API surface fixture coverage is {}/{}; missing {} subjects: {}",
        expected.len() - missing.len(),
        expected.len(),
        missing.len(),
        sample
    );
}

#[test]
fn type_strict_inverse_contract_is_explicit() {
    let root = cpython_dir().join("type");
    let files = collect_files(&root, ".py");
    assert!(
        files.len() >= MIN_TYPE_STRICT_FIXTURES,
        "expected at least {MIN_TYPE_STRICT_FIXTURES} type-strict fixtures, got {}",
        files.len()
    );

    for path in files {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        assert!(
            text.contains("mamba-strict-type:") || text.contains("strict_type"),
            "{} does not declare the strict-type contract",
            path.display()
        );
        // Inverse-classification markers (FIXTURE-LAYOUT type-strict convention):
        // the source prints `no_typeerror:` on the CPython-accepts branch and
        // `typeerror:` on the mamba-must-raise branch. type_wall_gen.py emits both
        // (plus a `setup_or_other:` branch for import/setup failures). Requiring
        // both keeps the contract honest — a fixture that dropped `no_typeerror:`
        // would no longer assert that CPython's accepting path is covered.
        assert!(
            text.contains("typeerror:") && text.contains("no_typeerror:"),
            "{} must emit both inverse-classification markers",
            path.display()
        );
    }
}

#[test]
fn perf_pins_gate_speed_and_memory_against_cpython() {
    let pins = collect_files(&cpython_harness_dir().join("config/perf/pins"), ".toml");
    assert!(
        pins.len() >= MIN_PERF_PINS,
        "expected at least {MIN_PERF_PINS} perf pins, got {}",
        pins.len()
    );

    for path in pins {
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        let parsed: Value = toml::from_str(&raw)
            .unwrap_or_else(|err| panic!("cannot parse {}: {err}", path.display()));

        let floor = parsed
            .get("floor")
            .and_then(Value::as_float)
            .unwrap_or_else(|| panic!("{} missing numeric floor", path.display()));
        assert!(
            floor <= 1.0,
            "{} allows mamba slower than CPython: floor={floor}",
            path.display()
        );

        let mem_floor = parsed
            .get("mem_floor")
            .and_then(Value::as_float)
            .unwrap_or_else(|| panic!("{} missing numeric mem_floor", path.display()));
        assert!(
            mem_floor >= 1.0,
            "{} allows mamba peak RSS above CPython: mem_floor={mem_floor}",
            path.display()
        );

        let fixture = parsed
            .get("fixture")
            .and_then(Value::as_str)
            .unwrap_or_else(|| panic!("{} missing fixture path", path.display()));
        let fixture_path = manifest_dir().join(fixture);
        assert!(
            fixture_path.exists(),
            "{} points at missing fixture {}",
            path.display(),
            fixture
        );
        // D5.2: speed gates measure CPU externally (getrusage / /usr/bin/time),
        // so fixtures no longer self-emit a self-timing marker. The old
        // "must self-emit a timing marker" contract is removed here (it forced the
        // self-timing anti-pattern); once D5.1 strips the markers tree-wide this
        // can flip to the inverse contract (a speed fixture must NOT self-time).
    }
}

#[test]
fn perf_baseline_sqlite_tool_is_present() {
    let tool = cpython_harness_dir().join("tools/perf_baseline.py");
    let raw = std::fs::read_to_string(&tool)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", tool.display()));
    for required in [
        "sqlite3",
        "cpython_perf_baseline",
        "internal_time_ns",
        "cpu_time_ns",
        "peak_rss_bytes",
        "--missing-only",
        "--ready-only",
        "--limit",
    ] {
        assert!(
            raw.contains(required),
            "{} missing required perf-baseline marker `{required}`",
            tool.display()
        );
    }
}

#[test]
fn cpython_status_reports_actionable_perf_baseline_readiness() {
    let tool = cpython_harness_dir().join("status.rs");
    let raw = std::fs::read_to_string(&tool)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", tool.display()));
    for required in [
        "baseline_missing_rows",
        "baseline_recordable_missing_rows",
        "baseline_stale_rows",
        "baseline_missing_cpu_rows",
        "baseline_missing_rss_rows",
        "missing_prereq_import",
        "fixture_sha256",
    ] {
        assert!(
            raw.contains(required),
            "{} missing required status marker `{required}`",
            tool.display()
        );
    }
}

#[test]
fn cpython_oracle_authoring_gate_is_present() {
    assert!(
        !cpython_dir().join("run.py").exists(),
        "tests/cpython/run.py is retired; use tests/harness/cpython plus tools/ instead"
    );

    let tool = cpython_harness_dir().join("tools/verify_cpython_oracle.py");
    let raw = std::fs::read_to_string(&tool)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", tool.display()));
    for required in [
        "No pass/fail results are stored",
        "bench/perf-baseline-owned",
        "pipeline-run-directive",
        ".cpython.expected",
        "cpython runtime pass",
        "--ready-only",
        "missing-prereq-import",
        "--progress-every",
    ] {
        assert!(
            raw.contains(required),
            "{} missing required CPython-oracle marker `{required}`",
            tool.display()
        );
    }
}

#[test]
fn py314_ast_type_fixtures_are_version_excluded_consistently() {
    for rel in [
        "tools/verify_cpython_oracle.py",
        "tools/strict_type_accounting.py",
    ] {
        let path = cpython_harness_dir().join(rel);
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        for required in [
            "std-libs/ast/Interpolation__init__value_as_expr_wrong.py",
            "std-libs/ast/TemplateStr__init__values_as_list_wrong.py",
            "(3, 14)",
        ] {
            assert!(
                raw.contains(required),
                "{} missing Py3.14-only AST type fixture exclusion marker `{required}`",
                path.display()
            );
        }
    }
}

#[test]
fn safety_contract_has_adversarial_fixtures_and_sandboxed_runner() {
    let security = fixture_files()
        .into_iter()
        .filter(|path| {
            fixture_rel(path)
                .components()
                .any(|part| part.as_os_str() == "security")
        })
        .count();
    assert!(
        security >= MIN_SECURITY_FIXTURES,
        "expected at least {MIN_SECURITY_FIXTURES} security fixtures, got {security}"
    );

    let compiler_resilience = collect_files(
        &cpython_dir().join("_regression/core/compiler_resilience"),
        ".py",
    );
    assert!(
        compiler_resilience.len() >= MIN_COMPILER_RESILIENCE_FIXTURES,
        "expected hostile-source compiler resilience fixtures, got {}",
        compiler_resilience.len()
    );

    let runner = std::fs::read_to_string(cpython_harness_dir().join("runner.rs"))
        .expect("read conformance runner");
    for needle in [
        "RLIMIT_AS",
        "RLIMIT_DATA",
        "RLIMIT_CPU",
        "RLIMIT_CORE",
        "MAMBA_CONFORMANCE_TIMEOUT_SECS",
    ] {
        assert!(
            runner.contains(needle),
            "conformance runner is missing sandbox/time bound marker `{needle}`"
        );
    }
}

#[test]
fn oracle_cache_contract_reports_warm_hit_metrics() {
    let runner = std::fs::read_to_string(cpython_harness_dir().join("runner.rs"))
        .expect("read conformance runner");
    for needle in [
        "ORACLE_CACHE_HITS",
        "ORACLE_CACHE_MISSES",
        "ORACLE_CACHE_DISABLED",
        "[oracle-cache]",
        "oracle hit={hit} miss={miss} disabled={disabled}",
        "record_oracle_cache_hit(path)",
        "record_oracle_cache_miss(path)",
        "record_oracle_cache_disabled(path)",
        "MAMBA_ORACLE_CACHE",
    ] {
        assert!(
            runner.contains(needle),
            "conformance runner is missing D5.3 oracle-cache metric marker `{needle}`"
        );
    }
}

#[test]
fn production_gate_can_report_d54_sut_rows_from_temp_db() {
    let gate = std::fs::read_to_string(cpython_harness_dir().join("tools/gate_check.py"))
        .expect("read production gate check");
    for needle in [
        "--db",
        "MAMBA_RESULTS_DB",
        "D5.3 cpython oracle rows",
        "D5.4 mamba SUT rows",
        "mamba verdicts",
        "outside-repo",
    ] {
        assert!(
            gate.contains(needle),
            "gate_check.py is missing D5.4 temp-DB reporting marker `{needle}`"
        );
    }
}
