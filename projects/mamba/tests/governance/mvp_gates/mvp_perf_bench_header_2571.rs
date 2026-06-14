//! MVP performance bench tier-header gate (closes #2571).
//!
//! Parent: #2530 (performance gate suite).
//!
//! Locks the contract of `scripts/perf_bench_header_check.py`. The
//! checker parses three `#`-prefixed directives in each perf
//! benchmark fixture:
//!
//!     # tier: <required|exploratory>
//!     # category: <numeric|recursion|workload|...>
//!     # inclusion_reason: <free-form one-line reason>
//!
//! Required fixtures must declare all three; the header `tier:`
//! value must match the manifest's `tier`. The harness emits parsed
//! values in its JSON output so a release reviewer can see the tier
//! per benchmark next to the timing.
//!
//! Acceptance (issue #2571):
//!
//!     1. A fixture without required tier metadata fails validation.
//!     2. Harness output includes tier per benchmark.
//!     3. Existing comments can be migrated without changing
//!        benchmark code (`#` directives, anywhere in the file).

use std::path::{Path, PathBuf};

use serde_json::Value;

fn checker_script() -> PathBuf {
    crate::common::project_root()
        .join("scripts")
        .join("perf_bench_header_check.py")
}

fn unique_dir(tag: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir()
        .join(format!("mamba-perf-bench-header-{tag}-{nanos}"));
    std::fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

fn run_checker(args: &[&str]) -> (i32, String, String) {
    let output = crate::common::run_python_script(&checker_script(), args);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
    )
}

fn run_checker_json(args: &[&str]) -> (i32, Value) {
    let mut full = vec!["--format", "json"];
    full.extend_from_slice(args);
    let (code, stdout, stderr) = run_checker(&full);
    let payload: Value =
        serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "checker JSON parse failed (code={code}): {e}\n--stdout--\n{stdout}\n--stderr--\n{stderr}"
            )
        });
    (code, payload)
}

fn write_manifest_and_fixtures(
    dir: &Path,
    body: &str,
    fixtures: &[(&str, &str)],
) -> PathBuf {
    let fx = dir.join("fixtures");
    std::fs::create_dir_all(&fx).unwrap();
    for (name, content) in fixtures {
        std::fs::write(fx.join(name), content).unwrap();
    }
    let path = dir.join("manifest.toml");
    std::fs::write(&path, body).unwrap();
    path
}

const MIN_VALID_MANIFEST_HEADER: &str = r#"
version = 1
manifest = "perf_benchmark"
parent_issue = 2530
issue = 2567
fixture_root = "fixtures"
categories = ["numeric", "recursion", "workload"]
tiers = ["required", "exploratory"]
timing_modes = ["internal", "process_wall"]
"#;

// ─── Shipped fixtures declare all three headers ──────────────────

#[test]
fn shipped_required_fixtures_have_all_three_tier_headers() {
    for (fixture, expected_tier) in &[
        ("int_sum.py", "required"),
        ("range_sum.py", "required"),
    ] {
        let body = std::fs::read_to_string(
            crate::common::project_root()
                .join("tests/cpython/_regression/core/bench")
                .join(fixture),
        )
        .expect("read shipped required fixture");
        assert!(
            body.contains(&format!("# tier: {expected_tier}")),
            "{fixture} must declare `# tier: {expected_tier}` header"
        );
        assert!(
            body.contains("# category:"),
            "{fixture} must declare `# category:` header"
        );
        assert!(
            body.contains("# inclusion_reason:"),
            "{fixture} must declare `# inclusion_reason:` header"
        );
    }
}

#[test]
fn shipped_manifest_passes_bench_header_check() {
    let (code, _stdout, stderr) = run_checker(&["--format", "text"]);
    assert_eq!(code, 0, "shipped manifest must pass; stderr={stderr}");
    assert!(
        stderr.contains("perf_bench_header_check: clean"),
        "stderr must report clean; got {stderr}"
    );
}

// ─── Acceptance 1: missing headers on required fails ─────────────

#[test]
fn required_fixture_missing_tier_header_fails() {
    let dir = unique_dir("no-tier");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "naked"
fixture = "naked.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#
    );
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        // Has `# category:` and `# inclusion_reason:` but no `# tier:`
        &[("naked.py", concat!(
            "# category: numeric\n",
            "# inclusion_reason: synthetic\n",
            "print('INTERNAL_TIME_NS=1')\n",
        ))],
    );
    let (code, payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "missing tier header must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(
        v.iter().any(|item| item["reason"]
            .as_str()
            .unwrap()
            .contains("tier:")),
        "violation must name the missing tier header; got {payload}"
    );
}

#[test]
fn required_fixture_missing_category_header_fails() {
    let dir = unique_dir("no-category");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "naked"
fixture = "naked.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#
    );
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        &[("naked.py", concat!(
            "# tier: required\n",
            "# inclusion_reason: synthetic\n",
            "print('INTERNAL_TIME_NS=1')\n",
        ))],
    );
    let (code, payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "missing category header must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(v.iter().any(|item| item["reason"]
        .as_str()
        .unwrap()
        .contains("category:")));
}

#[test]
fn required_fixture_missing_inclusion_reason_header_fails() {
    let dir = unique_dir("no-reason");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "naked"
fixture = "naked.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#
    );
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        &[("naked.py", concat!(
            "# tier: required\n",
            "# category: numeric\n",
            "print('INTERNAL_TIME_NS=1')\n",
        ))],
    );
    let (code, payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "missing inclusion_reason header must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(v.iter().any(|item| item["reason"]
        .as_str()
        .unwrap()
        .contains("inclusion_reason:")));
}

