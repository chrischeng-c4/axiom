// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
// CODEGEN-BEGIN
//! Native test runner — `jet test`.
//!
//! Replaces `@playwright/test` for pure-JS and (eventually) browser-backed
//! integration tests. See
//! `.aw/tech-design/projects/jet/logic/test-runner.md`.
//!
//! # MVP scope (v0)
//!
//! - Discover `**/*.spec.ts`, `**/*.test.ts` under the project root.
//! - Spawn one Node.js worker per spec file.
//! - Worker exposes `describe`, `test`, `expect` (5 core matchers) via an
//!   embedded JS runtime (see `runtime/test/index.ts`).
//! - Wire: NDJSON over stdin/stdout — worker emits lifecycle events, Rust
//!   emits commands.
//! - Reporters: `term` (stdout summary) + `json` (`.jet/test-results.json`).
//!
//! **Out of scope for v0** (pre-wired but no-op):
//!
//! - Browser fixtures (`page`, `browser`). A test that calls `page.locator(...)`
//!   will fail with `ReferenceError: page is not defined`. Use the
//!   `--playwright` escape hatch until Phase 4.
//! - Parallel workers (`workers > 1`).
//! - Trace viewer, HTML reporter, `test.extend()`.

pub mod config;
pub mod coverage;
pub mod discovery;
pub mod expect;
pub mod list_manifest;
pub mod reporter;
pub mod web_server;
pub mod wire;
pub mod worker;
pub mod worker_pool;

use anyhow::{Context, Result};
use std::path::Path;
use std::sync::Arc;

pub use config::RunnerConfig;
pub use coverage::{
    CoverageMetric, CoverageMetricKind, CoverageSummary, CoverageThresholdFailure,
    CoverageThresholds,
};
pub use reporter::{Outcome, Summary, TestReport};
pub use worker_pool::{parse_shard, partition_shard, ShardSpec};

/// Top-level entry point: runs all matching tests and returns a summary.
///
/// When `config.workers == 1`, runs serially (preserves pre-Phase-4b behavior).
/// When `config.workers > 1`, runs via `WorkerPool` with bounded concurrency.
/// When `config.shard` is set, filters specs to only the selected shard first.
/// When `html` reporter is configured, writes report to `config.report_dir`.
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
// @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
// @spec enhancement-html-reporter-for-native-test-runner-spec#R1
pub async fn run(config: RunnerConfig) -> Result<Summary> {
    let all_specs = discovery::scan(&config)?;

    // Boot [test.web_server] before discovery gates anything, so failures
    // surface before spec partitioning. Handle lifetime is the rest of
    // this function — drop kills the child.
    // @spec .aw/tech-design/projects/jet/logic/web-server.md#W2
    let _web_server = match &config.web_server {
        Some(ws_cfg) => Some(
            web_server::boot(ws_cfg, &config.project_root)
                .await
                .context("Failed to boot [test.web_server]")?,
        ),
        None => None,
    };

    // Apply shard partitioning before constructing the reporter so the
    // spec count reflects the actual work this process will do.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R3
    // GH #3616 — partition_shard now returns Result so a non-canonicalizable
    // spec path refuses to ship a non-deterministic shard assignment.
    let specs = partition_shard(&all_specs, config.shard)?;

    if specs.is_empty() {
        let reporter = reporter::MultiReporter::from_config(&config, config.project_root.clone());
        let summary = Summary::default();
        reporter.on_finish(&summary)?;
        return Ok(summary);
    }

    let reporter = Arc::new(reporter::MultiReporter::from_config(
        &config,
        config.project_root.clone(),
    ));
    reporter.on_start(&specs)?;

    // Delegate to WorkerPool — serial (workers==1) or parallel.
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R1
    // @spec enhancement-parallel-test-execution-sharding-for-native-test-r-spec#R2
    let workers = config.workers.max(1);
    let summary =
        worker_pool::WorkerPool::run(specs, workers, config.clone(), reporter.clone()).await;

    reporter.on_finish(&summary)?;

    // If the HTML reporter is active, emit the report now that the run is
    // complete and all TestReports are available in `summary.reports`.
    // @spec enhancement-html-reporter-for-native-test-runner-spec#R1
    if config
        .reporters
        .contains(&crate::test_runner::config::Reporter::Html)
    {
        let mut html_reporter = crate::reporter::HtmlReporter::new(&config.report_dir);
        if let Some((idx, total)) = config.shard {
            html_reporter.shard = Some((idx, total));
        }
        for report in &summary.reports {
            html_reporter.emit(report.clone());
        }
        crate::reporter::html::finalize_with_sidecar(&mut html_reporter)
            .context("HTML reporter failed to write report")?;
    }

    Ok(summary)
}

/// Convenience: run with the given project root and default config.
/// @spec .aw/tech-design/projects/jet/semantic/jet-test-runner.md#schema
pub async fn run_at(project_root: &Path) -> Result<Summary> {
    let cfg = RunnerConfig::default_for_root(project_root)
        .context("Failed to build default runner config")?;
    run(cfg).await
}
// CODEGEN-END
