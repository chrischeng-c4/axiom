// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
// CODEGEN-BEGIN
//! Evidence adapter boundary — the public surface for reading e2e/test
//! evidence without depending on UI, browser, or runner internals.
//!
//! Consumers fall into two families and both go through this module:
//!
//! 1. **Local-live mode** (`jet e2e open`, dev review shell) — a long-lived
//!    process that wants to observe the active run. It reads
//!    [`E2eEvidenceBundle`] snapshots and per-step [`E2eEvidenceEvent`]s as
//!    the run progresses.
//! 2. **Read-only report mode** (PM web report, agent post-mortem) — a
//!    short-lived consumer that loads a finished bundle from disk and
//!    renders/serializes it. It uses [`read_bundle_from_file`] and
//!    [`read_events_from_jsonl`] and never touches a Browser, runner, or
//!    DOM-tied UI module.
//!
//! ## Dependency direction
//!
//! ```text
//!   evidence (this module)         <- pure data + reader fns
//!     ^                ^
//!     |                |
//!   PM web report    open-mode UI / dev review
//!     (read-only)     (live + read)
//! ```
//!
//! Crucially, the arrow does **not** go the other way: nothing in this
//! module imports `crate::browser`, `crate::dev_server`, `crate::cdp_driver`,
//! the test_runner worker pool, or any UI rendering code. That invariant is
//! checked by the unit tests below.
//!
//! ## Stability
//!
//! [`EVIDENCE_SCHEMA_VERSION`] and the wire shape of [`E2eEvidenceBundle`]
//! follow the same versioning policy as the rest of the result envelope
//! family (see `result_envelope.rs`).
// @spec #2611

pub mod bundle;
pub mod writer;

use crate::e2e::{events_for_bundle, E2eEvidenceBundle, E2eEvidenceEvent};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub use crate::e2e::{
    E2eArtifactRef as EvidenceArtifactRef, E2eAssertionDetail as EvidenceAssertion,
    E2eCaseEvidence as EvidenceCase, E2eConsoleEntry as EvidenceConsole,
    E2eEvidenceBundle as EvidenceBundle, E2eEvidenceEvent as EvidenceEvent,
    E2eMode as EvidenceMode, E2eNetworkEntry as EvidenceNetwork, E2eProductStep as EvidenceStep,
    E2eSelectorContext as EvidenceSelector, E2eStepContext as EvidenceStepContext,
    E2eSummary as EvidenceSummary, EVIDENCE_SCHEMA_VERSION as SCHEMA_VERSION,
};

/// Load a finished evidence bundle from a `.evidence.json` file written by
/// `jet e2e run` (or any producer that follows the schema).
///
/// Used by the PM read-only report adapter and by agents doing post-mortem
/// analysis. Performs no UI work and opens no browser.
// @spec #2611
pub fn read_bundle_from_file(path: &Path) -> Result<E2eEvidenceBundle> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("reading evidence bundle from {}", path.display()))?;
    serde_json::from_slice::<E2eEvidenceBundle>(&bytes)
        .with_context(|| format!("parsing evidence bundle at {}", path.display()))
}

/// Load the per-event JSONL stream that accompanies an evidence bundle.
///
/// Each line is a [`E2eEvidenceEvent`]. Empty lines are skipped silently
/// so the reader stays tolerant of partially-written streams (the
/// open-mode adapter polls while a run is in progress). Unparseable
/// lines are also skipped, but emit a `tracing::warn!` so an actual
/// schema drift / mid-write truncation / encoding error surfaces in
/// the logs instead of silently shrinking the event set the HTML
/// reporter, PM agent, and conductor render from (GH #3255).
// @spec #2611
pub fn read_events_from_jsonl(path: &Path) -> Result<Vec<E2eEvidenceEvent>> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading evidence events from {}", path.display()))?;
    let mut events = Vec::new();
    for (idx, raw_line) in text.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        match serde_json::from_str::<E2eEvidenceEvent>(line) {
            Ok(event) => events.push(event),
            Err(err) => {
                tracing::warn!(
                    target: "jet::evidence",
                    path = %path.display(),
                    line_idx = idx,
                    error = %err,
                    "GH #3255 dropping malformed evidence event; \
                     reporter and PM agent will see a truncated event stream"
                );
            }
        }
    }
    Ok(events)
}

