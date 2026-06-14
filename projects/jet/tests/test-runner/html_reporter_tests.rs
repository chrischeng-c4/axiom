// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-tests.md#tests
// CODEGEN-BEGIN
//! Integration + unit tests for the HTML reporter.
//!
//! Covers T1-T9 from the spec Test Plan:
//! `.aw/changes/enhancement-html-reporter-for-native-test-runner/specs/
//!  enhancement-html-reporter-for-native-test-runner-spec.md`

use jet::reporter::html::HtmlReporter;
use jet::reporter::merge::merge_reports;
use jet::reporter::parser::parse_ndjson;
use jet::test_runner::config::Reporter;
use jet::test_runner::reporter::{Outcome, TestError, TestReport};
use std::path::PathBuf;
use tempfile::TempDir;

// ── Helper ────────────────────────────────────────────────────────────────────

fn make_report(name: &str, outcome: Outcome, duration_ms: u64) -> TestReport {
    TestReport {
        file: PathBuf::from("src/foo.spec.ts"),
        suite: vec!["Suite".to_string()],
        name: name.to_string(),
        outcome,
        duration_ms,
        error: None,
        trace_path: None,
        shard_index: None,
        shard_total: None,
        artifacts: Vec::new(),
    }
}

fn make_report_failed(name: &str, stack: &str, diff: Option<&str>) -> TestReport {
    TestReport {
        file: PathBuf::from("src/foo.spec.ts"),
        suite: vec![],
        name: name.to_string(),
        outcome: Outcome::Failed,
        duration_ms: 10,
        error: Some(TestError {
            message: "Expected 1 to be 2".to_string(),
            stack: Some(stack.to_string()),
            diff: diff.map(str::to_string),
            source_location: None,
        }),
        trace_path: None,
        shard_index: None,
        shard_total: None,
        artifacts: Vec::new(),
    }
}

// ── T1: HtmlReporter emits index.html + asset files ──────────────────────────

/// T1: After emitting 3 TestReports and calling finalize(), index.html,
/// report.js, and report.css all exist and index.html contains test names.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
#[test]
fn test_reporter_emits_index_html() {
    let tmp = TempDir::new().unwrap();
    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
    reporter.emit(make_report("passes correctly", Outcome::Passed, 5));
    reporter.emit(make_report_failed(
        "fails with message",
        "at foo.ts:10",
        None,
    ));
    reporter.emit(make_report("skipped test", Outcome::Skipped, 0));
    reporter.finalize().unwrap();

    let report_dir = tmp.path().join("report");
    assert!(
        report_dir.join("index.html").exists(),
        "index.html must exist"
    );
    assert!(
        report_dir.join("report.js").exists(),
        "report.js must exist"
    );
    assert!(
        report_dir.join("report.css").exists(),
        "report.css must exist"
    );

    let html = std::fs::read_to_string(report_dir.join("index.html")).unwrap();
    assert!(
        html.contains("passes correctly"),
        "html must contain first test name"
    );
    assert!(
        html.contains("fails with message"),
        "html must contain second test name"
    );
    assert!(
        html.contains("skipped test"),
        "html must contain third test name"
    );
}

// ── T2: Aggregate stats are rendered ─────────────────────────────────────────

/// T2: Stats panel shows correct total/passed/failed/skipped counts.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
#[test]
fn test_aggregate_stats_rendered() {
    let tmp = TempDir::new().unwrap();
    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
    reporter.emit(make_report("t1", Outcome::Passed, 10));
    reporter.emit(make_report("t2", Outcome::Passed, 20));
    reporter.emit(make_report_failed("t3", "stack", None));
    reporter.emit(make_report("t4", Outcome::Skipped, 0));
    reporter.finalize().unwrap();

    let html = std::fs::read_to_string(tmp.path().join("report/index.html")).unwrap();

    // Stats tile values should appear in the rendered output.
    // Total = 4
    assert!(html.contains(">4<"), "total count 4 must appear");
    // Passed = 2
    assert!(html.contains(">2<"), "passed count 2 must appear");
    // Failed = 1
    assert!(html.contains(">1<"), "failed count 1 must appear");
}

// ── T3: Test row contains required fields ─────────────────────────────────────

