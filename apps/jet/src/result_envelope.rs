// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Unified `jet test` / `jet e2e` result envelope schema (#2609).
//!
//! Both runners emit mode-specific JSON today — `Summary` from
//! `test_runner::reporter` carries `schema_version: "jet.test.result.v1"`,
//! and `E2eEvidenceBundle` from `e2e` carries its own evidence shape. UI
//! consumers and agents that want a single ingest path can lift either
//! source into the envelope defined here:
//!
//! ```text
//! { schema_version: "jet.result.envelope.v1",
//!   mode: "test" | "e2e_run" | "e2e_open",
//!   summary: { passed, failed, skipped, duration_ms },
//!   cases: [ { id, title, file, outcome, duration_ms, failure?, artifacts[] } ],
//!   artifacts: [ ... ],
//!   mode_data: { ... } }
//! ```
//!
//! `mode_data` is an opaque `serde_json::Value` so mode-specific fields
//! (e.g., E2e product steps, selectors, console logs) round-trip without
//! polluting the core schema. Consumers that only care about pass/fail
//! never have to peek inside.
//!
//! ## Versioning
//!
//! - [`SCHEMA_VERSION`] is bumped on breaking changes (renamed/removed
//!   fields, changed enum encodings). Adding new optional fields is
//!   non-breaking and does not bump the version.
//! - Source envelopes keep their own `schema_version`. The unified envelope
//!   embeds the source tag in `mode_data.source_schema_version` so a
//!   migration tool can recover the original.

use crate::e2e::{E2eEvidenceBundle, E2eMode};
use crate::test_runner::config::RunnerConfig;
use crate::test_runner::reporter::{Outcome, Summary, TestReport};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::warn;

/// Stable schema tag for the unified result envelope. Bumped on any
/// breaking change to the field shape. Adding new optional fields is
/// non-breaking.
pub const SCHEMA_VERSION: &str = "jet.result.envelope.v1";