/// Reconstruct the canonical event stream from a bundle.
///
/// Re-exported from `crate::e2e` so PM report adapters that only have the
/// bundle (no on-disk JSONL) can still drive timeline-style UIs from a pure
/// data source.
// @spec #2611
pub fn events_for(bundle: &E2eEvidenceBundle) -> Vec<E2eEvidenceEvent> {
    events_for_bundle(bundle)
}

/// Read-only report adapter for PM / agent post-mortem consumers.
///
/// Loads an evidence bundle and exposes a narrow read-only surface:
/// `summary()`, `cases()`, `failures()`, `artifacts()`, `events()`. By
/// design the adapter has **no** control-plane methods — no pause, no
/// next, no replay, no highlight, no fetch_live. Anything driving the
/// active run lives on the open-mode side; this adapter never touches
/// it, so a stale bundle on a developer's disk is enough.
///
/// Artifact paths resolve against a caller-supplied `evidence_root`
/// (typically the directory the `.evidence.json` was loaded from).
/// Missing artifacts surface as
/// [`ArtifactAvailability::Missing`] rather than I/O errors.
// @spec #2619
pub struct ReportAdapter {
    bundle: E2eEvidenceBundle,
    events: Vec<E2eEvidenceEvent>,
    evidence_root: PathBuf,
}