/// T3: Each rendered row contains: name, status badge class, duration.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
#[test]
fn test_test_row_contains_required_fields() {
    let tmp = TempDir::new().unwrap();
    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
    let mut rep = make_report("row-target", Outcome::Passed, 42);
    rep.file = PathBuf::from("src/target.spec.ts");
    reporter.emit(rep);
    reporter.finalize().unwrap();

    let html = std::fs::read_to_string(tmp.path().join("report/index.html")).unwrap();

    assert!(html.contains("row-target"), "row must contain test name");
    assert!(
        html.contains("badge-passed"),
        "row must contain status badge class"
    );
    assert!(html.contains("42ms"), "row must contain duration");
}

// ── T4: Parser reconstructs TestReports from NDJSON ──────────────────────────

/// T4: `parse_ndjson` reads a small NDJSON sample, returning the correct
/// count and test names.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R8
#[test]
fn test_parser_parses_ndjson() {
    let ndjson = r#"{"kind":"test_end","id":"a1","suite":["math"],"name":"adds","outcome":"passed","duration_ms":5,"error":null}
{"kind":"test_end","id":"a2","suite":[],"name":"subtracts","outcome":"failed","duration_ms":10,"error":{"message":"Expected 2 to be 3","stack":"at spec.ts:5","diff":"-2\n+3"}}
{"kind":"plan","file":"spec.ts","tests":[]}
"#;

    let reports = parse_ndjson(ndjson.as_bytes()).unwrap();
    // plan event is skipped; 2 testEnd events parsed
    assert_eq!(reports.len(), 2, "expected 2 TestReport entries");
    assert_eq!(reports[0].name, "adds");
    assert_eq!(reports[1].name, "subtracts");
    assert!(matches!(reports[0].outcome, Outcome::Passed));
    assert!(matches!(reports[1].outcome, Outcome::Failed));
    // Stack and diff are preserved.
    let err = reports[1].error.as_ref().unwrap();
    assert_eq!(err.stack.as_deref(), Some("at spec.ts:5"));
    assert_eq!(err.diff.as_deref(), Some("-2\n+3"));
}

// ── T5: Reporter flag parsing ─────────────────────────────────────────────────

/// T5: `Reporter::parse_list("list,html")` → two variants; `"html"` alone
/// produces one element.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R5
#[test]
fn test_reporter_flag_parses() {
    let kinds = Reporter::parse_list("list,html").unwrap();
    assert_eq!(kinds.len(), 2);
    assert!(kinds.contains(&Reporter::Term));
    assert!(kinds.contains(&Reporter::Html));

    let single = Reporter::parse_list("html").unwrap();
    assert_eq!(single.len(), 1);
    assert!(single.contains(&Reporter::Html));

    let three = Reporter::parse_list("term,json,html").unwrap();
    assert_eq!(three.len(), 3);

    let bad = Reporter::parse_list("unknown");
    assert!(bad.is_err(), "unknown reporter should be an error");
}

// ── T6: Merge deduplicates by test_id ────────────────────────────────────────

