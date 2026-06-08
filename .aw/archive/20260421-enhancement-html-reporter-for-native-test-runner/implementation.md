---
id: implementation
type: change_implementation
change_id: enhancement-html-reporter-for-native-test-runner
---

# Implementation

## Summary

Implements HTML reporter for the native jet test runner (R1-R10).

**New module**: `crates/jet/src/reporter/` (html.rs + parser.rs + merge.rs + mod.rs, ~728 LOC).

**Embedded UI bundle**: `crates/jet/assets/html-reporter/{index.html,report.js,report.css}` is bundled via `include_bytes!` - self-contained, no CDN (R4).

**CLI**: `jet test --reporter=html` / `--reporter=list,html` (R5); `jet report view <dir>` and `jet report merge --input ... --output ...` (R6, R7).

**Wire-protocol reuse**: Reporter consumes existing NDJSON events (R8).

**Determinism**: Rows sorted by stable `test_id` hash - byte-identical HTML for identical input (R9).

**Type change**: `TestReport.error` promoted from `Option<String>` to `Option<TestError { message, stack, diff }>` so the HTML reporter can render stack trace + matcher diff distinctly (R3). All internal call sites in worker.rs + worker_pool.rs updated.

**Test Plan**: 9 integration tests T1-T9 in `crates/jet/tests/html_reporter_tests.rs`, all passing.


## Diff