/// Outcome of resolving an artifact reference against the evidence root.
// @spec #2619
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactAvailability {
    /// The artifact exists on disk; `absolute` is the resolved path.
    Available { absolute: PathBuf },
    /// The artifact is referenced by the bundle but not present on disk
    /// (or the relative path could not be joined). The UI should render
    /// this as "unavailable" rather than crashing.
    Missing { reason: String },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
impl ReportAdapter {
    /// Build a read-only adapter from an in-memory bundle.
    ///
    /// `evidence_root` is the directory artifact paths are resolved
    /// against (e.g. the parent of the `.evidence.json` file).
    // @spec #2619
    pub fn from_bundle(bundle: E2eEvidenceBundle, evidence_root: PathBuf) -> Self {
        let events = events_for_bundle(&bundle);
        Self {
            bundle,
            events,
            evidence_root,
        }
    }

    /// Build a read-only adapter by loading an evidence bundle from disk.
    ///
    /// The accompanying `.events.jsonl` is loaded if present; otherwise
    /// the event stream is reconstructed from the bundle. Either way the
    /// adapter exposes a coherent timeline.
    // @spec #2619
    pub fn from_file(path: &Path) -> Result<Self> {
        let bundle = read_bundle_from_file(path)?;
        let evidence_root = resolve_evidence_root_or_warn(path);
        let jsonl_candidates: Vec<PathBuf> = {
            let stem = path
                .file_name()
                .and_then(|n| n.to_str())
                .and_then(|n| n.strip_suffix(".evidence.json"));
            match stem {
                Some(stem) => vec![evidence_root.join(format!("{stem}.events.jsonl"))],
                None => Vec::new(),
            }
        };
        // GH #3285 — when the companion .events.jsonl exists but
        // `read_events_from_jsonl` fails (permission denied, truncated
        // header, EIO), the prior `.ok()` swallowed the error and
        // transparently fell back to `events_for_bundle(&bundle)` — a
        // strictly lossy reconstruction. PM agents and HTML reporters
        // saw a different event stream than the on-disk JSONL would
        // have produced, with no breadcrumb. Surface read failures
        // via tracing::warn so the operator knows the reconstruction
        // is a fallback, not the recorded truth.
        let events = match jsonl_candidates.iter().find(|p| p.exists()) {
            None => events_for_bundle(&bundle),
            Some(jsonl_path) => match read_events_from_jsonl(jsonl_path) {
                Ok(events) => events,
                Err(err) => {
                    tracing::warn!(
                        target: "jet::evidence",
                        path = %jsonl_path.display(),
                        error = %err,
                        "GH #3285 failed to read evidence events JSONL; \
                         falling back to bundle-only reconstruction — \
                         reporter and PM agent will see a synthesized event stream"
                    );
                    events_for_bundle(&bundle)
                }
            },
        };
        Ok(Self {
            bundle,
            events,
            evidence_root,
        })
    }

    /// Summary block (counts + exit code).
    pub fn summary(&self) -> &EvidenceSummary {
        &self.bundle.summary
    }

    /// All cases in the bundle, in original order.
    pub fn cases(&self) -> &[EvidenceCase] {
        &self.bundle.cases
    }

    /// Cases whose outcome is not "passed" or "skipped".
    pub fn failures(&self) -> Vec<&EvidenceCase> {
        self.bundle
            .cases
            .iter()
            .filter(|c| c.outcome != "passed" && c.outcome != "skipped")
            .collect()
    }

    /// Top-level artifacts attached to the bundle (traces, screenshots).
    pub fn artifacts(&self) -> &[EvidenceArtifactRef] {
        &self.bundle.artifacts
    }

    /// Event timeline (reconstructed if no jsonl was loaded).
    pub fn events(&self) -> &[EvidenceEvent] {
        &self.events
    }

    /// The bundle this adapter was built from — exposed for consumers
    /// that want to render fields directly (mode, run_id, timestamps).
    pub fn bundle(&self) -> &EvidenceBundle {
        &self.bundle
    }

    /// Resolve an artifact path against the evidence root and report
    /// whether it actually exists on disk. The UI is expected to render
    /// `Missing` cases as "screenshot/trace unavailable", not to error.
    // @spec #2619
    pub fn resolve_artifact(&self, artifact: &EvidenceArtifactRef) -> ArtifactAvailability {
        let candidate = if artifact.path.is_absolute() {
            artifact.path.clone()
        } else {
            self.evidence_root.join(&artifact.path)
        };
        if candidate.exists() {
            ArtifactAvailability::Available {
                absolute: candidate,
            }
        } else {
            ArtifactAvailability::Missing {
                reason: format!("{} not present on disk", candidate.display()),
            }
        }
    }
}

/// GH #3797 — fallback evidence root used when the bundle path has no
/// parent component (e.g. the root `/` on Unix, or a pathological empty
/// `Path`). Kept as a named constant so call sites and tests pin the
/// same value.
pub(crate) const EVIDENCE_ROOT_NO_PARENT_FALLBACK: &str = ".";

/// GH #3797 — formatted warn shown when an evidence bundle path has no
/// `parent()` component. The prior code silently rewrote the root onto
/// ".", so two distinct parentless bundles shared the same lookup
/// directory and could pick up the wrong companion `.events.jsonl`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
pub(crate) fn format_evidence_no_parent_warn(path: &Path) -> String {
    format!(
        "gh3797: evidence bundle path has no parent for path={:?}; \
         falling back to {:?} — companion .events.jsonl lookup may \
         resolve against an unrelated cwd",
        path, EVIDENCE_ROOT_NO_PARENT_FALLBACK
    )
}

/// GH #3797 — resolve the evidence root for a bundle path.
///
/// `parent()` returns `Some(p)` for every well-formed relative or
/// absolute path that has a parent component; that arm stays silent.
/// The `None` arm is reached only by pathological inputs (root `/`,
/// empty path); surface a `tracing::warn!` so operators can see the
/// silent rewrite to "." that previously masked the issue.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
pub(crate) fn resolve_evidence_root_or_warn(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => {
            tracing::warn!(
                target: "jet::evidence",
                path = %path.display(),
                fallback = EVIDENCE_ROOT_NO_PARENT_FALLBACK,
                "{}",
                format_evidence_no_parent_warn(path)
            );
            PathBuf::from(EVIDENCE_ROOT_NO_PARENT_FALLBACK)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e::{
        E2eCaseEvidence, E2eEvidenceBundle, E2eEvidenceEvent, E2eMode, E2eProductStep,
        E2eStepContext, E2eSummary,
    };
    use std::path::PathBuf;

    fn sample_bundle() -> E2eEvidenceBundle {
        E2eEvidenceBundle {
            schema_version: SCHEMA_VERSION.to_string(),
            mode: E2eMode::Run,
            run_id: "test-run-1".to_string(),
            started_at_ms: 1_700_000_000_000,
            finished_at_ms: 1_700_000_001_500,
            summary: E2eSummary {
                passed: 1,
                failed: 0,
                skipped: 0,
                duration_ms: 1_500,
                exit_code: 0,
            },
            cases: vec![E2eCaseEvidence {
                id: "case-1".to_string(),
                title: "happy path".to_string(),
                file: PathBuf::from("e2e/happy.spec.ts"),
                outcome: "passed".to_string(),
                duration_ms: 1_500,
                steps: vec![E2eProductStep {
                    id: "step-1".to_string(),
                    title: "click button".to_string(),
                    status: "passed".to_string(),
                    duration_ms: 100,
                    assertion: None,
                    context: E2eStepContext::default(),
                }],
            }],
            artifacts: vec![],
            serve_session: None,
            browser_sessions: vec![],
            open_control: None,
        }
    }

    #[test]
    fn bundle_round_trips_through_disk_without_ui() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("run.evidence.json");
        let bundle = sample_bundle();
        std::fs::write(&path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();
        let loaded = read_bundle_from_file(&path).expect("loaded");
        assert_eq!(loaded.run_id, bundle.run_id);
        assert_eq!(loaded.cases.len(), 1);
        assert_eq!(loaded.cases[0].steps[0].title, "click button");
    }

    #[test]
    fn events_jsonl_tolerates_blank_and_unparseable_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("events.jsonl");
        let valid = E2eEvidenceEvent::RunStarted {
            run_id: "r".to_string(),
            mode: E2eMode::Run,
            ts_ms: 1,
        };
        let body = format!(
            "{}\n\nnot json\n{}\n",
            serde_json::to_string(&valid).unwrap(),
            serde_json::to_string(&valid).unwrap(),
        );
        std::fs::write(&path, body).unwrap();
        let events = read_events_from_jsonl(&path).expect("events");
        assert_eq!(events.len(), 2);
    }

    /// GH #3255 — Happy path: every line parses and the returned vec
    /// preserves order. Pins that the new `tracing::warn!` arm did not
    /// regress the success path.
    #[test]
    fn events_jsonl_returns_all_valid_events_in_order() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("events.jsonl");
        let e1 = E2eEvidenceEvent::RunStarted {
            run_id: "r".to_string(),
            mode: E2eMode::Run,
            ts_ms: 1,
        };
        let e2 = E2eEvidenceEvent::RunFinished {
            run_id: "r".to_string(),
            ts_ms: 9,
            passed: 1,
            failed: 0,
            skipped: 0,
            exit_code: 0,
        };
        let body = format!(
            "{}\n{}\n",
            serde_json::to_string(&e1).unwrap(),
            serde_json::to_string(&e2).unwrap(),
        );
        std::fs::write(&path, body).unwrap();
        let events = read_events_from_jsonl(&path).expect("events");
        assert_eq!(events.len(), 2);
        assert!(matches!(events[0], E2eEvidenceEvent::RunStarted { .. }));
        assert!(matches!(events[1], E2eEvidenceEvent::RunFinished { .. }));
    }

    /// GH #3255 — A malformed JSONL line in the middle of an otherwise
    /// valid stream used to silently drop without any diagnostic. The
    /// reader still returns the surrounding valid events (so the
    /// reporter renders something), but the new `tracing::warn!` arm
    /// surfaces the underlying integrity bug.
    ///
    /// Asserts: 5 lines (3 valid + 1 truncated + 1 wrong-schema) → 3
    /// events back.
    #[test]
    fn events_jsonl_preserves_valid_events_around_malformed_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("events.jsonl");
        let valid = E2eEvidenceEvent::RunStarted {
            run_id: "r".to_string(),
            mode: E2eMode::Run,
            ts_ms: 1,
        };
        let v = serde_json::to_string(&valid).unwrap();
        // Mid-write truncation, then valid, then a JSON object that
        // doesn't match the E2eEvidenceEvent schema, then two more
        // valid.
        let body = format!("{v}\n{{ \"type\": \"RunStart\"\n{v}\n{{\"unknown\":\"shape\"}}\n{v}\n");
        std::fs::write(&path, body).unwrap();

        let events = read_events_from_jsonl(&path).expect("events");
        assert_eq!(
            events.len(),
            3,
            "expected 3 valid events around 2 malformed lines, got {}",
            events.len()
        );
    }

    /// GH #3255 — File-level read failure must still bubble up as
    /// `Err` (no silent empty-vec fallback). Pins the function's
    /// outer contract.
    #[test]
    fn events_jsonl_errors_when_file_missing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("does-not-exist.jsonl");
        assert!(read_events_from_jsonl(&path).is_err());
    }

    #[test]
    fn events_for_synthesizes_run_started_and_finished() {
        let bundle = sample_bundle();
        let events = events_for(&bundle);
        assert!(events.len() >= 2);
        matches!(events.first(), Some(E2eEvidenceEvent::RunStarted { .. }));
        matches!(events.last(), Some(E2eEvidenceEvent::RunFinished { .. }));
    }

    /// Guardrail test: scan the non-test, non-comment lines of this module's
    /// source for `use` imports of UI / browser / live-runner module roots.
    /// If a refactor reaches for one of those, this test fires before the
    /// boundary regresses. The forbidden list is built from runtime
    /// concatenation so the test source itself does not contain the
    /// sentinel strings literally.
    #[test]
    fn evidence_module_does_not_import_ui_or_runner() {
        let src = include_str!("mod.rs");
        // Only scan the module body up to the `#[cfg(test)]` boundary so
        // the test sources themselves aren't searched.
        let body_end = src.find("#[cfg(test)]").unwrap_or(src.len());
        let body = &src[..body_end];
        let code: String = body
            .lines()
            .filter(|line| {
                let trimmed = line.trim_start();
                !trimmed.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");
        let c = "crate::";
        let forbidden: Vec<String> = vec![
            format!("{c}browser"),
            format!("{c}cdp_driver"),
            format!("{c}dev_server"),
            format!("{c}test_runner::worker"),
            format!("{c}test_runner::worker_pool"),
            "axum:".to_string() + ":",
            "tokio:".to_string() + ":net",
        ];
        for needle in &forbidden {
            assert!(
                !code.contains(needle),
                "evidence adapter must not depend on {needle}",
            );
        }
    }

    fn sample_bundle_with_artifacts(failure_diff: Option<&str>) -> E2eEvidenceBundle {
        use crate::e2e::{E2eArtifactRef, E2eAssertionDetail};
        let mut bundle = sample_bundle();
        bundle.cases.push(E2eCaseEvidence {
            id: "case-2".to_string(),
            title: "promotion".to_string(),
            file: PathBuf::from("e2e/promote.spec.ts"),
            outcome: "failed".to_string(),
            duration_ms: 300,
            steps: vec![E2eProductStep {
                id: "step-1".to_string(),
                title: "publish artifact".to_string(),
                status: "failed".to_string(),
                duration_ms: 200,
                assertion: failure_diff.map(|d| E2eAssertionDetail {
                    message: "expected shipped".to_string(),
                    stack: None,
                    diff: Some(d.to_string()),
                }),
                context: E2eStepContext::default(),
            }],
        });
        bundle.artifacts.push(E2eArtifactRef {
            kind: "screenshot".to_string(),
            path: PathBuf::from("screenshots/promotion.png"),
            label: Some("failure capture".to_string()),
        });
        bundle.summary.failed = 1;
        bundle.summary.exit_code = 1;
        bundle
    }

    #[test]
    fn report_adapter_exposes_summary_cases_failures_and_artifacts() {
        // @spec #2619
        let dir = tempfile::tempdir().unwrap();
        let bundle = sample_bundle_with_artifacts(Some("- shipped\n+ reviewing"));
        let adapter = ReportAdapter::from_bundle(bundle, dir.path().to_path_buf());
        assert_eq!(adapter.summary().failed, 1);
        assert_eq!(adapter.cases().len(), 2);
        let failures = adapter.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].id, "case-2");
        assert_eq!(adapter.artifacts().len(), 1);
        assert!(!adapter.events().is_empty());
    }

    #[test]
    fn report_adapter_resolves_present_artifact_to_absolute_path() {
        // @spec #2619
        let dir = tempfile::tempdir().unwrap();
        let screenshot_dir = dir.path().join("screenshots");
        std::fs::create_dir_all(&screenshot_dir).unwrap();
        let screenshot_path = screenshot_dir.join("promotion.png");
        std::fs::write(&screenshot_path, b"fake-png").unwrap();

        let bundle = sample_bundle_with_artifacts(None);
        let adapter = ReportAdapter::from_bundle(bundle, dir.path().to_path_buf());
        let availability = adapter.resolve_artifact(&adapter.artifacts()[0]);
        match availability {
            ArtifactAvailability::Available { absolute } => {
                assert!(absolute.ends_with("screenshots/promotion.png"));
                assert!(absolute.exists());
            }
            other => panic!("expected Available, got {other:?}"),
        }
    }

    #[test]
    fn report_adapter_degrades_missing_artifact_to_unavailable() {
        // @spec #2619 — missing screenshots/traces render as unavailable.
        let dir = tempfile::tempdir().unwrap();
        let bundle = sample_bundle_with_artifacts(None);
        let adapter = ReportAdapter::from_bundle(bundle, dir.path().to_path_buf());
        let availability = adapter.resolve_artifact(&adapter.artifacts()[0]);
        match availability {
            ArtifactAvailability::Missing { reason } => {
                assert!(reason.contains("not present on disk"));
            }
            other => panic!("expected Missing, got {other:?}"),
        }
    }

    #[test]
    fn report_adapter_from_file_loads_events_jsonl_when_present() {
        // @spec #2619
        let dir = tempfile::tempdir().unwrap();
        let bundle_path = dir.path().join("run.evidence.json");
        let events_path = dir.path().join("run.events.jsonl");
        let bundle = sample_bundle();
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();

        let custom_event = E2eEvidenceEvent::RunStarted {
            run_id: "custom".to_string(),
            mode: E2eMode::Run,
            ts_ms: 42,
        };
        std::fs::write(
            &events_path,
            serde_json::to_string(&custom_event).unwrap() + "\n",
        )
        .unwrap();

        let adapter = ReportAdapter::from_file(&bundle_path).expect("load");
        let events = adapter.events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            E2eEvidenceEvent::RunStarted { run_id, .. } => assert_eq!(run_id, "custom"),
            other => panic!("expected RunStarted, got {other:?}"),
        }
    }

    #[test]
    fn report_adapter_from_file_synthesizes_events_when_jsonl_missing() {
        // @spec #2619
        let dir = tempfile::tempdir().unwrap();
        let bundle_path = dir.path().join("run.evidence.json");
        let bundle = sample_bundle();
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();
        let adapter = ReportAdapter::from_file(&bundle_path).expect("load");
        let events = adapter.events();
        assert!(!events.is_empty());
        assert!(matches!(
            events.first(),
            Some(E2eEvidenceEvent::RunStarted { .. })
        ));
    }

    /// GH #3285 — when the companion `.events.jsonl` is present on
    /// disk but unreadable (permission denied), the prior `.ok()`
    /// silently fell back to bundle-only reconstruction with no
    /// breadcrumb. The fallback must still happen so a corrupt file
    /// doesn't prevent the report from loading, but the operator must
    /// see a warn line linking the synthesized events back to the
    /// JSONL that failed.
    #[cfg(unix)]
    #[test]
    fn report_adapter_from_file_falls_back_when_jsonl_unreadable() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().unwrap();
        let bundle_path = dir.path().join("run.evidence.json");
        let events_path = dir.path().join("run.events.jsonl");
        let bundle = sample_bundle();
        std::fs::write(&bundle_path, serde_json::to_vec_pretty(&bundle).unwrap()).unwrap();

        // A real JSONL with a distinctive run_id — if the reader silently
        // succeeded we'd see it in the adapter's events.
        let distinctive = E2eEvidenceEvent::RunStarted {
            run_id: "would-be-loaded-from-jsonl".to_string(),
            mode: E2eMode::Run,
            ts_ms: 7,
        };
        std::fs::write(
            &events_path,
            serde_json::to_string(&distinctive).unwrap() + "\n",
        )
        .unwrap();
        std::fs::set_permissions(&events_path, std::fs::Permissions::from_mode(0o000)).unwrap();

        let adapter = ReportAdapter::from_file(&bundle_path).expect("load must succeed");

        // Restore perms so tempdir cleanup works.
        let _ = std::fs::set_permissions(&events_path, std::fs::Permissions::from_mode(0o644));

        // Bundle reconstruction always yields RunStarted with the bundle's
        // run_id, never the JSONL's. If `.ok()` had silently succeeded
        // we'd see "would-be-loaded-from-jsonl" instead.
        let events = adapter.events();
        assert!(
            !events.is_empty(),
            "fallback reconstruction must still produce events"
        );
        match events.first() {
            Some(E2eEvidenceEvent::RunStarted { run_id, .. }) => {
                assert_eq!(
                    run_id, "test-run-1",
                    "events must come from bundle reconstruction, not the unreadable JSONL"
                );
            }
            other => panic!("expected first event to be RunStarted from bundle, got {other:?}"),
        }
    }

    /// Guardrail: the read-only adapter must NOT expose any control-plane
    /// methods. Anything pause/replay/next/highlight related is open-mode
    /// only and stays out of this surface. Scanning the source ensures a
    /// drive-by refactor cannot quietly add one.
    /// @spec #2619
    #[test]
    fn report_adapter_exposes_no_control_plane_methods() {
        let src = include_str!("mod.rs");
        let body_end = src.find("#[cfg(test)]").unwrap_or(src.len());
        let body = &src[..body_end];
        let pa = "p".to_string() + "ause";
        let re = "re".to_string() + "play";
        let nx = "ne".to_string() + "xt_case";
        let hi = "high".to_string() + "light";
        let live = "wait_for_live".to_string();
        let forbidden = [pa, re, nx, hi, live];
        for needle in &forbidden {
            assert!(
                !body.contains(&format!("fn {needle}")),
                "ReportAdapter must not expose fn {needle} — that lives in open mode"
            );
        }
    }
}