/// T6: When two shard dirs have overlapping test_id, the merged output
/// contains only one row per id.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
#[test]
fn test_merge_dedupes_by_test_id() {
    let tmp = TempDir::new().unwrap();
    let shard1 = tmp.path().join("shard1");
    let shard2 = tmp.path().join("shard2");
    let merged = tmp.path().join("merged");

    // Write NDJSON sidecars manually to both shard dirs.
    std::fs::create_dir_all(&shard1).unwrap();
    std::fs::create_dir_all(&shard2).unwrap();

    // shard1: rows A and B
    std::fs::write(
        shard1.join("results.ndjson"),
        r#"{"test_id":"id-aaa","name":"alpha","status":"passed","duration_ms":5,"file":"a.spec.ts"}
{"test_id":"id-bbb","name":"beta","status":"passed","duration_ms":6,"file":"b.spec.ts"}"#,
    )
    .unwrap();

    // shard2: rows B (duplicate) and C
    std::fs::write(
        shard2.join("results.ndjson"),
        r#"{"test_id":"id-bbb","name":"beta","status":"passed","duration_ms":6,"file":"b.spec.ts"}
{"test_id":"id-ccc","name":"gamma","status":"failed","duration_ms":7,"file":"c.spec.ts"}"#,
    )
    .unwrap();

    merge_reports(&[shard1, shard2], &merged).unwrap();

    let html = std::fs::read_to_string(merged.join("index.html")).unwrap();
    // Exactly one visible row per test name. Each row renders as
    // `<td class="test-name">NAME</td>`; the JSON data island also
    // contains `"name":"NAME"` so we assert the visible-row count only.
    assert_eq!(
        html.matches(r#"test-name">alpha<"#).count(),
        1,
        "alpha should have one visible row"
    );
    assert_eq!(
        html.matches(r#"test-name">gamma<"#).count(),
        1,
        "gamma should have one visible row"
    );
    // beta appears in both shards (last-writer wins), still one visible row.
    assert_eq!(
        html.matches(r#"test-name">beta<"#).count(),
        1,
        "beta should have one visible row after dedup"
    );
}

// ── T7: Merge shard info is aggregated ───────────────────────────────────────

/// T7: Merged HTML shows how many shards were merged.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
#[test]
fn test_merge_shard_info_aggregated() {
    let tmp = TempDir::new().unwrap();
    let s1 = tmp.path().join("s1");
    let s2 = tmp.path().join("s2");
    let out = tmp.path().join("out");

    std::fs::create_dir_all(&s1).unwrap();
    std::fs::create_dir_all(&s2).unwrap();

    std::fs::write(
        s1.join("results.ndjson"),
        r#"{"test_id":"t1","name":"test1","status":"passed","duration_ms":1,"file":"x.spec.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        s2.join("results.ndjson"),
        r#"{"test_id":"t2","name":"test2","status":"passed","duration_ms":2,"file":"y.spec.ts"}"#,
    )
    .unwrap();

    merge_reports(&[s1, s2], &out).unwrap();

    let html = std::fs::read_to_string(out.join("index.html")).unwrap();
    // Shard info should mention "2" shards.
    assert!(
        html.contains("Shard") && html.contains('2'),
        "merged HTML should reference 2 shards; got: {}",
        &html[..html.len().min(500)]
    );
}

// ── T8: Deterministic output ──────────────────────────────────────────────────

/// T8: Rendering the same set of reports twice produces byte-identical HTML.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R9
#[test]
fn test_deterministic_output() {
    let reports = vec![
        make_report("gamma", Outcome::Passed, 30),
        make_report("alpha", Outcome::Skipped, 0),
        make_report_failed("beta", "at x.ts:1", Some("-1\n+2")),
    ];

    let render = || {
        let tmp = TempDir::new().unwrap();
        let mut r = HtmlReporter::new(tmp.path().join("r"));
        for rep in &reports {
            r.emit(rep.clone());
        }
        r.finalize().unwrap();
        std::fs::read_to_string(tmp.path().join("r/index.html")).unwrap()
    };

    let first = render();
    let second = render();
    assert_eq!(
        first, second,
        "deterministic: identical input must produce identical HTML"
    );
}

// ── T9: Trace link is present in row when trace_path is set ──────────────────

/// T9: A TestReport with `trace_path = Some(...)` produces a `?trace=` or
/// `href=` reference in the rendered HTML row.
///
// @spec enhancement-html-reporter-for-native-test-runner-spec#R10
#[test]
fn test_trace_link_reference() {
    let tmp = TempDir::new().unwrap();
    let mut reporter = HtmlReporter::new(tmp.path().join("report"));

    let mut rep = make_report_failed("trace-test", "at bar.ts:5", None);
    rep.trace_path = Some(PathBuf::from("test-results/traces/trace-test.zip"));
    reporter.emit(rep);
    reporter.finalize().unwrap();

    let html = std::fs::read_to_string(tmp.path().join("report/index.html")).unwrap();

    // The trace path must appear as a trace link or query param.
    assert!(
        html.contains("trace-test.zip") || html.contains("?trace="),
        "html must contain trace reference: {}",
        &html[..html.len().min(1000)]
    );
    // The "View trace" text must be in the row.
    assert!(
        html.contains("View trace"),
        "html must contain 'View trace' button"
    );
}
// CODEGEN-END
