// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Failed-step DOM snapshot capture policy (#2882).
//!
//! When an e2e step fails, the screenshot tells you what the user saw
//! but not what the DOM actually looked like. This module adds the
//! sibling artifact: a serialized DOM snapshot (outer HTML of the
//! root element) captured at failure time, registered through the
//! existing `BundleManifest` plumbing so PM/dev reports can resolve
//! the file alongside the screenshot.
//!
//! Continuous DOM trace capture (multi-step timeline) is out of scope
//! — that lives in #2884 (trace) and the eventual full trace viewer.
//! This slice covers a single snapshot per failed step.
//!
//! Mirrors the policy/register split established by
//! [`crate::e2e_screenshots`] (#2726).
//!
//! Snapshots are stored as UTF-8 HTML so PM viewers can render them
//! in an iframe or open them in a tab; we tag them with content type
//! `text/html; charset=utf-8` to avoid sniffer guesses.

use crate::e2e::E2eArtifactRef;
use crate::e2e_screenshots::StepOutcome;
use crate::evidence_bundle::{BundleArtifact, BundleManifest};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Stable kind label registered through the bundle manifest for
/// step-scoped DOM snapshots.
pub const DOM_SNAPSHOT_ARTIFACT_KIND: &str = "step-dom-snapshot";

/// Configured capture mode. Same shape as
/// [`crate::e2e_screenshots::ScreenshotMode`] so a single config knob
/// can fan out to both — but it's a separate type so DOM snapshots can
/// be turned on/off independently when bundle size matters.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DomSnapshotMode {
    Off,
    #[default]
    OnFailure,
    Always,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl DomSnapshotMode {
    /// True when a step with this outcome should produce a snapshot.
    pub fn should_capture(self, outcome: StepOutcome) -> bool {
        match self {
            Self::Off => false,
            Self::OnFailure => matches!(outcome, StepOutcome::Failed | StepOutcome::TimedOut),
            Self::Always => true,
        }
    }
}

/// Build an [`E2eArtifactRef`] for a captured DOM snapshot and
/// register the matching [`BundleArtifact`] in the manifest so PM
/// reports can resolve the path through the bundle.
///
/// `id` should be stable across reruns of the same case+step. `path`
/// is relative to the bundle root and names a `.html` file holding
/// the serialized DOM.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn register_step_dom_snapshot(
    manifest: &mut BundleManifest,
    id: impl Into<String>,
    path: impl Into<PathBuf>,
    label: Option<String>,
) -> E2eArtifactRef {
    let id = id.into();
    let path = path.into();
    manifest.artifacts.push(BundleArtifact {
        id: id.clone(),
        kind: DOM_SNAPSHOT_ARTIFACT_KIND.to_string(),
        path: path.clone(),
        content_type: Some("text/html; charset=utf-8".to_string()),
    });
    E2eArtifactRef {
        kind: DOM_SNAPSHOT_ARTIFACT_KIND.to_string(),
        path,
        label,
    }
}

/// Predicate paired with [`register_step_dom_snapshot`]. Returns the
/// registered ref when the mode + outcome say capture should happen;
/// otherwise `None`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn maybe_register_failed_step_dom_snapshot(
    manifest: &mut BundleManifest,
    mode: DomSnapshotMode,
    outcome: StepOutcome,
    id: impl Into<String>,
    path: impl AsRef<Path>,
    label: Option<String>,
) -> Option<E2eArtifactRef> {
    if !mode.should_capture(outcome) {
        return None;
    }
    Some(register_step_dom_snapshot(
        manifest,
        id,
        path.as_ref().to_path_buf(),
        label,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};

    fn empty_manifest() -> BundleManifest {
        BundleManifest::new(
            "run-dom",
            BundleCommand::E2e,
            "jet",
            "cafef00d",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.51".into(),
                ci: None,
                node_version: None,
            },
        )
    }

    #[test]
    fn on_failure_mode_captures_only_failed_steps() {
        let mode = DomSnapshotMode::OnFailure;
        assert!(mode.should_capture(StepOutcome::Failed));
        assert!(mode.should_capture(StepOutcome::TimedOut));
        assert!(!mode.should_capture(StepOutcome::Passed));
        assert!(!mode.should_capture(StepOutcome::Skipped));
    }

    #[test]
    fn off_mode_captures_nothing() {
        let mode = DomSnapshotMode::Off;
        for o in [
            StepOutcome::Passed,
            StepOutcome::Failed,
            StepOutcome::TimedOut,
            StepOutcome::Skipped,
        ] {
            assert!(!mode.should_capture(o));
        }
    }

    #[test]
    fn always_mode_captures_every_outcome() {
        let mode = DomSnapshotMode::Always;
        assert!(mode.should_capture(StepOutcome::Passed));
        assert!(mode.should_capture(StepOutcome::Failed));
        assert!(mode.should_capture(StepOutcome::Skipped));
    }

    #[test]
    fn failed_step_links_to_dom_snapshot_in_bundle_manifest() {
        // Stop condition (#2882): one failed fixture links to a DOM
        // snapshot file resolvable through the manifest.
        let mut manifest = empty_manifest();
        let artifact_ref = register_step_dom_snapshot(
            &mut manifest,
            "case-1-step-3-dom",
            "artifacts/case-1-step-3.dom.html",
            Some("confirm order failed".into()),
        );
        assert_eq!(artifact_ref.kind, DOM_SNAPSHOT_ARTIFACT_KIND);
        assert_eq!(
            artifact_ref.path,
            PathBuf::from("artifacts/case-1-step-3.dom.html")
        );

        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-1-step-3-dom")
            .expect("registered in manifest");
        assert_eq!(entry.kind, DOM_SNAPSHOT_ARTIFACT_KIND);
        assert_eq!(
            entry.content_type.as_deref(),
            Some("text/html; charset=utf-8")
        );
    }

    #[test]
    fn maybe_register_skips_when_policy_says_no() {
        let mut manifest = empty_manifest();
        let res = maybe_register_failed_step_dom_snapshot(
            &mut manifest,
            DomSnapshotMode::OnFailure,
            StepOutcome::Passed,
            "case-x-step-1",
            "artifacts/should-not-exist.html",
            None,
        );
        assert!(res.is_none());
        assert!(manifest.artifacts.is_empty());
    }

    #[test]
    fn maybe_register_attaches_artifact_when_step_fails() {
        let mut manifest = empty_manifest();
        let res = maybe_register_failed_step_dom_snapshot(
            &mut manifest,
            DomSnapshotMode::OnFailure,
            StepOutcome::Failed,
            "case-1-step-2-dom",
            "artifacts/case-1-step-2.dom.html",
            Some("post-click DOM".into()),
        );
        let r = res.expect("expected an artifact ref");
        assert_eq!(r.kind, DOM_SNAPSHOT_ARTIFACT_KIND);
        assert_eq!(manifest.artifacts.len(), 1);
    }

    #[test]
    fn dom_snapshot_artifact_kind_is_distinct_from_screenshot() {
        // Pin the two artifacts apart so PM viewers can tell them
        // from the same manifest.
        assert_ne!(
            DOM_SNAPSHOT_ARTIFACT_KIND,
            crate::e2e_screenshots::SCREENSHOT_ARTIFACT_KIND
        );
    }
}
// CODEGEN-END