#[cfg(test)]
mod gh3797_evidence_root_no_parent_warn_tests {
    use super::*;
    use std::path::{Path, PathBuf};

    #[test]
    fn path_with_parent_stays_silent_and_returns_parent() {
        let p = Path::new("/runs/specs/login.evidence.json");
        let root = resolve_evidence_root_or_warn(p);
        assert_eq!(root, PathBuf::from("/runs/specs"));
    }

    #[test]
    fn root_path_falls_back_to_dot() {
        let p = Path::new("/");
        let root = resolve_evidence_root_or_warn(p);
        assert_eq!(root, PathBuf::from(EVIDENCE_ROOT_NO_PARENT_FALLBACK));
        assert_eq!(root, PathBuf::from("."));
    }

    #[test]
    fn relative_filename_returns_empty_parent_not_fallback() {
        // GH #3797 nuance: `Path::new("evidence.json").parent()` returns
        // Some("") on Unix, NOT None. So the warn does NOT fire on
        // bare relative filenames — that case stays silent.
        let p = Path::new("evidence.json");
        let root = resolve_evidence_root_or_warn(p);
        assert_eq!(root, PathBuf::from(""));
    }

    #[test]
    fn warn_helpers_pinned_for_discoverability() {
        let src = include_str!("mod.rs");
        assert!(src.contains("fn format_evidence_no_parent_warn"));
        assert!(src.contains("fn resolve_evidence_root_or_warn"));
        assert!(src.contains("EVIDENCE_ROOT_NO_PARENT_FALLBACK"));
    }

