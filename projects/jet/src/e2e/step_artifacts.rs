// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Read-only screenshot + artifact panel for the selected e2e step (#2888).
//!
//! Sibling of [`crate::e2e_step_panels`] (#2887). Where that module
//! projects console / network rows, this one projects the artifact
//! refs the runner attached to the selected step: the primary
//! screenshot for the preview tile, plus the generic artifact list
//! the UI renders as resolvable bundle-relative links.
//!
//! Video / trace playback UIs are out of scope.

use crate::e2e::{E2eArtifactRef, E2eStepContext};
use crate::e2e_screenshots::SCREENSHOT_ARTIFACT_KIND;
use crate::evidence_bundle::BundleManifest;
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Stable schema tag for [`StepArtifactsPanel`].
pub const STEP_ARTIFACTS_SCHEMA_VERSION: &str = "jet.e2e.step-artifacts.v1";

/// Why a path can't resolve through the bundle. Surfaced to the UI so
/// it renders a disabled link with a hint instead of a broken anchor.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactLinkUnresolvable {
    /// Path was absolute — bundle paths must be relative.
    AbsolutePath,
    /// Path escapes the bundle root via `..` segments.
    EscapesBundleRoot,
    /// The runner didn't register a matching artifact in the manifest.
    NotRegistered,
}

/// Result of resolving an artifact ref against the bundle manifest.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ArtifactLinkResolution {
    Resolved {
        bundle_path: PathBuf,
        content_type: Option<String>,
    },
    Unresolvable {
        reason: ArtifactLinkUnresolvable,
    },
}

/// One link row the UI renders in the artifact list.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLinkRow {
    pub kind: String,
    pub label: Option<String>,
    pub resolution: ArtifactLinkResolution,
}

/// Screenshot preview tile. `None` when no screenshot ref was
/// attached to the step.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenshotPreview {
    pub bundle_path: PathBuf,
    pub label: Option<String>,
    pub content_type: Option<String>,
}

/// The panel projection the inspector hands to the UI.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepArtifactsPanel {
    pub schema_version: String,
    /// Primary screenshot preview, when one is attached.
    pub screenshot: Option<ScreenshotPreview>,
    /// All artifact refs as link rows. Order matches the recorded
    /// order so the UI stays stable across reruns.
    pub links: Vec<ArtifactLinkRow>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl StepArtifactsPanel {
    /// Project the recorded step context into a panel. The screenshot
    /// preview is the first ref whose kind matches
    /// [`SCREENSHOT_ARTIFACT_KIND`] *and* resolves cleanly; everything
    /// else (including unresolvable screenshots) goes into `links`.
    pub fn from_context(context: &E2eStepContext, manifest: &BundleManifest) -> Self {
        let mut screenshot: Option<ScreenshotPreview> = None;
        let mut links: Vec<ArtifactLinkRow> = Vec::new();

        for shot in &context.screenshots {
            let resolution = resolve(shot, manifest);
            if screenshot.is_none() && shot.kind == SCREENSHOT_ARTIFACT_KIND {
                if let ArtifactLinkResolution::Resolved {
                    bundle_path,
                    content_type,
                } = &resolution
                {
                    screenshot = Some(ScreenshotPreview {
                        bundle_path: bundle_path.clone(),
                        label: shot.label.clone(),
                        content_type: content_type.clone(),
                    });
                    // Don't double-list the resolved primary preview.
                    continue;
                }
            }
            links.push(ArtifactLinkRow {
                kind: shot.kind.clone(),
                label: shot.label.clone(),
                resolution,
            });
        }

        Self {
            schema_version: STEP_ARTIFACTS_SCHEMA_VERSION.to_string(),
            screenshot,
            links,
        }
    }
}

fn resolve(reference: &E2eArtifactRef, manifest: &BundleManifest) -> ArtifactLinkResolution {
    if let Some(reason) = path_unportable(&reference.path) {
        return ArtifactLinkResolution::Unresolvable { reason };
    }
    let entry = manifest
        .artifacts
        .iter()
        .find(|a| a.kind == reference.kind && a.path == reference.path);
    match entry {
        Some(a) => ArtifactLinkResolution::Resolved {
            bundle_path: a.path.clone(),
            content_type: a.content_type.clone(),
        },
        None => ArtifactLinkResolution::Unresolvable {
            reason: ArtifactLinkUnresolvable::NotRegistered,
        },
    }
}