#[test]
fn required_fixture_header_tier_mismatch_fails() {
    let dir = unique_dir("tier-mismatch");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "lies"
fixture = "lies.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#
    );
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        // Header says exploratory; manifest says required.
        &[("lies.py", concat!(
            "# tier: exploratory\n",
            "# category: numeric\n",
            "# inclusion_reason: drifted from manifest\n",
            "print('INTERNAL_TIME_NS=1')\n",
        ))],
    );
    let (code, payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 1, "tier mismatch must gate");
    let v = payload["violations"].as_array().unwrap();
    assert!(v.iter().any(|item| item["reason"]
        .as_str()
        .unwrap()
        .contains("does not match manifest tier")));
}

#[test]
fn exploratory_fixture_missing_headers_does_not_gate() {
    let dir = unique_dir("explore-missing");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "ok"
fixture = "ok.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[[benchmarks]]
id = "legacy"
fixture = "legacy.py"
category = "numeric"
tier = "exploratory"
timing_mode = "process_wall"

[update]
location = "x"
command = "y"
"#
    );
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        &[
            ("ok.py", concat!(
                "# tier: required\n",
                "# category: numeric\n",
                "# inclusion_reason: synthetic\n",
                "print('INTERNAL_TIME_NS=1')\n",
            )),
            ("legacy.py", "x = 1\n"),
        ],
    );
    let (code, _payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "exploratory entries are not required to declare headers"
    );
}

// ─── Acceptance 2: harness output includes tier per benchmark ────

#[test]
fn json_output_includes_parsed_tier_category_and_inclusion_reason() {
    let (code, payload) = run_checker_json(&[]);
    assert_eq!(code, 0, "shipped manifest must pass; payload={payload}");
    let entries = payload["entries"].as_array().unwrap();
    assert!(!entries.is_empty());
    for e in entries {
        assert!(e.get("id").is_some());
        assert!(e.get("tier").is_some());
        assert!(e.get("tier_header").is_some(),
            "every entry must surface parsed tier_header (acceptance #2); got {e}");
        assert!(e.get("category_header").is_some());
        assert!(e.get("inclusion_reason_header").is_some());
    }
}

#[test]
fn json_output_distinguishes_required_from_exploratory_entries() {
    let (_code, payload) = run_checker_json(&[]);
    let entries = payload["entries"].as_array().unwrap();
    let required: Vec<&Value> = entries
        .iter()
        .filter(|e| e["tier"] == "required")
        .collect();
    let exploratory: Vec<&Value> = entries
        .iter()
        .filter(|e| e["tier"] == "exploratory")
        .collect();
    assert!(!required.is_empty(), "must have required cohort in output");
    assert!(
        !exploratory.is_empty(),
        "must have exploratory cohort in output"
    );
}

// ─── Acceptance 3: comment-based migration ───────────────────────

#[test]
fn headers_are_pure_comment_lines_in_shipped_fixtures() {
    // Acceptance #3: "Existing comments can be migrated without
    // changing benchmark code." The fixture is comment-based, so
    // adding the headers does not introduce executable statements.
    for fixture in &["int_sum.py", "range_sum.py", "fib30.py", "generator_sum.py"] {
        let body = std::fs::read_to_string(
            crate::common::project_root()
                .join("tests/cpython/_regression/core/bench")
                .join(fixture),
        )
        .expect("read shipped fixture");
        // Find each header line and assert it starts with '#'.
        for kw in &["# tier:", "# category:", "# inclusion_reason:"] {
            if let Some(start) = body.find(kw) {
                let line_start = body[..start]
                    .rfind('\n')
                    .map(|i| i + 1)
                    .unwrap_or(0);
                let line = &body[line_start..start + kw.len()];
                assert!(
                    line.trim_start().starts_with('#'),
                    "{fixture}: header {kw} must be on a comment line; line={line:?}"
                );
            }
        }
    }
}

#[test]
fn headers_can_appear_anywhere_in_the_file() {
    let dir = unique_dir("late-headers");
    let body = format!(
        r#"{MIN_VALID_MANIFEST_HEADER}
[[benchmarks]]
id = "ok"
fixture = "late.py"
category = "numeric"
tier = "required"
timing_mode = "internal"

[update]
location = "x"
command = "y"
"#
    );
    // Headers buried at the bottom of the file should still parse —
    // they are comment lines so Python ignores them in execution.
    let path = write_manifest_and_fixtures(
        &dir,
        &body,
        &[("late.py", concat!(
            "print('INTERNAL_TIME_NS=1')\n",
            "x = 1\n",
            "# Notes\n",
            "# tier: required\n",
            "# category: numeric\n",
            "# inclusion_reason: late headers are still valid (acceptance #3)\n",
        ))],
    );
    let (code, _payload) =
        run_checker_json(&["--manifest", path.to_str().unwrap()]);
    assert_eq!(code, 0, "headers anywhere in the file must validate");
}

// ─── Robustness ──────────────────────────────────────────────────

#[test]
fn checker_exits_101_when_manifest_missing() {
    let (code, _stdout, stderr) = run_checker(&[
        "--manifest",
        "/tmp/perf-bench-header-does-not-exist.toml",
    ]);
    assert_eq!(code, 101, "missing manifest must exit 101");
    assert!(
        stderr.contains("manifest missing"),
        "stderr must name missing manifest; got {stderr}"
    );
}

#[test]
fn checker_help_documents_manifest_and_format() {
    let (code, stdout, _stderr) = run_checker(&["--help"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("--manifest"));
    assert!(stdout.contains("--format"));
}

#[test]
fn text_output_lists_every_entry_with_its_header_tier() {
    let (_code, _stdout, stderr) = run_checker(&["--format", "text"]);
    // Acceptance #2: text output also surfaces tier per benchmark.
    assert!(
        stderr.contains("header_tier="),
        "text output must include `header_tier=`; got {stderr}"
    );
}