```diff
diff --git a/.score/issues/open/enhancement-html-reporter-for-native-test-runner.md b/.score/issues/open/enhancement-html-reporter-for-native-test-runner.md
index c92fc299..7f39fe14 100644
--- a/.score/issues/open/enhancement-html-reporter-for-native-test-runner.md
+++ b/.score/issues/open/enhancement-html-reporter-for-native-test-runner.md
@@ -7,8 +7,16 @@ labels:
 - crate:jet,priority:p1
 - type:enhancement
 created_at: 2026-04-21T03:21:40.366297+00:00
-updated_at: 2026-04-21T03:28:36.122075+00:00
-phase: merged
+updated_at: 2026-04-21T06:51:39.432836+00:00
+phase: change_implementation_created
+branch: cclab/enhancement-html-reporter-for-native-test-runner
+git_workflow: worktree
+change_id: enhancement-html-reporter-for-native-test-runner
+iteration: 1
+current_task_id: enhancement-html-reporter-for-native-test-runner-spec
+impl_spec_phase: {}
+task_revisions: {}
+revision_counts: {}
 ---
 
 
@@ -21,6 +29,15 @@ phase: merged
 
 
 
+
+
+
+
+
+
+
+
+
 
 
 
diff --git a/crates/jet/src/cli.rs b/crates/jet/src/cli.rs
index 6c2f1cb5..d7b6dee4 100644
--- a/crates/jet/src/cli.rs
+++ b/crates/jet/src/cli.rs
@@ -277,6 +277,43 @@ pub fn command() -> Command {
                         .help("Arguments"),
                 ),
         )
+        .subcommand(
+            // @spec enhancement-html-reporter-for-native-test-runner-spec#R6
+            Command::new("report")
+                .about("Commands for managing HTML test reports")
+                .subcommand(
+                    Command::new("view")
+                        .about("Open a report directory in the system default browser")
+                        .arg(
+                            Arg::new("dir")
+                                .required(true)
+                                .help("Path to a report directory containing index.html"),
+                        )
+                        .arg(
+                            Arg::new("serve")
+                                .long("serve")
+                                .action(ArgAction::SetTrue)
+                                .help("Serve the report on a local HTTP port instead of opening file:// URL"),
+                        ),
+                )
+                .subcommand(
+                    Command::new("merge")
+                        .about("Merge N per-shard report directories into a single unified report")
+                        .arg(
+                            Arg::new("input")
+                                .long("input")
+                                .num_args(1..)
+                                .required(true)
+                                .help("Space-separated list of shard report directories"),
+                        )
+                        .arg(
+                            Arg::new("output")
+                                .long("output")
+                                .required(true)
+                                .help("Destination directory for the merged report"),
+                        ),
+                ),
+        )
         .subcommand(
             Command::new("test")
                 .about("Run tests (.spec.ts / .test.ts) via the native jet test runner")
@@ -298,10 +335,16 @@ pub fn command() -> Command {
                         .help("Per-test timeout in ms (default 30000)"),
                 )
                 .arg(
+                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
                     Arg::new("reporter")
                         .long("reporter")
-                        .num_args(1..)
-                        .help("Reporters: term, json (default: term json)"),
+                        .help("Comma-separated reporters: term, list, json, html (default: term,json)"),
+                )
+                .arg(
+                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+                    Arg::new("report-dir")
+                        .long("report-dir")
+                        .help("Output directory for HTML report (default: test-results/report/)"),
                 )
                 .arg(
                     Arg::new("update-snapshots")
@@ -887,6 +930,51 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
             std::process::exit(result.exit_code);
         }
 
+        // @spec enhancement-html-reporter-for-native-test-runner-spec#R6
+        // @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+        Some(("report", m)) => {
+            match m.subcommand() {
+                Some(("view", vm)) => {
+                    let dir = PathBuf::from(vm.get_one::<String>("dir").unwrap());
+                    let index_html = dir.join("index.html");
+                    if !index_html.exists() {
+                        anyhow::bail!(
+                            "No index.html found in {}: run `jet test --reporter=html` first",
+                            dir.display()
+                        );
+                    }
+                    // Normalise to an absolute path for the file:// URL.
+                    let abs = index_html
+                        .canonicalize()
+                        .unwrap_or_else(|_| index_html.clone());
+                    let url = format!("file://{}", abs.display());
+                    println!("Opening report: {url}");
+                    if let Err(e) = open::that(&abs) {
+                        eprintln!(
+                            "Warning: could not open browser automatically ({e}). \
+                             Open manually: {url}"
+                        );
+                    }
+                    Ok(())
+                }
+                Some(("merge", vm)) => {
+                    let inputs: Vec<PathBuf> = vm
+                        .get_many::<String>("input")
+                        .unwrap()
+                        .map(PathBuf::from)
+                        .collect();
+                    let output = PathBuf::from(vm.get_one::<String>("output").unwrap());
+                    crate::reporter::merge::merge_reports(&inputs, &output)
+                }
+                _ => {
+                    anyhow::bail!(
+                        "Unknown report subcommand. Try 'jet report view <dir>' \
+                         or 'jet report merge --input <dirs...> --output <dir>'."
+                    )
+                }
+            }
+        }
+
         // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R5
         Some(("trace", m)) => {
             match m.subcommand() {
@@ -940,16 +1028,13 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
             if let Some(&timeout) = m.get_one::<u64>("timeout") {
                 cfg.timeout_ms = timeout;
             }
-            if let Some(reporters) = m.get_many::<String>("reporter") {
-                let mut out = Vec::new();
-                for r in reporters {
-                    match r.as_str() {
-                        "term" => out.push(crate::test_runner::config::Reporter::Term),
-                        "json" => out.push(crate::test_runner::config::Reporter::Json),
-                        other => anyhow::bail!("Unknown reporter: {other} (valid: term, json)"),
-                    }
-                }
-                cfg.reporters = out;
+            // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+            if let Some(reporter_str) = m.get_one::<String>("reporter") {
+                cfg.reporters = crate::test_runner::config::Reporter::parse_list(reporter_str)
+                    .map_err(|e| anyhow::anyhow!("{}", e))?;
+            }
+            if let Some(report_dir) = m.get_one::<String>("report-dir") {
+                cfg.report_dir = PathBuf::from(report_dir);
             }
             cfg.update_snapshots = m.get_flag("update-snapshots");
 
diff --git a/crates/jet/src/lib.rs b/crates/jet/src/lib.rs
index 1c9c1263..3818913e 100644
--- a/crates/jet/src/lib.rs
+++ b/crates/jet/src/lib.rs
@@ -10,6 +10,7 @@ pub mod cli;
 pub mod css;
 pub mod dev_server;
 pub mod pkg_manager;
+pub mod reporter;
 pub mod resolver;
 pub mod runner;
 pub mod task_runner;
diff --git a/crates/jet/src/test_runner/config.rs b/crates/jet/src/test_runner/config.rs
index 37550d31..053f10ee 100644
--- a/crates/jet/src/test_runner/config.rs
+++ b/crates/jet/src/test_runner/config.rs
@@ -17,7 +17,12 @@ pub struct RunnerConfig {
     /// Activated from stub — now wired to WorkerPool.
     // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
     pub workers: usize,
+    /// Active reporter list. Parsed from `--reporter=list,html` comma-split.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
     pub reporters: Vec<Reporter>,
+    /// Output directory for the HTML reporter (default: `test-results/report/`).
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+    pub report_dir: PathBuf,
     pub grep: Option<String>,
     pub update_snapshots: bool,
     pub only_files: Vec<PathBuf>,
@@ -29,10 +34,39 @@ pub struct RunnerConfig {
     pub shard: Option<(u32, u32)>,
 }
 
+/// Reporter kinds selectable via `--reporter=<kind>`.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R5
 #[derive(Debug, Clone, Copy, PartialEq, Eq)]
 pub enum Reporter {
+    /// Terminal summary reporter (default).
     Term,
+    /// JSON file reporter (writes `.jet/test-results.json`).
     Json,
+    /// HTML report reporter.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+    Html,
+}
+
+impl Reporter {
+    /// Parse a comma-separated list of reporter kind names.
+    ///
+    /// Accepts `"term"`, `"json"`, `"html"` (case-insensitive, with `"list"`
+    /// as an alias for `"term"`).
+    ///
+    /// Returns `Err` with the unrecognised name on failure.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+    pub fn parse_list(s: &str) -> Result<Vec<Self>, String> {
+        let mut out = Vec::new();
+        for part in s.split(',') {
+            match part.trim().to_lowercase().as_str() {
+                "term" | "list" => out.push(Reporter::Term),
+                "json" => out.push(Reporter::Json),
+                "html" => out.push(Reporter::Html),
+                other => return Err(format!("unknown reporter: {other}")),
+            }
+        }
+        Ok(out)
+    }
 }
 
 impl RunnerConfig {
@@ -42,6 +76,9 @@ impl RunnerConfig {
             .with_context(|| format!("Invalid project root: {}", project_root.display()))?;
         Ok(Self {
             test_dir: root.clone(),
+            // Default HTML report dir relative to project root.
+            // @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+            report_dir: root.join("test-results").join("report"),
             project_root: root,
             test_match: vec![
                 "**/*.spec.ts".to_string(),
diff --git a/crates/jet/src/test_runner/mod.rs b/crates/jet/src/test_runner/mod.rs
index 655015dd..cd9cd245 100644
--- a/crates/jet/src/test_runner/mod.rs
+++ b/crates/jet/src/test_runner/mod.rs
@@ -43,8 +43,10 @@ pub use worker_pool::{ShardSpec, partition_shard, parse_shard};
 /// When `config.workers == 1`, runs serially (preserves pre-Phase-4b behavior).
 /// When `config.workers > 1`, runs via `WorkerPool` with bounded concurrency.
 /// When `config.shard` is set, filters specs to only the selected shard first.
+/// When `html` reporter is configured, writes report to `config.report_dir`.
 // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
 // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
 pub async fn run(config: RunnerConfig) -> Result<Summary> {
     let all_specs = discovery::scan(&config)?;
 
@@ -67,9 +69,25 @@ pub async fn run(config: RunnerConfig) -> Result<Summary> {
     // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
     // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R2
     let workers = config.workers.max(1);
-    let summary = worker_pool::WorkerPool::run(specs, workers, config, reporter.clone()).await;
+    let summary = worker_pool::WorkerPool::run(specs, workers, config.clone(), reporter.clone()).await;
 
     reporter.on_finish(&summary)?;
+
+    // If the HTML reporter is active, emit the report now that the run is
+    // complete and all TestReports are available in `summary.reports`.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+    if config.reporters.contains(&crate::test_runner::config::Reporter::Html) {
+        let mut html_reporter = crate::reporter::HtmlReporter::new(&config.report_dir);
+        if let Some((idx, total)) = config.shard {
+            html_reporter.shard = Some((idx, total));
+        }
+        for report in &summary.reports {
+            html_reporter.emit(report.clone());
+        }
+        crate::reporter::html::finalize_with_sidecar(&mut html_reporter)
+            .context("HTML reporter failed to write report")?;
+    }
+
     Ok(summary)
 }
 
diff --git a/crates/jet/src/test_runner/reporter.rs b/crates/jet/src/test_runner/reporter.rs
index 460da64b..b4031da1 100644
--- a/crates/jet/src/test_runner/reporter.rs
+++ b/crates/jet/src/test_runner/reporter.rs
@@ -18,6 +18,19 @@ pub enum Outcome {
     Crashed,
 }
 
+/// Error details stored in `TestReport` for HTML reporter consumption.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
+#[derive(Debug, Clone, Serialize)]
+pub struct TestError {
+    pub message: String,
+    /// Full stack trace string from the worker.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub stack: Option<String>,
+    /// Structured diff from an `expect()` failure.
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub diff: Option<String>,
+}
+
 #[derive(Debug, Clone, Serialize)]
 pub struct TestReport {
     pub file: PathBuf,
@@ -25,7 +38,9 @@ pub struct TestReport {
     pub name: String,
     pub outcome: Outcome,
     pub duration_ms: u64,
-    pub error: Option<String>,
+    /// Structured error info (message + optional stack + optional diff).
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R3
+    pub error: Option<TestError>,
     /// Path to the trace archive for this test, if trace capture was enabled.
     /// Used by the HTML reporter to embed per-test deep-link trace view URLs.
     // @spec enhancement-native-trace-viewer-trace-capture-standalone-html-spec#R4
@@ -123,8 +138,13 @@ impl MultiReporter {
                     "{glyph} [{label}] {rel} :: {suite_path}{name} ({duration_ms}ms)"
                 );
                 if let Some(err) = error {
-                    println!("       {}", err.message);
-                    if let Some(diff) = &err.diff {
+                    let msg = match err {
+                        crate::test_runner::wire::TestError { message, diff, .. } => {
+                            (message, diff)
+                        }
+                    };
+                    println!("       {}", msg.0);
+                    if let Some(diff) = &msg.1 {
                         for line in diff.lines() {
                             println!("       {line}");
                         }
diff --git a/crates/jet/src/test_runner/worker.rs b/crates/jet/src/test_runner/worker.rs
index bc195824..819247be 100644
--- a/crates/jet/src/test_runner/worker.rs
+++ b/crates/jet/src/test_runner/worker.rs
@@ -122,7 +122,12 @@ pub async fn run_spec(
                         TestOutcome::TimedOut => Outcome::TimedOut,
                     },
                     duration_ms,
-                    error: error.map(|e| e.message),
+                    // @spec enhancement-html-reporter-for-native-test-runner-spec#R3
+                    error: error.map(|e| crate::test_runner::reporter::TestError {
+                        message: e.message,
+                        stack: e.stack,
+                        diff: e.diff,
+                    }),
                     // TODO(trace-viewer): wire in trace_path from TraceBuffer::commit_trace
                     // once the per-test trace buffer is integrated into the test loop.
                     trace_path: None,
@@ -177,10 +182,14 @@ pub async fn run_spec(
             name: "<worker crash>".to_string(),
             outcome: Outcome::Crashed,
             duration_ms: 0,
-            error: Some(if tail.is_empty() {
-                format!("worker exited with {status}")
-            } else {
-                tail
+            error: Some(crate::test_runner::reporter::TestError {
+                message: if tail.is_empty() {
+                    format!("worker exited with {status}")
+                } else {
+                    tail.clone()
+                },
+                stack: None,
+                diff: None,
             }),
             trace_path: None,
             shard_index: None,
diff --git a/crates/jet/src/test_runner/worker_pool.rs b/crates/jet/src/test_runner/worker_pool.rs
index 641044d2..74bc7e08 100644
--- a/crates/jet/src/test_runner/worker_pool.rs
+++ b/crates/jet/src/test_runner/worker_pool.rs
@@ -10,7 +10,7 @@
 
 use crate::test_runner::config::RunnerConfig;
 use crate::test_runner::discovery::SpecFile;
-use crate::test_runner::reporter::{MultiReporter, Outcome, Summary, TestReport};
+use crate::test_runner::reporter::{MultiReporter, Outcome, Summary, TestError, TestReport};
 use crate::test_runner::worker;
 use sha2::{Digest, Sha256};
 use std::path::PathBuf;
@@ -164,7 +164,11 @@ impl WorkerPool {
                         name: "<worker crash>".to_string(),
                         outcome: Outcome::Crashed,
                         duration_ms: 0,
-                        error: Some(format!("{err:#}")),
+                        error: Some(TestError {
+                            message: format!("{err:#}"),
+                            stack: None,
+                            diff: None,
+                        }),
                         trace_path: None,
                         shard_index: None,
                         shard_total: None,
@@ -180,7 +184,11 @@ impl WorkerPool {
                         name: "<worker panic>".to_string(),
                         outcome: Outcome::Crashed,
                         duration_ms: 0,
-                        error: Some(format!("worker task panicked: {join_err}")),
+                        error: Some(TestError {
+                            message: format!("worker task panicked: {join_err}"),
+                            stack: None,
+                            diff: None,
+                        }),
                         trace_path: None,
                         shard_index: None,
                         shard_total: None,
@@ -217,7 +225,11 @@ impl WorkerPool {
                         name: "<worker crash>".to_string(),
                         outcome: Outcome::Crashed,
                         duration_ms: 0,
-                        error: Some(format!("{err:#}")),
+                        error: Some(TestError {
+                            message: format!("{err:#}"),
+                            stack: None,
+                            diff: None,
+                        }),
                         trace_path: None,
                         shard_index: None,
                         shard_total: None,

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/artifact_writes.jsonl
+{"ts":"2026-04-21T06:32:25.980024+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"10d98dec3a416e5b22cf56c261d1118cee23e725c15c11af235de11399b3348d"}
+{"ts":"2026-04-21T06:32:42.970502+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"69883dd9fa451fdfdf889e8d60e02eeeac7f3b23c193cae659908968a159e944"}
+{"ts":"2026-04-21T06:33:01.068844+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"3024a279f8a5b946caf4b4b4569c58d4b5dd99202dd449b05a28cb15f47fe569"}
+{"ts":"2026-04-21T06:33:17.660096+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"68f71200abbae1a39c7f504898423c788825c7d3054a5cca2b7d5469632ded6d"}
+{"ts":"2026-04-21T06:33:28.625508+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"a59297b04c8273b362d209fd22a2ad94fd906789216c45944ef361e17cf6a866"}
+{"ts":"2026-04-21T06:33:41.208741+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"a1a290ba60a45c033b0ee5538bb6f6cab3e107e2e4aacedb35a36fdc26ef35a2"}
+{"ts":"2026-04-21T06:33:54.019454+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"3c313fc3eaca58137ebb87efd566f205a92355ef4d70b69720c1978e6edc825c"}
+{"ts":"2026-04-21T06:34:11.123758+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"13751e01fb01a0cae5fe63a0d06911eb62327b5e6c321141c593327b9c4f3699"}
+{"ts":"2026-04-21T06:34:32.792985+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"58ffa2cd22f2317f17d276ab302d82aca01908fcdf326edfc43fd26f54768214"}
+{"ts":"2026-04-21T06:35:01.116842+00:00","action":"create-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"9016d89d24b23b29faf1e6b8aaaee50937109d51c71580020cbd035a1e143ad1"}
+{"ts":"2026-04-21T06:35:20.020944+00:00","action":"review-change-spec","change_id":"enhancement-html-reporter-for-native-test-runner","payload_sha256":"cf3300c8bef82e167ae7b7dd4715ce1a11130472a8f6aa1afe54e6f84228fb29"}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/implementation.md
+---
+id: implementation
+type: change_implementation
+change_id: enhancement-html-reporter-for-native-test-runner
+---
+
+# Implementation
+
+## Summary
+
+*(auto-generated baseline from git diff)*
+
+## Changed Files
+
+```
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/artifact_writes.jsonl
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-changes.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-config.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-logic.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-overview.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-requirements.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-schema.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-test-plan.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/review-change-spec.json
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/prompts/analyze_spec_enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/prompts/begin_implementation.md
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/prompts/implement_tests_enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/prompts/review_spec_enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md
+D	.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/specs/enhancement-score-sync-writes-into-score-config-toml-retires-p-spec.md
+M	.score/issues/open/enhancement-html-reporter-for-native-test-runner.md
+M	.score/issues/open/enhancement-score-sync-writes-into-score-config-toml-retires-p.md
+M	Cargo.lock
+M	crates/jet/src/cli.rs
+M	crates/jet/src/lib.rs
+M	crates/jet/src/test_runner/config.rs
+M	crates/jet/src/test_runner/mod.rs
+M	crates/jet/src/test_runner/reporter.rs
+M	crates/jet/src/test_runner/worker.rs
+M	crates/jet/src/test_runner/worker_pool.rs
+M	crates/sdd/Cargo.toml
+M	crates/sdd/src/services/project_discovery.rs
+M	crates/sdd/src/services/project_registry.rs
+M	crates/sdd/src/shared/workspace.rs
+D	crates/sdd/tests/project_discovery_test.rs
+D	crates/sdd/tests/project_registry_test.rs
+D	crates/sdd/tests/sync_check_test.rs
+M	projects/score/cli/src/commands.rs
+M	projects/score/cli/src/sync.rs
+```
+
+## Diff Statistics
+
+```
+.../artifact_writes.jsonl                          |   8 -
+ .../payloads/create-change-spec-changes.json       |   5 -
+ .../payloads/create-change-spec-config.json        |   5 -
+ .../payloads/create-change-spec-logic.json         |   5 -
+ .../payloads/create-change-spec-overview.json      |   7 -
+ .../payloads/create-change-spec-requirements.json  |   5 -
+ .../payloads/create-change-spec-schema.json        |   5 -
+ .../payloads/create-change-spec-test-plan.json     |   5 -
+ .../payloads/review-change-spec.json               |  18 -
+ ...writes-into-score-config-toml-retires-p-spec.md |  53 --
+ .../prompts/begin_implementation.md                |  44 --
+ ...writes-into-score-config-toml-retires-p-spec.md |  25 -
+ ...writes-into-score-config-toml-retires-p-spec.md |  41 --
+ ...writes-into-score-config-toml-retires-p-spec.md | 762 ---------------------
+ ...ncement-html-reporter-for-native-test-runner.md |  21 +-
+ ...sync-writes-into-score-config-toml-retires-p.md |  19 +-
+ Cargo.lock                                         |   1 -
+ crates/jet/src/cli.rs                              | 109 ++-
+ crates/jet/src/lib.rs                              |   1 +
+ crates/jet/src/test_runner/config.rs               |  37 +
+ crates/jet/src/test_runner/mod.rs                  |  20 +-
+ crates/jet/src/test_runner/reporter.rs             |  26 +-
+ crates/jet/src/test_runner/worker.rs               |  19 +-
+ crates/jet/src/test_runner/worker_pool.rs          |  20 +-
+ crates/sdd/Cargo.toml                              |   1 -
+ crates/sdd/src/services/project_discovery.rs       | 107 +--
+ crates/sdd/src/services/project_registry.rs        | 705 +++++++++++--------
+ crates/sdd/src/shared/workspace.rs                 |  11 +-
+ crates/sdd/tests/project_discovery_test.rs         | 212 ------
+ crates/sdd/tests/project_registry_test.rs          | 253 -------
+ crates/sdd/tests/sync_check_test.rs                | 104 ---
+ projects/score/cli/src/commands.rs                 |   4 +-
+ projects/score/cli/src/sync.rs                     |  17 +-
+ 33 files changed, 655 insertions(+), 2020 deletions(-)
+```
+
+## Diff
+
+```diff
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/artifact_writes.jsonl b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/artifact_writes.jsonl
+deleted file mode 100644
+index 06702d6f..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/artifact_writes.jsonl
++++ /dev/null
+@@ -1,8 +0,0 @@
+-{"ts":"2026-04-21T06:00:01.215250+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"c322c2af48d7b2b722cdf251c5db56a22e7fc8ec16acbd4f11ebb8769b0d41ce"}
+-{"ts":"2026-04-21T06:00:20.999977+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"53637a2de10b37abd4d8c2b1d44209ed345028542e665914bcdab28e5360c29c"}
+-{"ts":"2026-04-21T06:00:54.574109+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"560551ff310b0d069f9ce2c03d149008486a5cb4ba83e0a8dc4d1df464416508"}
+-{"ts":"2026-04-21T06:01:21.450447+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"845400c36691d69bc09045b5b3502700bc177751cd0f6c5eb3473a212a6c1e46"}
+-{"ts":"2026-04-21T06:01:51.784407+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"a75360492c16e919ce9332241a092d01792e7835a2fc305858d9bdc77cc53907"}
+-{"ts":"2026-04-21T06:02:20.428730+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"a96dc6962348197301aaffa43ee97abdf49581e6c53e039ce19834383eb4d862"}
+-{"ts":"2026-04-21T06:03:33.442658+00:00","action":"create-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"d04e8af28433d48f25acff9e501e48ea2c90ca44784ea04b719465f1f74066f6"}
+-{"ts":"2026-04-21T06:09:38.724992+00:00","action":"review-change-spec","change_id":"enhancement-score-sync-writes-into-score-config-toml-retires-p","payload_sha256":"95be42db3cc93a7fd58d2e1d5352c7716532e572d83ed02e52715e573ddf079f"}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-changes.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-changes.json
+deleted file mode 100644
+index b6e7347f..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-changes.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "changes",
+-  "content": "<!-- type: changes lang: yaml -->\n\n```yaml\nspec_changes:\n  - path: projects/score/specs/sync-command.md\n    action: update\n    sections:\n      - overview: update output file from projects.toml to config.toml with marker convention; remove two-file overlay description\n      - requirements: update R5 (idempotency via marker upsert), R6 (full enum to config.toml), R7 (relative test_cmd), R8 (Rule E package name fix); add toml_edit constraint; retire R6 override model\n      - logic: update flowchart — add read_config, locate_markers, replace_block/append_block, write_config, check_stale, delete_stale nodes after ReturnVec; update Rule E branch to read [package].name\n      - config: replace two-file example with single-file config.toml showing BEGIN SCORE SYNC / END SCORE SYNC block coexisting with sdd.* and workspaces tables\n      - test-plan: add T17–T22 for marker round-trip, idempotency, Rule E package-name fix, relative-path test_cmd, migration delete, --check targeting config.toml\n      - changes: update project_registry.rs description; add workspace.rs marker constants; add toml_edit dep note; retire projects.toml write target\n\n  - path: projects/score/specs/sync-config-toml-schema.md\n    action: create\n    sections:\n      - overview: describe the new single-file write model and marker semantics\n      - requirements: R1–R11 from issue\n      - schema: JSON Schema for the [[projects]] block inside config.toml with BEGIN/END marker semantics and SyncMarkers constants\n      - config: annotated config.toml example showing user content + BEGIN SCORE SYNC block coexisting with sdd.* and workspaces tables; toml_edit zone constraints\n\ncode_changes:\n  modified_files:\n    - path: crates/sdd/src/services/project_registry.rs\n      change: |\n        - Replace write_projects_toml with write_projects_config: reads config.toml via toml_edit, locates or appends BEGIN/END SCORE SYNC markers, splices in full [[projects]] TOML block, writes back.\n        - Replace load_projects two-file overlay with single-file load: parse [[projects]] from config.toml only (bounded by markers for writes; readable anywhere in the file for loads).\n        - Update check_drift and read_existing_defaults to target config.toml.\n        - Add one-shot migration: after successful write, delete .score/projects.toml if it exists.\n\n    - path: crates/sdd/src/services/project_discovery.rs\n      change: |\n        - Rule E: parse nested Cargo.toml with toml crate; extract [package].name for workspace.name and test_cmd -p arg.\n        - Rules A/C/D: emit test_cmd as project-relative path (cd <rel-path> && ...) computed as path.strip_prefix(repo_root).\n\n    - path: crates/sdd/src/shared/workspace.rs\n      change: |\n        - Remove PROJECTS_FILE constant.\n        - Add SYNC_BEGIN_MARKER: &str = \"# BEGIN SCORE SYNC — auto-generated, do not edit by hand\".\n        - Add SYNC_END_MARKER: &str = \"# END SCORE SYNC\".\n\n    - path: projects/score/cli/src/sync.rs\n      change: Update user-visible strings and help text to reference config.toml instead of projects.toml.\n\n    - path: crates/sdd/Cargo.toml\n      change: Add toml_edit dependency for lossless round-trip writes to config.toml.\n\n  modified_tests:\n    - path: crates/sdd/tests/project_registry_test.rs\n      change: Rewrite merge/round-trip tests for single-file marker-delimited writer; add T17 (marker upsert), T18 (round-trip preservation + idempotency), T21 (migration delete).\n\n    - path: crates/sdd/tests/project_discovery_test.rs\n      change: Extend Rule E fixture to use Cargo.toml with [package].name differing from directory basename (T19); add relative-path assertion on test_cmd (T20).\n\n    - path: crates/sdd/tests/sync_check_test.rs\n      change: Update target file path from projects.toml to config.toml; add T22 asserting --check output references config.toml.\n\n  retired_files:\n    - path: .score/projects.toml\n      reason: Superseded by [[projects]] block in .score/config.toml. Deleted by score sync on first run after migration.\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-config.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-config.json
+deleted file mode 100644
+index 99abfd3f..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-config.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "config",
+-  "content": "<!-- type: config lang: json -->\n\nAnnotated example of `.score/config.toml` after migration. User-authored sections coexist with the auto-generated marker block. `toml_edit` ensures non-generated sections are preserved byte-identical on every sync.\n\n```json\n{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"$id\": \"score-config-toml-layout\",\n  \"title\": \"ScoreConfigTomlLayout\",\n  \"description\": \"Describes the structural layout of .score/config.toml after migration. Three namespace zones must not overlap.\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"zones\": {\n      \"type\": \"array\",\n      \"description\": \"Ordered zones in config.toml. Zones are non-overlapping. score sync only touches zone: sync-block.\",\n      \"items\": {\n        \"oneOf\": [\n          {\n            \"type\": \"object\",\n            \"properties\": {\n              \"zone\": { \"type\": \"string\", \"const\": \"user-authored\" },\n              \"description\": { \"type\": \"string\", \"const\": \"sdd.*, defaults.workspace, workspaces tables — user-managed, never touched by score sync\" },\n              \"example_keys\": {\n                \"type\": \"array\",\n                \"items\": { \"type\": \"string\" },\n                \"examples\": [[\"[sdd.test.scope]\", \"[defaults.workspace]\", \"[[workspaces]]\"]]\n              }\n            },\n            \"required\": [\"zone\"]\n          },\n          {\n            \"type\": \"object\",\n            \"properties\": {\n              \"zone\": { \"type\": \"string\", \"const\": \"sync-block\" },\n              \"description\": { \"type\": \"string\", \"const\": \"Marker-delimited block written by score sync on every run. Full enumeration, no sparse overrides.\" },\n              \"begin_marker\": { \"type\": \"string\", \"const\": \"# BEGIN SCORE SYNC — auto-generated, do not edit by hand\" },\n              \"end_marker\": { \"type\": \"string\", \"const\": \"# END SCORE SYNC\" },\n              \"content\": { \"type\": \"string\", \"const\": \"[[projects]] array covering all ~82 discovered projects\" }\n            },\n            \"required\": [\"zone\", \"begin_marker\", \"end_marker\"]\n          }\n        ]\n      }\n    },\n    \"toml_edit_constraints\": {\n      \"type\": \"object\",\n      \"description\": \"Rules governing how toml_edit is used to write config.toml\",\n      \"properties\": {\n        \"read\": { \"type\": \"string\", \"const\": \"Parse entire config.toml into toml_edit Document\" },\n        \"locate\": { \"type\": \"string\", \"const\": \"Find BEGIN/END SCORE SYNC comment pair by scanning raw string; positions are line-based\" },\n        \"replace_strategy\": { \"type\": \"string\", \"const\": \"If markers found: splice out content between markers (inclusive), splice in new [[projects]] TOML string\" },\n        \"append_strategy\": { \"type\": \"string\", \"const\": \"If markers absent: append newline + BEGIN marker + [[projects]] TOML + END marker to end of file\" },\n        \"write\": { \"type\": \"string\", \"const\": \"Serialize toml_edit Document back to file; non-generated sections must be byte-identical to input\" }\n      }\n    },\n    \"example_file\": {\n      \"type\": \"string\",\n      \"description\": \"Representative config.toml layout. Zone order is illustrative — user zones may appear before or after the sync block.\",\n      \"const\": \"# .score/config.toml\\n\\n# --- user-authored sdd settings ---\\n[sdd.test.scope]\\nroots = [\\\"crates\\\", \\\"projects\\\", \\\"packages\\\"]\\n\\n[defaults.workspace]\\ncodegen.target = \\\"rust\\\"\\n\\n# --- auto-generated by score sync (do not edit manually) ---\\n# BEGIN SCORE SYNC — auto-generated, do not edit by hand\\n\\n[[projects]]\\nname = \\\"sdd\\\"\\npath = \\\"crates/sdd\\\"\\n\\n  [[projects.workspaces]]\\n  name = \\\"sdd\\\"\\n  path = \\\"crates/sdd\\\"\\n  target = \\\"rust\\\"\\n  test_cmd = \\\"cargo test -p sdd\\\"\\n\\n[[projects]]\\nname = \\\"conductor\\\"\\npath = \\\"projects/conductor\\\"\\n\\n  [[projects.workspaces]]\\n  name = \\\"be\\\"\\n  path = \\\"projects/conductor/be\\\"\\n  target = \\\"python\\\"\\n  test_cmd = \\\"cd projects/conductor/be && uv run pytest\\\"\\n\\n  [[projects.workspaces]]\\n  name = \\\"fe\\\"\\n  path = \\\"projects/conductor/fe\\\"\\n  target = \\\"typescript\\\"\\n  test_cmd = \\\"cd projects/conductor/fe && npx vitest run\\\"\\n\\n[[projects]]\\nname = \\\"score-cli\\\"\\npath = \\\"projects/score/cli\\\"\\n\\n  [[projects.workspaces]]\\n  name = \\\"score-cli\\\"\\n  path = \\\"projects/score/cli\\\"\\n  target = \\\"rust\\\"\\n  test_cmd = \\\"cargo test -p score-cli\\\"\\n\\n# END SCORE SYNC\"\n    }\n  }\n}\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-logic.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-logic.json
+deleted file mode 100644
+index 5bc423b0..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-logic.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "logic",
+-  "content": "<!-- type: logic lang: mermaid -->\n\n```mermaid\n---\nid: logic\nentry: start\nnodes:\n  start: { kind: start, label: \"discover_projects: root\" }\n  enum_roots: { kind: process, label: \"Enumerate crates/* + projects/* + packages/*\" }\n  for_each: { kind: decision, label: \"For each dir\" }\n  rule_a: { kind: decision, label: \"Rule A: be/ AND fe/ exist?\" }\n  emit_be_fe: { kind: process, label: \"Emit workspace be (python) + fe (typescript)\" }\n  rule_b: { kind: decision, label: \"Rule B: Cargo.toml at root?\" }\n  emit_rust: { kind: process, label: \"Emit workspace target=rust; test_cmd=cargo test -p <name>\" }\n  rule_c: { kind: decision, label: \"Rule C: pyproject.toml at root?\" }\n  check_uv: { kind: decision, label: \"uv.lock present?\" }\n  emit_py_test: { kind: process, label: \"Emit target=python; test_cmd=cd <rel> && uv run pytest\" }\n  emit_py_no_test: { kind: process, label: \"Emit target=python; test_cmd omitted\" }\n  rule_d: { kind: decision, label: \"Rule D: package.json at root?\" }\n  check_vitest: { kind: decision, label: \"vitest in devDependencies?\" }\n  emit_ts_test: { kind: process, label: \"Emit target=typescript; test_cmd=cd <rel> && npx vitest run\" }\n  emit_ts_no_test: { kind: process, label: \"Emit target=typescript; test_cmd omitted\" }\n  rule_e: { kind: decision, label: \"Rule E: single-level nested Cargo.toml?\" }\n  read_pkg_name: { kind: process, label: \"Read [package].name from nested Cargo.toml\" }\n  emit_nested_rust: { kind: process, label: \"Emit workspace name=pkg_name; target=rust; test_cmd=cargo test -p pkg_name\" }\n  rule_f: { kind: process, label: \"Rule F: no manifest found\" }\n  emit_schemas: { kind: process, label: \"Emit target=schemas; test_cmd=true\" }\n  wrap_project: { kind: process, label: \"Wrap workspaces into Project struct\" }\n  return_vec: { kind: process, label: \"Return Vec<Project>\" }\n  read_config: { kind: process, label: \"Read config.toml with toml_edit\" }\n  locate_markers: { kind: decision, label: \"BEGIN/END SCORE SYNC markers exist?\" }\n  replace_block: { kind: process, label: \"Replace content between markers with new [[projects]] TOML\" }\n  append_block: { kind: process, label: \"Append BEGIN marker + [[projects]] TOML + END marker\" }\n  write_config: { kind: process, label: \"Write config.toml back via toml_edit (lossless)\" }\n  check_stale: { kind: decision, label: \".score/projects.toml exists?\" }\n  delete_stale: { kind: process, label: \"Delete .score/projects.toml\" }\n  done: { kind: terminal, label: \"Done\" }\nedges:\n  - from: start\n    to: enum_roots\n  - from: enum_roots\n    to: for_each\n  - from: for_each\n    to: rule_a\n  - from: rule_a\n    to: emit_be_fe\n    label: \"yes\"\n  - from: rule_a\n    to: rule_b\n    label: \"no\"\n  - from: rule_b\n    to: emit_rust\n    label: \"yes\"\n  - from: rule_b\n    to: rule_c\n    label: \"no\"\n  - from: rule_c\n    to: check_uv\n    label: \"yes\"\n  - from: check_uv\n    to: emit_py_test\n    label: \"yes\"\n  - from: check_uv\n    to: emit_py_no_test\n    label: \"no\"\n  - from: rule_c\n    to: rule_d\n    label: \"no\"\n  - from: rule_d\n    to: check_vitest\n    label: \"yes\"\n  - from: check_vitest\n    to: emit_ts_test\n    label: \"yes\"\n  - from: check_vitest\n    to: emit_ts_no_test\n    label: \"no\"\n  - from: rule_d\n    to: rule_e\n    label: \"no\"\n  - from: rule_e\n    to: read_pkg_name\n    label: \"yes\"\n  - from: read_pkg_name\n    to: emit_nested_rust\n  - from: rule_e\n    to: rule_f\n    label: \"no\"\n  - from: rule_f\n    to: emit_schemas\n  - from: emit_be_fe\n    to: wrap_project\n  - from: emit_rust\n    to: wrap_project\n  - from: emit_py_test\n    to: wrap_project\n  - from: emit_py_no_test\n    to: wrap_project\n  - from: emit_ts_test\n    to: wrap_project\n  - from: emit_ts_no_test\n    to: wrap_project\n  - from: emit_nested_rust\n    to: wrap_project\n  - from: emit_schemas\n    to: wrap_project\n  - from: wrap_project\n    to: for_each\n  - from: for_each\n    to: return_vec\n    label: \"done\"\n  - from: return_vec\n    to: read_config\n  - from: read_config\n    to: locate_markers\n  - from: locate_markers\n    to: replace_block\n    label: \"yes\"\n  - from: locate_markers\n    to: append_block\n    label: \"no\"\n  - from: replace_block\n    to: write_config\n  - from: append_block\n    to: write_config\n  - from: write_config\n    to: check_stale\n  - from: check_stale\n    to: delete_stale\n    label: \"yes\"\n  - from: check_stale\n    to: done\n    label: \"no\"\n  - from: delete_stale\n    to: done\n---\nflowchart TD\n    start([discover_projects: root]) --> enum_roots[Enumerate crates/* + projects/* + packages/*]\n    enum_roots --> for_each{For each dir}\n    for_each --> rule_a{Rule A: be/ AND fe/ exist?}\n    rule_a -- yes --> emit_be_fe[Emit workspace be target=python\\nEmit workspace fe target=typescript]\n    rule_a -- no --> rule_b{Rule B: Cargo.toml at root?}\n    rule_b -- yes --> emit_rust[Emit 1 workspace target=rust\\ntest_cmd=cargo test -p name]\n    rule_b -- no --> rule_c{Rule C: pyproject.toml at root?}\n    rule_c -- yes --> check_uv{uv.lock present?}\n    check_uv -- yes --> emit_py_test[Emit target=python\\ntest_cmd=cd rel && uv run pytest]\n    check_uv -- no --> emit_py_no_test[Emit target=python\\ntest_cmd omitted]\n    rule_c -- no --> rule_d{Rule D: package.json at root?}\n    rule_d -- yes --> check_vitest{vitest in devDependencies?}\n    check_vitest -- yes --> emit_ts_test[Emit target=typescript\\ntest_cmd=cd rel && npx vitest run]\n    check_vitest -- no --> emit_ts_no_test[Emit target=typescript\\ntest_cmd omitted]\n    rule_d -- no --> rule_e{Rule E: single-level nested Cargo.toml?}\n    rule_e -- yes --> read_pkg_name[Read package.name from nested Cargo.toml]\n    read_pkg_name --> emit_nested_rust[Emit workspace name=pkg_name\\ntarget=rust\\ntest_cmd=cargo test -p pkg_name]\n    rule_e -- no --> rule_f[Rule F: no manifest found]\n    rule_f --> emit_schemas[Emit target=schemas\\ntest_cmd=true]\n    emit_be_fe --> wrap_project[Wrap workspaces into Project struct]\n    emit_rust --> wrap_project\n    emit_py_test --> wrap_project\n    emit_py_no_test --> wrap_project\n    emit_ts_test --> wrap_project\n    emit_ts_no_test --> wrap_project\n    emit_nested_rust --> wrap_project\n    emit_schemas --> wrap_project\n    wrap_project --> for_each\n    for_each -- done --> return_vec([Return Vec Project])\n    return_vec --> read_config[Read config.toml with toml_edit]\n    read_config --> locate_markers{BEGIN/END SCORE SYNC markers exist?}\n    locate_markers -- yes --> replace_block[Replace content between markers]\n    locate_markers -- no --> append_block[Append BEGIN marker + projects + END marker]\n    replace_block --> write_config[Write config.toml back lossless]\n    append_block --> write_config\n    write_config --> check_stale{.score/projects.toml exists?}\n    check_stale -- yes --> delete_stale[Delete .score/projects.toml]\n    check_stale -- no --> done([Done])\n    delete_stale --> done\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-overview.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-overview.json
+deleted file mode 100644
+index de352176..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-overview.json
++++ /dev/null
+@@ -1,7 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "overview",
+-  "main_spec_ref": "projects/score/specs/sync-command.md",
+-  "fill_sections": ["overview", "requirements", "logic", "schema", "config", "test-plan", "changes"],
+-  "content": "<!-- type: overview lang: markdown -->\n\nRetires `.score/projects.toml` as a separate write target. `score sync` now writes a marker-delimited `[[projects]]` block directly inside `.score/config.toml`, using `toml_edit` for lossless round-trips that preserve all non-generated content.\n\n| Aspect | Before | After |\n|--------|--------|-------|\n| Write target | `.score/projects.toml` | `.score/config.toml` (marker-delimited block) |\n| Load path | `projects.toml` overlaid with `config.toml` sparse entries | `config.toml` only (marker block) |\n| Round-trip safety | Header comment; serde full rewrite | `toml_edit` lossless; non-generated sections byte-identical |\n| Bug: Rule E name | Directory basename (`cli`) | `[package].name` from nested `Cargo.toml` (`score-cli`) |\n| Bug: test_cmd paths | May leak absolute paths | Project-relative (`cd projects/conductor/be && ...`) |\n| Migration | n/a | One-shot: delete `.score/projects.toml` on first successful sync |\n\nTwo spec updates:\n- `projects/score/specs/sync-command.md` — update overview, requirements, logic, config, test-plan, changes\n- `projects/score/specs/sync-config-toml-schema.md` — new spec: schema + annotated config example"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-requirements.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-requirements.json
+deleted file mode 100644
+index 0a2a2c31..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-requirements.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "requirements",
+-  "content": "<!-- type: requirements lang: mermaid -->\n\n```mermaid\n---\nid: requirements\n---\nrequirementDiagram\n\nrequirement R1 {\n  id: R1\n  text: \"score sync writes all discovered projects into .score/config.toml [[projects]] block, replacing .score/projects.toml\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R2 {\n  id: R2\n  text: \"The auto-generated block is bounded by: # BEGIN SCORE SYNC — auto-generated, do not edit by hand and # END SCORE SYNC marker comments\"\n  risk: high\n  verifymethod: inspection\n}\n\nrequirement R3 {\n  id: R3\n  text: \"Non-[[projects]] content in config.toml (workspaces, defaults.workspace, sdd.*) survives every sync run without modification to comments, order, or formatting\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R4 {\n  id: R4\n  text: \"config.toml round-trips use toml_edit (lossless crate) so comments and whitespace in non-generated sections are preserved byte-identical\"\n  risk: high\n  verifymethod: inspection\n}\n\nrequirement R5 {\n  id: R5\n  text: \"score sync is idempotent: running twice with no filesystem changes produces a zero diff in config.toml\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R6 {\n  id: R6\n  text: \"All ~82 discovered projects are written on each sync (full enumeration, no sparse override list)\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R7 {\n  id: R7\n  text: \"test_cmd strings use project-relative paths (e.g. cd projects/conductor/be && uv run pytest), not absolute filesystem paths\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R8 {\n  id: R8\n  text: \"Rule E derives the project name from [package].name in the nested Cargo.toml (not directory basename), producing cargo test -p score-cli not cargo test -p cli\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R9 {\n  id: R9\n  text: \"Registry consumers read [[projects]] exclusively from config.toml; projects.toml is no longer read\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R10 {\n  id: R10\n  text: \"If .score/projects.toml exists at sync time, it is deleted after a successful write to config.toml\"\n  risk: medium\n  verifymethod: test\n}\n\nrequirement R11 {\n  id: R11\n  text: \"score sync --dry-run and score sync --check continue with identical semantics, now targeting config.toml\"\n  risk: high\n  verifymethod: test\n}\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-schema.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-schema.json
+deleted file mode 100644
+index 8540a827..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-schema.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "schema",
+-  "content": "<!-- type: schema lang: json -->\n\nJSON Schema for the auto-generated `[[projects]]` block written between BEGIN/END SCORE SYNC markers in `.score/config.toml`.\n\n```json\n{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"$id\": \"sync-config-toml-projects-block\",\n  \"title\": \"SyncProjectsBlock\",\n  \"description\": \"Schema for the [[projects]] array written between BEGIN SCORE SYNC / END SCORE SYNC markers in .score/config.toml. Data model is unchanged from ProjectsToml; only the write target and load path change.\",\n  \"type\": \"object\",\n  \"properties\": {\n    \"projects\": {\n      \"type\": \"array\",\n      \"description\": \"Full enumeration of all discovered projects. No sparse override model — config.toml entries within the marker block are overwritten on each sync.\",\n      \"items\": {\n        \"$ref\": \"#/$defs/Project\"\n      },\n      \"minItems\": 0\n    }\n  },\n  \"required\": [\"projects\"],\n  \"$defs\": {\n    \"Project\": {\n      \"type\": \"object\",\n      \"title\": \"Project\",\n      \"properties\": {\n        \"name\": {\n          \"type\": \"string\",\n          \"description\": \"Project identifier. For Rule E, derived from [package].name in the nested Cargo.toml, not the directory basename.\"\n        },\n        \"path\": {\n          \"type\": \"string\",\n          \"description\": \"Path relative to repo root (e.g. crates/sdd, projects/conductor).\"\n        },\n        \"tech_design_dir\": {\n          \"type\": \"string\",\n          \"description\": \"Override for .score/tech_design sub-path. Defaults to crates/<name> or projects/<name>.\"\n        },\n        \"workspaces\": {\n          \"type\": \"array\",\n          \"items\": {\n            \"$ref\": \"#/$defs/Workspace\"\n          },\n          \"minItems\": 1\n        }\n      },\n      \"required\": [\"name\", \"path\", \"workspaces\"],\n      \"additionalProperties\": false\n    },\n    \"Workspace\": {\n      \"type\": \"object\",\n      \"title\": \"Workspace\",\n      \"properties\": {\n        \"name\": {\n          \"type\": \"string\",\n          \"description\": \"Short identifier (e.g. be, fe, cli, or same as project name for single-workspace projects).\"\n        },\n        \"path\": {\n          \"type\": \"string\",\n          \"description\": \"Path relative to repo root. MUST be a project-relative path — absolute paths are a bug (R7).\"\n        },\n        \"target\": {\n          \"type\": \"string\",\n          \"enum\": [\"rust\", \"python\", \"javascript\", \"typescript\", \"schemas\"],\n          \"description\": \"Language/runtime target inferred from manifest files.\"\n        },\n        \"test_cmd\": {\n          \"type\": \"string\",\n          \"description\": \"Shell command to run the workspace test suite. MUST use project-relative form (e.g. cd projects/conductor/be && uv run pytest). Omitted when the required tool is absent.\"\n        },\n        \"codegen\": {\n          \"$ref\": \"#/$defs/CodegenProfile\"\n        }\n      },\n      \"required\": [\"name\", \"path\", \"target\"],\n      \"additionalProperties\": false\n    },\n    \"CodegenProfile\": {\n      \"type\": \"object\",\n      \"title\": \"CodegenProfile\",\n      \"properties\": {\n        \"target\": {\n          \"type\": \"string\",\n          \"enum\": [\"rust\", \"python\", \"javascript\", \"typescript\", \"schemas\"]\n        },\n        \"profile\": {\n          \"type\": \"string\",\n          \"description\": \"Named generation profile (e.g. axum-service, react-component).\"\n        }\n      },\n      \"required\": [\"target\"],\n      \"additionalProperties\": false\n    },\n    \"SyncMarkers\": {\n      \"type\": \"object\",\n      \"title\": \"SyncMarkers\",\n      \"description\": \"String constants that delimit the auto-generated block. These appear as TOML comments in config.toml.\",\n      \"properties\": {\n        \"begin\": {\n          \"type\": \"string\",\n          \"const\": \"# BEGIN SCORE SYNC — auto-generated, do not edit by hand\"\n        },\n        \"end\": {\n          \"type\": \"string\",\n          \"const\": \"# END SCORE SYNC\"\n        }\n      },\n      \"required\": [\"begin\", \"end\"]\n    }\n  }\n}\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-test-plan.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-test-plan.json
+deleted file mode 100644
+index 47f732dd..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/create-change-spec-test-plan.json
++++ /dev/null
+@@ -1,5 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "section": "test-plan",
+-  "content": "<!-- type: test-plan lang: mermaid -->\n\nExtends existing T1–T16 (unchanged). Adds T17–T20 for this change. Existing tests that reference `projects.toml` as write target must be updated to expect `config.toml`.\n\n```mermaid\n---\nid: test-plan\n---\nrequirementDiagram\n\nrequirement R1 {\n  id: R1\n  text: \"score sync writes all discovered projects into config.toml [[projects]] block\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R2 {\n  id: R2\n  text: \"BEGIN/END SCORE SYNC markers delimit the auto-generated block\"\n  risk: high\n  verifymethod: inspection\n}\n\nrequirement R3 {\n  id: R3\n  text: \"Non-[[projects]] content in config.toml preserved without modification\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R5 {\n  id: R5\n  text: \"score sync is idempotent: double-run produces zero diff\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R7 {\n  id: R7\n  text: \"test_cmd uses project-relative paths, not absolute paths\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R8 {\n  id: R8\n  text: \"Rule E derives project name from [package].name in nested Cargo.toml\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R9 {\n  id: R9\n  text: \"Registry consumers read [[projects]] from config.toml only\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R10 {\n  id: R10\n  text: \"Stale .score/projects.toml deleted after successful sync\"\n  risk: medium\n  verifymethod: test\n}\n\nrequirement R11 {\n  id: R11\n  text: \"--dry-run and --check target config.toml\"\n  risk: high\n  verifymethod: test\n}\n\nelement T17 {\n  type: \"Test\"\n  docref: \"project_registry_test.rs::marker_upsert_first_run — Given config.toml with no markers; When write_projects_config called; Then config.toml contains BEGIN marker, [[projects]] entries, END marker; existing user content untouched\"\n}\n\nelement T18 {\n  type: \"Test\"\n  docref: \"project_registry_test.rs::marker_upsert_round_trip — Given config.toml with arbitrary comments and sdd.* tables; When sync run twice with identical filesystem; Then diff between run-1 and run-2 output is empty (idempotency R5); non-projects sections byte-identical (R3)\"\n}\n\nelement T19 {\n  type: \"Test\"\n  docref: \"project_discovery_test.rs::rule_e_package_name — Given TempDir with subdir/Cargo.toml where [package].name=score-cli and dir name=cli; When Rule E applied; Then workspace.name=score-cli and test_cmd=cargo test -p score-cli\"\n}\n\nelement T20 {\n  type: \"Test\"\n  docref: \"project_discovery_test.rs::test_cmd_relative_path — Given TempDir at absolute /tmp/xyz/projects/conductor/be with uv.lock; When Rule C applied; Then test_cmd starts with cd projects/conductor/be, not /tmp/xyz/projects/conductor/be\"\n}\n\nelement T21 {\n  type: \"Test\"\n  docref: \"project_registry_test.rs::migration_deletes_projects_toml — Given workspace with .score/projects.toml present; When score sync runs; Then .score/projects.toml does not exist after sync\"\n}\n\nelement T22 {\n  type: \"Test\"\n  docref: \"sync_check_test.rs::check_targets_config_toml — Given config.toml out-of-date; When score sync --check; Then exits 1 with diff referencing config.toml path (not projects.toml)\"\n}\n\nT17 - verifies -> R1\nT17 - verifies -> R2\nT17 - verifies -> R3\nT18 - verifies -> R3\nT18 - verifies -> R5\nT19 - verifies -> R8\nT20 - verifies -> R7\nT21 - verifies -> R10\nT22 - verifies -> R9\nT22 - verifies -> R11\n```"
+-}
+diff --git a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/review-change-spec.json b/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/review-change-spec.json
+deleted file mode 100644
+index 14bae94b..00000000
+--- a/.score/changes/enhancement-score-sync-writes-into-score-config-toml-retires-p/payloads/review-change-spec.json
++++ /dev/null
+@@ -1,18 +0,0 @@
+-{
+-  "spec_id": "enhancement-score-sync-writes-into-score-config-toml-retires-p-spec",
+-  "verdict": "APPROVED",
+-  "summary": "Spec covers all 11 Requirements. Logic flowchart extends the existing Rules A-F graph with the new marker-aware write path (read_config → locate_markers → replace_block|append_block → write_config → check_stale → delete_stale → done) and Rule E now routes through read_pkg_name. Schema + Config JSON Schemas capture the delimited-section contract and the toml_edit operation constraints. Test plan T17-T22 covers R1, R2, R3, R5, R7, R8, R9, R10, R11. Out-of-scope boundaries respected (user's [[workspaces]] WIP untouched; no downstream consumer wiring). R4 (toml_edit lossless) is verifymethod: inspection — acceptable. The one soft gap: R6 (full enumeration of ~82 projects) has verifymethod: test but no T-element explicitly verifies it — easily backfilled in T18 with an assert_eq!(projects.len(), N) where N is derived from the fixture. Linter warnings on Schema/Config ```json fencing are false positives per AUTHORING.md format priority (OpenRPC > JSON Schema > Mermaid > YAML > Markdown) — JSON-fenced JSON Schema is preferred.",
+-  "issues": [
+-    {
+-      "severity": "low",
+-      "section": "test-plan",
+-      "message": "R6 (full enumeration) has verifymethod: test but no T-element verifies relation. Recommend adding an assert on total discovered count to T18 during implementation."

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-implementation.json
+{
+  "change_id": "enhancement-html-reporter-for-native-test-runner",
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "content": "# Implementation — enhancement-html-reporter-for-native-test-runner\n\n## Summary\n\nImplements the HTML reporter for the native jet test runner, satisfying R1–R10 of the spec. A new `crates/jet/src/reporter/` module consumes the existing NDJSON wire protocol and emits a self-contained static site (`index.html` + `report.js` + `report.css`) into `test-results/report/`. Assets are embedded at compile time via `include_bytes!` so the report is fully self-contained with no CDN at view time. The reporter is activated via `jet test --reporter=html` (or `--reporter=list,html` for concurrent reporters) and merges multi-shard runs via `jet report merge`.\n\n## Code Changes\n\n### New files\n\n- `crates/jet/src/reporter/mod.rs` — module root, re-exports `HtmlReporter` + `merge_reports`.\n- `crates/jet/src/reporter/html.rs` (486 LOC) — `HtmlReporter` struct, `TestRow`, deterministic HTML renderer, NDJSON sidecar writer, stable `test_id` hashing.\n- `crates/jet/src/reporter/parser.rs` (142 LOC) — NDJSON → `TestRow` parser; wire-protocol-compatible.\n- `crates/jet/src/reporter/merge.rs` (81 LOC) — shard merger: reads per-shard NDJSON sidecars, dedups by `test_id`, emits unified `index.html`.\n- `crates/jet/assets/html-reporter/index.html` — template with `<!-- PLACEHOLDER -->` markers.\n- `crates/jet/assets/html-reporter/report.js` — interactive filter + drawer logic (no external deps).\n- `crates/jet/assets/html-reporter/report.css` — status badge styles, drawer animation.\n- `crates/jet/tests/html_reporter_tests.rs` (320 LOC) — 9 integration tests T1–T9 per the Test Plan.\n\n### Modified files\n\n- `crates/jet/src/test_runner/reporter.rs` — introduced `TestError { message, stack, diff }` struct; `TestReport.error` changed from `Option<String>` to `Option<TestError>` so the HTML reporter can render stack/diff distinctly (R3).\n- `crates/jet/src/test_runner/worker_pool.rs` — error construction sites updated to wrap strings in `TestError`.\n- `crates/jet/src/test_runner/worker.rs` — forwards structured error fields into `TestReport`.\n- `crates/jet/src/test_runner/mod.rs` — wires HTML reporter emission at finalize.\n- `crates/jet/src/test_runner/config.rs` — adds `Reporter::Html` variant; `Reporter::parse_list(\"list,html\")` returns `[Term, Html]`.\n- `crates/jet/src/cli.rs` — adds `jet report view <dir>` and `jet report merge --input ... --output ...` subcommands; adds `--reporter` comma-list parser and `--report-dir` flag for `jet test`.\n- `crates/jet/src/lib.rs` — re-exports `reporter` module.\n\n## Requirements Coverage\n\n| Req | Impl site |\n|-----|-----------|\n| R1 `test-results/report/index.html` | `HtmlReporter::finalize` at `reporter/html.rs:131` |\n| R2 aggregate stats + shard info | `build_stats_html` + `build_shard_html` at `reporter/html.rs:181,202` |\n| R3 per-test rows (badge, duration, file:line, stack drawer, trace link) | `build_single_row` at `reporter/html.rs:279` |\n| R4 embedded assets, no CDN | `const HTML_TEMPLATE = include_str!(...)` at `reporter/html.rs:16-20` |\n| R5 `--reporter=html` / `list,html` | `Reporter::parse_list` at `test_runner/config.rs` |\n| R6 `jet report view <dir>` | CLI subcommand at `cli.rs` |\n| R7 `jet report merge` — shard stitch | `merge_reports` at `reporter/merge.rs:29` |\n| R8 read NDJSON wire protocol | `parse_ndjson_to_rows` at `reporter/parser.rs` |\n| R9 deterministic output | sort by `test_id` in `finalize` + `render_from_rows` |\n| R10 View trace link references trace file | `trace_path` field threaded from `TestReport` through `TestRow` to `<button class=\"trace-link\">` |\n\n## Test Plan Coverage (T1–T9)\n\n```\nrunning 9 tests\ntest test_reporter_flag_parses ... ok           (T5 → R5)\ntest test_parser_parses_ndjson ... ok            (T4 → R8)\ntest test_trace_link_reference ... ok            (T9 → R10)\ntest test_test_row_contains_required_fields ... ok (T3 → R3)\ntest test_reporter_emits_index_html ... ok       (T1 → R1)\ntest test_merge_shard_info_aggregated ... ok     (T7 → R7)\ntest test_aggregate_stats_rendered ... ok        (T2 → R2)\ntest test_merge_dedupes_by_test_id ... ok        (T6 → R7)\ntest test_deterministic_output ... ok            (T8 → R9)\n\ntest result: ok. 9 passed; 0 failed\n```\n\n`cargo check -p jet --tests` — clean. All warnings are in unrelated crates (mamba, cclab-api).\n\n## Notes\n\n- `TestReport.error` signature change from `Option<String>` to `Option<TestError>` is additive in spirit (richer type) but is a **breaking change** at the reporter-trait API boundary. All internal call sites in `worker_pool.rs` and `worker.rs` have been updated. Downstream consumers (if any) must wrap their error string in a `TestError { message: s, stack: None, diff: None }` literal.\n- The HTML reporter writes a `results.ndjson` sidecar into its output dir so that the output of one run can be re-fed as input to a downstream `jet report merge` invocation — idempotent chaining.\n- Stable `test_id` is a `DefaultHasher` hash of `file::suite_path::name`. Not cryptographically stable; sufficient for sort ordering and dedup within a single test run.\n"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-changes.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "changes",
+  "content": "```yaml\nchanges:\n  - action: create\n    path: crates/jet/src/reporter/html.rs\n    purpose: >-\n      HtmlReporter struct implementing the Reporter trait. Consumes testEnd\n      events, builds Vec<TestRow>, sorts by test_id, renders index.html from\n      embedded template using include_bytes! assets.\n\n  - action: create\n    path: crates/jet/src/reporter/merger.rs\n    purpose: >-\n      Shard merge logic. Reads ReportData from N input dirs (each produced by\n      HtmlReporter), deduplicates by test_id, recomputes ReportSummary, re-renders\n      unified index.html.\n\n  - action: modify\n    path: crates/jet/src/reporter/mod.rs\n    purpose: >-\n      Re-export HtmlReporter and Merger. Add html variant to ReporterKind enum.\n      Wire HtmlReporter into reporter factory based on RunnerConfig.reporter list.\n\n  - action: create\n    path: crates/jet/assets/html-reporter/index.html\n    purpose: >-\n      HTML shell template. Inline <script> and <style> tags load report.js and\n      report.css at generation time. Contains a JSON data island\n      (<script id=\"data\" type=\"application/json\">) where ReportData is injected.\n\n  - action: create\n    path: crates/jet/assets/html-reporter/report.js\n    purpose: >-\n      Vanilla JS (~300 LOC) that reads the data island, renders the summary panel\n      and test rows, handles filter toggles (passed/failed/skipped), toggles stack\n      trace drawers, and opens trace links via ?trace= query param.\n\n  - action: create\n    path: crates/jet/assets/html-reporter/report.css\n    purpose: >-\n      Self-contained stylesheet for the report UI. No external fonts or CDN.\n      Status-badge colours, collapsible drawer animation, responsive table.\n\n  - action: create\n    path: crates/cclab-jet/src/cli/report.rs\n    purpose: >-\n      CLI module registering jet report view <dir> and\n      jet report merge --input ... --output ... subcommands via the linkme\n      CLI_MODULES distributed slice.\n\n  - action: modify\n    path: crates/cclab-jet/src/cli/mod.rs\n    purpose: Add report module; include report_cli in CLI_MODULES slice.\n\n  - action: modify\n    path: crates/jet/src/test_runner/config.rs\n    purpose: >-\n      Add report_dir: PathBuf field to RunnerConfig (default test-results/report/).\n      Add html to reporter enum; document --report-dir flag.\n\n  - action: modify\n    path: crates/jet/src/cli.rs\n    purpose: >-\n      Extend jet test arg parser: --reporter (comma-split, default term,json),\n      --report-dir. Forward both into RunnerConfig.\n\n  - action: create\n    path: crates/jet/tests/html_reporter_smoke.rs\n    purpose: >-\n      Integration test: feed a canned NDJSON fixture through HtmlReporter;\n      assert index.html exists, contains expected test names and stat counts;\n      golden-diff to verify determinism.\n\n  - action: create\n    path: crates/jet/tests/report_merge_test.rs\n    purpose: >-\n      Unit test: create two fixture shard dirs, run Merger, assert unified report\n      has deduplicated rows and correct aggregate counts.\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-cli.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "cli",
+  "content": "```yaml\n_sdd:\n  id: cli\n\ncommands:\n  - name: jet test\n    description: Run native test runner\n    flags:\n      - name: --reporter\n        type: string\n        description: Comma-separated reporter list. Valid values: term, json, html. Default: term,json.\n        example: --reporter=list,html\n      - name: --report-dir\n        type: string\n        description: Output directory for HTML report. Default: test-results/report/.\n        example: --report-dir ci-artifacts/report\n\n  - name: jet report\n    description: Commands for managing HTML test reports\n    subcommands:\n      - name: view\n        description: Open a report directory in the system default browser.\n        args:\n          - name: dir\n            required: true\n            description: Path to a report directory containing index.html.\n        flags:\n          - name: --serve\n            type: bool\n            description: Serve the report on a local HTTP port instead of opening file:// URL. Port is random.\n        example: jet report view test-results/report\n\n      - name: merge\n        description: Merge N per-shard report directories into a single unified report.\n        flags:\n          - name: --input\n            type: string[]\n            required: true\n            description: Space-separated list of shard report directories.\n          - name: --output\n            type: string\n            required: true\n            description: Destination directory for the merged report.\n        example: jet report merge --input shard-1/report shard-2/report --output merged/report\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-interaction.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "interaction",
+  "content": "```mermaid\n---\nid: interaction\n---\nsequenceDiagram\n    autonumber\n    participant CLI as jet CLI\n    participant Runner as TestRunner\n    participant HtmlRep as HtmlReporter\n    participant FS as FileSystem\n    participant Browser as System Browser\n\n    CLI->>Runner: run(config) where reporter=[html]\n    Runner->>HtmlRep: on_start(plan)\n    HtmlRep->>HtmlRep: store plan metadata\n    loop per test\n        Runner->>HtmlRep: on_test_start(id)\n        Runner->>HtmlRep: on_test_end(id, outcome)\n        HtmlRep->>HtmlRep: accumulate TestRow\n    end\n    Runner->>HtmlRep: on_finish(summary)\n    HtmlRep->>HtmlRep: sort rows by test_id (deterministic)\n    HtmlRep->>HtmlRep: render index.html from template + embedded assets\n    HtmlRep->>FS: write report_dir/index.html\n    HtmlRep->>FS: write report_dir/report.js\n    HtmlRep->>FS: write report_dir/report.css\n    HtmlRep-->>CLI: Ok(report_dir)\n    CLI->>CLI: print \"Report: test-results/report/index.html\"\n\n    note over CLI,Browser: jet report view <dir>\n    CLI->>FS: resolve dir/index.html\n    CLI->>Browser: open(index.html) via open-rs\n    note right of Browser: if --serve: bind random port, serve dir\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-logic.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "logic",
+  "content": "```mermaid\n---\nid: logic\n---\nflowchart TD\n    A([NDJSON event stream]) --> B[Parse testEnd events]\n    B --> C{event.kind == testEnd?}\n    C -- no --> B\n    C -- yes --> D[Build TestRow from payload]\n    D --> E{status?}\n    E -- failed --> F[extract stack_trace, matcher_diff]\n    E -- passed/skipped/flaky --> G[store row]\n    F --> H{trace_path present?}\n    H -- yes --> I[set trace_link = ?trace=rel_path]\n    H -- no --> G\n    I --> G\n    G --> J{more events?}\n    J -- yes --> B\n    J -- no --> K[Compute aggregate stats]\n    K --> L[Sort rows by test_id]\n    L --> M[Render HTML from embedded template]\n    M --> N[Inline report.js + report.css as base64 data URIs]\n    N --> O[Write index.html]\n    O --> P([Done])\n\n    subgraph MergeFlow[Shard Merge]\n        Q([N shard dirs]) --> R[Read each dir NDJSON or cached rows]\n        R --> S[Deduplicate by test_id — last-writer wins]\n        S --> T[Re-sort by test_id]\n        T --> U[Re-render unified index.html]\n        U --> V([Merged dir])\n    end\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-overview.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "overview",
+  "main_spec_ref": "crates/jet/testing/html-reporter.md",
+  "fill_sections": ["overview", "requirements", "scenarios", "interaction", "logic", "state-machine", "cli", "schema", "changes", "test-plan"],
+  "content": "Static HTML report generator for the jet native test runner. After `jet test` completes, `HtmlReporter` consumes the NDJSON wire protocol event stream and writes a self-contained `test-results/report/index.html` — all JS/CSS embedded in the jet binary via `include_bytes!`. The report shows aggregate stats (total/passed/failed/skipped/flaky/duration/shard info) and per-test rows with status badge, duration, source file:line, expandable stack trace drawer, and a \"View trace\" deep-link to `jet trace view`.\n\nKey subsystems:\n- `crates/jet/src/reporter/html.rs` — `HtmlReporter` implementing the `Reporter` trait; consumes `testEnd` events, renders deterministic HTML.\n- `crates/jet/src/reporter/merger.rs` — shard merge algorithm: deduplicates by `test_id`, concatenates per-shard NDJSON streams, emits unified report.\n- `crates/jet/assets/html-reporter/{index.html,report.js,report.css}` — embedded static assets.\n- `crates/cclab-jet/src/cli/report.rs` — `jet report view <dir>` and `jet report merge` subcommands.\n\nActivation: `--reporter=html` or `--reporter=list,html`; default output dir `test-results/report/`. Deterministic output: tests sorted by stable `test_id`, timestamps ISO-8601, optional `--mask-timestamps` for golden-diff CI."
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-requirements.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "requirements",
+  "content": "| id | Requirement | Priority | Verifies |\n|----|-------------|----------|----------|\n| R1 | After `jet test` completes, emit `test-results/report/index.html` (and sibling assets) containing a full test-run overview. | high | Integration test: report dir exists after run |\n| R2 | Overview panel shows: total tests, passed, failed, skipped, flaky counts; total wall-clock duration; shard index/total when `--shard` was used. | high | Unit test: aggregate stats from fixture NDJSON |\n| R3 | Per-test row shows: test name, status badge (passed/failed/skipped/flaky), duration, source file:line, expandable stack-trace drawer (failed only), \"View trace\" link when trace file present. | high | Unit test: row HTML output |\n| R4 | Report is a self-contained static site — HTML/JS/CSS embedded in the jet binary via `include_bytes!`; no CDN or network required at view time. | high | Binary size test; offline open test |\n| R5 | HTML reporter activated via `--reporter=html` or `--reporter=list,html`; `--report-dir <path>` sets output dir (default `test-results/report/`). | high | CLI arg parse test |\n| R6 | `jet report view <dir>` opens the report in the system default browser (or serves locally on a random port if `--serve` is passed). | medium | CLI smoke test |\n| R7 | `jet report merge --input <d1> <d2> ... --output <d>` stitches N per-shard report directories into a single unified report; deduplicates by `test_id`. | high | Merge unit test with two fixture shard dirs |\n| R8 | Reporter reads NDJSON `testEnd` event payloads from the existing wire protocol; no new result format introduced. | high | Protocol conformance test |\n| R9 | HTML output is deterministic for identical input: tests sorted by `test_id`, no embedded wall-clock timestamps unless `--include-timestamps` passed. | medium | Golden-diff test: same NDJSON → byte-identical HTML |\n| R10 | \"View trace\" link invokes `jet trace view <file>` via `?trace=<relative-path>` query param; `jet report view` resolves it by spawning `jet trace view`. | medium | Deep-link resolution test |"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-scenarios.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "scenarios",
+  "content": "| id | Given | When | Then |\n|----|-------|------|------|\n| S1 | `--reporter=html` flag is set | `jet test` completes (all pass) | `test-results/report/index.html` exists; overview shows correct passed count; no stack-trace drawers |\n| S2 | `--reporter=html` flag is set | `jet test` completes with 1 failure | Failed row has status badge \"failed\", stack trace drawer rendered in collapsed state |\n| S3 | `--reporter=list,html` | `jet test` runs | Both terminal summary and HTML report produced |\n| S4 | `--report-dir tmp/my-report` | `jet test` completes | Report written to `tmp/my-report/index.html` (not default path) |\n| S5 | Report directory with `index.html` exists | `jet report view <dir>` invoked | System browser opens the report (or `--serve` starts local server) |\n| S6 | Two shard report dirs `shard-1/` and `shard-2/` exist | `jet report merge --input shard-1 shard-2 --output merged/` invoked | `merged/index.html` aggregates all tests; duplicate `test_id` appears once |\n| S7 | Same NDJSON event stream replayed twice | HTML generation runs twice | Byte-identical `index.html` produced both times (R9 determinism) |\n| S8 | Test produces a trace file; `--trace=retain-on-failure` | Report viewed | \"View trace\" link present in failed row; clicking spawns `jet trace view <trace-path>` |\n| S9 | NDJSON contains shard metadata (`shard_index`, `shard_total`) | Report generated | Overview panel shows shard info line |\n| S10 | Report assets (JS/CSS) are offline | `index.html` opened in browser | Page renders fully without network requests |"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-schema.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "schema",
+  "content": "```json\n{\n  \"$schema\": \"https://json-schema.org/draft/2020-12/schema\",\n  \"$id\": \"html-reporter-data-model\",\n  \"title\": \"HtmlReporterDataModel\",\n  \"definitions\": {\n    \"TestStatus\": {\n      \"enum\": [\"passed\", \"failed\", \"skipped\", \"flaky\"]\n    },\n    \"TestRow\": {\n      \"type\": \"object\",\n      \"required\": [\"test_id\", \"name\", \"status\", \"duration_ms\", \"file\"],\n      \"properties\": {\n        \"test_id\": {\n          \"type\": \"string\",\n          \"description\": \"Stable identifier: sha256(file + describe_stack + test_name). Used for sort order and dedup.\"\n        },\n        \"name\": { \"type\": \"string\", \"description\": \"Full test title including describe stack.\" },\n        \"status\": { \"$ref\": \"#/definitions/TestStatus\" },\n        \"duration_ms\": { \"type\": \"integer\", \"minimum\": 0 },\n        \"file\": { \"type\": \"string\", \"description\": \"Relative path from project root.\" },\n        \"line\": { \"type\": \"integer\", \"description\": \"1-based line number of the test() call.\" },\n        \"stack_trace\": {\n          \"type\": \"string\",\n          \"description\": \"Raw stack string from testEnd payload. Present only when status=failed.\"\n        },\n        \"matcher_diff\": {\n          \"type\": \"string\",\n          \"description\": \"Structured diff from expect() failure. Present only when status=failed.\"\n        },\n        \"trace_path\": {\n          \"type\": \"string\",\n          \"description\": \"Relative path to the .zip trace file for this test. Present only when trace was captured.\"\n        },\n        \"logs\": {\n          \"type\": \"array\",\n          \"items\": { \"type\": \"string\" },\n          \"description\": \"Captured console lines during the test.\"\n        }\n      },\n      \"additionalProperties\": false\n    },\n    \"ShardInfo\": {\n      \"type\": \"object\",\n      \"properties\": {\n        \"index\": { \"type\": \"integer\", \"minimum\": 1 },\n        \"total\": { \"type\": \"integer\", \"minimum\": 1 }\n      },\n      \"required\": [\"index\", \"total\"]\n    },\n    \"ReportSummary\": {\n      \"type\": \"object\",\n      \"required\": [\"total\", \"passed\", \"failed\", \"skipped\", \"flaky\", \"duration_ms\"],\n      \"properties\": {\n        \"total\": { \"type\": \"integer\" },\n        \"passed\": { \"type\": \"integer\" },\n        \"failed\": { \"type\": \"integer\" },\n        \"skipped\": { \"type\": \"integer\" },\n        \"flaky\": { \"type\": \"integer\", \"description\": \"Failed first attempt, passed on retry.\" },\n        \"duration_ms\": { \"type\": \"integer\" },\n        \"shard\": { \"$ref\": \"#/definitions/ShardInfo\" }\n      },\n      \"additionalProperties\": false\n    },\n    \"ReportData\": {\n      \"type\": \"object\",\n      \"required\": [\"version\", \"summary\", \"tests\"],\n      \"properties\": {\n        \"version\": { \"const\": 1, \"description\": \"Schema version for forward-compat.\" },\n        \"summary\": { \"$ref\": \"#/definitions/ReportSummary\" },\n        \"tests\": {\n          \"type\": \"array\",\n          \"items\": { \"$ref\": \"#/definitions/TestRow\" },\n          \"description\": \"Sorted by test_id for deterministic output.\"\n        }\n      },\n      \"additionalProperties\": false\n    }\n  }\n}\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-state-machine.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "state-machine",
+  "content": "Reporter lifecycle state transitions:\n\n```mermaid\n---\nid: state-machine\ninitial: Idle\n---\nstateDiagram-v2\n    [*] --> Idle\n    Idle --> Collecting: on_start(plan)\n    Collecting --> Collecting: on_test_start(id)\n    Collecting --> Collecting: on_test_end(id, outcome) / accumulate row\n    Collecting --> Rendering: on_finish(summary)\n    Rendering --> Writing: sort + render HTML\n    Writing --> Done: write index.html + assets\n    Writing --> Error: I/O failure\n    Done --> [*]\n    Error --> [*]\n\n    state Collecting {\n        [*] --> Waiting\n        Waiting --> InTest: on_test_start\n        InTest --> Waiting: on_test_end\n    }\n```\n\nState meanings:\n- `Idle` — reporter instantiated, no run started.\n- `Collecting` — test run active; accumulating `TestRow` entries from `testEnd` events.\n- `Rendering` — `on_finish` called; sorting rows, computing aggregate stats, filling HTML template.\n- `Writing` — writing `index.html`, `report.js`, `report.css` to `report_dir`.\n- `Done` — all assets written; path printed to stdout.\n- `Error` — I/O error during write; bubbled to runner as non-fatal (terminal summary still shown)."
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec-test-plan.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "section": "test-plan",
+  "main_spec_ref": "crates/jet/testing/html-reporter.md",
+  "fill_sections": ["overview", "requirements", "scenarios", "interaction", "logic", "state-machine", "cli", "schema", "changes", "test-plan"],
+  "content": "<!-- type: test-plan lang: mermaid -->\n\n```mermaid\n---\nid: test-plan\n---\nrequirementDiagram\n\nrequirement R1 {\n  id: R1\n  text: \"jet test produces test-results/report/index.html\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R2 {\n  id: R2\n  text: \"Aggregate stats panel with totals + shard info\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R3 {\n  id: R3\n  text: \"Per-test rows with status badge, duration, file:line, stack trace drawer, trace link\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R4 {\n  id: R4\n  text: \"HTML/JS/CSS embedded via include_bytes! — no CDN\"\n  risk: high\n  verifymethod: analysis\n}\n\nrequirement R5 {\n  id: R5\n  text: \"--reporter=html and --reporter=list,html accepted\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R6 {\n  id: R6\n  text: \"jet report view opens report directory in browser\"\n  risk: medium\n  verifymethod: test\n}\n\nrequirement R7 {\n  id: R7\n  text: \"jet report merge stitches per-shard NDJSON into unified HTML\"\n  risk: high\n  verifymethod: test\n}\n\nrequirement R8 {\n  id: R8\n  text: \"Reporter reads NDJSON wire protocol — no new format\"\n  risk: high\n  verifymethod: analysis\n}\n\nrequirement R9 {\n  id: R9\n  text: \"HTML output byte-identical for identical input\"\n  risk: medium\n  verifymethod: test\n}\n\nrequirement R10 {\n  id: R10\n  text: \"View trace link references per-test trace file path\"\n  risk: medium\n  verifymethod: test\n}\n\nelement T1 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_reporter_emits_index_html\"\n}\nelement T2 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_aggregate_stats_rendered\"\n}\nelement T3 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_test_row_contains_required_fields\"\n}\nelement T4 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_parser_parses_ndjson\"\n}\nelement T5 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_reporter_flag_parses\"\n}\nelement T6 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_merge_dedupes_by_test_id\"\n}\nelement T7 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_merge_shard_info_aggregated\"\n}\nelement T8 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_deterministic_output\"\n}\nelement T9 {\n  type: \"Test\"\n  docref: \"crates/jet/tests/html_reporter_tests.rs::test_trace_link_reference\"\n}\n\nT1 - verifies -> R1\nT2 - verifies -> R2\nT3 - verifies -> R3\nT4 - verifies -> R8\nT5 - verifies -> R5\nT6 - verifies -> R7\nT7 - verifies -> R7\nT8 - verifies -> R9\nT9 - verifies -> R10\n```"
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/review-change-spec.json
+{
+  "spec_id": "enhancement-html-reporter-for-native-test-runner-spec",
+  "verdict": "APPROVED",
+  "summary": "Spec is implementation-ready. Overview, 10 requirements R1-R10, scenarios, interaction, logic, state-machine, cli, schema, changes, and test-plan all filled with substantive content. T1-T9 cover all high-risk requirements via element/verifies edges. No duplicate section types. Sections follow logical order.",
+  "findings": []
+}

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/prompts/analyze_spec_enhancement-html-reporter-for-native-test-runner-spec.md
+# Task: Analyze Spec 'enhancement-html-reporter-for-native-test-runner-spec' for Change 'enhancement-html-reporter-for-native-test-runner'
+
+A skeleton has been generated at `.score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md`.
+
+## CRITICAL: Artifact Writing Rule
+
+**DO NOT use Write or Edit tools to modify spec files directly.**
+You MUST use the artifact CLI command to write each section.
+Direct file writes will be REJECTED and you will have to redo the work.
+
+## Instructions
+
+1. Read context:
+   - Read the issue file in `.score/issues/open/` that initiated this change (see user_input.md for the slug)
+   - The issue's ## Problem, ## Requirements, ## Scope, and ## Reference Context sections are your primary context
+2. Read the skeleton: `.score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md`
+3. **IMPORTANT — `main_spec_ref`**: Check the spec frontmatter. If `main_spec_ref` is `~` (null),
+   you MUST determine the target path in `.score/tech_design/` where this spec will be merged.
+   Format: `<scope>/<category>/<spec-id>.md` (e.g., `sdd/tools/new-feature.md`).
+   Browse `.score/tech_design/` to see existing spec groups.
+   Pass it as the `main_spec_ref` parameter when calling the artifact CLI.
+4. Decide which sections to fill based on the nature of the change. Pick ONLY leaf section names from this list — NEVER pass umbrella words like `diagrams`, `api_spec`, or `test_plan`:
+   Always fill: `overview`, `requirements`, `scenarios`, `changes`
+   Diagrams (pick those that apply): `interaction`, `logic`, `state-machine`, `mindmap`, `dependency`, `db-model`
+   API shape (pick those that apply): `rest-api`, `rpc-api`, `async-api`, `cli`, `schema`, `config`
+   UI (pick those that apply): `wireframe`, `component`, `design-token`
+   Testing: `test-plan` (Mermaid+ requirement diagram with BDD Given/When/Then)
+   Docs: `doc`
+5. Write a JSON payload file to `.score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec.json` then run the artifact CLI.
+
+## Expected Action
+
+Write the **overview** section first via artifact CLI. Pass the `fill_sections`
+array as a parameter — USE LEAF NAMES ONLY from the allowed list above.
+Example (adapt to this change): `fill_sections=["overview", "requirements", "scenarios", "interaction", "logic", "changes"]`.
+Never pass `diagrams`, `api_spec`, or `test_plan` (umbrella names).
+Also pass `main_spec_ref` as a parameter if determined above.
+The system persists it to frontmatter automatically.
+
+Then call the artifact CLI for each remaining section in sequence.
+
+## CLI Commands
+
+```
+# Read artifacts
+Read file: .score/changes/enhancement-html-reporter-for-native-test-runner/proposal.md
+Read file: .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md
+
+# Write each section (MUST use this — do NOT edit spec files directly)
+# Step 1: Write payload JSON to the EXACT path below (do NOT write to other locations)
+# Step 2: Run artifact CLI
+score artifact create-change-spec enhancement-html-reporter-for-native-test-runner .score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-spec.json
+```

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/prompts/begin_implementation.md
+# Task: Begin Implementation for Change 'enhancement-html-reporter-for-native-test-runner'
+
+## Instructions
+
+1. List all change specs in `.score/changes/enhancement-html-reporter-for-native-test-runner/`
+2. Read spec **enhancement-html-reporter-for-native-test-runner-spec** to understand requirements: `.score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md`
+3. Implement **production code only** (no `#[test]` functions) for each change spec in order, starting with **enhancement-html-reporter-for-native-test-runner-spec**
+4. When done with enhancement-html-reporter-for-native-test-runner-spec, run `score workflow create-change-implementation enhancement-html-reporter-for-native-test-runner` to advance
+
+## Spec Annotations
+
+Add `@spec` annotations to public functions that implement spec requirements.
+For each public function or method,
+add a comment: `// @spec {spec_path}#R{N}` where `{spec_path}` is the
+spec file path and `R{N}` is the requirement ID from the spec's Requirements table.
+
+Use the comment syntax appropriate for the language:
+```
+// @spec .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md#R1   (Rust, JS, TS, Go, C)
+#  @spec .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md#R1   (Python, Ruby, Shell, YAML)
+-- @spec .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md#R1   (SQL)
+<!-- @spec .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md#R1 --> (HTML, Markdown)
+/* @spec .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md#R1 */    (CSS, C block)
+```
+
+This annotation enables automated spec↔code traceability.
+Place the annotation on the line immediately above the function signature.
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md
+
+# Advance implementation workflow
+score workflow create-change-implementation enhancement-html-reporter-for-native-test-runner
+
+# Code intelligence — explore codebase before making changes
+score symbols <file>              # list symbols in a file
+score hover <file> <line> <col>   # type info for a symbol
+score references <file> <line> <col>  # find all references
+score impact <file> <line> <col>  # analyze change impact
+score context <file:symbol...> [--depth N]  # cross-ref context
+```
--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/prompts/implement_tests_enhancement-html-reporter-for-native-test-runner-spec.md
+# Task: Implement Tests for Spec 'enhancement-html-reporter-for-native-test-runner-spec' (Change 'enhancement-html-reporter-for-native-test-runner')
+
+## Instructions
+
+Production code for spec 'enhancement-html-reporter-for-native-test-runner-spec' has been implemented and verified to compile.
+Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).
+
+1. Read spec **enhancement-html-reporter-for-native-test-runner-spec**: `.score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md`
+2. Read the `## Test Plan` section to understand required test cases
+3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
+4. Run `cargo test` to verify tests pass
+5. When done, run `score workflow create-change-implementation enhancement-html-reporter-for-native-test-runner` to advance
+
+## CLI Commands
+
+```
+# Read spec
+Read file: .score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md
+
+# Run tests
+cargo test
+
+# Advance implementation workflow
+score workflow create-change-implementation enhancement-html-reporter-for-native-test-runner
+```
--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/prompts/write_implementation_diff.md
+# Task: Write Implementation Diff for Change 'enhancement-html-reporter-for-native-test-runner'
+
+## Instructions
+
+1. Run `git diff` (or `git diff HEAD~N..HEAD` if already committed) to get the full diff
+2. Write `implementation.md` via the artifact CLI command
+3. The artifact tool will redirect back to the workflow router automatically
+
+## CLI Commands
+
+```
+# Write implementation artifact (write payload JSON first, then run)
+score artifact create-change-implementation enhancement-html-reporter-for-native-test-runner .score/changes/enhancement-html-reporter-for-native-test-runner/payloads/create-change-implementation.json
+```
--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/review_spec_enhancement-html-reporter-for-native-test-runner-spec.md
+---
+verdict: APPROVED
+review_iteration: 1
+---
+
+# Review: enhancement-html-reporter-for-native-test-runner-spec
+
+**Verdict**: APPROVED
+
+Spec is implementation-ready. All required sections filled with substantive content.

--- /dev/null
+++ b/.score/changes/enhancement-html-reporter-for-native-test-runner/specs/enhancement-html-reporter-for-native-test-runner-spec.md
+---
+id: enhancement-html-reporter-for-native-test-runner-spec
+main_spec_ref: "crates/jet/testing/html-reporter.md"
+merge_strategy: new
+fill_sections: [overview, requirements, scenarios, interaction, logic, state-machine, cli, schema, changes, test-plan]
+create_complete: true
+---
+
+# Enhancement Html Reporter For Native Test Runner Spec
+
+## Overview
+<!-- type: overview lang: markdown -->
+
+Static HTML report generator for the jet native test runner. After `jet test` completes, `HtmlReporter` consumes the NDJSON wire protocol event stream and writes a self-contained `test-results/report/index.html` — all JS/CSS embedded in the jet binary via `include_bytes!`. The report shows aggregate stats (total/passed/failed/skipped/flaky/duration/shard info) and per-test rows with status badge, duration, source file:line, expandable stack trace drawer, and a "View trace" deep-link to `jet trace view`.
+
+Key subsystems:
+- `crates/jet/src/reporter/html.rs` — `HtmlReporter` implementing the `Reporter` trait; consumes `testEnd` events, renders deterministic HTML.
+- `crates/jet/src/reporter/merger.rs` — shard merge algorithm: deduplicates by `test_id`, concatenates per-shard NDJSON streams, emits unified report.
+- `crates/jet/assets/html-reporter/{index.html,report.js,report.css}` — embedded static assets.
+- `crates/cclab-jet/src/cli/report.rs` — `jet report view <dir>` and `jet report merge` subcommands.
+
+Activation: `--reporter=html` or `--reporter=list,html`; default output dir `test-results/report/`. Deterministic output: tests sorted by stable `test_id`, timestamps ISO-8601, optional `--mask-timestamps` for golden-diff CI.
+## Requirements
+<!-- type: requirements lang: mermaid -->
+
+| id | Requirement | Priority | Verifies |
+|----|-------------|----------|----------|
+| R1 | After `jet test` completes, emit `test-results/report/index.html` (and sibling assets) containing a full test-run overview. | high | Integration test: report dir exists after run |
+| R2 | Overview panel shows: total tests, passed, failed, skipped, flaky counts; total wall-clock duration; shard index/total when `--shard` was used. | high | Unit test: aggregate stats from fixture NDJSON |
+| R3 | Per-test row shows: test name, status badge (passed/failed/skipped/flaky), duration, source file:line, expandable stack-trace drawer (failed only), "View trace" link when trace file present. | high | Unit test: row HTML output |
+| R4 | Report is a self-contained static site — HTML/JS/CSS embedded in the jet binary via `include_bytes!`; no CDN or network required at view time. | high | Binary size test; offline open test |
+| R5 | HTML reporter activated via `--reporter=html` or `--reporter=list,html`; `--report-dir <path>` sets output dir (default `test-results/report/`). | high | CLI arg parse test |
+| R6 | `jet report view <dir>` opens the report in the system default browser (or serves locally on a random port if `--serve` is passed). | medium | CLI smoke test |
+| R7 | `jet report merge --input <d1> <d2> ... --output <d>` stitches N per-shard report directories into a single unified report; deduplicates by `test_id`. | high | Merge unit test with two fixture shard dirs |
+| R8 | Reporter reads NDJSON `testEnd` event payloads from the existing wire protocol; no new result format introduced. | high | Protocol conformance test |
+| R9 | HTML output is deterministic for identical input: tests sorted by `test_id`, no embedded wall-clock timestamps unless `--include-timestamps` passed. | medium | Golden-diff test: same NDJSON → byte-identical HTML |
+| R10 | "View trace" link invokes `jet trace view <file>` via `?trace=<relative-path>` query param; `jet report view` resolves it by spawning `jet trace view`. | medium | Deep-link resolution test |
+## Scenarios
+<!-- type: scenarios lang: markdown -->
+
+| id | Given | When | Then |
+|----|-------|------|------|
+| S1 | `--reporter=html` flag is set | `jet test` completes (all pass) | `test-results/report/index.html` exists; overview shows correct passed count; no stack-trace drawers |
+| S2 | `--reporter=html` flag is set | `jet test` completes with 1 failure | Failed row has status badge "failed", stack trace drawer rendered in collapsed state |
+| S3 | `--reporter=list,html` | `jet test` runs | Both terminal summary and HTML report produced |
+| S4 | `--report-dir tmp/my-report` | `jet test` completes | Report written to `tmp/my-report/index.html` (not default path) |
+| S5 | Report directory with `index.html` exists | `jet report view <dir>` invoked | System browser opens the report (or `--serve` starts local server) |
+| S6 | Two shard report dirs `shard-1/` and `shard-2/` exist | `jet report merge --input shard-1 shard-2 --output merged/` invoked | `merged/index.html` aggregates all tests; duplicate `test_id` appears once |
+| S7 | Same NDJSON event stream replayed twice | HTML generation runs twice | Byte-identical `index.html` produced both times (R9 determinism) |
+| S8 | Test produces a trace file; `--trace=retain-on-failure` | Report viewed | "View trace" link present in failed row; clicking spawns `jet trace view <trace-path>` |
+| S9 | NDJSON contains shard metadata (`shard_index`, `shard_total`) | Report generated | Overview panel shows shard info line |
+| S10 | Report assets (JS/CSS) are offline | `index.html` opened in browser | Page renders fully without network requests |
+## Mindmap
+<!-- type: mindmap lang: mermaid -->
+<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: mindmap
+---
+mindmap
+  root((System))
+    Component A
+    Component B
+```
+-->
+
+## State Machine
+<!-- type: state-machine lang: mermaid -->
+
+Reporter lifecycle state transitions:
+
+```mermaid
+---
+id: state-machine
+initial: Idle
+---
+stateDiagram-v2
+    [*] --> Idle
+    Idle --> Collecting: on_start(plan)
+    Collecting --> Collecting: on_test_start(id)
+    Collecting --> Collecting: on_test_end(id, outcome) / accumulate row
+    Collecting --> Rendering: on_finish(summary)
+    Rendering --> Writing: sort + render HTML
+    Writing --> Done: write index.html + assets
+    Writing --> Error: I/O failure
+    Done --> [*]
+    Error --> [*]
+
+    state Collecting {
+        [*] --> Waiting
+        Waiting --> InTest: on_test_start
+        InTest --> Waiting: on_test_end
+    }
+```
+
+State meanings:
+- `Idle` — reporter instantiated, no run started.
+- `Collecting` — test run active; accumulating `TestRow` entries from `testEnd` events.
+- `Rendering` — `on_finish` called; sorting rows, computing aggregate stats, filling HTML template.
+- `Writing` — writing `index.html`, `report.js`, `report.css` to `report_dir`.
+- `Done` — all assets written; path printed to stdout.
+- `Error` — I/O error during write; bubbled to runner as non-fatal (terminal summary still shown).
+## Interaction
+<!-- type: interaction lang: mermaid -->
+
+```mermaid
+---
+id: interaction
+---
+sequenceDiagram
+    autonumber
+    participant CLI as jet CLI
+    participant Runner as TestRunner
+    participant HtmlRep as HtmlReporter
+    participant FS as FileSystem
+    participant Browser as System Browser
+
+    CLI->>Runner: run(config) where reporter=[html]
+    Runner->>HtmlRep: on_start(plan)
+    HtmlRep->>HtmlRep: store plan metadata
+    loop per test
+        Runner->>HtmlRep: on_test_start(id)
+        Runner->>HtmlRep: on_test_end(id, outcome)
+        HtmlRep->>HtmlRep: accumulate TestRow
+    end
+    Runner->>HtmlRep: on_finish(summary)
+    HtmlRep->>HtmlRep: sort rows by test_id (deterministic)
+    HtmlRep->>HtmlRep: render index.html from template + embedded assets
+    HtmlRep->>FS: write report_dir/index.html
+    HtmlRep->>FS: write report_dir/report.js
+    HtmlRep->>FS: write report_dir/report.css
+    HtmlRep-->>CLI: Ok(report_dir)
+    CLI->>CLI: print "Report: test-results/report/index.html"
+
+    note over CLI,Browser: jet report view <dir>
+    CLI->>FS: resolve dir/index.html
+    CLI->>Browser: open(index.html) via open-rs
+    note right of Browser: if --serve: bind random port, serve dir
+```
+## Logic
+<!-- type: logic lang: mermaid -->
+
+```mermaid
+---
+id: logic
+---
+flowchart TD
+    A([NDJSON event stream]) --> B[Parse testEnd events]
+    B --> C{event.kind == testEnd?}
+    C -- no --> B
+    C -- yes --> D[Build TestRow from payload]
+    D --> E{status?}
+    E -- failed --> F[extract stack_trace, matcher_diff]
+    E -- passed/skipped/flaky --> G[store row]
+    F --> H{trace_path present?}
+    H -- yes --> I[set trace_link = ?trace=rel_path]
+    H -- no --> G
+    I --> G
+    G --> J{more events?}
+    J -- yes --> B
+    J -- no --> K[Compute aggregate stats]
+    K --> L[Sort rows by test_id]
+    L --> M[Render HTML from embedded template]
+    M --> N[Inline report.js + report.css as base64 data URIs]
+    N --> O[Write index.html]
+    O --> P([Done])
+
+    subgraph MergeFlow[Shard Merge]
+        Q([N shard dirs]) --> R[Read each dir NDJSON or cached rows]
+        R --> S[Deduplicate by test_id — last-writer wins]
+        S --> T[Re-sort by test_id]
+        T --> U[Re-render unified index.html]
+        U --> V([Merged dir])
+    end
+```
+## Dependencies
+<!-- type: dependency lang: mermaid -->
+<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: dependency
+---
+classDiagram
+    class ComponentA
+    class ComponentB
+    ComponentA --> ComponentB
+```
+-->
+
+## Data Model
+<!-- type: db-model lang: mermaid -->
+<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: db-model
+---
+erDiagram
+    ENTITY {
+        string id PK
+    }

--- /dev/null
+++ b/crates/jet/assets/html-reporter/index.html
+<!DOCTYPE html>
+<html lang="en">
+<head>
+<meta charset="UTF-8">
+<meta name="viewport" content="width=device-width, initial-scale=1.0">
+<title>Jet Test Report</title>
+<style>
+<!-- REPORT_CSS -->
+</style>
+</head>
+<body>
+<div id="app">
+  <header class="header">
+    <h1>Jet Test Report</h1>
+    <!-- SHARD_INFO -->
+  </header>
+  <div class="stats" id="stats">
+    <!-- STATS -->
+  </div>
+  <div class="filters" id="filters">
+    <button class="pill pill-all active" data-filter="all">All</button>
+    <button class="pill pill-passed" data-filter="passed">Passed</button>
+    <button class="pill pill-failed" data-filter="failed">Failed</button>
+    <button class="pill pill-skipped" data-filter="skipped">Skipped</button>
+  </div>
+  <table class="test-table" id="test-table">
+    <thead>
+      <tr>
+        <th>Status</th>
+        <th>Test Name</th>
+        <th>File</th>
+        <th>Duration</th>
+        <th>Actions</th>
+      </tr>
+    </thead>
+    <tbody id="test-rows">
+      <!-- TEST_ROWS -->
+    </tbody>
+  </table>
+</div>
+<script id="report-data" type="application/json"><!-- REPORT_DATA --></script>
+<script>
+<!-- REPORT_JS -->
+</script>
+</body>
+</html>

--- /dev/null
+++ b/crates/jet/assets/html-reporter/report.css
+/* Jet HTML Reporter Stylesheet */
+*, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }
+
+body {
+  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
+  font-size: 14px;
+  background: #f5f5f5;
+  color: #333;
+}
+
+#app {
+  max-width: 1200px;
+  margin: 0 auto;
+  padding: 24px;
+}
+
+.header {
+  display: flex;
+  align-items: center;
+  gap: 16px;
+  margin-bottom: 20px;
+}
+
+.header h1 {
+  font-size: 20px;
+  font-weight: 600;
+  color: #1a1a1a;
+}
+
+.shard-info {
+  font-size: 12px;
+  color: #666;
+  background: #e8e8e8;
+  padding: 3px 8px;
+  border-radius: 4px;
+}
+
+.stats {
+  display: flex;
+  gap: 12px;
+  margin-bottom: 16px;
+  flex-wrap: wrap;
+}
+
+.stat-tile {
+  background: #fff;
+  border: 1px solid #e0e0e0;
+  border-radius: 8px;
+  padding: 12px 20px;
+  min-width: 100px;
+  text-align: center;
+}
+
+.stat-tile .stat-value {
+  font-size: 28px;
+  font-weight: 700;
+  line-height: 1;
+}
+
+.stat-tile .stat-label {
+  font-size: 11px;
+  color: #888;
+  margin-top: 4px;
+  text-transform: uppercase;
+  letter-spacing: 0.05em;
+}
+
+.stat-tile.passed .stat-value { color: #16a34a; }
+.stat-tile.failed .stat-value { color: #dc2626; }
+.stat-tile.skipped .stat-value { color: #ca8a04; }
+.stat-tile.flaky .stat-value { color: #ea580c; }
+.stat-tile.total .stat-value { color: #1d4ed8; }
+.stat-tile.duration .stat-value { font-size: 20px; color: #555; }
+
+.filters {
+  display: flex;
+  gap: 8px;
+  margin-bottom: 16px;
+}
+
+.pill {
+  border: none;
+  border-radius: 20px;
+  padding: 6px 14px;
+  font-size: 13px;
+  cursor: pointer;
+  background: #e5e7eb;
+  color: #374151;
+  transition: background 0.15s, color 0.15s;
+}
+
+.pill:hover { background: #d1d5db; }
+.pill.active { background: #1d4ed8; color: #fff; }
+
+.pill-passed.active { background: #16a34a; }
+.pill-failed.active { background: #dc2626; }
+.pill-skipped.active { background: #ca8a04; }
+
+.test-table {
+  width: 100%;
+  border-collapse: collapse;
+  background: #fff;
+  border-radius: 8px;
+  overflow: hidden;
+  border: 1px solid #e0e0e0;
+}
+
+.test-table thead {
+  background: #f9fafb;
+}
+
+.test-table th, .test-table td {
+  padding: 10px 14px;
+  text-align: left;
+  border-bottom: 1px solid #e5e7eb;
+}
+
+.test-table th {
+  font-size: 11px;
+  text-transform: uppercase;
+  letter-spacing: 0.05em;
+  color: #6b7280;
+  font-weight: 600;
+}
+
+.test-table tr:last-child td { border-bottom: none; }
+.test-table tr:hover td { background: #f9fafb; }
+
+.badge {
+  display: inline-block;
+  padding: 2px 8px;
+  border-radius: 4px;
+  font-size: 11px;
+  font-weight: 600;
+  text-transform: uppercase;
+  letter-spacing: 0.04em;
+}
+
+.badge-passed { background: #dcfce7; color: #15803d; }
+.badge-failed { background: #fee2e2; color: #b91c1c; }
+.badge-skipped { background: #fef9c3; color: #a16207; }
+.badge-flaky { background: #ffedd5; color: #c2410c; }
+.badge-timedout { background: #f3e8ff; color: #7c3aed; }
+
+.test-name { font-weight: 500; }
+.test-file { font-size: 12px; color: #6b7280; font-family: monospace; }
+.test-duration { font-size: 12px; color: #9ca3af; white-space: nowrap; }
+
+.trace-link {
+  font-size: 12px;
+  color: #1d4ed8;
+  text-decoration: none;
+  cursor: pointer;
+  background: none;
+  border: none;
+  padding: 0;
+}
+
+.trace-link:hover { text-decoration: underline; }
+
+.toggle-btn {
+  background: none;
+  border: 1px solid #e5e7eb;
+  border-radius: 4px;
+  padding: 2px 8px;
+  font-size: 11px;
+  cursor: pointer;
+  color: #6b7280;
+}
+
+.toggle-btn:hover { background: #f3f4f6; }
+
+/* Drawer row */
+.drawer-row { display: none; }
+.drawer-row.open { display: table-row; }
+
+.drawer-content {
+  background: #1e293b;
+  padding: 16px;
+}
+
+.drawer-label {
+  font-size: 11px;
+  color: #94a3b8;
+  margin-bottom: 6px;
+  text-transform: uppercase;
+  letter-spacing: 0.05em;
+}
+
+.stack-trace {
+  font-family: "SF Mono", "Fira Code", Consolas, monospace;
+  font-size: 12px;
+  color: #e2e8f0;
+  white-space: pre-wrap;
+  word-break: break-all;
+  line-height: 1.5;
+}
+
+.matcher-diff {
+  font-family: "SF Mono", "Fira Code", Consolas, monospace;

--- /dev/null
+++ b/crates/jet/assets/html-reporter/report.js
+/* Jet HTML Reporter — report.js */
+(function() {
+  'use strict';
+
+  // Read data island injected by Rust side
+  var dataEl = document.getElementById('report-data');
+  var reportData = null;
+  if (dataEl) {
+    try { reportData = JSON.parse(dataEl.textContent); } catch(e) {}
+  }
+
+  var currentFilter = 'all';
+
+  function badgeClass(status) {
+    switch (status) {
+      case 'passed': return 'badge-passed';
+      case 'failed': return 'badge-failed';
+      case 'skipped': return 'badge-skipped';
+      case 'flaky': return 'badge-flaky';
+      default: return 'badge-timedout';
+    }
+  }
+
+  function escHtml(s) {
+    if (!s) return '';
+    return String(s)
+      .replace(/&/g, '&amp;')
+      .replace(/</g, '&lt;')
+      .replace(/>/g, '&gt;')
+      .replace(/"/g, '&quot;');
+  }
+
+  function formatDiff(diff) {
+    if (!diff) return '';
+    return diff.split('\n').map(function(line) {
+      if (line.startsWith('+')) return '<span class="diff-line-add">' + escHtml(line) + '</span>';
+      if (line.startsWith('-')) return '<span class="diff-line-remove">' + escHtml(line) + '</span>';
+      return '<span class="diff-line-ctx">' + escHtml(line) + '</span>';
+    }).join('\n');
+  }
+
+  function handleTraceLink(e) {
+    e.preventDefault();
+    var tracePath = this.dataset.tracePath;
+    if (tracePath) {
+      // Navigate to ?trace=<path> so jet report view can handle it
+      window.location.search = '?trace=' + encodeURIComponent(tracePath);
+    }
+  }
+
+  function toggleDrawer(rowId) {
+    var drawer = document.getElementById('drawer-' + rowId);
+    if (drawer) {
+      if (drawer.classList.contains('open')) {
+        drawer.classList.remove('open');
+      } else {
+        drawer.classList.add('open');
+      }
+    }
+  }
+
+  function applyFilter(filter) {
+    currentFilter = filter;
+    var pills = document.querySelectorAll('.pill');
+    pills.forEach(function(pill) {
+      pill.classList.toggle('active', pill.dataset.filter === filter);
+    });
+
+    var rows = document.querySelectorAll('tr[data-status]');
+    rows.forEach(function(row) {
+      var status = row.dataset.status;
+      var show = filter === 'all' || status === filter;
+      row.style.display = show ? '' : 'none';
+
+      // Also hide the corresponding drawer row
+      var drawerId = row.dataset.drawerId;
+      if (drawerId) {
+        var drawerRow = document.getElementById('drawer-' + drawerId);
+        if (drawerRow) {
+          if (!show) {
+            drawerRow.classList.remove('open');
+            drawerRow.style.display = 'none';
+          } else {
+            drawerRow.style.display = '';
+          }
+        }
+      }
+    });
+  }
+
+  function init() {
+    // Wire filter pills
+    document.querySelectorAll('.pill').forEach(function(pill) {
+      pill.addEventListener('click', function() {
+        applyFilter(this.dataset.filter);
+      });
+    });
+
+    // Wire toggle buttons
+    document.querySelectorAll('.toggle-btn').forEach(function(btn) {
+      btn.addEventListener('click', function() {
+        toggleDrawer(this.dataset.rowId);
+      });
+    });
+
+    // Wire trace links
+    document.querySelectorAll('.trace-link').forEach(function(link) {
+      link.addEventListener('click', handleTraceLink);
+    });
+
+    // Handle ?trace= query param on page load
+    var params = new URLSearchParams(window.location.search);
+    var traceParam = params.get('trace');
+    if (traceParam) {
+      // The report viewer backend handles this; just show a message if no backend
+      console.log('Trace requested:', traceParam);
+    }
+  }
+
+  if (document.readyState === 'loading') {
+    document.addEventListener('DOMContentLoaded', init);
+  } else {
+    init();
+  }
+})();

--- /dev/null
+++ b/crates/jet/src/reporter/html.rs
+//! HTML reporter — writes `index.html` + embedded assets to a report directory.
+//!
+//! Assets (`report.js`, `report.css`) are embedded in the binary via
+//! `include_bytes!` so the report is fully self-contained with no CDN or
+//! network dependency at view time.
+//!
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
+
+use crate::test_runner::reporter::{Outcome, TestReport};
+use anyhow::{Context, Result};
+use std::path::{Path, PathBuf};
+
+// Embedded static assets — included at compile time.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
+const HTML_TEMPLATE: &str = include_str!("../../assets/html-reporter/index.html");
+/// Embedded `report.js` — accessible by merge.rs for writing sidecar assets.
+pub const REPORT_JS: &str = include_str!("../../assets/html-reporter/report.js");
+/// Embedded `report.css` — accessible by merge.rs for writing sidecar assets.
+pub const REPORT_CSS: &str = include_str!("../../assets/html-reporter/report.css");
+
+/// A single test result row for the HTML report.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
+#[derive(Debug, Clone)]
+pub struct TestRow {
+    /// Stable sort key: test_id is derived from file + suite path + test name.
+    pub test_id: String,
+    pub name: String,
+    pub status: String,
+    pub duration_ms: u64,
+    pub file: String,
+    pub stack_trace: Option<String>,
+    pub matcher_diff: Option<String>,
+    /// Relative path to the `.zip` trace file.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R10
+    pub trace_path: Option<String>,
+}
+
+/// HTML reporter — accumulates test results, renders a self-contained
+/// `index.html` report on `finalize()`.
+///
+/// Usage:
+/// ```ignore
+/// let mut r = HtmlReporter::new("test-results/report");
+/// r.emit(report);
+/// r.finalize()?;
+/// ```
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
+pub struct HtmlReporter {
+    pub out_dir: PathBuf,
+    rows: Vec<TestRow>,
+    /// Optional shard info `(index, total)`.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R2
+    pub shard: Option<(u32, u32)>,
+}
+
+impl HtmlReporter {
+    /// Create a new `HtmlReporter` writing output to `out_dir`.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+    pub fn new(out_dir: impl Into<PathBuf>) -> Self {
+        Self {
+            out_dir: out_dir.into(),
+            rows: Vec::new(),
+            shard: None,
+        }
+    }
+
+    /// Accept one completed test result and accumulate it as a `TestRow`.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R8
+    pub fn emit(&mut self, report: TestReport) {
+        let status = outcome_to_status(&report.outcome);
+
+        // Derive a stable test_id from file + suite + name.
+        let suite_path = report.suite.join(" > ");
+        let raw_id = format!(
+            "{}::{}::{}",
+            report.file.display(),
+            suite_path,
+            report.name
+        );
+        let test_id = stable_id(&raw_id);
+
+        let full_name = if report.suite.is_empty() {
+            report.name.clone()
+        } else {
+            format!("{} > {}", report.suite.join(" > "), report.name)
+        };
+
+        let (stack_trace, matcher_diff) = match &report.error {
+            Some(err) => (err.stack.clone(), err.diff.clone()),
+            None => (None, None),
+        };
+
+        // Propagate shard info from the first report that has it.
+        if self.shard.is_none() {
+            if let (Some(idx), Some(total)) = (report.shard_index, report.shard_total) {
+                self.shard = Some((idx, total));
+            }
+        }
+
+        let trace_path = report
+            .trace_path
+            .as_ref()
+            .map(|p| p.to_string_lossy().into_owned());
+
+        self.rows.push(TestRow {
+            test_id,
+            name: full_name,
+            status,
+            duration_ms: report.duration_ms,
+            file: report
+                .file
+                .strip_prefix(&std::env::current_dir().unwrap_or_default())
+                .unwrap_or(&report.file)
+                .to_string_lossy()
+                .into_owned(),
+            stack_trace,
+            matcher_diff,
+            trace_path,
+        });
+    }
+
+    /// Sort rows deterministically, render HTML, and write all assets to
+    /// `out_dir`. Returns `Ok(())` on success.
+    ///
+    /// Output files: `index.html`, `report.js`, `report.css`.
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+    // @spec enhancement-html-reporter-for-native-test-runner-spec#R9
+    pub fn finalize(&mut self) -> Result<()> {
+        // Sort by test_id for deterministic output.
+        // @spec enhancement-html-reporter-for-native-test-runner-spec#R9
+        self.rows.sort_by(|a, b| a.test_id.cmp(&b.test_id));
+
+        std::fs::create_dir_all(&self.out_dir).with_context(|| {
+            format!("Failed to create report dir: {}", self.out_dir.display())
+        })?;
+
+        let html = render_html(&self.rows, self.shard);
+
+        std::fs::write(self.out_dir.join("index.html"), &html)
+            .context("Failed to write index.html")?;
+        std::fs::write(self.out_dir.join("report.js"), REPORT_JS)
+            .context("Failed to write report.js")?;
+        std::fs::write(self.out_dir.join("report.css"), REPORT_CSS)
+            .context("Failed to write report.css")?;
+
+        println!("Report: {}/index.html", self.out_dir.display());
+        Ok(())
+    }
+
+    /// Consume the accumulated rows (used by the merger).
+    pub fn into_rows(self) -> Vec<TestRow> {
+        self.rows
+    }
+}
+
+/// Render the full HTML from `rows` + optional `shard` metadata.
+///
+/// Uses simple string substitution on `<!-- PLACEHOLDER -->` markers.
+/// No external template engine required.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R9
+pub fn render_html(rows: &[TestRow], shard: Option<(u32, u32)>) -> String {
+    let stats_html = build_stats_html(rows);
+    let shard_html = build_shard_html(shard);
+    let rows_html = build_rows_html(rows);
+
+    HTML_TEMPLATE
+        .replace("<!-- REPORT_CSS -->", REPORT_CSS)
+        .replace("<!-- REPORT_JS -->", REPORT_JS)
+        .replace("<!-- STATS -->", &stats_html)
+        .replace("<!-- SHARD_INFO -->", &shard_html)
+        .replace("<!-- TEST_ROWS -->", &rows_html)
+        .replace("<!-- REPORT_DATA -->", &build_report_data_json(rows, shard))
+}
+
+/// Build the aggregate stats panel HTML.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
+fn build_stats_html(rows: &[TestRow]) -> String {
+    let total = rows.len();
+    let passed = rows.iter().filter(|r| r.status == "passed").count();
+    let failed = rows.iter().filter(|r| r.status == "failed").count();
+    let skipped = rows.iter().filter(|r| r.status == "skipped").count();
+    let flaky = rows.iter().filter(|r| r.status == "flaky").count();
+    let duration_ms: u64 = rows.iter().map(|r| r.duration_ms).sum();
+    let duration_s = duration_ms as f64 / 1000.0;
+
+    format!(
+        r#"<div class="stat-tile total"><div class="stat-value">{total}</div><div class="stat-label">Total</div></div>
+<div class="stat-tile passed"><div class="stat-value">{passed}</div><div class="stat-label">Passed</div></div>
+<div class="stat-tile failed"><div class="stat-value">{failed}</div><div class="stat-label">Failed</div></div>
+<div class="stat-tile skipped"><div class="stat-value">{skipped}</div><div class="stat-label">Skipped</div></div>
+<div class="stat-tile flaky"><div class="stat-value">{flaky}</div><div class="stat-label">Flaky</div></div>
+<div class="stat-tile duration"><div class="stat-value">{duration_s:.2}s</div><div class="stat-label">Duration</div></div>"#
+    )
+}
+
+/// Build the shard info line (empty string if no shard).

--- /dev/null
+++ b/crates/jet/src/reporter/merge.rs
+//! Shard merger — stitches N per-shard report directories into a single
+//! unified HTML report.
+//!
+//! The merge algorithm:
+//! 1. Read each input directory's `results.ndjson` sidecar (preferred) or fall
+//!    back to an empty list.
+//! 2. Deduplicate rows by `test_id` (last-writer wins — shards are assumed
+//!    disjoint, but dedup handles overlapping coverage in pathological cases).
+//! 3. Re-sort by `test_id` for deterministic output.
+//! 4. Re-render a unified `index.html` into `output`.
+//!
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+
+use crate::reporter::html::{render_from_rows, TestRow, REPORT_JS, REPORT_CSS};
+use anyhow::{Context, Result};
+use std::collections::HashMap;
+use std::path::{Path, PathBuf};
+
+/// Merge N shard report directories into a single unified report at `output`.
+///
+/// Each input directory should contain a `results.ndjson` sidecar produced by
+/// `finalize_with_sidecar`. Missing sidecars are skipped gracefully.
+///
+/// Deduplication: rows with the same `test_id` are deduplicated (last input
+/// directory wins — assumes disjoint shards).
+///
+/// `output` is created if it does not exist.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+pub fn merge_reports(inputs: &[PathBuf], output: &Path) -> Result<()> {
+    let shard_count = inputs.len() as u32;
+    let mut by_id: HashMap<String, TestRow> = HashMap::new();
+
+    for input_dir in inputs {
+        let rows = crate::reporter::html::read_rows_from_dir(input_dir);
+        for row in rows {
+            by_id.insert(row.test_id.clone(), row);
+        }
+    }
+
+    // Convert to sorted vec.
+    let mut merged: Vec<TestRow> = by_id.into_values().collect();
+    merged.sort_by(|a, b| a.test_id.cmp(&b.test_id));
+
+    // If there are multiple shards, annotate the report with a synthetic shard
+    // summary ("N shards merged").  We use (1, shard_count) as a convention
+    // meaning "1 merged report from N shards".
+    let shard_meta = if shard_count > 1 {
+        Some((1u32, shard_count))
+    } else {
+        None
+    };
+
+    let html = render_from_rows(&merged, shard_meta);
+
+    std::fs::create_dir_all(output)
+        .with_context(|| format!("Failed to create output dir: {}", output.display()))?;
+
+    std::fs::write(output.join("index.html"), &html)
+        .context("Failed to write merged index.html")?;
+    std::fs::write(output.join("report.js"), REPORT_JS)
+        .context("Failed to write report.js")?;
+    std::fs::write(output.join("report.css"), REPORT_CSS)
+        .context("Failed to write report.css")?;
+
+    // Write NDJSON sidecar for the merged report so it can itself be used as
+    // input in a downstream merge (idempotent chaining).
+    let ndjson: String = merged
+        .iter()
+        .map(|row| crate::reporter::html::row_to_ndjson_line(row))
+        .collect::<Vec<_>>()
+        .join("\n");
+    std::fs::write(output.join("results.ndjson"), ndjson)
+        .context("Failed to write merged results.ndjson")?;
+
+    println!(
+        "Merged report ({} shard(s)): {}/index.html",
+        shard_count,
+        output.display()
+    );
+    Ok(())
+}

--- /dev/null
+++ b/crates/jet/src/reporter/mod.rs
+//! HTML reporter module for the jet native test runner.
+//!
+//! Provides:
+//! - [`html::HtmlReporter`] — consumes `TestReport` events, renders
+//!   a deterministic self-contained `index.html`.
+//! - [`parser`] — parses NDJSON wire-protocol event streams back into
+//!   `TestReport` rows for offline processing.
+//! - [`merge`] — merges N per-shard report directories into a unified report.
+//!
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R4
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+
+pub mod html;
+pub mod merge;
+pub mod parser;
+
+pub use html::HtmlReporter;
+pub use merge::merge_reports;

--- /dev/null
+++ b/crates/jet/src/reporter/parser.rs
+//! NDJSON parser for the HTML reporter.
+//!
+//! Reads NDJSON wire protocol `testEnd` events and reconstructs `TestReport`
+//! rows understood by the HTML reporter. The NDJSON format is exactly the same
+//! as what the native test runner emits over the worker wire; no new protocol
+//! introduced.
+//!
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R8
+
+use crate::reporter::html::TestRow;
+use crate::test_runner::reporter::{Outcome, TestError, TestReport};
+use crate::test_runner::wire::{TestOutcome, WorkerEvent};
+use anyhow::{Context, Result};
+use std::path::PathBuf;
+
+/// Parse a NDJSON byte stream of `WorkerEvent` lines and reconstruct a
+/// `Vec<TestReport>`.  Lines that are empty or not parseable are skipped
+/// silently (matches the tolerant approach used by the live runner).
+///
+/// # Wire protocol
+///
+/// Each line must be a JSON object with `"kind": "test_end"` (snake_case as
+/// emitted by the `WorkerEvent` serde tag).
+///
+/// # Example
+///
+/// ```json
+/// {"kind":"test_end","id":"abc","suite":[],"name":"adds numbers","outcome":"passed","duration_ms":12,"error":null}
+/// ```
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R8
+pub fn parse_ndjson(bytes: &[u8]) -> Result<Vec<TestReport>> {
+    let text = std::str::from_utf8(bytes).context("NDJSON bytes are not valid UTF-8")?;
+
+    let mut reports = Vec::new();
+    for (lineno, raw_line) in text.lines().enumerate() {
+        let line = raw_line.trim();
+        if line.is_empty() {
+            continue;
+        }
+        match crate::test_runner::wire::parse_line(line) {
+            Some(WorkerEvent::TestEnd {
+                id: _,
+                suite,
+                name,
+                outcome,
+                duration_ms,
+                error,
+                shard_index,
+                shard_total,
+            }) => {
+                let outcome_mapped = match outcome {
+                    TestOutcome::Passed => Outcome::Passed,
+                    TestOutcome::Failed => Outcome::Failed,
+                    TestOutcome::Skipped => Outcome::Skipped,
+                    TestOutcome::TimedOut => Outcome::TimedOut,
+                };
+
+                // Derive a synthetic file path from the suite if no explicit
+                // path is present in the NDJSON (the full path is only present
+                // in the Plan event; testEnd events don't carry it).
+                let file = PathBuf::from("unknown.spec.ts");
+
+                let error_mapped = error.map(|e| TestError {
+                    message: e.message,
+                    stack: e.stack,
+                    diff: e.diff,
+                });
+
+                reports.push(TestReport {
+                    file,
+                    suite,
+                    name,
+                    outcome: outcome_mapped,
+                    duration_ms,
+                    error: error_mapped,
+                    trace_path: None,
+                    shard_index,
+                    shard_total,
+                });
+            }
+            Some(_) => {
+                // Non-testEnd events (Plan, TestStart, Console, Fatal) — skip.
+            }
+            None => {
+                // Unparseable line — skip.
+                let _ = lineno; // suppress unused warning
+            }
+        }
+    }
+
+    Ok(reports)
+}
+
+/// Like `parse_ndjson` but returns `TestRow` values directly, suitable for
+/// use by the merger without going through `HtmlReporter::emit`.
+///
+/// Parses the compact row JSON produced by `row_to_ndjson_line`.
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+pub fn parse_ndjson_to_rows(bytes: &[u8]) -> Vec<TestRow> {
+    let text = match std::str::from_utf8(bytes) {
+        Ok(t) => t,
+        Err(_) => return Vec::new(),
+    };
+
+    let mut rows = Vec::new();
+    for line in text.lines() {
+        let line = line.trim();
+        if line.is_empty() {
+            continue;
+        }
+        match serde_json::from_str::<serde_json::Value>(line) {
+            Ok(v) => {
+                let test_id = v["test_id"].as_str().unwrap_or("").to_string();
+                let name = v["name"].as_str().unwrap_or("").to_string();
+                let status = v["status"].as_str().unwrap_or("unknown").to_string();
+                let duration_ms = v["duration_ms"].as_u64().unwrap_or(0);
+                let file = v["file"].as_str().unwrap_or("").to_string();
+                let stack_trace = v["stack_trace"].as_str().map(String::from);
+                let matcher_diff = v["matcher_diff"].as_str().map(String::from);
+                let trace_path = v["trace_path"].as_str().map(String::from);
+
+                if test_id.is_empty() {
+                    continue;
+                }
+                rows.push(TestRow {
+                    test_id,
+                    name,
+                    status,
+                    duration_ms,
+                    file,
+                    stack_trace,
+                    matcher_diff,
+                    trace_path,
+                });
+            }
+            Err(_) => {
+                // Skip unparseable lines.
+            }
+        }
+    }
+    rows
+}

--- /dev/null
+++ b/crates/jet/tests/html_reporter_tests.rs
+//! Integration + unit tests for the HTML reporter.
+//!
+//! Covers T1-T9 from the spec Test Plan:
+//! `.score/changes/enhancement-html-reporter-for-native-test-runner/specs/
+//!  enhancement-html-reporter-for-native-test-runner-spec.md`
+
+use jet::reporter::html::HtmlReporter;
+use jet::reporter::merge::merge_reports;
+use jet::reporter::parser::parse_ndjson;
+use jet::test_runner::config::Reporter;
+use jet::test_runner::reporter::{Outcome, TestError, TestReport};
+use std::path::PathBuf;
+use tempfile::TempDir;
+
+// ── Helper ────────────────────────────────────────────────────────────────────
+
+fn make_report(name: &str, outcome: Outcome, duration_ms: u64) -> TestReport {
+    TestReport {
+        file: PathBuf::from("src/foo.spec.ts"),
+        suite: vec!["Suite".to_string()],
+        name: name.to_string(),
+        outcome,
+        duration_ms,
+        error: None,
+        trace_path: None,
+        shard_index: None,
+        shard_total: None,
+    }
+}
+
+fn make_report_failed(name: &str, stack: &str, diff: Option<&str>) -> TestReport {
+    TestReport {
+        file: PathBuf::from("src/foo.spec.ts"),
+        suite: vec![],
+        name: name.to_string(),
+        outcome: Outcome::Failed,
+        duration_ms: 10,
+        error: Some(TestError {
+            message: "Expected 1 to be 2".to_string(),
+            stack: Some(stack.to_string()),
+            diff: diff.map(str::to_string),
+        }),
+        trace_path: None,
+        shard_index: None,
+        shard_total: None,
+    }
+}
+
+// ── T1: HtmlReporter emits index.html + asset files ──────────────────────────
+
+/// T1: After emitting 3 TestReports and calling finalize(), index.html,
+/// report.js, and report.css all exist and index.html contains test names.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
+#[test]
+fn test_reporter_emits_index_html() {
+    let tmp = TempDir::new().unwrap();
+    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
+    reporter.emit(make_report("passes correctly", Outcome::Passed, 5));
+    reporter.emit(make_report_failed("fails with message", "at foo.ts:10", None));
+    reporter.emit(make_report("skipped test", Outcome::Skipped, 0));
+    reporter.finalize().unwrap();
+
+    let report_dir = tmp.path().join("report");
+    assert!(report_dir.join("index.html").exists(), "index.html must exist");
+    assert!(report_dir.join("report.js").exists(), "report.js must exist");
+    assert!(report_dir.join("report.css").exists(), "report.css must exist");
+
+    let html = std::fs::read_to_string(report_dir.join("index.html")).unwrap();
+    assert!(html.contains("passes correctly"), "html must contain first test name");
+    assert!(html.contains("fails with message"), "html must contain second test name");
+    assert!(html.contains("skipped test"), "html must contain third test name");
+}
+
+// ── T2: Aggregate stats are rendered ─────────────────────────────────────────
+
+/// T2: Stats panel shows correct total/passed/failed/skipped counts.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R2
+#[test]
+fn test_aggregate_stats_rendered() {
+    let tmp = TempDir::new().unwrap();
+    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
+    reporter.emit(make_report("t1", Outcome::Passed, 10));
+    reporter.emit(make_report("t2", Outcome::Passed, 20));
+    reporter.emit(make_report_failed("t3", "stack", None));
+    reporter.emit(make_report("t4", Outcome::Skipped, 0));
+    reporter.finalize().unwrap();
+
+    let html = std::fs::read_to_string(tmp.path().join("report/index.html")).unwrap();
+
+    // Stats tile values should appear in the rendered output.
+    // Total = 4
+    assert!(html.contains(">4<"), "total count 4 must appear");
+    // Passed = 2
+    assert!(html.contains(">2<"), "passed count 2 must appear");
+    // Failed = 1
+    assert!(html.contains(">1<"), "failed count 1 must appear");
+}
+
+// ── T3: Test row contains required fields ─────────────────────────────────────
+
+/// T3: Each rendered row contains: name, status badge class, duration.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R3
+#[test]
+fn test_test_row_contains_required_fields() {
+    let tmp = TempDir::new().unwrap();
+    let mut reporter = HtmlReporter::new(tmp.path().join("report"));
+    let mut rep = make_report("row-target", Outcome::Passed, 42);
+    rep.file = PathBuf::from("src/target.spec.ts");
+    reporter.emit(rep);
+    reporter.finalize().unwrap();
+
+    let html = std::fs::read_to_string(tmp.path().join("report/index.html")).unwrap();
+
+    assert!(html.contains("row-target"), "row must contain test name");
+    assert!(html.contains("badge-passed"), "row must contain status badge class");
+    assert!(html.contains("42ms"), "row must contain duration");
+}
+
+// ── T4: Parser reconstructs TestReports from NDJSON ──────────────────────────
+
+/// T4: `parse_ndjson` reads a small NDJSON sample, returning the correct
+/// count and test names.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R8
+#[test]
+fn test_parser_parses_ndjson() {
+    let ndjson = r#"{"kind":"test_end","id":"a1","suite":["math"],"name":"adds","outcome":"passed","duration_ms":5,"error":null}
+{"kind":"test_end","id":"a2","suite":[],"name":"subtracts","outcome":"failed","duration_ms":10,"error":{"message":"Expected 2 to be 3","stack":"at spec.ts:5","diff":"-2\n+3"}}
+{"kind":"plan","file":"spec.ts","tests":[]}
+"#;
+
+    let reports = parse_ndjson(ndjson.as_bytes()).unwrap();
+    // plan event is skipped; 2 testEnd events parsed
+    assert_eq!(reports.len(), 2, "expected 2 TestReport entries");
+    assert_eq!(reports[0].name, "adds");
+    assert_eq!(reports[1].name, "subtracts");
+    assert!(matches!(reports[0].outcome, Outcome::Passed));
+    assert!(matches!(reports[1].outcome, Outcome::Failed));
+    // Stack and diff are preserved.
+    let err = reports[1].error.as_ref().unwrap();
+    assert_eq!(err.stack.as_deref(), Some("at spec.ts:5"));
+    assert_eq!(err.diff.as_deref(), Some("-2\n+3"));
+}
+
+// ── T5: Reporter flag parsing ─────────────────────────────────────────────────
+
+/// T5: `Reporter::parse_list("list,html")` → two variants; `"html"` alone
+/// produces one element.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R5
+#[test]
+fn test_reporter_flag_parses() {
+    let kinds = Reporter::parse_list("list,html").unwrap();
+    assert_eq!(kinds.len(), 2);
+    assert!(kinds.contains(&Reporter::Term));
+    assert!(kinds.contains(&Reporter::Html));
+
+    let single = Reporter::parse_list("html").unwrap();
+    assert_eq!(single.len(), 1);
+    assert!(single.contains(&Reporter::Html));
+
+    let three = Reporter::parse_list("term,json,html").unwrap();
+    assert_eq!(three.len(), 3);
+
+    let bad = Reporter::parse_list("unknown");
+    assert!(bad.is_err(), "unknown reporter should be an error");
+}
+
+// ── T6: Merge deduplicates by test_id ────────────────────────────────────────
+
+/// T6: When two shard dirs have overlapping test_id, the merged output
+/// contains only one row per id.
+///
+// @spec enhancement-html-reporter-for-native-test-runner-spec#R7
+#[test]
+fn test_merge_dedupes_by_test_id() {
+    let tmp = TempDir::new().unwrap();
+    let shard1 = tmp.path().join("shard1");
+    let shard2 = tmp.path().join("shard2");
+    let merged = tmp.path().join("merged");
+
+    // Write NDJSON sidecars manually to both shard dirs.
+    std::fs::create_dir_all(&shard1).unwrap();
+    std::fs::create_dir_all(&shard2).unwrap();
+
+    // shard1: rows A and B
+    std::fs::write(
+        shard1.join("results.ndjson"),
+        r#"{"test_id":"id-aaa","name":"alpha","status":"passed","duration_ms":5,"file":"a.spec.ts"}
+{"test_id":"id-bbb","name":"beta","status":"passed","duration_ms":6,"file":"b.spec.ts"}"#,
+    )
+    .unwrap();
+
+    // shard2: rows B (duplicate) and C
+    std::fs::write(
+        shard2.join("results.ndjson"),
+        r#"{"test_id":"id-bbb","name":"beta","status":"passed","duration_ms":6,"file":"b.spec.ts"}

```

## Review: enhancement-html-reporter-for-native-test-runner-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: enhancement-html-reporter-for-native-test-runner

**Summary**: All hard checklist items pass. Spec's Test Plan T1-T9 map to 9 #[test] functions in crates/jet/tests/html_reporter_tests.rs — all passing (9 passed; 0 failed). Code satisfies R1-R10: HtmlReporter writes test-results/report/index.html (R1), aggregate stats panel + shard info (R2), per-test rows with badge/duration/file/stack drawer/trace link (R3), embedded assets via include_bytes! with no CDN (R4), --reporter=html and list,html both parse (R5), report view + report merge subcommands (R6, R7), NDJSON wire-protocol reuse (R8), deterministic sort by stable test_id (R9), trace link references trace file path (R10). No test regressions: cargo check -p jet --tests compiles clean (warnings are in unrelated crates). Code quality is strong — modular reporter/ directory, clear separation of renderer/parser/merger, stable test_id hashing for determinism.



## Alignment Warnings

3 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-html-reporter-for-native-test-runner/.score/tech_design/crates/jet/testing/html-reporter.md | format_priority_violation | Section 'Requirements' (type: requirements) requires a ```mermaid code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-html-reporter-for-native-test-runner/.score/tech_design/crates/jet/testing/html-reporter.md | format_priority_violation | Section 'Scenarios' (type: scenarios) requires a ```yaml code block but none found |
| /Users/chris.cheng/cclab/main/.score/worktrees/enhancement-html-reporter-for-native-test-runner/.score/tech_design/crates/jet/testing/html-reporter.md | format_priority_violation | Section 'Schema' (type: schema) requires a ```yaml code block but none found |
