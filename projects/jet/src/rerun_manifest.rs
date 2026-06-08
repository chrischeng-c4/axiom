// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-src.md#schema
// CODEGEN-BEGIN
//! Failure-only rerun manifest projection (#2720).
//!
//! Both `jet test` and `jet e2e` already record per-case rerun hints,
//! source locations, and artifact paths inside the unified
//! [`ResultEnvelope`](crate::result_envelope::ResultEnvelope). Agents and CI that just want
//! to "rerun whatever failed" should not have to walk that envelope to
//! filter failures out of passes. This module flattens the failed cases
//! into a small, agent-readable manifest:
//!
//! ```text
//! { schema_version: "jet.rerun.manifest.v1",
//!   mode: "test" | "e2e_run" | "e2e_open",
//!   failed_count: <u32>,
//!   entries: [
//!     { id, title, file, outcome,
//!       rerun_command?, failure_message?, source_location?,
//!       artifacts: [{ kind, path? }] }
//!   ] }
//! ```
//!
//! A passing run emits an empty `entries` array (not absent) so consumers
//! can always parse the same shape. `outcome` is one of
//! `failed | timed_out | crashed` — the failure lexicon shared with the
//! envelope.
//!
//! ## Versioning
//!
//! [`RERUN_MANIFEST_SCHEMA_VERSION`] is bumped on breaking changes (renamed
//! or removed fields). New optional fields are non-breaking.

use crate::result_envelope::{
    FailureArtifactRef, ResultCase, ResultEnvelope, ResultMode, ResultSourceLocation,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Stable schema tag for the failure-only rerun manifest. Bumped on any
/// breaking change to the field shape. Adding new optional fields is
/// non-breaking.
pub const RERUN_MANIFEST_SCHEMA_VERSION: &str = "jet.rerun.manifest.v1";

/// Manifest projection of a [`ResultEnvelope`] containing only failed
/// cases. Passing runs still serialize with `entries: []` so consumers
/// see a stable shape.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerunManifest {
    pub schema_version: String,
    pub mode: ResultMode,
    pub failed_count: u32,
    pub entries: Vec<RerunEntry>,
}

/// One failed-case row in the manifest.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerunEntry {
    pub id: String,
    pub title: String,
    pub file: PathBuf,
    /// One of `failed | timed_out | crashed`. Skipped/passed cases are
    /// never present in the manifest.
    pub outcome: String,
    /// Pasteable shell command for rerunning this case in isolation. Lifted
    /// verbatim from `ResultFailure::rerun_hint`; `None` when the runner
    /// did not attach one (e.g., degraded metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rerun_command: Option<String>,
    /// First user-frame source location parsed from the failure stack, when
    /// available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_location: Option<ResultSourceLocation>,
    /// Short failure message lifted from `ResultFailure::message`. Useful
    /// for agents that want to triage without loading the full envelope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_message: Option<String>,
    /// Failure-scoped artifact references resolved through the bundle
    /// manifest (e.g., screenshot, trace, diff). Empty when nothing is
    /// attached.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub artifacts: Vec<FailureArtifactRef>,
}

/// Sentinel surfaced into `failure_message` when a failure-outcome
/// case ships without `ResultFailure` metadata. The accompanying
/// `tracing::warn!` is the audit trail; the sentinel keeps the gap
/// visible to a human reading the rerun manifest.
// @spec gh3791 — silent empty rerun manifest entry for wrong-shape envelope
pub(crate) const MISSING_FAILURE_METADATA_SENTINEL: &str =
    "<missing failure metadata for failure-outcome case>";

/// Format the warn string emitted when a failure-outcome case lacks
/// `ResultFailure`. Pinned in tests so a rename without grep breaks
/// the suite.
// @spec gh3791
pub(crate) fn format_rerun_manifest_missing_failure_warn(case_id: &str, outcome: &str) -> String {
    format!(
        "gh3791: rerun manifest entry id={case_id:?} outcome={outcome:?} has no \
         ResultFailure attached — upstream envelope is wrong-shape; \
         rerun command, source location, artifacts will be empty"
    )
}