fn path_unportable(path: &Path) -> Option<ArtifactLinkUnresolvable> {
    if path.is_absolute() {
        return Some(ArtifactLinkUnresolvable::AbsolutePath);
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Some(ArtifactLinkUnresolvable::EscapesBundleRoot);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::e2e_dom_snapshot::register_step_dom_snapshot;
    use crate::e2e_screenshots::register_step_screenshot;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};

    fn empty_manifest() -> BundleManifest {
        BundleManifest::new(
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
        )
    }

    #[test]
    fn failed_step_shows_screenshot_preview_when_available() {
        // Stop condition (#2888): selected failed step displays
        // screenshot when available.
        let mut manifest = empty_manifest();
        let shot_ref = register_step_screenshot(
            &mut manifest,
            "case-1-step-3",
            "artifacts/case-1-step-3.png",
            Some("confirm order failed".into()),
        );
        let mut ctx = E2eStepContext::default();
        ctx.screenshots.push(shot_ref);

        let panel = StepArtifactsPanel::from_context(&ctx, &manifest);
        let preview = panel.screenshot.expect("screenshot preview");
        assert_eq!(
            preview.bundle_path,
            PathBuf::from("artifacts/case-1-step-3.png")
        );
        assert_eq!(preview.content_type.as_deref(), Some("image/png"));
        // Primary preview is not duplicated into the link list.
        assert!(panel.links.is_empty());
    }

    #[test]
    fn artifact_links_resolve_through_bundle_paths() {
        // Stop condition (#2888): artifact links resolve through
        // bundle paths.
        let mut manifest = empty_manifest();
        let shot_ref = register_step_screenshot(
            &mut manifest,
            "case-2-step-1",
            "artifacts/case-2-step-1.png",
            None,
        );
        let dom_ref = register_step_dom_snapshot(
            &mut manifest,
            "case-2-step-1-dom",
            "artifacts/case-2-step-1.dom.html",
            None,
        );
        let mut ctx = E2eStepContext::default();
        ctx.screenshots.push(shot_ref);
        ctx.screenshots.push(dom_ref);

        let panel = StepArtifactsPanel::from_context(&ctx, &manifest);
        assert!(panel.screenshot.is_some());
        assert_eq!(panel.links.len(), 1);
        match &panel.links[0].resolution {
            ArtifactLinkResolution::Resolved {
                bundle_path,
                content_type,
            } => {
                assert_eq!(
                    bundle_path,
                    &PathBuf::from("artifacts/case-2-step-1.dom.html")
                );
                assert_eq!(content_type.as_deref(), Some("text/html; charset=utf-8"));
            }
            other => panic!("expected resolved, got {other:?}"),
        }
    }

    #[test]
    fn unregistered_artifact_renders_disabled_link_with_reason() {
        let manifest = empty_manifest();
        let mut ctx = E2eStepContext::default();
        ctx.screenshots.push(E2eArtifactRef {
            kind: "step-screenshot".into(),
            path: "artifacts/missing.png".into(),
            label: None,
        });
        let panel = StepArtifactsPanel::from_context(&ctx, &manifest);
        assert!(panel.screenshot.is_none());
        assert_eq!(panel.links.len(), 1);
        match &panel.links[0].resolution {
            ArtifactLinkResolution::Unresolvable { reason } => {
                assert_eq!(*reason, ArtifactLinkUnresolvable::NotRegistered);
            }
            other => panic!("expected unresolvable, got {other:?}"),
        }
    }

    #[test]
    fn absolute_path_and_traversal_are_marked_unresolvable() {
        let manifest = empty_manifest();
        let mut ctx = E2eStepContext::default();
        ctx.screenshots.push(E2eArtifactRef {
            kind: "step-screenshot".into(),
            path: "/tmp/leak.png".into(),
            label: None,
        });
        ctx.screenshots.push(E2eArtifactRef {
            kind: "step-screenshot".into(),
            path: "../leak.png".into(),
            label: None,
        });
        let panel = StepArtifactsPanel::from_context(&ctx, &manifest);
        assert_eq!(panel.links.len(), 2);
        assert!(matches!(
            &panel.links[0].resolution,
            ArtifactLinkResolution::Unresolvable {
                reason: ArtifactLinkUnresolvable::AbsolutePath
            }
        ));
        assert!(matches!(
            &panel.links[1].resolution,
            ArtifactLinkResolution::Unresolvable {
                reason: ArtifactLinkUnresolvable::EscapesBundleRoot
            }
        ));
    }

    #[test]
    fn empty_step_yields_no_preview_and_no_links() {
        let manifest = empty_manifest();
        let panel = StepArtifactsPanel::from_context(&E2eStepContext::default(), &manifest);
        assert!(panel.screenshot.is_none());
        assert!(panel.links.is_empty());
    }

    #[test]
    fn panel_round_trips_through_json() {
        let mut manifest = empty_manifest();
        let shot_ref =
            register_step_screenshot(&mut manifest, "case-x", "artifacts/x.png", Some("x".into()));
        let mut ctx = E2eStepContext::default();
        ctx.screenshots.push(shot_ref);
        let panel = StepArtifactsPanel::from_context(&ctx, &manifest);
        let json = serde_json::to_string(&panel).unwrap();
        let back: StepArtifactsPanel = serde_json::from_str(&json).unwrap();
        assert_eq!(back, panel);
    }
}
// CODEGEN-END
