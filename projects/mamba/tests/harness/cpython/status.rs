//! Fast CPython conformance status reporter.
//!
//! This is intentionally a harness-level tool, not a fixture. It summarizes the
//! CPython replacement corpus and local perf/oracle readiness without spawning
//! every fixture. Use it before expensive conformance/oracle runs:
//!
//!     cargo test -p mamba --test cpython_status
//!     cargo test -p mamba --test cpython_status -- --json

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::{json, Value as JsonValue};
use toml::Value as TomlValue;

#[path = "harness_common.rs"]
mod common;
use common::{collect_files, fixture_sha256_opt as fixture_sha256};

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn cpython_dir() -> PathBuf {
    manifest_dir().join("tests/cpython")
}

fn cpython_harness_dir() -> PathBuf {
    manifest_dir().join("tests/harness/cpython")
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

fn tool_mamba(text: &str) -> Option<TomlValue> {
    let toml_src = extract_script_toml(text)?;
    let parsed: TomlValue = toml::from_str(&toml_src).ok()?;
    parsed
        .get("tool")
        .and_then(|tool| tool.get("mamba"))
        .cloned()
}

fn meta_str<'a>(meta: &'a TomlValue, key: &str) -> Option<&'a str> {
    meta.get(key).and_then(TomlValue::as_str)
}

#[derive(Default)]
struct FixtureSummary {
    total: usize,
    migrated: usize,
    legacy: usize,
    invalid_metadata: usize,
    xfail_empty: usize,
    xfail_nonempty: usize,
    stale_cpython_subjects: usize,
    by_bucket: BTreeMap<String, usize>,
    by_dimension: BTreeMap<String, usize>,
}

fn fixture_summary() -> FixtureSummary {
    let mut summary = FixtureSummary::default();
    for path in collect_files(&cpython_dir(), ".py") {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with("_stub.py"))
            || path.components().any(|part| part.as_os_str() == "_invalid")
        {
            continue;
        }

        summary.total += 1;
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        if text.contains("subject = \"cpython.test_") {
            summary.stale_cpython_subjects += 1;
        }

        let Some(meta) = tool_mamba(&text) else {
            summary.legacy += 1;
            continue;
        };

        let Some(bucket) = meta_str(&meta, "bucket") else {
            summary.invalid_metadata += 1;
            continue;
        };
        let Some(dimension) = meta_str(&meta, "dimension") else {
            summary.invalid_metadata += 1;
            continue;
        };

        summary.migrated += 1;
        *summary.by_bucket.entry(bucket.to_string()).or_default() += 1;
        *summary
            .by_dimension
            .entry(dimension.to_string())
            .or_default() += 1;
        match meta_str(&meta, "xfail") {
            Some("") => summary.xfail_empty += 1,
            Some(_) => summary.xfail_nonempty += 1,
            None => {}
        }
    }
    summary
}