/// Result-source mode. Agents key off this to route mode-specific data
/// out of `mode_data`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultMode {
    /// `jet test` run — unit / component / frontend-integration suite.
    Test,
    /// `jet e2e run` — Playwright-like agent / CI mode.
    E2eRun,
    /// `jet e2e open` — local dev review mode.
    E2eOpen,
    /// `jet e2e manual` — human-readable manual documentation mode.
    E2eManual,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResultSummary {
    pub passed: u32,
    pub failed: u32,
    pub skipped: u32,
    pub duration_ms: u64,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSourceLocation {
    pub file: PathBuf,
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultFailure {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<String>,
    /// Optional human-readable command for replaying just this case.
    /// `jet test` populates this via `Summary::rerun_hint`; `jet e2e`
    /// populates it via `e2e_case_rerun_hint`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerun_hint: Option<String>,
    /// First user-frame source location parsed from the stack, when
    /// available. Skipped when the worker did not attach a stack.
    // @spec #2610
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<ResultSourceLocation>,
    /// Failure-scoped artifact references resolved through the bundle
    /// manifest (#2875). Empty when nothing is attached; missing refs
    /// are kept explicit (path/kind None) — see [`FailureArtifactRef`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<FailureArtifactRef>,
    /// Configured per-test timeout budget in milliseconds, populated
    /// only for timeout failures (#2869). Pair with the case's
    /// `duration_ms` to render `elapsed/budget` evidence; agents key off
    /// `Some(_)` here to recognise timeout-flavoured failures without
    /// having to parse `failure.message`.
    // @spec #2869
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_budget_ms: Option<u64>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultArtifact {
    pub kind: String,
    pub path: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Failure-scoped reference to an artifact in the evidence bundle (#2875).
///
/// Identifies the artifact by its stable bundle id and carries the
/// kind/path resolved through the manifest at projection time. When
/// the bundle manifest does not know about `id` (stale evidence, lost
/// upload, etc.) `path` and `kind` are `None` — that's the documented
/// "missing artifact represented explicitly" state, in contrast to an
/// empty `Vec` which means "no artifacts attached".
// @spec #2875
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FailureArtifactRef {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl FailureArtifactRef {
    /// Resolve a list of artifact ids against a bundle manifest. Ids
    /// that do not match an entry are returned with `path: None` and
    /// `kind: None` so the consumer can render an explicit "missing"
    /// marker instead of silently dropping the reference.
    // @spec #2875
    pub fn resolve_all(
        ids: impl IntoIterator<Item = impl Into<String>>,
        manifest: &crate::evidence_bundle::BundleManifest,
    ) -> Vec<Self> {
        ids.into_iter()
            .map(|id| {
                let id = id.into();
                match manifest.artifacts.iter().find(|a| a.id == id) {
                    Some(a) => Self {
                        id,
                        kind: Some(a.kind.clone()),
                        path: Some(a.path.clone()),
                    },
                    None => Self {
                        id,
                        kind: None,
                        path: None,
                    },
                }
            })
            .collect()
    }

    /// True when the ref could not be resolved against the manifest.
    pub fn is_missing(&self) -> bool {
        self.path.is_none()
    }

    /// Build the standard `expected / actual / diff` triplet of
    /// references for a failed golden assertion (#2870). All three
    /// ids are resolved against the bundle so the resulting refs
    /// carry bundle-relative paths and survive `serde` round-trips.
    ///
    /// An id with no matching manifest entry is preserved as a
    /// missing-ref (see [`Self::is_missing`]) rather than dropped —
    /// the consumer can render an explicit "expected snapshot not
    /// recorded" marker instead of silently losing the slot.
    // @spec #2870
    pub fn golden_triplet(
        expected_id: impl Into<String>,
        actual_id: impl Into<String>,
        diff_id: impl Into<String>,
        manifest: &crate::evidence_bundle::BundleManifest,
    ) -> Vec<Self> {
        Self::resolve_all(
            [expected_id.into(), actual_id.into(), diff_id.into()],
            manifest,
        )
    }
}

/// Per-attempt record for a retried case (#2721).
///
/// When the runner retries a case, earlier attempts go here; the
/// outer [`ResultCase`] still carries the final status and (when
/// the run ultimately failed) the final failure record. Empty when
/// the case ran once and was not retried.
///
/// `failure` is the failure that ended the attempt — it survives
/// even when a later attempt passed, so reports can show *why*
/// the case was retried without scanning logs.
// @spec #2721
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRetryAttempt {
    /// 0-indexed attempt number. The final attempt lives on the
    /// outer `ResultCase`, so a case retried twice has entries
    /// `attempt: 0, 1` here and the third attempt on the case itself.
    pub attempt: u32,
    /// Outcome of this single attempt (e.g., `failed`, `timed_out`).
    /// Passed attempts are never recorded; passing terminates retry.
    pub outcome: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<ResultFailure>,
    /// Artifacts captured for this attempt. Kept per-attempt so a
    /// retry-pass case can still surface the failed-attempt
    /// screenshot/trace for triage.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ResultArtifact>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultCase {
    pub id: String,
    pub title: String,
    pub file: PathBuf,
    /// One of `passed | failed | skipped | timed_out | crashed`. Mode-specific
    /// runners may map their own outcomes onto this lexicon.
    pub outcome: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<ResultFailure>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ResultArtifact>,
    /// Earlier retry attempts. Empty when the case ran once. The final
    /// attempt is reflected by the case's own `outcome`/`failure`.
    // @spec #2721
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retries: Vec<ResultRetryAttempt>,
}

/// Worker scheduling metadata captured into the unified envelope so
/// evidence reflects how the run was scheduled (parallel worker pool
/// vs. serial fallback).
///
/// `serial_fallback` is true whenever the runner executes serially —
/// either because `--workers=1` was supplied (operator-forced) or
/// because the host has only one logical CPU. Keeping the two
/// signals distinct lets CI flag "workers=1 by accident" runs.
// @spec #2710
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerScheduling {
    /// Number of concurrent workers requested for the run.
    pub worker_count: usize,
    /// True when the run executed serially (workers == 1).
    pub serial_fallback: bool,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl WorkerScheduling {
    /// Derive worker-scheduling metadata from a resolved `RunnerConfig`.
    // @spec #2710
    pub fn from_config(config: &RunnerConfig) -> Self {
        let worker_count = config.workers.max(1);
        Self {
            worker_count,
            serial_fallback: worker_count == 1,
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultEnvelope {
    pub schema_version: String,
    pub mode: ResultMode,
    pub summary: ResultSummary,
    pub cases: Vec<ResultCase>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<ResultArtifact>,
    /// Mode-specific data (e.g., E2e product steps, selectors, console
    /// logs, open-mode control protocol). Opaque to the core schema.
    /// `source_schema_version` lives inside this value so migrations can
    /// recover the originating envelope.
    pub mode_data: serde_json::Value,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl ResultEnvelope {
    /// Lift a `jet test` `Summary` into the unified envelope.
    ///
    /// `project_root` is used to derive the per-case `rerun_hint`.
    pub fn from_test_summary(summary: &Summary, project_root: &std::path::Path) -> Self {
        let cases = summary
            .reports
            .iter()
            .map(|r| test_report_to_case(r, project_root, summary))
            .collect();
        let mode_data = serde_json::json!({
            "source_schema_version": summary.schema_version,
        });
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: ResultMode::Test,
            summary: ResultSummary {
                passed: summary.passed,
                failed: summary.failed,
                skipped: summary.skipped,
                duration_ms: summary.duration_ms,
            },
            cases,
            artifacts: Vec::new(),
            mode_data,
        }
    }

    /// Same as [`Self::from_test_summary`] but additionally records the
    /// run's worker-scheduling metadata into `mode_data.worker_scheduling`.
    // @spec #2710
    pub fn from_test_summary_with_config(
        summary: &Summary,
        project_root: &std::path::Path,
        config: &RunnerConfig,
    ) -> Self {
        Self::from_test_summary(summary, project_root)
            .with_worker_scheduling(WorkerScheduling::from_config(config))
    }

    /// Merge worker-scheduling metadata into the envelope's `mode_data`.
    ///
    /// Idempotent: calling this twice with the same scheduling produces
    /// a byte-equivalent envelope. Existing `mode_data` keys are kept.
    // @spec #2710
    pub fn with_worker_scheduling(mut self, scheduling: WorkerScheduling) -> Self {
        let value = serde_json::to_value(&scheduling).expect("WorkerScheduling serialises to JSON");
        if let Some(map) = self.mode_data.as_object_mut() {
            map.insert("worker_scheduling".to_string(), value);
        } else {
            // mode_data was non-object (e.g., null) — replace with a
            // fresh object that still carries the scheduling key.
            self.mode_data = serde_json::json!({ "worker_scheduling": value });
        }
        self
    }

    /// Return references to the `limit` slowest cases, ordered by
    /// `duration_ms` descending. Used by terminal reporters to render
    /// "slowest tests" summaries from an envelope without re-parsing
    /// the per-case JSONL stream.
    // @spec #2721
    pub fn slowest_cases(&self, limit: usize) -> Vec<&ResultCase> {
        let mut sorted: Vec<&ResultCase> = self.cases.iter().collect();
        sorted.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));
        sorted.truncate(limit);
        sorted
    }

    /// Lift an `E2eEvidenceBundle` into the unified envelope. The mode is
    /// inferred from the bundle's `mode` field.
    pub fn from_e2e_bundle(bundle: &E2eEvidenceBundle) -> Self {
        let mode = match bundle.mode {
            E2eMode::Run => ResultMode::E2eRun,
            E2eMode::Open => ResultMode::E2eOpen,
            E2eMode::Manual => ResultMode::E2eManual,
        };
        let cases = bundle
            .cases
            .iter()
            .map(|c| ResultCase {
                id: c.id.clone(),
                title: c.title.clone(),
                file: c.file.clone(),
                outcome: c.outcome.clone(),
                duration_ms: c.duration_ms,
                retries: Vec::new(),
                failure: c.steps.iter().find_map(|s| {
                    s.assertion.as_ref().map(|a| ResultFailure {
                        message: a.message.clone(),
                        stack: a.stack.clone(),
                        diff: a.diff.clone(),
                        rerun_hint: Some(e2e_case_rerun_hint(c, &bundle.mode)),
                        source_location: a
                            .stack
                            .as_deref()
                            .and_then(
                                crate::test_runner::reporter::SourceLocation::parse_from_stack,
                            )
                            .map(|loc| ResultSourceLocation {
                                file: loc.file,
                                line: loc.line,
                                column: loc.column,
                            }),
                        artifacts: Vec::new(),
                        timeout_budget_ms: None,
                    })
                }),
                artifacts: c
                    .steps
                    .iter()
                    .flat_map(|s| s.context.screenshots.iter())
                    .map(|a| ResultArtifact {
                        kind: a.kind.clone(),
                        path: a.path.clone(),
                        label: a.label.clone(),
                    })
                    .collect(),
            })
            .collect();
        let artifacts = bundle
            .artifacts
            .iter()
            .map(|a| ResultArtifact {
                kind: a.kind.clone(),
                path: a.path.clone(),
                label: a.label.clone(),
            })
            .collect();
        let mode_data = serde_json::json!({
            "source_schema_version": bundle.schema_version,
            "run_id": bundle.run_id,
            "started_at_ms": bundle.started_at_ms,
            "finished_at_ms": bundle.finished_at_ms,
            "open_control": bundle.open_control,
        });
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            mode,
            summary: ResultSummary {
                passed: bundle.summary.passed,
                failed: bundle.summary.failed,
                skipped: bundle.summary.skipped,
                duration_ms: bundle.summary.duration_ms,
            },
            cases,
            artifacts,
            mode_data,
        }
    }
}

/// Build a deterministic `jet e2e` replay command for a single case.
/// Mirrors `Summary::rerun_hint` on the test side.
// @spec #2610
pub fn e2e_case_rerun_hint(case: &crate::e2e::E2eCaseEvidence, mode: &E2eMode) -> String {
    let sub = match mode {
        E2eMode::Run => "run",
        E2eMode::Open => "open",
        E2eMode::Manual => "manual",
    };
    format!(
        "jet e2e {} {} --grep '{}'",
        sub,
        case.file.display(),
        case.title
    )
}

fn outcome_str(o: Outcome) -> &'static str {
    match o {
        Outcome::Passed => "passed",
        Outcome::Failed => "failed",
        Outcome::Skipped => "skipped",
        Outcome::TimedOut => "timed_out",
        Outcome::Crashed => "crashed",
    }
}

fn test_report_to_case(
    r: &TestReport,
    project_root: &std::path::Path,
    summary: &Summary,
) -> ResultCase {
    let suite_path = if r.suite.is_empty() {
        String::new()
    } else {
        format!("{} > ", r.suite.join(" > "))
    };
    let title = format!("{}{}", suite_path, r.name);
    let id = format!("{}::{}", r.file.display(), title);
    let failure = r.error.as_ref().map(|e| ResultFailure {
        message: e.message.clone(),
        stack: e.stack.clone(),
        diff: e.diff.clone(),
        rerun_hint: Some(Summary::rerun_hint(r, project_root)),
        source_location: e.source_location.as_ref().map(|loc| ResultSourceLocation {
            file: loc.file.clone(),
            line: loc.line,
            column: loc.column,
        }),
        artifacts: Vec::new(),
        timeout_budget_ms: None,
    });
    let artifacts = r
        .artifacts
        .iter()
        .map(|p| ResultArtifact {
            kind: "screenshot".to_string(),
            path: p.clone(),
            label: None,
        })
        .collect();
    let _ = summary; // reserved for future cross-case enrichment
    ResultCase {
        id,
        title,
        file: r.file.clone(),
        outcome: outcome_str(r.outcome).to_string(),
        duration_ms: r.duration_ms,
        failure,
        artifacts,
        retries: Vec::new(),
    }
}

/// Configuration for [`emit_artifacts`]. Always emits `<prefix>.result.json`
/// and `<prefix>.result.jsonl`. YAML and failures text are opt-in so simple
/// CI consumers don't pay the cost.
// @spec #2612
#[derive(Debug, Clone)]
pub struct EmitterOptions {
    /// Directory the artifacts are written into. Created if missing.
    pub dir: PathBuf,
    /// Filename stem (e.g., `run-2026-05-19T12-00-00`).
    pub prefix: String,
    pub emit_yaml: bool,
    pub emit_failures_txt: bool,
}

/// Filesystem locations of the artifacts written by [`emit_artifacts`].
// @spec #2612
#[derive(Debug, Clone)]
pub struct EmittedArtifacts {
    pub json: PathBuf,
    pub jsonl: PathBuf,
    pub yaml: Option<PathBuf>,
    pub failures_txt: Option<PathBuf>,
}

/// Write the agent-readable artifact set for a result envelope.
///
/// - `<prefix>.result.json` — full envelope, pretty-printed.
/// - `<prefix>.result.jsonl` — streaming form: one `case_finished` line per
///   case followed by a `summary` line. Useful for `tail -f`-style ingest.
/// - `<prefix>.result.yaml` (opt-in) — same shape as JSON, for humans.
/// - `<prefix>.failures.txt` (opt-in) — concise failure-first summary
///   listing `[FAIL] title / file / message / rerun` for each failed case.
///
/// File paths are deterministic given `dir` + `prefix`, so automation can
/// reference them without re-discovering.
// @spec #2612
pub fn emit_artifacts(env: &ResultEnvelope, opts: &EmitterOptions) -> Result<EmittedArtifacts> {
    std::fs::create_dir_all(&opts.dir)
        .with_context(|| format!("creating {}", opts.dir.display()))?;

    let json = opts.dir.join(format!("{}.result.json", opts.prefix));
    let jsonl = opts.dir.join(format!("{}.result.jsonl", opts.prefix));

    std::fs::write(
        &json,
        serde_json::to_vec_pretty(env).context("serializing result envelope as JSON")?,
    )
    .with_context(|| format!("writing {}", json.display()))?;

    let mut jsonl_body = String::new();
    for case in &env.cases {
        let line = serde_json::json!({
            "kind": "case_finished",
            "id": case.id,
            "title": case.title,
            "file": case.file,
            "outcome": case.outcome,
            "duration_ms": case.duration_ms,
            "failure": case.failure,
        });
        jsonl_body.push_str(&serde_json::to_string(&line)?);
        jsonl_body.push('\n');
    }
    let summary_line = serde_json::json!({
        "kind": "summary",
        "schema_version": env.schema_version,
        "mode": env.mode,
        "summary": env.summary,
    });
    jsonl_body.push_str(&serde_json::to_string(&summary_line)?);
    jsonl_body.push('\n');
    std::fs::write(&jsonl, jsonl_body).with_context(|| format!("writing {}", jsonl.display()))?;

    let yaml = if opts.emit_yaml {
        let path = opts.dir.join(format!("{}.result.yaml", opts.prefix));
        let body = serde_yaml::to_string(env).context("serializing result envelope as YAML")?;
        std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        Some(path)
    } else {
        None
    };

    let failures_txt = if opts.emit_failures_txt {
        let path = opts.dir.join(format!("{}.failures.txt", opts.prefix));
        let body = format_failures_text(env);
        std::fs::write(&path, body).with_context(|| format!("writing {}", path.display()))?;
        Some(path)
    } else {
        None
    };

    Ok(EmittedArtifacts {
        json,
        jsonl,
        yaml,
        failures_txt,
    })
}

/// Sentinel surfaced into `.failures.txt` when a `Failure` carries an
/// empty `message`. The accompanying `tracing::warn!` is the audit trail;
/// the sentinel keeps the gap visible to a human tailing the file.
// @spec gh3787 — silent empty failure.message fallback
pub(crate) const EMPTY_FAILURE_MESSAGE_SENTINEL: &str = "<empty failure message>";

/// Format the warn string emitted when [`format_failures_text`] sees a
/// failure whose `message` is empty. Exposed as `pub(crate)` so tests can
/// pin both the text and the renaming surface.
// @spec gh3787 — distinguishes truly-empty messages from leading-newline messages
pub(crate) fn format_result_envelope_empty_failure_message_warn(title: &str) -> String {
    format!(
        "gh3787: failure.message is empty for case title={title:?}; \
         upstream reporter dropped the diagnostic — rendering sentinel \
         {EMPTY_FAILURE_MESSAGE_SENTINEL:?}"
    )
}

/// Pick the first line of `message` for the failures-text `message:` row.
///
/// `str::lines` yields zero items for the empty string. The legacy code
/// silently coerced that to `""`, which made a dropped-by-reporter message
/// indistinguishable from a message whose first line is empty (e.g.
/// `"\n..."` where `lines().next()` is `Some("")`). This helper warns on
/// the truly-empty path and returns a sentinel so the gap is visible.
// @spec gh3787
pub(crate) fn first_failure_message_line_or_warn(message: &str, title: &str) -> String {
    match message.lines().next() {
        Some(line) => line.to_string(),
        None => {
            warn!(
                "{}",
                format_result_envelope_empty_failure_message_warn(title)
            );
            EMPTY_FAILURE_MESSAGE_SENTINEL.to_string()
        }
    }
}

/// Render the failures-first text summary used by [`emit_artifacts`].
///
/// Exposed publicly so callers that want to print the same text inline
/// (e.g., terminal reporter at end of run) can reuse the format.
// @spec #2612
pub fn format_failures_text(env: &ResultEnvelope) -> String {
    let mut out = String::new();
    let mut failures = env.cases.iter().filter(|c| c.failure.is_some()).peekable();
    if failures.peek().is_none() {
        out.push_str("No failures.\n");
        return out;
    }
    for case in failures {
        let failure = case.failure.as_ref().expect("filtered");
        out.push_str(&format!("[FAIL] {}\n", case.title));
        let file_line = match &failure.source_location {
            Some(loc) => match loc.column {
                Some(col) => format!("  file: {}:{}:{}\n", loc.file.display(), loc.line, col),
                None => format!("  file: {}:{}\n", loc.file.display(), loc.line),
            },
            None => format!("  file: {}\n", case.file.display()),
        };
        out.push_str(&file_line);
        let first_msg_line = first_failure_message_line_or_warn(&failure.message, &case.title);
        out.push_str(&format!("  message: {first_msg_line}\n"));
        if let Some(hint) = &failure.rerun_hint {
            out.push_str(&format!("  rerun: {hint}\n"));
        }
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_runner::reporter::{Outcome, Summary, TestError, TestReport};
    use std::path::Path;

    fn sample_test_summary() -> Summary {
        Summary {
            passed: 2,
            failed: 1,
            skipped: 1,
            duration_ms: 12,
            reports: vec![
                TestReport {
                    file: PathBuf::from("/tmp/proj/specs/a.spec.ts"),
                    suite: vec!["math".to_string()],
                    name: "adds".to_string(),
                    outcome: Outcome::Passed,
                    duration_ms: 1,
                    error: None,
                    trace_path: None,
                    shard_index: None,
                    shard_total: None,
                    artifacts: vec![],
                    steps: Vec::new(),
                },
                TestReport {
                    file: PathBuf::from("/tmp/proj/specs/a.spec.ts"),
                    suite: vec!["math".to_string()],
                    name: "bad".to_string(),
                    outcome: Outcome::Failed,
                    duration_ms: 2,
                    error: Some(TestError {
                        message: "expected 5".to_string(),
                        stack: Some("at /tmp/proj/specs/a.spec.ts:7:10".to_string()),
                        diff: Some("- 4\n+ 5".to_string()),
                        source_location: None,
                    }),
                    trace_path: None,
                    shard_index: None,
                    shard_total: None,
                    artifacts: vec![],
                    steps: Vec::new(),
                },
            ],
            ..Summary::default()
        }
    }

    #[test]
    fn from_test_summary_maps_core_fields() {
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));

        assert_eq!(env.schema_version, SCHEMA_VERSION.to_string());
        assert_eq!(env.mode, ResultMode::Test);
        assert_eq!(env.summary.passed, 2);
        assert_eq!(env.summary.failed, 1);
        assert_eq!(env.summary.skipped, 1);
        assert_eq!(env.cases.len(), 2);

        let failed = env
            .cases
            .iter()
            .find(|c| c.outcome == "failed")
            .expect("failed case present");
        let failure = failed.failure.as_ref().expect("failure payload");
        assert_eq!(failure.message, "expected 5");
        assert!(failure.diff.is_some(), "diff propagates");
        let hint = failure.rerun_hint.as_deref().unwrap();
        assert!(hint.starts_with("jet test"), "rerun_hint: {hint}");
        assert!(hint.contains("math > bad"), "rerun_hint: {hint}");
    }

    #[test]
    fn worker_scheduling_serial_when_workers_one() {
        // @spec #2710 — workers=1 → serial_fallback=true.
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.workers = 1;
        let sched = WorkerScheduling::from_config(&cfg);
        assert_eq!(sched.worker_count, 1);
        assert!(sched.serial_fallback);
    }

    #[test]
    fn worker_scheduling_parallel_when_workers_gt_one() {
        // @spec #2710 — workers>1 → serial_fallback=false.
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.workers = 4;
        let sched = WorkerScheduling::from_config(&cfg);
        assert_eq!(sched.worker_count, 4);
        assert!(!sched.serial_fallback);
    }

    #[test]
    fn from_test_summary_with_config_records_worker_count() {
        // @spec #2710 — evidence carries the selected worker count.
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.workers = 3;
        let summary = sample_test_summary();
        let env =
            ResultEnvelope::from_test_summary_with_config(&summary, Path::new("/tmp/proj"), &cfg);

        let sched = env
            .mode_data
            .get("worker_scheduling")
            .expect("worker_scheduling key in mode_data");
        assert_eq!(sched["worker_count"], 3);
        assert_eq!(sched["serial_fallback"], false);
        // Existing keys are preserved.
        assert_eq!(
            env.mode_data["source_schema_version"],
            crate::test_runner::reporter::SCHEMA_VERSION
        );
    }

    #[test]
    fn worker_scheduling_round_trips_through_envelope_json() {
        // @spec #2710 — the worker_scheduling block survives JSON round-trip.
        let tmp = tempfile::tempdir().unwrap();
        let mut cfg = RunnerConfig::default_for_root(tmp.path()).unwrap();
        cfg.workers = 1;
        let summary = sample_test_summary();
        let env =
            ResultEnvelope::from_test_summary_with_config(&summary, Path::new("/tmp/proj"), &cfg);
        let json = serde_json::to_string(&env).expect("serialize");
        let decoded: ResultEnvelope = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.mode_data["worker_scheduling"]["worker_count"], 1);
        assert_eq!(
            decoded.mode_data["worker_scheduling"]["serial_fallback"],
            true
        );
    }

    #[test]
    fn with_worker_scheduling_is_idempotent() {
        // @spec #2710 — applying the same scheduling twice yields the
        // same envelope, so downstream tools can re-tag safely.
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let sched = WorkerScheduling {
            worker_count: 2,
            serial_fallback: false,
        };
        let once = env.clone().with_worker_scheduling(sched.clone());
        let twice = once.clone().with_worker_scheduling(sched);
        assert_eq!(
            serde_json::to_string(&once).unwrap(),
            serde_json::to_string(&twice).unwrap()
        );
    }

    #[test]
    fn envelope_round_trips_through_json() {
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let json = serde_json::to_string(&env).expect("serialize");
        let decoded: ResultEnvelope = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(decoded.schema_version, SCHEMA_VERSION.to_string());
        assert_eq!(decoded.mode, ResultMode::Test);
        assert_eq!(decoded.summary.failed, 1);
        assert_eq!(decoded.cases.len(), 2);
        assert_eq!(
            decoded.mode_data["source_schema_version"],
            crate::test_runner::reporter::SCHEMA_VERSION
        );
    }

    #[test]
    fn mode_serializes_snake_case() {
        let v = serde_json::to_value(&ResultMode::E2eRun).unwrap();
        assert_eq!(v.as_str(), Some("e2e_run"));
        let v = serde_json::to_value(&ResultMode::E2eOpen).unwrap();
        assert_eq!(v.as_str(), Some("e2e_open"));
        let v = serde_json::to_value(&ResultMode::E2eManual).unwrap();
        assert_eq!(v.as_str(), Some("e2e_manual"));
        let v = serde_json::to_value(&ResultMode::Test).unwrap();
        assert_eq!(v.as_str(), Some("test"));
    }

    #[test]
    fn emit_artifacts_writes_json_and_jsonl_by_default() {
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let dir = tempfile::tempdir().expect("tempdir");
        let opts = EmitterOptions {
            dir: dir.path().to_path_buf(),
            prefix: "run".to_string(),
            emit_yaml: false,
            emit_failures_txt: false,
        };
        let written = emit_artifacts(&env, &opts).expect("emit");
        assert!(written.json.exists());
        assert!(written.jsonl.exists());
        assert!(written.yaml.is_none());
        assert!(written.failures_txt.is_none());

        let jsonl_body = std::fs::read_to_string(&written.jsonl).unwrap();
        let lines: Vec<&str> = jsonl_body.lines().collect();
        assert_eq!(lines.len(), env.cases.len() + 1, "case lines + 1 summary");
        let summary_v: serde_json::Value =
            serde_json::from_str(lines.last().unwrap()).expect("summary line");
        assert_eq!(summary_v["kind"], "summary");
        assert_eq!(summary_v["summary"]["failed"], 1);
    }

    #[test]
    fn emit_artifacts_writes_yaml_and_failures_text_when_requested() {
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let dir = tempfile::tempdir().expect("tempdir");
        let opts = EmitterOptions {
            dir: dir.path().to_path_buf(),
            prefix: "run".to_string(),
            emit_yaml: true,
            emit_failures_txt: true,
        };
        let written = emit_artifacts(&env, &opts).expect("emit");
        let yaml_path = written.yaml.expect("yaml path");
        let failures_path = written.failures_txt.expect("failures path");
        assert!(yaml_path.exists());
        assert!(failures_path.exists());

        // YAML deserializes back into the envelope.
        let yaml_body = std::fs::read_to_string(&yaml_path).unwrap();
        let decoded: ResultEnvelope = serde_yaml::from_str(&yaml_body).expect("yaml decode");
        assert_eq!(decoded.summary.failed, 1);

        let failures_body = std::fs::read_to_string(&failures_path).unwrap();
        assert!(failures_body.contains("[FAIL] math > bad"));
        assert!(failures_body.contains("message: expected 5"));
        assert!(failures_body.contains("rerun: jet test"));
    }

    #[test]
    fn yaml_output_is_field_equivalent_to_json_for_same_envelope() {
        // @spec #2871 — YAML and JSON projections must agree field-by-field
        // on the same envelope. Normalise both into serde_json::Value and
        // compare; YAML's repr can differ on whitespace but the decoded
        // shape must match the JSON projection byte-for-byte.
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let dir = tempfile::tempdir().expect("tempdir");
        let opts = EmitterOptions {
            dir: dir.path().to_path_buf(),
            prefix: "run".to_string(),
            emit_yaml: true,
            emit_failures_txt: false,
        };
        let written = emit_artifacts(&env, &opts).expect("emit");
        let yaml_path = written.yaml.expect("yaml path");

        let json_body = std::fs::read_to_string(&written.json).expect("json read");
        let yaml_body = std::fs::read_to_string(&yaml_path).expect("yaml read");

        let json_value: serde_json::Value = serde_json::from_str(&json_body).expect("json parse");
        // Round-trip YAML through serde_yaml -> serde_json::Value so we
        // can compare the two trees with a single equality check.
        let yaml_value: serde_json::Value = serde_yaml::from_str::<serde_yaml::Value>(&yaml_body)
            .and_then(|v| serde_yaml::from_value::<serde_json::Value>(v))
            .expect("yaml->json value");

        assert_eq!(
            json_value, yaml_value,
            "JSON and YAML projections of the same envelope must be field-equivalent",
        );

        // Spot-check the spec acceptance fields are visible in YAML.
        // Envelope-level artifacts skip-serialize when empty; the fixture
        // has no top-level artifacts, so we only assert run-status fields
        // here (#2871 acceptance: "YAML output includes run status,
        // failures, and artifacts" — covered by the structural equality
        // above for any fixture that does carry artifacts).
        assert!(yaml_body.contains("schema_version"), "{yaml_body}");
        assert!(yaml_body.contains("summary:"), "{yaml_body}");
        assert!(yaml_body.contains("cases:"), "{yaml_body}");
        assert!(yaml_body.contains("failure:"), "{yaml_body}");
    }

    #[test]
    fn format_failures_text_reports_no_failures_when_empty() {
        let env = ResultEnvelope {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: ResultMode::Test,
            summary: ResultSummary::default(),
            cases: vec![],
            artifacts: vec![],
            mode_data: serde_json::Value::Null,
        };
        let text = format_failures_text(&env);
        assert_eq!(text, "No failures.\n");
    }

    #[test]
    fn emit_artifacts_paths_are_deterministic() {
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let dir = tempfile::tempdir().expect("tempdir");
        let opts = EmitterOptions {
            dir: dir.path().to_path_buf(),
            prefix: "deterministic".to_string(),
            emit_yaml: true,
            emit_failures_txt: true,
        };
        let a = emit_artifacts(&env, &opts).expect("emit a");
        let b = emit_artifacts(&env, &opts).expect("emit b");
        assert_eq!(a.json, b.json);
        assert_eq!(a.jsonl, b.jsonl);
        assert_eq!(a.yaml, b.yaml);
        assert_eq!(a.failures_txt, b.failures_txt);
        assert_eq!(a.json, dir.path().join("deterministic.result.json"),);
    }

    #[test]
    fn timeout_failure_carries_status_elapsed_budget_and_rerun() {
        // @spec #2869 — timeout failures must surface:
        //   * outcome == "timed_out"
        //   * elapsed duration (ResultCase.duration_ms)
        //   * configured timeout budget (ResultFailure.timeout_budget_ms)
        //   * rerun guidance (ResultFailure.rerun_hint)
        // Agents parse all four off the envelope without any UI.
        let case = ResultCase {
            id: "specs/slow.spec.ts::slow > times out".to_string(),
            title: "slow > times out".to_string(),
            file: PathBuf::from("specs/slow.spec.ts"),
            outcome: "timed_out".to_string(),
            duration_ms: 5012,
            failure: Some(ResultFailure {
                message: "Test timed out after 5000ms".to_string(),
                stack: None,
                diff: None,
                rerun_hint: Some("jet test specs/slow.spec.ts -g 'slow > times out'".into()),
                source_location: None,
                artifacts: Vec::new(),
                timeout_budget_ms: Some(5000),
            }),
            artifacts: Vec::new(),
            retries: Vec::new(),
        };
        let env = ResultEnvelope {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: ResultMode::Test,
            summary: ResultSummary {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 5012,
            },
            cases: vec![case],
            artifacts: vec![],
            mode_data: serde_json::Value::Null,
        };

        // Agent-side parse: all four facts come straight off the envelope.
        let case = env.cases.first().unwrap();
        assert_eq!(case.outcome, "timed_out");
        assert_eq!(case.duration_ms, 5012, "elapsed duration");
        let failure = case.failure.as_ref().unwrap();
        assert_eq!(failure.timeout_budget_ms, Some(5000), "configured budget");
        assert!(
            failure
                .rerun_hint
                .as_deref()
                .unwrap()
                .starts_with("jet test"),
            "rerun guidance preserved for timeout failures",
        );

        // JSON round-trip keeps the timeout budget field visible.
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains(r#""timeout_budget_ms":5000"#), "{json}");
        let back: ResultEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(
            back.cases[0].failure.as_ref().unwrap().timeout_budget_ms,
            Some(5000),
        );

        // Failure text rendering surfaces the configured timeout so
        // a tail of failures.txt is enough for triage.
        let text = format_failures_text(&env);
        assert!(text.contains("Test timed out after 5000ms"), "{text}");
        assert!(text.contains("rerun: jet test"), "{text}");
    }

    #[test]
    fn retry_pass_case_preserves_earlier_attempt_failure_and_artifacts() {
        // @spec #2721 — when a case fails the first attempt but passes
        // on retry, the envelope must carry:
        //   * outer outcome == "passed" (final status)
        //   * retries[].outcome == "failed" (earlier attempt status)
        //   * retries[].failure with the original message
        //   * retries[].artifacts so the failed-attempt screenshot
        //     survives even though the case ultimately passed
        let case = ResultCase {
            id: "specs/flaky.spec.ts::flaky > intermittent".to_string(),
            title: "flaky > intermittent".to_string(),
            file: PathBuf::from("specs/flaky.spec.ts"),
            outcome: "passed".to_string(),
            duration_ms: 24,
            failure: None,
            artifacts: vec![],
            retries: vec![ResultRetryAttempt {
                attempt: 0,
                outcome: "failed".to_string(),
                duration_ms: 12,
                failure: Some(ResultFailure {
                    message: "transient network error".into(),
                    stack: None,
                    diff: None,
                    rerun_hint: None,
                    source_location: None,
                    artifacts: vec![],
                    timeout_budget_ms: None,
                }),
                artifacts: vec![ResultArtifact {
                    kind: "screenshot".into(),
                    path: PathBuf::from("artifacts/attempt-0.png"),
                    label: Some("attempt-0".into()),
                }],
            }],
        };
        let env = ResultEnvelope {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: ResultMode::Test,
            summary: ResultSummary {
                passed: 1,
                failed: 0,
                skipped: 0,
                duration_ms: 24,
            },
            cases: vec![case],
            artifacts: vec![],
            mode_data: serde_json::Value::Null,
        };

        let case = env.cases.first().unwrap();
        assert_eq!(case.outcome, "passed", "final status is the retry");
        assert!(
            case.failure.is_none(),
            "retry-pass clears top-level failure"
        );
        assert_eq!(case.retries.len(), 1, "earlier attempt recorded");
        let earlier = &case.retries[0];
        assert_eq!(earlier.attempt, 0);
        assert_eq!(earlier.outcome, "failed");
        assert!(
            earlier
                .failure
                .as_ref()
                .unwrap()
                .message
                .contains("transient"),
            "earlier failure preserved",
        );
        assert_eq!(earlier.artifacts.len(), 1, "attempt artifact preserved");

        // Schema fixture must serialise cleanly so downstream tooling
        // can ingest the retry record.
        let json = serde_json::to_string(&env).unwrap();
        assert!(json.contains(r#""retries":["#), "{json}");
        assert!(json.contains(r#""attempt":0"#), "{json}");
        assert!(json.contains(r#""transient network error""#), "{json}");
        let back: ResultEnvelope = serde_json::from_str(&json).unwrap();
        assert_eq!(back.cases[0].retries.len(), 1);
    }

    #[test]
    fn case_without_retries_skips_serialising_empty_field() {
        // @spec #2721 — non-retried cases must not emit a noisy
        // `"retries":[]` key so existing envelopes stay byte-stable.
        let summary = sample_test_summary();
        let env = ResultEnvelope::from_test_summary(&summary, Path::new("/tmp/proj"));
        let json = serde_json::to_string(&env).unwrap();
        assert!(!json.contains("\"retries\""), "{json}");
    }

    #[test]
    fn slowest_cases_orders_by_duration_descending() {
        // @spec #2721 — text output can summarise slowest cases from
        // evidence. The envelope exposes a helper so reporters don't
        // need to re-sort the case list themselves.
        let env = ResultEnvelope {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: ResultMode::Test,
            summary: ResultSummary::default(),
            cases: vec![
                ResultCase {
                    id: "a".into(),
                    title: "a".into(),
                    file: PathBuf::from("a.spec.ts"),
                    outcome: "passed".into(),
                    duration_ms: 10,
                    failure: None,
                    artifacts: vec![],
                    retries: vec![],
                },
                ResultCase {
                    id: "b".into(),
                    title: "b".into(),
                    file: PathBuf::from("b.spec.ts"),
                    outcome: "passed".into(),
                    duration_ms: 500,
                    failure: None,
                    artifacts: vec![],
                    retries: vec![],
                },
                ResultCase {
                    id: "c".into(),
                    title: "c".into(),
                    file: PathBuf::from("c.spec.ts"),
                    outcome: "passed".into(),
                    duration_ms: 100,
                    failure: None,
                    artifacts: vec![],
                    retries: vec![],
                },
            ],
            artifacts: vec![],
            mode_data: serde_json::Value::Null,
        };
        let slowest = env.slowest_cases(2);
        assert_eq!(slowest.len(), 2);
        assert_eq!(slowest[0].id, "b", "longest first");
        assert_eq!(slowest[1].id, "c", "second longest");

        let all = env.slowest_cases(10);
        assert_eq!(all.len(), 3, "limit larger than cases returns all");
    }

    #[test]
    fn golden_diff_failure_links_expected_actual_and_diff_artifacts() {
        // @spec #2870 — a failed golden assertion must link three
        // artifacts: expected, actual, diff. All three paths must be
        // bundle-relative (so the bundle can move) and the trio must
        // survive serde round-trips into the result envelope.
        use crate::evidence_bundle::{
            BundleArtifact, BundleCommand, BundleEnvironment, BundleManifest,
        };

        let mut manifest = BundleManifest::new(
            "run-golden",
            BundleCommand::Test,
            "jet",
            "cafef00d",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.51".into(),
                ci: None,
                node_version: None,
            },
        );
        manifest.artifacts.push(BundleArtifact {
            id: "snap-1-expected".into(),
            kind: "golden_expected".into(),
            path: PathBuf::from("artifacts/snap-1.expected.txt"),
            content_type: Some("text/plain".into()),
        });
        manifest.artifacts.push(BundleArtifact {
            id: "snap-1-actual".into(),
            kind: "golden_actual".into(),
            path: PathBuf::from("artifacts/snap-1.actual.txt"),
            content_type: Some("text/plain".into()),
        });
        manifest.artifacts.push(BundleArtifact {
            id: "snap-1-diff".into(),
            kind: "golden_diff".into(),
            path: PathBuf::from("artifacts/snap-1.diff.txt"),
            content_type: Some("text/plain".into()),
        });

        let refs = FailureArtifactRef::golden_triplet(
            "snap-1-expected",
            "snap-1-actual",
            "snap-1-diff",
            &manifest,
        );

        // Stop condition: a fixture failure contains expected, actual,
        // and diff refs — in that order — and each is bundle-relative.
        assert_eq!(refs.len(), 3);
        assert_eq!(refs[0].id, "snap-1-expected");
        assert_eq!(refs[0].kind.as_deref(), Some("golden_expected"));
        assert_eq!(refs[1].id, "snap-1-actual");
        assert_eq!(refs[1].kind.as_deref(), Some("golden_actual"));
        assert_eq!(refs[2].id, "snap-1-diff");
        assert_eq!(refs[2].kind.as_deref(), Some("golden_diff"));
        for r in &refs {
            let p = r.path.as_ref().expect("resolved triplet has a path");
            assert!(
                p.is_relative(),
                "golden artifact path must stay bundle-relative: {p:?}",
            );
            assert!(
                p.starts_with("artifacts/"),
                "paths must live under the artifacts/ dir: {p:?}",
            );
        }

        // The trio attaches to a ResultFailure and round-trips through JSON.
        let failure = ResultFailure {
            message: "Snapshot mismatch for snap-1".into(),
            stack: None,
            diff: Some("--- expected\n+++ actual\n@@\n-hi\n+ho\n".into()),
            rerun_hint: Some("jet test specs/snap.spec.ts -g 'snap-1' --update-snapshots".into()),
            source_location: None,
            artifacts: refs,
            timeout_budget_ms: None,
        };
        let json = serde_json::to_string(&failure).unwrap();
        assert!(json.contains(r#""id":"snap-1-expected""#), "{json}");
        assert!(json.contains(r#""id":"snap-1-actual""#), "{json}");
        assert!(json.contains(r#""id":"snap-1-diff""#), "{json}");
        assert!(json.contains(r#""kind":"golden_diff""#), "{json}");
        assert!(
            json.contains(r#""path":"artifacts/snap-1.diff.txt""#),
            "diff path must serialise bundle-relative: {json}",
        );

        let back: ResultFailure = serde_json::from_str(&json).unwrap();
        assert_eq!(back.artifacts.len(), 3);
        assert_eq!(back.artifacts[0].id, "snap-1-expected");
        assert_eq!(back.artifacts[2].kind.as_deref(), Some("golden_diff"));
    }

    #[test]
    fn golden_triplet_records_missing_artifacts_explicitly() {
        // @spec #2870 — when the producer fails to register one of the
        // three golden artifacts (e.g. no `expected` baseline yet), the
        // ref must still appear with an explicit missing marker so the
        // report does not silently lose the slot.
        use crate::evidence_bundle::{
            BundleArtifact, BundleCommand, BundleEnvironment, BundleManifest,
        };

        let mut manifest = BundleManifest::new(
            "run-golden-missing",
            BundleCommand::Test,
            "jet",
            "cafef00d",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.51".into(),
                ci: None,
                node_version: None,
            },
        );
        // Only actual + diff registered — expected is intentionally
        // absent (new snapshot, no baseline yet).
        manifest.artifacts.push(BundleArtifact {
            id: "snap-2-actual".into(),
            kind: "golden_actual".into(),
            path: PathBuf::from("artifacts/snap-2.actual.txt"),
            content_type: None,
        });
        manifest.artifacts.push(BundleArtifact {
            id: "snap-2-diff".into(),
            kind: "golden_diff".into(),
            path: PathBuf::from("artifacts/snap-2.diff.txt"),
            content_type: None,
        });

        let refs = FailureArtifactRef::golden_triplet(
            "snap-2-expected",
            "snap-2-actual",
            "snap-2-diff",
            &manifest,
        );

        assert_eq!(refs.len(), 3);
        assert!(refs[0].is_missing(), "expected baseline should be missing");
        assert_eq!(refs[0].id, "snap-2-expected");
        assert_eq!(refs[0].path, None);
        assert_eq!(refs[0].kind, None);
        assert!(!refs[1].is_missing());
        assert!(!refs[2].is_missing());
    }

    #[test]
    fn failure_artifact_refs_resolve_through_bundle_manifest() {
        // @spec #2875 — a failure record can list related artifacts by
        // stable id/path resolved against the bundle manifest, and
        // missing refs are represented explicitly (path/kind None).
        use crate::evidence_bundle::{
            BundleArtifact, BundleCommand, BundleEnvironment, BundleManifest,
        };

        let mut manifest = BundleManifest::new(
            "run-art",
            BundleCommand::E2e,
            "jet",
            "cafef00d",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.51".into(),
                ci: None,
                node_version: None,
            },
        );
        manifest.artifacts.push(BundleArtifact {
            id: "page-1".into(),
            kind: "screenshot".into(),
            path: PathBuf::from("artifacts/page-1.png"),
            content_type: Some("image/png".into()),
        });
        manifest.artifacts.push(BundleArtifact {
            id: "console-1".into(),
            kind: "log".into(),
            path: PathBuf::from("artifacts/console-1.log"),
            content_type: None,
        });

        let refs = FailureArtifactRef::resolve_all(
            ["page-1", "console-1", "missing-from-bundle"],
            &manifest,
        );

        assert_eq!(refs.len(), 3);
        // Resolved: id + kind + path from manifest.
        assert_eq!(refs[0].id, "page-1");
        assert_eq!(refs[0].kind.as_deref(), Some("screenshot"));
        assert_eq!(refs[0].path, Some(PathBuf::from("artifacts/page-1.png")));
        assert!(!refs[0].is_missing());
        assert_eq!(refs[1].id, "console-1");
        assert_eq!(refs[1].kind.as_deref(), Some("log"));
        // Explicit missing: id retained, kind+path None.
        assert_eq!(refs[2].id, "missing-from-bundle");
        assert!(refs[2].is_missing());
        assert_eq!(refs[2].kind, None);
        assert_eq!(refs[2].path, None);

        // Attaching them to a ResultFailure round-trips through JSON
        // and keeps the missing entry distinguishable from a resolved
        // one (the missing entry serialises with neither `kind` nor
        // `path`, the resolved one with both).
        let failure = ResultFailure {
            message: "boom".into(),
            stack: None,
            diff: None,
            rerun_hint: None,
            source_location: None,
            artifacts: refs,
            timeout_budget_ms: None,
        };
        let json = serde_json::to_string(&failure).unwrap();
        assert!(
            json.contains(r#""id":"page-1""#)
                && json.contains(r#""kind":"screenshot""#)
                && json.contains(r#""path":"artifacts/page-1.png""#),
            "{json}"
        );
        assert!(
            json.contains(r#"{"id":"missing-from-bundle"}"#),
            "missing artifact must serialise as id-only: {json}",
        );

        let back: ResultFailure = serde_json::from_str(&json).unwrap();
        assert_eq!(back.artifacts.len(), 3);
        assert!(back.artifacts[2].is_missing());
    }

    #[test]
    fn e2e_failing_case_exposes_focused_rerun_command() {
        // @spec #2874 — failure evidence includes a copy/paste rerun
        // command derived from resolved case metadata (no hand-written
        // strings). Two fixture examples are required: a `jet test`
        // failure (covered by from_test_summary_populates_case_failure)
        // and a `jet e2e` failure (this fixture).
        use crate::e2e::{
            E2eAssertionDetail, E2eCaseEvidence, E2eEvidenceBundle, E2eMode, E2eProductStep,
            E2eStepContext, E2eSummary,
        };

        let bundle = E2eEvidenceBundle {
            schema_version: "jet.e2e.evidence.v1".to_string(),
            mode: E2eMode::Run,
            run_id: "run-fail".to_string(),
            started_at_ms: 0,
            finished_at_ms: 500,
            summary: E2eSummary {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 500,
                exit_code: 1,
            },
            cases: vec![E2eCaseEvidence {
                id: "checkout-1".to_string(),
                title: "checkout flow".to_string(),
                file: PathBuf::from("e2e/checkout.case.ts"),
                outcome: "failed".to_string(),
                duration_ms: 500,
                steps: vec![E2eProductStep {
                    id: "step-3".to_string(),
                    title: "confirm order".to_string(),
                    status: "failed".to_string(),
                    duration_ms: 120,
                    assertion: Some(E2eAssertionDetail {
                        message: "expected order id".to_string(),
                        stack: None,
                        diff: None,
                    }),
                    context: E2eStepContext::default(),
                }],
            }],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        };

        let env = ResultEnvelope::from_e2e_bundle(&bundle);
        let case = env.cases.first().expect("one e2e case");
        let failure = case.failure.as_ref().expect("case is failed");
        let hint = failure
            .rerun_hint
            .as_deref()
            .expect("e2e failure must expose a rerun command");
        // Resolved from case metadata: subcommand, file path, --grep title.
        assert!(hint.starts_with("jet e2e run "), "{hint}");
        assert!(hint.contains("e2e/checkout.case.ts"), "{hint}");
        assert!(hint.contains("--grep 'checkout flow'"), "{hint}");
    }

    #[test]
    fn rerun_hint_degrades_to_none_when_assertion_metadata_is_missing() {
        // @spec #2874 — unknown metadata degrades to a documented
        // unavailable state. `failure` is constructed off the first
        // step assertion; a failed case with no assertion yields no
        // failure record and therefore no rerun_hint surface.
        use crate::e2e::{E2eCaseEvidence, E2eEvidenceBundle, E2eMode, E2eSummary};

        let bundle = E2eEvidenceBundle {
            schema_version: "jet.e2e.evidence.v1".to_string(),
            mode: E2eMode::Run,
            run_id: "run-bare".to_string(),
            started_at_ms: 0,
            finished_at_ms: 1,
            summary: E2eSummary {
                passed: 0,
                failed: 1,
                skipped: 0,
                duration_ms: 1,
                exit_code: 1,
            },
            cases: vec![E2eCaseEvidence {
                id: "bare".into(),
                title: "bare".into(),
                file: PathBuf::from("e2e/bare.case.ts"),
                outcome: "failed".into(),
                duration_ms: 1,
                steps: vec![],
            }],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        };

        let env = ResultEnvelope::from_e2e_bundle(&bundle);
        let case = env.cases.first().unwrap();
        assert!(
            case.failure.is_none(),
            "no assertion metadata => no failure surface (rerun_hint stays absent)",
        );
    }

    #[test]
    fn e2e_bundle_maps_into_envelope() {
        use crate::e2e::{E2eCaseEvidence, E2eEvidenceBundle, E2eMode, E2eSummary};

        let bundle = E2eEvidenceBundle {
            schema_version: "jet.e2e.evidence.v1".to_string(),
            mode: E2eMode::Run,
            run_id: "run-123".to_string(),
            started_at_ms: 1,
            finished_at_ms: 1000,
            summary: E2eSummary {
                passed: 3,
                failed: 0,
                skipped: 0,
                duration_ms: 999,
                exit_code: 0,
            },
            cases: vec![E2eCaseEvidence {
                id: "case-1".to_string(),
                title: "cart flow".to_string(),
                file: PathBuf::from("/tmp/e2e/cart.case.ts"),
                outcome: "passed".to_string(),
                duration_ms: 500,
                steps: vec![],
            }],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        };

        let env = ResultEnvelope::from_e2e_bundle(&bundle);
        assert_eq!(env.mode, ResultMode::E2eRun);
        assert_eq!(env.summary.passed, 3);
        assert_eq!(env.cases.len(), 1);
        assert_eq!(env.cases[0].id, "case-1");
        assert_eq!(env.mode_data["run_id"], "run-123");
        assert_eq!(
            env.mode_data["source_schema_version"],
            "jet.e2e.evidence.v1"
        );
    }

    // gh3787: silent empty failure.message fallback warns + emits sentinel.
    mod gh3787_empty_failure_message_warn_tests {
        use super::*;

        fn env_with_failure_message(title: &str, message: &str) -> ResultEnvelope {
            ResultEnvelope {
                schema_version: SCHEMA_VERSION.to_string(),
                mode: ResultMode::Test,
                summary: ResultSummary {
                    passed: 0,
                    failed: 1,
                    skipped: 0,
                    duration_ms: 1,
                },
                cases: vec![ResultCase {
                    id: format!("specs/x.spec.ts::{title}"),
                    title: title.to_string(),
                    file: PathBuf::from("specs/x.spec.ts"),
                    outcome: "failed".to_string(),
                    duration_ms: 1,
                    failure: Some(ResultFailure {
                        message: message.to_string(),
                        stack: None,
                        diff: None,
                        rerun_hint: None,
                        source_location: None,
                        artifacts: Vec::new(),
                        timeout_budget_ms: None,
                    }),
                    artifacts: Vec::new(),
                    retries: Vec::new(),
                }],
                artifacts: Vec::new(),
                mode_data: serde_json::Value::Null,
            }
        }

        #[test]
        fn empty_message_emits_sentinel_via_helper() {
            // 1) Truly-empty message yields the sentinel and the rendered row.
            let rendered = first_failure_message_line_or_warn("", "case-one");
            assert_eq!(rendered, EMPTY_FAILURE_MESSAGE_SENTINEL);
            let env = env_with_failure_message("case-one", "");
            let text = format_failures_text(&env);
            assert!(
                text.contains(&format!("  message: {EMPTY_FAILURE_MESSAGE_SENTINEL}\n")),
                "expected sentinel-rendered message row, got:\n{text}"
            );
        }

        #[test]
        fn single_line_non_empty_message_renders_unchanged() {
            // 2) Single-line non-empty path matches the legacy output.
            let rendered = first_failure_message_line_or_warn("expected 5", "case");
            assert_eq!(rendered, "expected 5");
            assert_ne!(rendered, EMPTY_FAILURE_MESSAGE_SENTINEL);
        }

        #[test]
        fn multi_line_message_picks_first_line() {
            // 3) Multi-line message uses the first line, no sentinel.
            let rendered = first_failure_message_line_or_warn(
                "headline\nstack frame 1\nstack frame 2",
                "case",
            );
            assert_eq!(rendered, "headline");
            assert_ne!(rendered, EMPTY_FAILURE_MESSAGE_SENTINEL);
        }

        #[test]
        fn leading_newline_keeps_empty_first_line_no_warn() {
            // 4) `"\n..."` yields Some("") from lines().next() — different
            // shape from truly-empty; helper must NOT emit the sentinel.
            let rendered = first_failure_message_line_or_warn("\nbody", "case");
            assert_eq!(rendered, "");
            assert_ne!(rendered, EMPTY_FAILURE_MESSAGE_SENTINEL);
        }

        #[test]
        fn warn_helper_name_pinned_for_discoverability() {
            // 5) Pin the helper name so a rename without grep breaks tests.
            let w = format_result_envelope_empty_failure_message_warn("case");
            assert!(
                !w.is_empty(),
                "format_result_envelope_empty_failure_message_warn returned empty string",
            );
        }

        #[test]
        fn warn_string_includes_gh3787_issue_tag() {
            // 6) Issue tag is the audit trail anchor.
            let w = format_result_envelope_empty_failure_message_warn("case");
            assert!(w.contains("gh3787"), "missing gh3787 tag: {w}");
        }

        #[test]
        fn warn_string_distinct_from_sibling_silent_fallback_families() {
            // 7) Sibling-distinctness vs prior silent-fallback warns —
            // log filtering must split them.
            let w = format_result_envelope_empty_failure_message_warn("case");
            assert!(
                !w.contains("gh3776"),
                "must not overlap reporter/html warn: {w}"
            );
            assert!(!w.contains("gh3774"), "must not overlap asset warn: {w}");
            assert!(
                !w.contains("gh3772"),
                "must not overlap nx project-name warn: {w}"
            );
            assert!(
                !w.contains("gh3770"),
                "must not overlap browser/page warn: {w}"
            );
            assert!(
                !w.contains("gh3768"),
                "must not overlap browser/locator warn: {w}"
            );
            assert!(
                !w.contains("gh3765"),
                "must not overlap nx rel-root warn: {w}"
            );
            assert!(
                !w.contains("gh3763"),
                "must not overlap reporter/parser warn: {w}"
            );
        }

        #[test]
        fn mixed_failures_only_empty_messages_use_sentinel() {
            // 8) Multiple failures with mixed empty/non-empty messages —
            // only the empty one emits the sentinel; ordering preserved.
            let env = ResultEnvelope {
                schema_version: SCHEMA_VERSION.to_string(),
                mode: ResultMode::Test,
                summary: ResultSummary {
                    passed: 0,
                    failed: 2,
                    skipped: 0,
                    duration_ms: 2,
                },
                cases: vec![
                    ResultCase {
                        id: "a".to_string(),
                        title: "case-a".to_string(),
                        file: PathBuf::from("specs/x.spec.ts"),
                        outcome: "failed".to_string(),
                        duration_ms: 1,
                        failure: Some(ResultFailure {
                            message: "".to_string(),
                            stack: None,
                            diff: None,
                            rerun_hint: None,
                            source_location: None,
                            artifacts: Vec::new(),
                            timeout_budget_ms: None,
                        }),
                        artifacts: Vec::new(),
                        retries: Vec::new(),
                    },
                    ResultCase {
                        id: "b".to_string(),
                        title: "case-b".to_string(),
                        file: PathBuf::from("specs/x.spec.ts"),
                        outcome: "failed".to_string(),
                        duration_ms: 1,
                        failure: Some(ResultFailure {
                            message: "expected 7".to_string(),
                            stack: None,
                            diff: None,
                            rerun_hint: None,
                            source_location: None,
                            artifacts: Vec::new(),
                            timeout_budget_ms: None,
                        }),
                        artifacts: Vec::new(),
                        retries: Vec::new(),
                    },
                ],
                artifacts: Vec::new(),
                mode_data: serde_json::Value::Null,
            };
            let text = format_failures_text(&env);
            let sentinel_count = text.matches(EMPTY_FAILURE_MESSAGE_SENTINEL).count();
            assert_eq!(sentinel_count, 1, "exactly one sentinel expected: {text}");
            let idx_a = text.find("[FAIL] case-a").expect("case-a present");
            let idx_b = text.find("[FAIL] case-b").expect("case-b present");
            assert!(idx_a < idx_b, "ordering preserved: {text}");
            assert!(
                text.contains("expected 7"),
                "non-empty msg untouched: {text}"
            );
        }

        #[test]
        fn warn_helper_handles_title_with_special_chars() {
            // 9) Title containing newlines / colons must not corrupt the
            // warn string — Debug-quoted via {title:?}.
            let w = format_result_envelope_empty_failure_message_warn("tricky\n: title");
            assert!(w.contains("gh3787"), "{w}");
            assert!(w.contains("tricky"), "title preserved in warn: {w}");
        }

        #[test]
        fn full_render_integration_pass_empty_normal() {
            // 10) Happy-path integration: 1 pass (no failure section),
            // 1 empty-message fail (sentinel), 1 normal fail (untouched).
            let env = ResultEnvelope {
                schema_version: SCHEMA_VERSION.to_string(),
                mode: ResultMode::Test,
                summary: ResultSummary {
                    passed: 1,
                    failed: 2,
                    skipped: 0,
                    duration_ms: 3,
                },
                cases: vec![
                    ResultCase {
                        id: "p".to_string(),
                        title: "passes".to_string(),
                        file: PathBuf::from("specs/x.spec.ts"),
                        outcome: "passed".to_string(),
                        duration_ms: 1,
                        failure: None,
                        artifacts: Vec::new(),
                        retries: Vec::new(),
                    },
                    ResultCase {
                        id: "e".to_string(),
                        title: "empty-msg".to_string(),
                        file: PathBuf::from("specs/x.spec.ts"),
                        outcome: "failed".to_string(),
                        duration_ms: 1,
                        failure: Some(ResultFailure {
                            message: "".to_string(),
                            stack: None,
                            diff: None,
                            rerun_hint: None,
                            source_location: None,
                            artifacts: Vec::new(),
                            timeout_budget_ms: None,
                        }),
                        artifacts: Vec::new(),
                        retries: Vec::new(),
                    },
                    ResultCase {
                        id: "n".to_string(),
                        title: "normal-fail".to_string(),
                        file: PathBuf::from("specs/x.spec.ts"),
                        outcome: "failed".to_string(),
                        duration_ms: 1,
                        failure: Some(ResultFailure {
                            message: "AssertionError: bad math".to_string(),
                            stack: None,
                            diff: None,
                            rerun_hint: None,
                            source_location: None,
                            artifacts: Vec::new(),
                            timeout_budget_ms: None,
                        }),
                        artifacts: Vec::new(),
                        retries: Vec::new(),
                    },
                ],
                artifacts: Vec::new(),
                mode_data: serde_json::Value::Null,
            };
            let text = format_failures_text(&env);
            assert!(
                !text.contains("[FAIL] passes"),
                "passes not in failures: {text}"
            );
            assert!(text.contains("[FAIL] empty-msg"), "{text}");
            assert!(text.contains("[FAIL] normal-fail"), "{text}");
            assert!(text.contains(EMPTY_FAILURE_MESSAGE_SENTINEL), "{text}");
            assert!(
                text.contains("  message: AssertionError: bad math\n"),
                "{text}"
            );
        }
    }
}
// CODEGEN-END