/// Build a single [`RerunEntry`] from a failure-outcome case, warning
/// when the cross-field invariant (`is_failure_outcome(case.outcome)`
/// implies `case.failure.is_some()`) is violated.
// @spec gh3791
pub(crate) fn rerun_entry_from_case_or_warn(case: &ResultCase) -> RerunEntry {
    match case.failure.as_ref() {
        Some(f) => RerunEntry {
            id: case.id.clone(),
            title: case.title.clone(),
            file: case.file.clone(),
            outcome: case.outcome.clone(),
            rerun_command: f.rerun_hint.clone(),
            source_location: f.source_location.clone(),
            failure_message: Some(f.message.clone()),
            artifacts: f.artifacts.clone(),
        },
        None => {
            tracing::warn!(
                "{}",
                format_rerun_manifest_missing_failure_warn(&case.id, &case.outcome),
            );
            RerunEntry {
                id: case.id.clone(),
                title: case.title.clone(),
                file: case.file.clone(),
                outcome: case.outcome.clone(),
                rerun_command: None,
                source_location: None,
                failure_message: Some(MISSING_FAILURE_METADATA_SENTINEL.to_string()),
                artifacts: Vec::new(),
            }
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
impl RerunManifest {
    /// Build a failure-only manifest from a [`ResultEnvelope`].
    ///
    /// Passing runs return an empty `entries` array (not absent), so the
    /// schema shape stays consistent across runs.
    pub fn from_envelope(env: &ResultEnvelope) -> Self {
        let mut entries: Vec<RerunEntry> = Vec::new();
        for case in &env.cases {
            if !is_failure_outcome(&case.outcome) {
                continue;
            }
            entries.push(rerun_entry_from_case_or_warn(case));
        }
        Self {
            schema_version: RERUN_MANIFEST_SCHEMA_VERSION.to_string(),
            mode: env.mode,
            failed_count: entries.len() as u32,
            entries,
        }
    }

    /// True when no failed/timed-out/crashed cases were recorded.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Write the manifest as pretty-printed JSON at `path`. Always overwrites.
/// @spec .aw/tech-design/projects/jet/semantic/jet-src.md#schema
pub fn write_manifest(manifest: &RerunManifest, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let body = serde_json::to_vec_pretty(manifest).context("serializing rerun manifest")?;
    std::fs::write(path, body).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

fn is_failure_outcome(outcome: &str) -> bool {
    matches!(outcome, "failed" | "timed_out" | "crashed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result_envelope::{ResultArtifact, ResultFailure, ResultSummary, SCHEMA_VERSION};
    use std::path::PathBuf;

    fn passing_case(id: &str, title: &str) -> ResultCase {
        ResultCase {
            id: id.into(),
            title: title.into(),
            file: PathBuf::from("specs/ok.spec.ts"),
            outcome: "passed".into(),
            duration_ms: 5,
            failure: None,
            artifacts: vec![],
            retries: vec![],
        }
    }

    fn failing_case(id: &str, title: &str, outcome: &str) -> ResultCase {
        ResultCase {
            id: id.into(),
            title: title.into(),
            file: PathBuf::from("specs/bad.spec.ts"),
            outcome: outcome.into(),
            duration_ms: 12,
            failure: Some(ResultFailure {
                message: format!("expected pass, got {outcome}"),
                stack: None,
                diff: None,
                rerun_hint: Some(format!("jet test specs/bad.spec.ts -g '{title}'")),
                source_location: Some(ResultSourceLocation {
                    file: PathBuf::from("specs/bad.spec.ts"),
                    line: 17,
                    column: Some(3),
                }),
                artifacts: vec![FailureArtifactRef {
                    id: "screenshot-1".into(),
                    kind: Some("screenshot".into()),
                    path: Some(PathBuf::from("artifacts/screenshot-1.png")),
                }],
                timeout_budget_ms: None,
            }),
            artifacts: vec![],
            retries: vec![],
        }
    }

    fn envelope(cases: Vec<ResultCase>) -> ResultEnvelope {
        let failed = cases.iter().filter(|c| c.outcome != "passed").count() as u32;
        let passed = cases.iter().filter(|c| c.outcome == "passed").count() as u32;
        ResultEnvelope {
            schema_version: SCHEMA_VERSION.into(),
            mode: ResultMode::Test,
            summary: ResultSummary {
                passed,
                failed,
                skipped: 0,
                duration_ms: 100,
            },
            cases,
            artifacts: vec![] as Vec<ResultArtifact>,
            mode_data: serde_json::json!({}),
        }
    }

    #[test]
    fn manifest_filters_to_failed_cases_only() {
        let env = envelope(vec![
            passing_case("a", "ok one"),
            failing_case("b", "boom", "failed"),
            passing_case("c", "ok two"),
        ]);
        let manifest = RerunManifest::from_envelope(&env);
        assert_eq!(manifest.failed_count, 1);
        assert_eq!(manifest.entries.len(), 1);
        assert_eq!(manifest.entries[0].id, "b");
        assert_eq!(manifest.entries[0].outcome, "failed");
    }

    #[test]
    fn manifest_includes_timed_out_and_crashed_outcomes() {
        let env = envelope(vec![
            failing_case("a", "slow", "timed_out"),
            failing_case("b", "boom", "crashed"),
            passing_case("c", "ok"),
        ]);
        let manifest = RerunManifest::from_envelope(&env);
        assert_eq!(manifest.failed_count, 2);
        let outcomes: Vec<&str> = manifest
            .entries
            .iter()
            .map(|e| e.outcome.as_str())
            .collect();
        assert!(outcomes.contains(&"timed_out"));
        assert!(outcomes.contains(&"crashed"));
    }

    #[test]
    fn passing_run_emits_empty_entries_with_stable_schema() {
        let env = envelope(vec![
            passing_case("a", "ok one"),
            passing_case("b", "ok two"),
        ]);
        let manifest = RerunManifest::from_envelope(&env);
        assert!(manifest.is_empty());
        assert_eq!(manifest.failed_count, 0);
        assert_eq!(manifest.schema_version, RERUN_MANIFEST_SCHEMA_VERSION);
        let json = serde_json::to_string(&manifest).unwrap();
        assert!(json.contains("\"entries\":[]"), "json: {json}");
    }

    #[test]
    fn entry_carries_source_location_artifacts_and_rerun_command() {
        let env = envelope(vec![failing_case("a", "boom", "failed")]);
        let manifest = RerunManifest::from_envelope(&env);
        let entry = &manifest.entries[0];
        let rerun = entry.rerun_command.as_deref().expect("rerun_command");
        assert!(rerun.starts_with("jet test"));
        assert!(rerun.contains("boom"));
        let loc = entry.source_location.as_ref().expect("source_location");
        assert_eq!(loc.line, 17);
        assert_eq!(loc.column, Some(3));
        assert_eq!(entry.artifacts.len(), 1);
        assert_eq!(entry.artifacts[0].kind.as_deref(), Some("screenshot"));
        assert!(entry
            .failure_message
            .as_deref()
            .unwrap()
            .contains("expected pass"));
    }

    #[test]
    fn missing_rerun_hint_or_source_location_degrade_to_none() {
        // Failure recorded but without rerun_hint or source_location — e.g.,
        // degraded worker metadata. Manifest entry must still serialize.
        let env = envelope(vec![ResultCase {
            id: "a".into(),
            title: "degraded".into(),
            file: PathBuf::from("specs/x.spec.ts"),
            outcome: "failed".into(),
            duration_ms: 1,
            failure: Some(ResultFailure {
                message: "no metadata".into(),
                stack: None,
                diff: None,
                rerun_hint: None,
                source_location: None,
                artifacts: vec![],
                timeout_budget_ms: None,
            }),
            artifacts: vec![],
            retries: vec![],
        }]);
        let manifest = RerunManifest::from_envelope(&env);
        let entry = &manifest.entries[0];
        assert!(entry.rerun_command.is_none());
        assert!(entry.source_location.is_none());
        assert!(entry.artifacts.is_empty());
    }

    #[test]
    fn write_manifest_round_trips_through_disk() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let path = tmp.path().join("manifest.json");
        let env = envelope(vec![failing_case("a", "boom", "failed")]);
        let manifest = RerunManifest::from_envelope(&env);
        write_manifest(&manifest, &path).expect("write");
        let body = std::fs::read_to_string(&path).expect("read");
        let parsed: RerunManifest = serde_json::from_str(&body).expect("decode");
        assert_eq!(parsed.failed_count, 1);
        assert_eq!(parsed.schema_version, RERUN_MANIFEST_SCHEMA_VERSION);
        assert_eq!(parsed.entries[0].title, "boom");
    }

    // gh3791: failure-outcome case with `failure: None` warns + sentinel.
    mod gh3791_missing_failure_metadata_warn_tests {
        use super::*;

        fn failure_outcome_no_failure(id: &str, outcome: &str) -> ResultCase {
            ResultCase {
                id: id.into(),
                title: format!("case-{id}"),
                file: PathBuf::from("specs/x.spec.ts"),
                outcome: outcome.into(),
                duration_ms: 1,
                failure: None,
                artifacts: vec![],
                retries: vec![],
            }
        }

        #[test]
        fn well_shaped_failure_case_renders_unchanged() {
            // 1) Failure with `Some(failure)` produces the same entry as
            // today — sentinel must NOT appear.
            let env = envelope(vec![failing_case("a", "boom", "failed")]);
            let manifest = RerunManifest::from_envelope(&env);
            let entry = &manifest.entries[0];
            assert!(entry.rerun_command.is_some());
            assert!(entry.source_location.is_some());
            assert_eq!(entry.artifacts.len(), 1);
            let msg = entry.failure_message.as_deref().expect("failure_message");
            assert!(msg.contains("expected pass"));
            assert!(
                !msg.contains(MISSING_FAILURE_METADATA_SENTINEL),
                "well-shaped case must not carry sentinel: {msg}",
            );
        }

        #[test]
        fn failure_outcome_with_none_failure_emits_sentinel() {
            // 2) Failure-outcome case with `failure: None` carries the
            // sentinel in `failure_message`; other fields stay None/empty.
            let env = envelope(vec![failure_outcome_no_failure("a", "failed")]);
            let manifest = RerunManifest::from_envelope(&env);
            assert_eq!(manifest.failed_count, 1);
            let entry = &manifest.entries[0];
            assert_eq!(entry.id, "a");
            assert_eq!(entry.outcome, "failed");
            assert!(entry.rerun_command.is_none());
            assert!(entry.source_location.is_none());
            assert!(entry.artifacts.is_empty());
            assert_eq!(
                entry.failure_message.as_deref(),
                Some(MISSING_FAILURE_METADATA_SENTINEL),
            );
        }

        #[test]
        fn passing_case_filtered_regardless_of_failure_shape() {
            // 3) Passing case never appears in the manifest — even if
            // `failure` is None (which is the normal pass shape).
            let env = envelope(vec![passing_case("p", "ok")]);
            let manifest = RerunManifest::from_envelope(&env);
            assert_eq!(manifest.failed_count, 0);
            assert!(manifest.entries.is_empty());
        }

        #[test]
        fn skipped_case_filtered_out() {
            // 4) Skipped case filtered regardless of failure shape.
            let env = envelope(vec![ResultCase {
                id: "s".into(),
                title: "skipped".into(),
                file: PathBuf::from("specs/x.spec.ts"),
                outcome: "skipped".into(),
                duration_ms: 0,
                failure: None,
                artifacts: vec![],
                retries: vec![],
            }]);
            let manifest = RerunManifest::from_envelope(&env);
            assert_eq!(manifest.failed_count, 0);
            assert!(manifest.entries.is_empty());
        }

        #[test]
        fn timed_out_and_crashed_outcomes_warn_when_failure_is_none() {
            // 5) Sentinel applies to every failure-outcome lexicon entry.
            let env = envelope(vec![
                failure_outcome_no_failure("t", "timed_out"),
                failure_outcome_no_failure("c", "crashed"),
            ]);
            let manifest = RerunManifest::from_envelope(&env);
            assert_eq!(manifest.failed_count, 2);
            for entry in &manifest.entries {
                assert_eq!(
                    entry.failure_message.as_deref(),
                    Some(MISSING_FAILURE_METADATA_SENTINEL),
                );
            }
        }

        #[test]
        fn warn_helper_name_pinned_for_discoverability() {
            // 6) Pin the helper name so a silent rename breaks tests.
            let w = format_rerun_manifest_missing_failure_warn("a", "failed");
            assert!(!w.is_empty());
        }

        #[test]
        fn warn_string_includes_gh3791_issue_tag() {
            // 7) Issue tag anchors the audit trail.
            let w = format_rerun_manifest_missing_failure_warn("a", "failed");
            assert!(w.contains("gh3791"), "missing gh3791 tag: {w}");
            assert!(w.contains("\"a\""), "missing case id: {w}");
            assert!(w.contains("\"failed\""), "missing outcome: {w}");
        }

        #[test]
        fn warn_string_distinct_from_prior_silent_fallback_families() {
            // 8) Sibling-distinctness vs every prior warn family.
            let w = format_rerun_manifest_missing_failure_warn("a", "failed");
            for prior in [
                "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
                "gh3789",
            ] {
                assert!(!w.contains(prior), "must not overlap {prior}: {w}");
            }
        }

        #[test]
        fn mixed_cases_only_missing_failure_carries_sentinel() {
            // 9) Multiple cases — only the `failure: None` one gets the
            // sentinel; manifest ordering preserved; failed_count matches.
            let env = envelope(vec![
                failing_case("a", "well-shaped", "failed"),
                failure_outcome_no_failure("b", "failed"),
                passing_case("c", "ok"),
                failing_case("d", "another-good", "crashed"),
            ]);
            let manifest = RerunManifest::from_envelope(&env);
            assert_eq!(manifest.failed_count, 3);
            assert_eq!(manifest.entries[0].id, "a");
            assert_eq!(manifest.entries[1].id, "b");
            assert_eq!(manifest.entries[2].id, "d");
            assert!(!manifest.entries[0]
                .failure_message
                .as_deref()
                .unwrap()
                .contains(MISSING_FAILURE_METADATA_SENTINEL));
            assert_eq!(
                manifest.entries[1].failure_message.as_deref(),
                Some(MISSING_FAILURE_METADATA_SENTINEL),
            );
            assert!(!manifest.entries[2]
                .failure_message
                .as_deref()
                .unwrap()
                .contains(MISSING_FAILURE_METADATA_SENTINEL));
        }

        #[test]
        fn json_round_trip_preserves_sentinel() {
            // 10) Manifest containing the sentinel must serialize and
            // deserialize cleanly.
            let env = envelope(vec![failure_outcome_no_failure("a", "failed")]);
            let manifest = RerunManifest::from_envelope(&env);
            let body = serde_json::to_string(&manifest).expect("encode");
            assert!(body.contains(MISSING_FAILURE_METADATA_SENTINEL), "{body}");
            let parsed: RerunManifest = serde_json::from_str(&body).expect("decode");
            assert_eq!(parsed.failed_count, 1);
            assert_eq!(
                parsed.entries[0].failure_message.as_deref(),
                Some(MISSING_FAILURE_METADATA_SENTINEL),
            );
        }
    }
}
// CODEGEN-END
