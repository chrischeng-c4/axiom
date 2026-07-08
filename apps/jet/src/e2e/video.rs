// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Case-level video artifact registration (#2884).
//!
//! Some flake investigations are easier to triage from a video than
//! from a screenshot + DOM snapshot pair — UI animation glitches and
//! focus-trap races in particular. This module adds the optional
//! capture flag and the helper that registers the artifact through
//! the existing bundle manifest plumbing.
//!
//! Compression, upload, and retention policy are out of scope. We
//! also require the registered path to be relative to the bundle so
//! the evidence bundle stays portable (absolute paths or `..`
//! traversal would break offline PM report viewing).

use crate::e2e::E2eArtifactRef;
use crate::evidence_bundle::{BundleArtifact, BundleManifest};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Stable kind label registered through the bundle manifest for
/// case-scoped videos.
pub const VIDEO_ARTIFACT_KIND: &str = "case-video";

/// Configured video capture mode. Off by default — video is heavy
/// and rarely necessary on green runs.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VideoMode {
    #[default]
    Off,
    On,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl VideoMode {
    pub fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

/// Why a video could not be registered. The runner surfaces these to
/// the per-case error log so the reviewer doesn't silently miss a
/// video that was supposed to land.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoRegistrationError {
    /// Path was absolute. The bundle is portable; only paths inside
    /// the bundle root are allowed.
    AbsolutePath,
    /// Path contained a `..` segment that would escape the bundle
    /// root.
    EscapesBundleRoot,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl std::fmt::Display for VideoRegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AbsolutePath => {
                write!(f, "video path must be relative to the evidence bundle")
            }
            Self::EscapesBundleRoot => {
                write!(f, "video path must not contain `..` segments")
            }
        }
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl std::error::Error for VideoRegistrationError {}

/// Build an [`E2eArtifactRef`] for a captured video and register the
/// matching [`BundleArtifact`] in the manifest. Returns an error if
/// the path would break bundle portability.
///
/// `id` should be stable across reruns of the same case. `path` is
/// relative to the bundle root and names a video file (typically
/// `.webm` for CDP screencast frames).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn register_case_video(
    manifest: &mut BundleManifest,
    id: impl Into<String>,
    path: impl Into<PathBuf>,
    label: Option<String>,
) -> Result<E2eArtifactRef, VideoRegistrationError> {
    let id = id.into();
    let path = path.into();
    check_portable(&path)?;
    let content_type = content_type_for(&path);
    manifest.artifacts.push(BundleArtifact {
        id: id.clone(),
        kind: VIDEO_ARTIFACT_KIND.to_string(),
        path: path.clone(),
        content_type: Some(content_type.to_string()),
    });
    Ok(E2eArtifactRef {
        kind: VIDEO_ARTIFACT_KIND.to_string(),
        path,
        label,
    })
}

/// Predicate paired with [`register_case_video`]. Returns the
/// registered ref only when the mode is `On`; otherwise `Ok(None)`
/// and the manifest is untouched.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn maybe_register_case_video(
    manifest: &mut BundleManifest,
    mode: VideoMode,
    id: impl Into<String>,
    path: impl AsRef<Path>,
    label: Option<String>,
) -> Result<Option<E2eArtifactRef>, VideoRegistrationError> {
    if !mode.is_on() {
        return Ok(None);
    }
    Ok(Some(register_case_video(
        manifest,
        id,
        path.as_ref().to_path_buf(),
        label,
    )?))
}

fn check_portable(path: &Path) -> Result<(), VideoRegistrationError> {
    if path.is_absolute() {
        return Err(VideoRegistrationError::AbsolutePath);
    }
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(VideoRegistrationError::EscapesBundleRoot);
    }
    Ok(())
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("webm") => "video/webm",
        Some("mp4") => "video/mp4",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};

    fn empty_manifest() -> BundleManifest {
        BundleManifest::new(
            "run-video",
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
    fn video_off_by_default() {
        assert_eq!(VideoMode::default(), VideoMode::Off);
        assert!(!VideoMode::default().is_on());
    }

    #[test]
    fn fixture_run_emits_registered_video_artifact_when_enabled() {
        // Stop condition (#2884): a fixture run emits a registered
        // video artifact when enabled.
        let mut manifest = empty_manifest();
        let res = maybe_register_case_video(
            &mut manifest,
            VideoMode::On,
            "case-buy-video",
            "artifacts/case-buy.webm",
            Some("buy flow video".into()),
        );
        let r = res.unwrap().expect("expected an artifact ref");
        assert_eq!(r.kind, VIDEO_ARTIFACT_KIND);
        assert_eq!(r.path, PathBuf::from("artifacts/case-buy.webm"));

        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-buy-video")
            .expect("registered");
        assert_eq!(entry.kind, VIDEO_ARTIFACT_KIND);
        assert_eq!(entry.content_type.as_deref(), Some("video/webm"));
    }

    #[test]
    fn disabled_mode_registers_nothing() {
        let mut manifest = empty_manifest();
        let res = maybe_register_case_video(
            &mut manifest,
            VideoMode::Off,
            "case-x-video",
            "artifacts/should-not-exist.webm",
            None,
        )
        .unwrap();
        assert!(res.is_none());
        assert!(manifest.artifacts.is_empty());
    }

    #[test]
    fn absolute_path_is_rejected_to_keep_bundle_portable() {
        // Stop condition (#2884): video path is relative to the
        // evidence bundle.
        let mut manifest = empty_manifest();
        let err = register_case_video(&mut manifest, "case-bad-video", "/tmp/case.webm", None)
            .unwrap_err();
        assert_eq!(err, VideoRegistrationError::AbsolutePath);
        assert!(manifest.artifacts.is_empty());
    }

    #[test]
    fn parent_dir_traversal_is_rejected() {
        let mut manifest = empty_manifest();
        let err = register_case_video(&mut manifest, "case-escape", "../leaked/case.webm", None)
            .unwrap_err();
        assert_eq!(err, VideoRegistrationError::EscapesBundleRoot);
        assert!(manifest.artifacts.is_empty());
    }

    #[test]
    fn mp4_extension_gets_mp4_content_type() {
        let mut manifest = empty_manifest();
        register_case_video(&mut manifest, "case-mp4", "artifacts/case.mp4", None).unwrap();
        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-mp4")
            .unwrap();
        assert_eq!(entry.content_type.as_deref(), Some("video/mp4"));
    }

    #[test]
    fn video_kind_is_distinct_from_other_artifact_kinds() {
        assert_ne!(
            VIDEO_ARTIFACT_KIND,
            crate::e2e_screenshots::SCREENSHOT_ARTIFACT_KIND
        );
        assert_ne!(
            VIDEO_ARTIFACT_KIND,
            crate::e2e_dom_snapshot::DOM_SNAPSHOT_ARTIFACT_KIND
        );
        assert_ne!(VIDEO_ARTIFACT_KIND, crate::e2e_trace::TRACE_ARTIFACT_KIND);
    }
}
// CODEGEN-END