fn repo_rel(path: &Path) -> String {
    path.strip_prefix(manifest_dir())
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn baseline_db() -> PathBuf {
    std::env::var("MAMBA_CPYTHON_PERF_BASELINE_DB")
        .map(PathBuf::from)
        .unwrap_or_else(|_| cpython_dir().join(".cache/perf/cpython_baseline.sqlite"))
}

#[derive(Clone)]
struct PerfPin {
    rel_path: String,
    issue: Option<i64>,
    lib: String,
    fixture_rel: String,
    fixture_path: PathBuf,
    prereq_imports: Vec<String>,
}

impl PerfPin {
    fn brief(&self) -> String {
        match self.issue {
            Some(issue) => format!("#{} {} {}", issue, self.lib, self.rel_path),
            None => format!("{} {}", self.lib, self.rel_path),
        }
    }

    fn as_json(&self) -> JsonValue {
        json!({
            "pin": self.rel_path,
            "issue": self.issue,
            "lib": self.lib,
            "fixture": self.fixture_rel,
        })
    }
}

fn load_perf_pins() -> (Vec<PerfPin>, Vec<String>) {
    let mut pins = Vec::new();
    let mut malformed = Vec::new();

    for path in collect_files(&cpython_harness_dir().join("config/perf/pins"), ".toml") {
        let rel_path = repo_rel(&path);
        let raw = match std::fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(err) => {
                malformed.push(format!("{rel_path}: read failed: {err}"));
                continue;
            }
        };
        let parsed: TomlValue = match toml::from_str(&raw) {
            Ok(parsed) => parsed,
            Err(err) => {
                malformed.push(format!("{rel_path}: TOML parse failed: {err}"));
                continue;
            }
        };

        let issue = parsed.get("issue").and_then(TomlValue::as_integer);
        let lib = parsed
            .get("lib")
            .and_then(TomlValue::as_str)
            .unwrap_or("<missing-lib>")
            .to_string();
        let Some(fixture_rel) = parsed.get("fixture").and_then(TomlValue::as_str) else {
            malformed.push(format!("{rel_path}: missing fixture"));
            continue;
        };
        let prereq_imports = parsed
            .get("prereq_imports")
            .and_then(TomlValue::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(TomlValue::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();

        pins.push(PerfPin {
            rel_path,
            issue,
            lib,
            fixture_rel: fixture_rel.to_string(),
            fixture_path: manifest_dir().join(fixture_rel),
            prereq_imports,
        });
    }

    (pins, malformed)
}

fn load_baseline_rows(db: &Path) -> Result<Vec<JsonValue>, String> {
    if !db.exists() {
        return Ok(Vec::new());
    }
    let script = r#"
import json
import sqlite3
import sys

conn = sqlite3.connect(sys.argv[1])
conn.row_factory = sqlite3.Row
rows = conn.execute(
    "SELECT pin_path, issue, lib, fixture, fixture_sha256, samples, "
    "internal_time_ns, cpu_time_ns, peak_rss_bytes, python, platform, "
    "argv, captured_at_unix FROM cpython_perf_baseline"
).fetchall()
print(json.dumps([dict(row) for row in rows], sort_keys=True))
"#;
    let db_arg = db.to_string_lossy().to_string();
    let output = Command::new(common::python3_bin())
        .args(["-c", script, &db_arg])
        .output()
        .map_err(|err| format!("failed to run python sqlite query: {err}"))?;
    if !output.status.success() {
        return Err(format!(
            "sqlite query failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("failed to parse sqlite JSON rows: {err}"))
}

#[derive(Default)]
struct PerfBaselineSummary {
    db_path: PathBuf,
    db_exists: bool,
    rows: usize,
    pins: usize,
    malformed_pins: Vec<String>,
    missing_rows: Vec<PerfPin>,
    recordable_missing_rows: Vec<PerfPin>,
    stale_rows: Vec<PerfPin>,
    missing_cpu_rows: Vec<PerfPin>,
    missing_rss_rows: Vec<PerfPin>,
    missing_fixtures: Vec<PerfPin>,
    missing_prereq_imports: Vec<(PerfPin, String)>,
    query_error: Option<String>,
}

fn json_str<'a>(value: &'a JsonValue, key: &str) -> Option<&'a str> {
    value.get(key).and_then(JsonValue::as_str)
}

fn json_is_null(value: &JsonValue, key: &str) -> bool {
    value.get(key).map_or(true, JsonValue::is_null)
}

fn perf_baseline_summary() -> PerfBaselineSummary {
    let db = baseline_db();
    let (pins, malformed_pins) = load_perf_pins();
    let mut summary = PerfBaselineSummary {
        db_path: db.clone(),
        db_exists: db.exists(),
        rows: 0,
        pins: pins.len(),
        malformed_pins,
        ..Default::default()
    };

    let rows = match load_baseline_rows(&db) {
        Ok(rows) => rows,
        Err(err) => {
            summary.query_error = Some(err);
            return summary;
        }
    };
    summary.rows = rows.len();

    let rows_by_pin: BTreeMap<String, JsonValue> = rows
        .into_iter()
        .filter_map(|row| {
            let pin_path = json_str(&row, "pin_path")?.to_string();
            Some((pin_path, row))
        })
        .collect();

    let missing_import_modules = {
        let modules = pins
            .iter()
            .flat_map(|pin| pin.prereq_imports.iter().cloned())
            .collect();
        missing_python_modules(modules)
    };

    for pin in pins {
        let missing_fixture = !pin.fixture_path.exists();
        if missing_fixture {
            summary.missing_fixtures.push(pin.clone());
        }
        let missing_prereqs: Vec<String> = pin
            .prereq_imports
            .iter()
            .filter(|import_module| missing_import_modules.contains(*import_module))
            .cloned()
            .collect();
        for import_module in &pin.prereq_imports {
            if missing_import_modules.contains(import_module) {
                summary
                    .missing_prereq_imports
                    .push((pin.clone(), import_module.clone()));
            }
        }

        let Some(row) = rows_by_pin.get(&pin.rel_path) else {
            if !missing_fixture && missing_prereqs.is_empty() {
                summary.recordable_missing_rows.push(pin.clone());
            }
            summary.missing_rows.push(pin);
            continue;
        };

        if let Some(actual_hash) = fixture_sha256(&pin.fixture_path) {
            if json_str(row, "fixture_sha256") != Some(actual_hash.as_str()) {
                summary.stale_rows.push(pin.clone());
            }
        }
        if json_is_null(row, "cpu_time_ns") {
            summary.missing_cpu_rows.push(pin.clone());
        }
        if json_is_null(row, "peak_rss_bytes") {
            summary.missing_rss_rows.push(pin);
        }
    }

    summary
}

fn first_import_module(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(rest) = line.strip_prefix("import ") {
            let module = rest
                .split('#')
                .next()
                .unwrap_or(rest)
                .split(',')
                .next()
                .unwrap_or(rest)
                .split(" as ")
                .next()
                .unwrap_or(rest)
                .trim();
            if !module.is_empty() {
                return Some(module.to_string());
            }
        }
        if let Some(rest) = line.strip_prefix("from ") {
            let module = rest.split(" import ").next().unwrap_or(rest).trim();
            if !module.is_empty() && !module.starts_with('.') {
                return Some(module.to_string());
            }
        }
    }
    None
}

fn missing_python_modules(modules: BTreeSet<String>) -> BTreeSet<String> {
    if modules.is_empty() {
        return BTreeSet::new();
    }

    let script = r#"
import importlib
import sys

for module in sys.argv[1:]:
    try:
        importlib.import_module(module)
    except Exception:
        print(module)
"#;
    let output = Command::new(common::python3_bin())
        .arg("-c")
        .arg(script)
        .args(modules.iter())
        .output();
    let Ok(output) = output else {
        return modules;
    };
    if !output.status.success() {
        return modules;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

fn missing_third_party_imports() -> Vec<(String, String)> {
    let root = cpython_dir().join("3rd-libs");
    let mut checked = BTreeSet::new();
    let mut modules_by_lib = Vec::new();
    if !root.exists() {
        return Vec::new();
    }

    let entries = std::fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", root.display()));
    for entry in entries {
        let path = entry.expect("read_dir entry").path();
        if !path.is_dir() {
            continue;
        }
        let lib = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<non-utf8>");
        if lib.starts_with('_') {
            continue;
        }

        let import_module = ["surface.py", "behavior.py"]
            .iter()
            .filter_map(|file| std::fs::read_to_string(path.join(file)).ok())
            .find_map(|text| first_import_module(&text))
            .unwrap_or_else(|| lib.replace('_', "."));

        if checked.insert(import_module.clone()) {
            modules_by_lib.push((lib.to_string(), import_module));
        }
    }

    let missing_modules = missing_python_modules(
        modules_by_lib
            .iter()
            .map(|(_, import_module)| import_module.clone())
            .collect(),
    );
    let mut missing = Vec::new();
    for (lib, import_module) in modules_by_lib {
        if missing_modules.contains(&import_module) {
            missing.push((lib, import_module));
        }
    }
    missing.sort();
    missing
}

fn cpython_denominator_summary() -> JsonValue {
    let script = manifest_dir().join("tools/cpython_regrtest_inventory.py");
    let output = Command::new(common::python3_bin())
        .arg(&script)
        .args(["--json", "--top", "20"])
        .output();
    let output = match output {
        Ok(output) => output,
        Err(err) => {
            return json!({
                "available": false,
                "error": format!("failed to launch {}: {err}", script.display()),
            });
        }
    };
    if !output.status.success() {
        return json!({
            "available": false,
            "exit_status": output.status.to_string(),
            "stdout": String::from_utf8_lossy(&output.stdout),
            "stderr": String::from_utf8_lossy(&output.stderr),
        });
    }
    let parsed: JsonValue = match serde_json::from_slice(&output.stdout) {
        Ok(parsed) => parsed,
        Err(err) => {
            return json!({
                "available": false,
                "error": format!("failed to parse denominator JSON: {err}"),
                "stdout": String::from_utf8_lossy(&output.stdout),
            });
        }
    };
    let candidates = parsed
        .get("cpython_test_case_candidates")
        .cloned()
        .unwrap_or_else(|| json!({}));
    json!({
        "available": true,
        "python": parsed.get("python").cloned().unwrap_or(JsonValue::Null),
        "cpython_test_root": parsed.get("cpython_test_root").cloned().unwrap_or(JsonValue::Null),
        "test_py_files": candidates.get("test_py_files").cloned().unwrap_or(JsonValue::Null),
        "regrtest_modules": parsed.get("cpython_regrtest_modules").cloned().unwrap_or(JsonValue::Null),
        "static_test_defs_in_regrtest_modules": parsed.get("static_test_defs_in_regrtest_modules").cloned().unwrap_or(JsonValue::Null),
        "static_test_defs_in_all_test_files": parsed.get("static_test_defs_in_all_test_files").cloned().unwrap_or(JsonValue::Null),
        "exact_fixture_lib_matches": parsed.get("exact_fixture_lib_matches").cloned().unwrap_or(JsonValue::Null),
        "source_fixture_matches": parsed.get("source_fixture_matches").cloned().unwrap_or(JsonValue::Null),
        "no_fixture_lib_or_source_match": parsed.get("no_fixture_lib_or_source_match").cloned().unwrap_or(JsonValue::Null),
        "ownership": parsed.get("denominator_ownership").cloned().unwrap_or_else(|| json!({})),
        "top_unowned_modules": parsed.get("top_no_fixture_lib_or_source_match").cloned().unwrap_or_else(|| json!([])),
    })
}

fn pin_vec_json(pins: &[PerfPin]) -> Vec<JsonValue> {
    pins.iter().map(PerfPin::as_json).collect()
}

fn pin_import_vec_json(items: &[(PerfPin, String)]) -> Vec<JsonValue> {
    items
        .iter()
        .map(|(pin, import_module)| {
            let mut value = pin.as_json();
            if let Some(obj) = value.as_object_mut() {
                obj.insert("import".to_string(), json!(import_module));
            }
            value
        })
        .collect()
}

fn print_pins(label: &str, pins: &[PerfPin]) {
    println!("  {label}: {}", pins.len());
    for pin in pins.iter().take(20) {
        println!("    {}", pin.brief());
    }
    if pins.len() > 20 {
        println!("    ... {} more", pins.len() - 20);
    }
}

fn print_pin_imports(label: &str, items: &[(PerfPin, String)]) {
    println!("  {label}: {}", items.len());
    for (pin, import_module) in items.iter().take(20) {
        println!("    {}: import {}", pin.brief(), import_module);
    }
    if items.len() > 20 {
        println!("    ... {} more", items.len() - 20);
    }
}

fn main() {
    let json_mode = std::env::args().any(|arg| arg == "--json");
    let fixtures = fixture_summary();
    let denominator = cpython_denominator_summary();
    let perf = perf_baseline_summary();
    let missing_3p = missing_third_party_imports();

    if json_mode {
        let missing_3p_json: Vec<_> = missing_3p
            .iter()
            .map(|(lib, import_module)| json!({"lib": lib, "import": import_module}))
            .collect();
        println!(
            "{}",
            json!({
                "fixtures": {
                    "total": fixtures.total,
                    "migrated": fixtures.migrated,
                    "legacy": fixtures.legacy,
                    "invalid_metadata": fixtures.invalid_metadata,
                    "xfail_empty": fixtures.xfail_empty,
                    "xfail_nonempty": fixtures.xfail_nonempty,
                    "stale_cpython_subjects": fixtures.stale_cpython_subjects,
                    "by_bucket": fixtures.by_bucket,
                    "by_dimension": fixtures.by_dimension,
                },
                "cpython_denominator": denominator,
                "perf": {
                    "pins": perf.pins,
                    "baseline_db": perf.db_path,
                    "baseline_db_exists": perf.db_exists,
                    "baseline_rows": perf.rows,
                    "baseline_missing_rows": perf.missing_rows.len(),
                    "baseline_recordable_missing_rows": perf.recordable_missing_rows.len(),
                    "baseline_stale_rows": perf.stale_rows.len(),
                    "baseline_missing_cpu_rows": perf.missing_cpu_rows.len(),
                    "baseline_missing_rss_rows": perf.missing_rss_rows.len(),
                    "missing_fixture_count": perf.missing_fixtures.len(),
                    "missing_prereq_import_count": perf.missing_prereq_imports.len(),
                    "malformed_pin_count": perf.malformed_pins.len(),
                    "query_error": perf.query_error,
                    "missing_row_pins": pin_vec_json(&perf.missing_rows),
                    "recordable_missing_row_pins": pin_vec_json(&perf.recordable_missing_rows),
                    "stale_row_pins": pin_vec_json(&perf.stale_rows),
                    "missing_cpu_pins": pin_vec_json(&perf.missing_cpu_rows),
                    "missing_rss_pins": pin_vec_json(&perf.missing_rss_rows),
                    "missing_fixture_pins": pin_vec_json(&perf.missing_fixtures),
                    "missing_prereq_import_pins": pin_import_vec_json(&perf.missing_prereq_imports),
                    "malformed_pins": perf.malformed_pins,
                },
                "third_party": {
                    "missing_imports": missing_3p_json,
                    "missing_import_count": missing_3p.len(),
                }
            })
        );
        return;
    }

    println!("CPython conformance status");
    println!("  fixtures total: {}", fixtures.total);
    println!("  migrated: {}", fixtures.migrated);
    println!("  legacy: {}", fixtures.legacy);
    println!("  invalid metadata: {}", fixtures.invalid_metadata);
    println!("  xfail empty/pass-intended: {}", fixtures.xfail_empty);
    println!("  xfail nonempty/mamba-gap: {}", fixtures.xfail_nonempty);
    println!(
        "  stale CPython subjects: {}",
        fixtures.stale_cpython_subjects
    );
    println!("  by bucket: {:?}", fixtures.by_bucket);
    println!("  by dimension: {:?}", fixtures.by_dimension);
    println!("  denominator available: {}", denominator["available"]);
    if denominator["available"].as_bool() == Some(true) {
        let ownership = &denominator["ownership"];
        println!("  denominator CPython: {}", denominator["python"]);
        println!(
            "  denominator test root: {}",
            denominator["cpython_test_root"]
        );
        println!(
            "  denominator test .py files: {}",
            denominator["test_py_files"]
        );
        println!(
            "  denominator regrtest modules: {}",
            denominator["regrtest_modules"]
        );
        println!(
            "  denominator static test defs: {}",
            denominator["static_test_defs_in_regrtest_modules"]
        );
        println!("  denominator ownership pass: {}", ownership["pass"]);
        println!(
            "  denominator owned modules: {}",
            ownership["owned_modules"]
        );
        println!(
            "  denominator unowned modules: {}",
            ownership["unowned_modules"]
        );
        println!(
            "  denominator unowned static test defs: {}",
            ownership["unowned_static_test_defs"]
        );
        if let Some(items) = denominator["top_unowned_modules"].as_array() {
            for item in items.iter().take(10) {
                println!(
                    "    unowned denominator: {} defs={} key={}",
                    item["module"], item["static_test_defs"], item["key"]
                );
            }
        }
    } else if let Some(err) = denominator["error"].as_str() {
        println!("  denominator error: {err}");
    }
    println!("  perf pins: {}", perf.pins);
    println!("  perf baseline db: {}", perf.db_path.display());
    println!("  perf baseline db exists: {}", perf.db_exists);
    println!("  perf baseline rows: {}", perf.rows);
    if let Some(err) = &perf.query_error {
        println!("  perf baseline query error: {err}");
    }
    if !perf.malformed_pins.is_empty() {
        println!("  malformed perf pins: {}", perf.malformed_pins.len());
        for item in perf.malformed_pins.iter().take(20) {
            println!("    {item}");
        }
        if perf.malformed_pins.len() > 20 {
            println!("    ... {} more", perf.malformed_pins.len() - 20);
        }
    }
    print_pins(
        "perf baseline recordable missing rows",
        &perf.recordable_missing_rows,
    );
    print_pins("perf baseline missing rows", &perf.missing_rows);
    print_pins("perf baseline stale rows", &perf.stale_rows);
    print_pins("perf baseline missing cpu rows", &perf.missing_cpu_rows);
    print_pins("perf baseline missing rss rows", &perf.missing_rss_rows);
    print_pins("perf missing fixtures", &perf.missing_fixtures);
    print_pin_imports("perf missing prereq imports", &perf.missing_prereq_imports);
    println!("  missing 3rd-party imports: {}", missing_3p.len());
    for (lib, import_module) in missing_3p.iter().take(20) {
        println!("    {lib}: import {import_module}");
    }
    if missing_3p.len() > 20 {
        println!("    ... {} more", missing_3p.len() - 20);
    }
}