    #[test]
    fn each_warn_string_carries_gh3797_tag() {
        let p = Path::new("/");
        let s = format_evidence_no_parent_warn(p);
        assert!(s.starts_with("gh3797:"), "missing gh3797 tag: {s:?}");
        assert!(s.contains("evidence bundle path has no parent"));
    }

    #[test]
    fn warn_distinct_from_prior_silent_fallback_families() {
        let s = format_evidence_no_parent_warn(Path::new("/"));
        for tag in [
            "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
            "gh3789", "gh3791", "gh3793", "gh3795",
        ] {
            assert!(!s.contains(tag), "gh3797 warn must not carry {tag}: {s:?}");
        }
    }

    #[test]
    fn fallback_constant_matches_legacy_string() {
        assert_eq!(EVIDENCE_ROOT_NO_PARENT_FALLBACK, ".");
    }

    #[test]
    fn warn_names_companion_jsonl_consequence() {
        let s = format_evidence_no_parent_warn(Path::new("/"));
        assert!(
            s.contains(".events.jsonl"),
            "warn should explain the silent fallback consequence: {s:?}"
        );
    }

    #[test]
    fn two_distinct_root_paths_still_share_fallback_root() {
        // Pins the documented behaviour: the fallback IS "." for any
        // parentless path. The warn is the only thing that distinguishes
        // them — operators get a breadcrumb instead of silent collision.
        let r1 = resolve_evidence_root_or_warn(Path::new("/"));
        let r2 = resolve_evidence_root_or_warn(Path::new("/"));
        assert_eq!(r1, r2);
        assert_eq!(r1, PathBuf::from("."));
    }

    #[test]
    fn happy_path_resolve_joins_against_parent_dir() {
        let bundle_path = Path::new("/runs/2026-05-21/run.evidence.json");
        let root = resolve_evidence_root_or_warn(bundle_path);
        let companion = root.join("run.events.jsonl");
        assert_eq!(
            companion,
            PathBuf::from("/runs/2026-05-21/run.events.jsonl")
        );
    }
}
// CODEGEN-END
