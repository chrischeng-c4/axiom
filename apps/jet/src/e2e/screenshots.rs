// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Failed-step screenshot capture policy (#2726).
//!
//! Step-scoped artifact capture knobs ride on the existing
//! [`E2eArtifactRef`] / [`BundleManifest`] plumbing. This module adds
//! the policy layer that decides *when* a screenshot is captured and
//! the helper that registers the artifact through the bundle so PM
//! reports and rerun manifests can resolve it.
//!
//! Three modes:
//!
//! - `Off` — never capture; useful for fast CI smoke runs.
//! - `OnFailure` — capture only for failed steps (default).
//! - `Always` — capture for every step; useful for triaging flaky
//!   product flows.
//!
//! DOM / trace / video artifacts are out of scope for this slice
//! (split into #2882 / #2883 / #2884).

use crate::e2e::E2eArtifactRef;
use crate::evidence_bundle::{BundleArtifact, BundleManifest};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Stable kind label registered through the bundle manifest for
/// step-scoped screenshots.
pub const SCREENSHOT_ARTIFACT_KIND: &str = "step-screenshot";

/// Step outcome the runner reports when deciding whether to capture.
/// Only `Failed` and `TimedOut` count as failures for this policy.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepOutcome {
    Passed,
    Failed,
    TimedOut,
    Skipped,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StepOutcome {
    fn is_failure(self) -> bool {
        matches!(self, Self::Failed | Self::TimedOut)
    }
}

/// Configured capture mode.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScreenshotMode {
    Off,
    #[default]
    OnFailure,
    Always,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl ScreenshotMode {
    /// True when a step with this outcome should produce a screenshot.
    pub fn should_capture(self, outcome: StepOutcome) -> bool {
        match self {
            Self::Off => false,
            Self::OnFailure => outcome.is_failure(),
            Self::Always => true,
        }
    }
}

/// Build an [`E2eArtifactRef`] for a captured screenshot and register
/// the matching [`BundleArtifact`] in the manifest so PM reports
/// can resolve the path through the bundle.
///
/// `id` should be stable across reruns of the same case+step so
/// downstream tooling can dedupe. `path` is relative to the bundle
/// root.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn register_step_screenshot(
    manifest: &mut BundleManifest,
    id: impl Into<String>,
    path: impl Into<PathBuf>,
    label: Option<String>,
) -> E2eArtifactRef {
    let id = id.into();
    let path = path.into();
    manifest.artifacts.push(BundleArtifact {
        id: id.clone(),
        kind: SCREENSHOT_ARTIFACT_KIND.to_string(),
        path: path.clone(),
        content_type: Some("image/png".to_string()),
    });
    E2eArtifactRef {
        kind: SCREENSHOT_ARTIFACT_KIND.to_string(),
        path,
        label,
    }
}

/// Convenience predicate paired with [`register_step_screenshot`].
/// Returns the registered ref when the mode + outcome say capture
/// should happen; otherwise `None`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn maybe_register_failed_step_screenshot(
    manifest: &mut BundleManifest,
    mode: ScreenshotMode,
    outcome: StepOutcome,
    id: impl Into<String>,
    path: impl AsRef<Path>,
    label: Option<String>,
) -> Option<E2eArtifactRef> {
    if !mode.should_capture(outcome) {
        return None;
    }
    Some(register_step_screenshot(
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
            "run-shot",
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
        let mode = ScreenshotMode::OnFailure;
        assert!(mode.should_capture(StepOutcome::Failed));
        assert!(mode.should_capture(StepOutcome::TimedOut));
        assert!(!mode.should_capture(StepOutcome::Passed));
        assert!(!mode.should_capture(StepOutcome::Skipped));
    }

    #[test]
    fn off_mode_captures_nothing() {
        let mode = ScreenshotMode::Off;
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
    fn always_mode_captures_passed_and_failed() {
        let mode = ScreenshotMode::Always;
        assert!(mode.should_capture(StepOutcome::Passed));
        assert!(mode.should_capture(StepOutcome::Failed));
        assert!(mode.should_capture(StepOutcome::Skipped));
    }

    #[test]
    fn register_step_screenshot_attaches_to_bundle_manifest() {
        // Stop condition (#2726): a failed fixture links to one
        // screenshot artifact registered through the bundle manifest.
        let mut manifest = empty_manifest();
        let artifact_ref = register_step_screenshot(
            &mut manifest,
            "case-1-step-3",
            "artifacts/case-1-step-3.png",
            Some("confirm order failed".into()),
        );
        assert_eq!(artifact_ref.kind, SCREENSHOT_ARTIFACT_KIND);
        assert_eq!(
            artifact_ref.path,
            PathBuf::from("artifacts/case-1-step-3.png")
        );
        assert_eq!(artifact_ref.label.as_deref(), Some("confirm order failed"));

        // Bundle manifest now resolves the id back to the same path
        // so PM report rendering finds the file.
        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-1-step-3")
            .expect("registered in manifest");
        assert_eq!(entry.kind, SCREENSHOT_ARTIFACT_KIND);
        assert_eq!(entry.path, PathBuf::from("artifacts/case-1-step-3.png"));
        assert_eq!(entry.content_type.as_deref(), Some("image/png"));
    }

    #[test]
    fn maybe_register_skips_when_policy_says_no() {
        let mut manifest = empty_manifest();
        let res = maybe_register_failed_step_screenshot(
            &mut manifest,
            ScreenshotMode::OnFailure,
            StepOutcome::Passed,
            "case-x-step-1",
            Path::new("artifacts/case-x-step-1.png"),
            None,
        );
        assert!(res.is_none());
        assert!(manifest.artifacts.is_empty(), "no registration on pass");
    }

    #[test]
    fn maybe_register_captures_on_failure_by_default() {
        let mut manifest = empty_manifest();
        let res = maybe_register_failed_step_screenshot(
            &mut manifest,
            ScreenshotMode::default(),
            StepOutcome::Failed,
            "case-x-step-1",
            Path::new("artifacts/case-x-step-1.png"),
            None,
        );
        assert!(res.is_some());
        assert_eq!(manifest.artifacts.len(), 1);
    }
}
// CODEGEN-END
