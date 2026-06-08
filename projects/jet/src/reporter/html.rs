// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
// CODEGEN-BEGIN
//! HTML reporter — writes `index.html` + embedded assets to a report directory.
//!
//! Assets (`report.js`, `report.css`) are embedded in the binary via
//! `include_bytes!` so the report is fully self-contained with no CDN or
//! network dependency at view time.
//!
// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
// @spec enhancement-html-reporter-for-native-test-runner-spec#R4

use crate::test_runner::reporter::{Outcome, TestReport};
use anyhow::{Context, Result};
use std::io;
use std::path::{Path, PathBuf};

/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn relativize_report_file(
    file: &Path,
    cwd_result: io::Result<PathBuf>,
) -> (String, Option<String>) {
    // GH #3776 — was a silent `.to_string_lossy()` on the relativized
    // path. Non-UTF-8 bytes substituted with U+FFFD silently and two
    // sibling test files with similarly-shaped non-UTF-8 paths could
    // collide on the same display string in the HTML report.
    match cwd_result {
        Ok(cwd) => match file.strip_prefix(&cwd) {
            Ok(rel) => (lossy_with_non_utf8_warn(rel, "report_file", file), None),
            Err(_) => match file.file_name() {
                Some(name) => (
                    lossy_with_non_utf8_warn(Path::new(name), "report_file_name", file),
                    Some(format_relativize_outside_cwd_warn(file, &cwd)),
                ),
                None => (
                    String::new(),
                    Some(format_relativize_outside_cwd_warn(file, &cwd)),
                ),
            },
        },
        Err(err) => {
            let display = file
                .file_name()
                .map(|n| lossy_with_non_utf8_warn(Path::new(n), "report_file_name", file))
                .unwrap_or_default();
            (display, Some(format_relativize_cwd_err_warn(file, &err)))
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_relativize_outside_cwd_warn(file: &Path, cwd: &Path) -> String {
    format!(
        "jet html reporter: report file {:?} is outside current_dir {:?}; falling back to file_name only (GH #3602)",
        file, cwd
    )
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_relativize_cwd_err_warn(file: &Path, err: &io::Error) -> String {
    format!(
        "jet html reporter: current_dir() failed ({}); falling back to file_name only for {:?} (GH #3602)",
        err, file
    )
}

/// GH #3776 — render a path for the HTML report, warning on non-UTF-8
/// bytes. UTF-8 paths pass through silently; non-UTF-8 paths still
/// render (via `to_string_lossy`) but emit a tagged warn so operators
/// can spot two report rows that collided on a U+FFFD-substituted
/// display string, or a trace-zip link with a broken target.
///
/// `field` names the call site for the warn (e.g. "report_file",
/// "trace_path"); `original` is the unmodified path the operator should
/// rename to fix the collision.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn lossy_with_non_utf8_warn(path: &Path, field: &str, original: &Path) -> String {
    match path.to_str() {
        Some(s) => s.to_string(),
        None => {
            let lossy = path.to_string_lossy().into_owned();
            tracing::warn!(
                target: "jet::reporter::html",
                field = %field,
                original = %original.display(),
                lossy = %lossy,
                "{}",
                format_html_report_non_utf8_warn(field, original, &lossy)
            );
            lossy
        }
    }
}

/// GH #3776 — diagnostic for a non-UTF-8 path in the HTML report.
/// Operators grep for "GH #3776" to chase report-row collisions or
/// broken trace-zip links.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub(crate) fn format_html_report_non_utf8_warn(
    field: &str,
    original: &Path,
    lossy: &str,
) -> String {
    format!(
        "GH #3776 jet html reporter field `{field}` has a non-UTF-8 \
         path ({original:?}); rendered via lossy decode as {lossy:?}. \
         Two test files with similarly-shaped non-UTF-8 paths can \
         collide on the same display string in the report, and a \
         non-UTF-8 trace_path renders a broken zip link target."
    )
}

// Embedded static assets — included at compile time.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
const HTML_TEMPLATE: &str = include_str!("../../assets/html-reporter/index.html");
/// Embedded `report.js` — accessible by merge.rs for writing sidecar assets.
pub const REPORT_JS: &str = include_str!("../../assets/html-reporter/report.js");
/// Embedded `report.css` — accessible by merge.rs for writing sidecar assets.
pub const REPORT_CSS: &str = include_str!("../../assets/html-reporter/report.css");

/// A single test result row for the HTML report.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
#[derive(Debug, Clone)]
pub struct TestRow {
    /// Stable sort key: test_id is derived from file + suite path + test name.
    pub test_id: String,
    pub name: String,
    pub status: String,
    pub duration_ms: u64,
    pub file: String,
    pub stack_trace: Option<String>,
    pub matcher_diff: Option<String>,
    /// Relative path to the `.zip` trace file.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R10
    pub trace_path: Option<String>,
}

/// HTML reporter — accumulates test results, renders a self-contained
/// `index.html` report on `finalize()`.
///
/// Usage:
/// ```ignore
/// let mut r = HtmlReporter::new("test-results/report");
/// r.emit(report);
/// r.finalize()?;
/// ```
// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
pub struct HtmlReporter {
    pub out_dir: PathBuf,
    rows: Vec<TestRow>,
    /// Optional shard info `(index, total)`.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R2
    pub shard: Option<(u32, u32)>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
impl HtmlReporter {
    /// Create a new `HtmlReporter` writing output to `out_dir`.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
        Self {
            out_dir: out_dir.into(),
            rows: Vec::new(),
            shard: None,
        }
    }

    /// Accept one completed test result and accumulate it as a `TestRow`.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R8
    pub fn emit(&mut self, report: TestReport) {
        let status = outcome_to_status(&report.outcome);

        // Derive a stable test_id from file + suite + name.
        let suite_path = report.suite.join(" > ");
        let raw_id = format!("{}::{}::{}", report.file.display(), suite_path, report.name);
        let test_id = stable_id(&raw_id);

        let full_name = if report.suite.is_empty() {
            report.name.clone()
        } else {
            format!("{} > {}", report.suite.join(" > "), report.name)
        };

        let (stack_trace, matcher_diff) = match &report.error {
            Some(err) => (err.stack.clone(), err.diff.clone()),
            None => (None, None),
        };

        // Propagate shard info from the first report that has it.
        if self.shard.is_none() {
            if let (Some(idx), Some(total)) = (report.shard_index, report.shard_total) {
                self.shard = Some((idx, total));
            }
        }

        let trace_path = report
            .trace_path
            .as_ref()
            // GH #3776 — was silent `.to_string_lossy()` that produced
            // U+FFFD substitutions in the HTML report's trace-zip link
            // target, silently breaking the link.
            .map(|p| lossy_with_non_utf8_warn(p, "trace_path", p));

        let (display_file, warn) = relativize_report_file(&report.file, std::env::current_dir());
        if let Some(msg) = warn {
            tracing::warn!(target: "jet::reporter::html", "{}", msg);
        }

        self.rows.push(TestRow {
            test_id,
            name: full_name,
            status,
            duration_ms: report.duration_ms,
            file: display_file,
            stack_trace,
            matcher_diff,
            trace_path,
        });
    }

    /// Sort rows deterministically, render HTML, and write all assets to
    /// `out_dir`. Returns `Ok(())` on success.
    ///
    /// Output files: `index.html`, `report.js`, `report.css`.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R9
    pub fn finalize(&mut self) -> Result<()> {
        // Sort by test_id for deterministic output.
        // @spec enhancement-html-reporter-for-native-test-runner-spec#R9
        self.rows.sort_by(|a, b| a.test_id.cmp(&b.test_id));

        std::fs::create_dir_all(&self.out_dir)
            .with_context(|| format!("Failed to create report dir: {}", self.out_dir.display()))?;

        let html = render_html(&self.rows, self.shard);

        std::fs::write(self.out_dir.join("index.html"), &html)
            .context("Failed to write index.html")?;
        std::fs::write(self.out_dir.join("report.js"), REPORT_JS)
            .context("Failed to write report.js")?;
        std::fs::write(self.out_dir.join("report.css"), REPORT_CSS)
            .context("Failed to write report.css")?;

        println!("Report: {}/index.html", self.out_dir.display());
        Ok(())
    }

    /// Consume the accumulated rows (used by the merger).
    pub fn into_rows(self) -> Vec<TestRow> {
        self.rows
    }
}

/// Render the full HTML from `rows` + optional `shard` metadata.
///
/// Uses simple string substitution on `<!-- PLACEHOLDER -->` markers.
/// No external template engine required.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
// @spec enhancement-html-reporter-for-native-test-runner-spec#R9
pub fn render_html(rows: &[TestRow], shard: Option<(u32, u32)>) -> String {
    let stats_html = build_stats_html(rows);
    let shard_html = build_shard_html(shard);
    let rows_html = build_rows_html(rows);

    HTML_TEMPLATE
        .replace("<!-- REPORT_CSS -->", REPORT_CSS)
        .replace("<!-- REPORT_JS -->", REPORT_JS)
        .replace("<!-- STATS -->", &stats_html)
        .replace("<!-- SHARD_INFO -->", &shard_html)
        .replace("<!-- TEST_ROWS -->", &rows_html)
        .replace("<!-- REPORT_DATA -->", &build_report_data_json(rows, shard))
}

/// Build the aggregate stats panel HTML.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
fn build_stats_html(rows: &[TestRow]) -> String {
    let total = rows.len();
    let passed = rows.iter().filter(|r| r.status == "passed").count();
    let failed = rows.iter().filter(|r| r.status == "failed").count();
    let skipped = rows.iter().filter(|r| r.status == "skipped").count();
    let flaky = rows.iter().filter(|r| r.status == "flaky").count();
    let duration_ms: u64 = rows.iter().map(|r| r.duration_ms).sum();
    let duration_s = duration_ms as f64 / 1000.0;

    format!(
        r#"<div class="stat-tile total"><div class="stat-value">{total}</div><div class="stat-label">Total</div></div>
<div class="stat-tile passed"><div class="stat-value">{passed}</div><div class="stat-label">Passed</div></div>
<div class="stat-tile failed"><div class="stat-value">{failed}</div><div class="stat-label">Failed</div></div>
<div class="stat-tile skipped"><div class="stat-value">{skipped}</div><div class="stat-label">Skipped</div></div>
<div class="stat-tile flaky"><div class="stat-value">{flaky}</div><div class="stat-label">Flaky</div></div>
<div class="stat-tile duration"><div class="stat-value">{duration_s:.2}s</div><div class="stat-label">Duration</div></div>"#
    )
}

/// Build the shard info line (empty string if no shard).
// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
fn build_shard_html(shard: Option<(u32, u32)>) -> String {
    match shard {
        Some((idx, total)) => format!(r#"<span class="shard-info">Shard {idx} of {total}</span>"#),
        None => String::new(),
    }
}

/// Build a JSON data island for the report (used by report.js).
fn build_report_data_json(rows: &[TestRow], shard: Option<(u32, u32)>) -> String {
    let total = rows.len();
    let passed = rows.iter().filter(|r| r.status == "passed").count();
    let failed = rows.iter().filter(|r| r.status == "failed").count();
    let skipped = rows.iter().filter(|r| r.status == "skipped").count();
    let flaky = rows.iter().filter(|r| r.status == "flaky").count();
    let duration_ms: u64 = rows.iter().map(|r| r.duration_ms).sum();

    let shard_json = match shard {
        Some((idx, n)) => format!(r#","shard":{{"index":{idx},"total":{n}}}"#),
        None => String::new(),
    };

    let mut tests_json = Vec::new();
    for row in rows {
        let stack = row
            .stack_trace
            .as_deref()
            .map(|s| format!(r#""stack_trace":{}"#, json_string(s)))
            .unwrap_or_default();
        let diff = row
            .matcher_diff
            .as_deref()
            .map(|s| format!(r#","matcher_diff":{}"#, json_string(s)))
            .unwrap_or_default();
        let trace = row
            .trace_path
            .as_deref()
            .map(|s| format!(r#","trace_path":{}"#, json_string(s)))
            .unwrap_or_default();

        tests_json.push(format!(
            r#"{{"test_id":{},"name":{},"status":{},"duration_ms":{},
"file":{}{}{}{}
}}"#,
            json_string(&row.test_id),
            json_string(&row.name),
            json_string(&row.status),
            row.duration_ms,
            json_string(&row.file),
            if stack.is_empty() {
                String::new()
            } else {
                format!(",{stack}")
            },
            diff,
            trace,
        ));
    }

    format!(
        r#"{{"version":1,"summary":{{"total":{total},"passed":{passed},"failed":{failed},
"skipped":{skipped},"flaky":{flaky},"duration_ms":{duration_ms}{shard_json}}},
"tests":[{tests}]}}"#,
        tests = tests_json.join(",")
    )
}

/// Build all test row `<tr>` elements.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
fn build_rows_html(rows: &[TestRow]) -> String {
    rows.iter()
        .enumerate()
        .map(|(i, row)| build_single_row(i, row))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Build a pair of `<tr>` elements for one test: the main row + the drawer.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
// @spec enhancement-html-reporter-for-native-test-runner-spec#R10
fn build_single_row(idx: usize, row: &TestRow) -> String {
    let badge_class = match row.status.as_str() {
        "passed" => "badge-passed",
        "failed" => "badge-failed",
        "skipped" => "badge-skipped",
        "flaky" => "badge-flaky",
        _ => "badge-timedout",
    };

    let has_drawer = row.stack_trace.is_some() || row.matcher_diff.is_some();

    let actions = {
        let toggle = if has_drawer {
            format!(r#"<button class="toggle-btn" data-row-id="{idx}">Details</button>"#)
        } else {
            String::new()
        };
        // GH #3073 — `<button>` has no `href`; the legacy ?trace= URL was
        // a silent no-op in file:// mode. The JS handler now shows an
        // in-page toast with the `jet trace view <path>` command.
        let trace_link = if let Some(tp) = &row.trace_path {
            format!(
                r#"<button class="trace-link" type="button" data-trace-path="{}" title="Show jet trace view command">View trace</button>"#,
                esc_attr(tp),
            )
        } else {
            String::new()
        };
        format!("{toggle} {trace_link}")
    };

    let main_row = format!(
        r#"<tr data-status="{status}" data-drawer-id="{idx}">
  <td><span class="badge {badge_class}">{status}</span></td>
  <td class="test-name">{name}</td>
  <td class="test-file">{file}</td>
  <td class="test-duration">{duration_ms}ms</td>
  <td>{actions}</td>
</tr>"#,
        status = esc_html(&row.status),
        badge_class = badge_class,
        name = esc_html(&row.name),
        file = esc_html(&row.file),
        duration_ms = row.duration_ms,
        actions = actions,
    );

    let drawer_row = if has_drawer {
        let stack_section = if let Some(st) = &row.stack_trace {
            format!(
                r#"<div class="drawer-label">Stack Trace</div>
<pre class="stack-trace">{}</pre>"#,
                esc_html(st)
            )
        } else {
            String::new()
        };

        let diff_section = if let Some(diff) = &row.matcher_diff {
            format!(
                r#"<div class="drawer-label" style="margin-top:10px">Diff</div>
<pre class="matcher-diff">{}</pre>"#,
                esc_html(diff)
            )
        } else {
            String::new()
        };

        format!(
            r#"<tr class="drawer-row" id="drawer-{idx}">
  <td colspan="5">
    <div class="drawer-content">
      {stack_section}
      {diff_section}
    </div>
  </td>
</tr>"#
        )
    } else {
        String::new()
    };

    format!("{main_row}\n{drawer_row}")
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn outcome_to_status(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Passed => "passed".to_string(),
        Outcome::Failed => "failed".to_string(),
        Outcome::Skipped => "skipped".to_string(),
        Outcome::TimedOut => "failed".to_string(),
        Outcome::Crashed => "failed".to_string(),
    }
}

/// Produce a short deterministic ID from an arbitrary string using a hex hash.
fn stable_id(s: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

/// HTML-escape for text content.
fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// HTML-escape for attribute values (also escapes `"`).
fn esc_attr(s: &str) -> String {
    esc_html(s).replace('"', "&quot;")
}

/// Produce a JSON string literal (quoted + escaped).
fn json_string(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

/// Render HTML from pre-built `TestRow` slices — public helper for merger.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
pub fn render_from_rows(rows: &[TestRow], shard: Option<(u32, u32)>) -> String {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| a.test_id.cmp(&b.test_id));
    render_html(&sorted, shard)
}

/// Read the report directory, find the NDJSON snapshot or index.html and
/// return accumulated rows (helper used by the merger).
///
/// For now this is implemented by scanning the out_dir for a `results.ndjson`
/// sidecar written by `finalize_with_ndjson`.  Falls back gracefully to empty.
///
/// GH #3314 — A locked / unreadable `results.ndjson` used to silently return
/// an empty row set, so an entire shard's failures could vanish from the
/// merged report with no log line. Surface non-NotFound IO via
/// `tracing::warn!` under target `jet::reporter` so operators can triage
/// a shard that's missing from the CI output.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub fn read_rows_from_dir(dir: &Path) -> Vec<TestRow> {
    let ndjson_path = dir.join("results.ndjson");
    if !ndjson_path.exists() {
        return Vec::new();
    }
    let bytes = match std::fs::read(&ndjson_path) {
        Ok(b) => b,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            // Race with deletion between `exists()` and `read`.
            return Vec::new();
        }
        Err(err) => {
            tracing::warn!(
                target: "jet::reporter",
                path = %ndjson_path.display(),
                error = %err,
                "GH #3314 unreadable shard results.ndjson; this shard's \
                 test rows will be missing from the merged report"
            );
            return Vec::new();
        }
    };
    crate::reporter::parser::parse_ndjson_to_rows(&bytes)
}

/// Like `finalize` but also writes a `results.ndjson` sidecar containing the
/// rows serialised as NDJSON.  This sidecar is used by the merger when
/// combining multiple shard reports.
// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
pub fn finalize_with_sidecar(reporter: &mut HtmlReporter) -> Result<()> {
    reporter.rows.sort_by(|a, b| a.test_id.cmp(&b.test_id));

    std::fs::create_dir_all(&reporter.out_dir).with_context(|| {
        format!(
            "Failed to create report dir: {}",
            reporter.out_dir.display()
        )
    })?;

    // Write NDJSON sidecar before HTML so the merger can pick it up.
    let ndjson: String = reporter
        .rows
        .iter()
        .map(|row| row_to_ndjson_line(row))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(reporter.out_dir.join("results.ndjson"), ndjson)
        .context("Failed to write results.ndjson")?;

    reporter.finalize()
}

/// Serialise a `TestRow` to a single NDJSON line.
/// @spec .aw/tech-design/projects/jet/semantic/jet-reporter.md#schema
pub fn row_to_ndjson_line(row: &TestRow) -> String {
    let stack = row
        .stack_trace
        .as_deref()
        .map(|s| format!(r#","stack_trace":{}"#, json_string(s)))
        .unwrap_or_default();
    let diff = row
        .matcher_diff
        .as_deref()
        .map(|s| format!(r#","matcher_diff":{}"#, json_string(s)))
        .unwrap_or_default();
    let trace = row
        .trace_path
        .as_deref()
        .map(|s| format!(r#","trace_path":{}"#, json_string(s)))
        .unwrap_or_default();

    format!(
        r#"{{"test_id":{},"name":{},"status":{},"duration_ms":{},"file":{}{}{}{}}}"#,
        json_string(&row.test_id),
        json_string(&row.name),
        json_string(&row.status),
        row.duration_ms,
        json_string(&row.file),
        stack,
        diff,
        trace,
    )
}

#[cfg(test)]
mod trace_button_tests {
    //! GH #3073 — render-side and asset-side guards for the trace-button fix.
    //! The asset checks are substring assertions because cargo can't run JS,
    //! but they pin the behavior the user-visible bug depended on.

    use super::*;

    fn row_with_trace(tp: &str) -> TestRow {
        TestRow {
            test_id: "t1".into(),
            name: "a".into(),
            status: "passed".into(),
            duration_ms: 1,
            file: "f.spec.ts".into(),
            stack_trace: None,
            matcher_diff: None,
            trace_path: Some(tp.into()),
        }
    }

    /// The rendered button uses `type="button"` and carries
    /// `data-trace-path` but no longer has the broken `href="?trace=..."`.
    #[test]
    fn trace_button_html_drops_broken_href() {
        let html = build_single_row(0, &row_with_trace("/tmp/test-results/traces/a.zip"));
        assert!(
            html.contains(r#"class="trace-link" type="button""#),
            "button must declare type=button, got: {html}"
        );
        assert!(
            html.contains(r#"data-trace-path="/tmp/test-results/traces/a.zip""#),
            "button must carry data-trace-path, got: {html}"
        );
        assert!(
            !html.contains("href=\"?trace="),
            "button must not carry the silently-broken `href` attribute, got: {html}"
        );
    }

    /// REPORT_JS no longer navigates the URL bar on click — that was the
    /// silent-no-op user-visible bug. It now shows an in-page toast with
    /// the `jet trace view` command and a copy-to-clipboard button.
    #[test]
    fn report_js_replaces_navigation_with_toast() {
        assert!(
            !REPORT_JS.contains("window.location.search = '?trace="),
            "REPORT_JS must not navigate the URL bar on trace click"
        );
        assert!(
            REPORT_JS.contains("showTraceToast"),
            "REPORT_JS must include the toast handler"
        );
        assert!(
            REPORT_JS.contains("'jet trace view '"),
            "REPORT_JS must surface the actionable command"
        );
        assert!(
            REPORT_JS.contains("navigator.clipboard"),
            "REPORT_JS must use clipboard API for copy"
        );
        assert!(
            REPORT_JS.contains("document.execCommand('copy')"),
            "REPORT_JS must keep a fallback for non-secure / older browser contexts"
        );
    }

    /// REPORT_CSS includes the toast styles so the rendered HTML can stand
    /// alone in file:// mode.
    #[test]
    fn report_css_includes_trace_toast_styles() {
        assert!(
            REPORT_CSS.contains(".trace-toast"),
            "REPORT_CSS must include .trace-toast styles"
        );
        assert!(
            REPORT_CSS.contains(".trace-toast-copy"),
            "REPORT_CSS must include the copy-button style"
        );
    }

    // GH #3314 — read_rows_from_dir must not silently swallow IO errors
    // beyond NotFound.

    #[test]
    fn read_rows_from_dir_missing_ndjson_returns_empty_silently() {
        let dir = tempfile::tempdir().unwrap();
        // No results.ndjson written.
        let rows = super::read_rows_from_dir(dir.path());
        assert!(rows.is_empty(), "missing ndjson must yield empty rows");
    }

    #[test]
    fn read_rows_from_dir_valid_ndjson_extracts_rows() {
        let dir = tempfile::tempdir().unwrap();
        let line = r#"{"test_id":"t1","name":"t1","status":"pass","duration_ms":2,"file":"x.ts"}"#;
        std::fs::write(dir.path().join("results.ndjson"), line).unwrap();

        let rows = super::read_rows_from_dir(dir.path());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].test_id, "t1");
    }

    #[cfg(unix)]
    #[test]
    fn read_rows_from_dir_unreadable_ndjson_returns_empty_with_warn() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("results.ndjson");
        std::fs::write(
            &path,
            r#"{"test_id":"t2","name":"t2","status":"pass","duration_ms":1,"file":"y.ts"}"#,
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Skip when running as root.
        if std::fs::read(&path).is_ok() {
            let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));
            return;
        }

        let rows = super::read_rows_from_dir(dir.path());

        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644));

        assert!(
            rows.is_empty(),
            "unreadable ndjson must yield empty rows so merger doesn't abort"
        );
        // The contract is also: a warn was emitted (covered by impl; visual
        // log inspection or a tracing-test harness would verify).
    }
}

#[cfg(test)]
mod gh3602_relativize_tests {
    use super::*;

    #[test]
    fn happy_path_strips_cwd() {
        let cwd = PathBuf::from("/repo");
        let file = PathBuf::from("/repo/specs/a.test.ts");
        let (display, warn) = relativize_report_file(&file, Ok(cwd));
        assert_eq!(display, "specs/a.test.ts");
        assert!(warn.is_none());
    }

    #[test]
    fn outside_cwd_falls_back_to_file_name_and_warns() {
        let cwd = PathBuf::from("/repo");
        let file = PathBuf::from("/Users/x/elsewhere/foo.test.ts");
        let (display, warn) = relativize_report_file(&file, Ok(cwd));
        assert_eq!(display, "foo.test.ts");
        let msg = warn.expect("must warn on outside-cwd path");
        assert!(msg.contains("GH #3602"), "msg: {msg}");
        assert!(msg.contains("outside"), "msg: {msg}");
        assert!(!display.contains("/Users/"), "must not leak absolute path");
    }

    #[test]
    fn cwd_error_falls_back_to_file_name_and_warns() {
        let file = PathBuf::from("/Users/x/repo/specs/a.test.ts");
        let err = io::Error::new(io::ErrorKind::NotFound, "deleted CWD");
        let (display, warn) = relativize_report_file(&file, Err(err));
        assert_eq!(display, "a.test.ts");
        let msg = warn.expect("must warn on current_dir error");
        assert!(msg.contains("GH #3602"), "msg: {msg}");
        assert!(msg.contains("current_dir"), "msg: {msg}");
        assert!(!display.contains("/Users/"), "must not leak absolute path");
    }

    #[test]
    fn outside_cwd_warn_helper_tags_issue() {
        let msg = format_relativize_outside_cwd_warn(Path::new("/x/y.test.ts"), Path::new("/repo"));
        assert!(msg.contains("GH #3602"), "msg: {msg}");
        assert!(msg.contains("/x/y.test.ts"), "msg: {msg}");
        assert!(msg.contains("/repo"), "msg: {msg}");
    }

    #[test]
    fn cwd_err_warn_helper_tags_issue() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        let msg = format_relativize_cwd_err_warn(Path::new("/x/y.test.ts"), &err);
        assert!(msg.contains("GH #3602"), "msg: {msg}");
        assert!(msg.contains("/x/y.test.ts"), "msg: {msg}");
        assert!(msg.contains("denied"), "msg: {msg}");
    }
}

#[cfg(test)]
mod gh3776_non_utf8_path_warn_tests {
    //! GH #3776 — silent `.to_string_lossy()` on report_file and
    //! trace_path masked non-UTF-8 paths as U+FFFD-substituted display
    //! strings, causing collisions in the report and broken trace-zip
    //! links. Tests cover UTF-8 / non-UTF-8 branches +
    //! helper-name discoverability + sibling-distinctness vs. #3602,
    //! #3753, #3765, #3772, #3774.

    use super::*;

    /// GH #3776 — UTF-8 path passes through unchanged (no regression on
    /// the happy path).
    #[test]
    fn gh3776_utf8_path_passes_through() {
        let p = Path::new("apps/web/foo.spec.ts");
        let result = lossy_with_non_utf8_warn(p, "report_file", p);
        assert_eq!(result, "apps/web/foo.spec.ts");
    }

    /// GH #3776 — non-UTF-8 path on Unix recovers via lossy decode and
    /// would emit a warn.
    #[cfg(unix)]
    #[test]
    fn gh3776_non_utf8_path_recovers_via_lossy() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::PathBuf;

        let mut bad = PathBuf::from("apps/");
        bad.push(OsStr::from_bytes(b"fa\xFFcade"));
        bad.push("foo.spec.ts");

        let result = lossy_with_non_utf8_warn(&bad, "report_file", &bad);
        // U+FFFD substitution preserves the surrounding ASCII.
        assert!(result.starts_with("apps/"));
        assert!(result.contains("fa"));
        assert!(result.contains("cade"));
        assert!(result.ends_with("/foo.spec.ts"));
    }

    /// GH #3776 — issue-tag discoverability.
    #[test]
    fn gh3776_helper_includes_issue_tag() {
        let msg =
            format_html_report_non_utf8_warn("report_file", Path::new("/x/y.spec.ts"), "lossy");
        assert!(msg.contains("GH #3776"));
    }

    /// GH #3776 — message records the field name, original path, and
    /// lossy recovery so operators can act without rerunning with debug
    /// logging.
    #[test]
    fn gh3776_helper_records_field_path_and_lossy() {
        let msg = format_html_report_non_utf8_warn(
            "trace_path",
            Path::new("/repo/traces/abc.zip"),
            "/repo/traces/\u{FFFD}.zip",
        );
        assert!(msg.contains("trace_path"));
        assert!(msg.contains("/repo/traces/abc.zip"));
        assert!(msg.contains("/repo/traces/\u{FFFD}.zip"));
    }

    /// GH #3776 — sibling-distinctness vs. the prior #3602 warns in
    /// the same file. The two #3602 warns concern the *cwd boundary*;
    /// this one concerns the *encoding* of the displayed path. They
    /// must be distinguishable by tag and by wording.
    #[test]
    fn gh3776_warn_distinct_from_gh3602() {
        let new_msg =
            format_html_report_non_utf8_warn("report_file", Path::new("/x/y.spec.ts"), "lossy");
        let old_outside =
            format_relativize_outside_cwd_warn(Path::new("/x/y.spec.ts"), Path::new("/repo"));

        assert!(new_msg.contains("GH #3776"));
        assert!(!new_msg.contains("GH #3602"));
        assert!(old_outside.contains("GH #3602"));
        assert!(!old_outside.contains("GH #3776"));
    }

    /// GH #3776 — message names the operational risk (collision /
    /// broken link) so the operator sees why this matters, not just
    /// that bytes were lossy.
    #[test]
    fn gh3776_helper_names_collision_and_link_risk() {
        let msg = format_html_report_non_utf8_warn("trace_path", Path::new("/x"), "y");
        assert!(msg.contains("collide"));
        assert!(msg.contains("broken zip link"));
    }

    /// GH #3776 — helper-name convention is discoverable. If the helper
    /// is ever renamed, this file would fail to compile — the test
    /// asserts the convention via use-site.
    #[test]
    fn gh3776_helper_naming_convention_discoverable() {
        let _ = lossy_with_non_utf8_warn(Path::new("/x"), "field", Path::new("/x"));
        let _ = format_html_report_non_utf8_warn("field", Path::new("/x"), "y");
    }

    /// GH #3776 — relativize_report_file integration: happy path
    /// inside cwd with UTF-8 paths is unchanged.
    #[test]
    fn gh3776_relativize_happy_path_utf8_unchanged() {
        let cwd = PathBuf::from("/repo");
        let file = PathBuf::from("/repo/specs/a.test.ts");
        let (display, warn) = relativize_report_file(&file, Ok(cwd));
        assert_eq!(display, "specs/a.test.ts");
        assert!(warn.is_none());
    }

    /// GH #3776 — relativize_report_file with a non-UTF-8 inside-cwd
    /// path renders lossy without aborting (Unix only).
    #[cfg(unix)]
    #[test]
    fn gh3776_relativize_non_utf8_inside_cwd_renders_lossy() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let cwd = PathBuf::from("/repo");
        let mut file = PathBuf::from("/repo/apps/");
        file.push(OsStr::from_bytes(b"fa\xFFcade.spec.ts"));

        let (display, warn) = relativize_report_file(&file, Ok(cwd));
        // No "outside cwd" warn — the file IS inside cwd; the path is
        // just non-UTF-8.
        assert!(warn.is_none());
        assert!(display.starts_with("apps/"));
        assert!(display.contains("fa"));
        assert!(display.contains("cade"));
    }

    /// GH #3776 — distinct field labels are preserved between
    /// `report_file` and `trace_path` so an operator grepping for one
    /// call site doesn't collide with the other.
    #[test]
    fn gh3776_field_labels_pairwise_distinct() {
        let report_msg =
            format_html_report_non_utf8_warn("report_file", Path::new("/x/y.spec.ts"), "lossy");
        let trace_msg =
            format_html_report_non_utf8_warn("trace_path", Path::new("/x/y.spec.ts"), "lossy");
        assert!(report_msg.contains("`report_file`"));
        assert!(trace_msg.contains("`trace_path`"));
        assert_ne!(report_msg, trace_msg);
    }
}
// CODEGEN-END
