// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
// CODEGEN-BEGIN
//! Case-level trace artifact registration (#2883).
//!
//! Each e2e case can optionally produce one trace artifact — a CDP
//! trace file the dev viewer can replay step-by-step. Unlike the
//! screenshot/DOM artifacts that scope to a single step, the trace
//! is case-wide. This module owns the on/off policy and the helper
//! that registers the artifact through the bundle manifest so PM
//! reports can resolve the path.
//!
//! Trace UI / visualization is out of scope.

use crate::e2e::E2eArtifactRef;
use crate::evidence_bundle::{BundleArtifact, BundleManifest};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Stable kind label registered through the bundle manifest for
/// case-scoped traces.
pub const TRACE_ARTIFACT_KIND: &str = "case-trace";

/// Configured trace capture mode. Off by default — traces are heavy
/// and we don't want them landing on every CI run.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TraceMode {
    #[default]
    Off,
    On,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
impl TraceMode {
    pub fn is_on(self) -> bool {
        matches!(self, Self::On)
    }
}

/// Build an [`E2eArtifactRef`] for a captured trace and register the
/// matching [`BundleArtifact`] in the manifest.
///
/// `id` should be stable across reruns of the same case so downstream
/// tooling can dedupe. `path` is relative to the bundle root and
/// names a CDP trace file (typically `.json` or `.zip`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn register_case_trace(
    manifest: &mut BundleManifest,
    id: impl Into<String>,
    path: impl Into<PathBuf>,
    label: Option<String>,
) -> E2eArtifactRef {
    let id = id.into();
    let path = path.into();
    let content_type = content_type_for(&path);
    manifest.artifacts.push(BundleArtifact {
        id: id.clone(),
        kind: TRACE_ARTIFACT_KIND.to_string(),
        path: path.clone(),
        content_type: Some(content_type.to_string()),
    });
    E2eArtifactRef {
        kind: TRACE_ARTIFACT_KIND.to_string(),
        path,
        label,
    }
}

/// Predicate paired with [`register_case_trace`]. Returns the
/// registered ref only when the mode is `On`; otherwise `None` and
/// the manifest is untouched.
/// @spec .aw/tech-design/projects/jet/semantic/jet-e2e.md#schema
pub fn maybe_register_case_trace(
    manifest: &mut BundleManifest,
    mode: TraceMode,
    id: impl Into<String>,
    path: impl AsRef<Path>,
    label: Option<String>,
) -> Option<E2eArtifactRef> {
    if !mode.is_on() {
        return None;
    }
    Some(register_case_trace(
        manifest,
        id,
        path.as_ref().to_path_buf(),
        label,
    ))
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("zip") => "application/zip",
        Some("json") => "application/json",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BundleManifest};

    fn empty_manifest() -> BundleManifest {
        BundleManifest::new(
            "run-trace",
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
    fn trace_off_by_default() {
        assert_eq!(TraceMode::default(), TraceMode::Off);
        assert!(!TraceMode::default().is_on());
    }

    #[test]
    fn enabled_trace_capture_creates_registered_artifact() {
        // Stop condition (#2883): enabled trace capture creates a
        // registered artifact in the bundle manifest.
        let mut manifest = empty_manifest();
        let res = maybe_register_case_trace(
            &mut manifest,
            TraceMode::On,
            "case-buy-trace",
            "artifacts/case-buy.trace.zip",
            Some("buy flow trace".into()),
        );
        let r = res.expect("expected an artifact ref");
        assert_eq!(r.kind, TRACE_ARTIFACT_KIND);
        assert_eq!(r.path, PathBuf::from("artifacts/case-buy.trace.zip"));

        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-buy-trace")
            .expect("registered");
        assert_eq!(entry.kind, TRACE_ARTIFACT_KIND);
        assert_eq!(entry.content_type.as_deref(), Some("application/zip"));
    }

    #[test]
    fn disabled_trace_capture_produces_no_artifact_and_no_failure() {
        // Stop condition (#2883): disabled trace capture produces no
        // artifact and no failure — fixture can flip on/off freely.
        let mut manifest = empty_manifest();
        let res = maybe_register_case_trace(
            &mut manifest,
            TraceMode::Off,
            "case-x-trace",
            "artifacts/should-not-exist.zip",
            None,
        );
        assert!(res.is_none());
        assert!(manifest.artifacts.is_empty());
    }

    #[test]
    fn fixture_toggles_trace_on_then_off_across_two_cases() {
        // Stop condition (#2883): a fixture can toggle trace capture
        // on/off — manifest reflects exactly one entry.
        let mut manifest = empty_manifest();
        maybe_register_case_trace(
            &mut manifest,
            TraceMode::On,
            "case-a-trace",
            "artifacts/case-a.trace.json",
            None,
        );
        maybe_register_case_trace(
            &mut manifest,
            TraceMode::Off,
            "case-b-trace",
            "artifacts/case-b.trace.json",
            None,
        );
        let trace_entries: Vec<&BundleArtifact> = manifest
            .artifacts
            .iter()
            .filter(|a| a.kind == TRACE_ARTIFACT_KIND)
            .collect();
        assert_eq!(trace_entries.len(), 1);
        assert_eq!(trace_entries[0].id, "case-a-trace");
        assert_eq!(
            trace_entries[0].content_type.as_deref(),
            Some("application/json")
        );
    }

    #[test]
    fn unknown_extension_falls_back_to_octet_stream() {
        let mut manifest = empty_manifest();
        register_case_trace(&mut manifest, "case-x-trace", "artifacts/case-x.dat", None);
        let entry = manifest
            .artifacts
            .iter()
            .find(|a| a.id == "case-x-trace")
            .unwrap();
        assert_eq!(
            entry.content_type.as_deref(),
            Some("application/octet-stream")
        );
    }

    #[test]
    fn trace_kind_is_distinct_from_screenshot_and_dom_snapshot() {
        assert_ne!(
            TRACE_ARTIFACT_KIND,
            crate::e2e_screenshots::SCREENSHOT_ARTIFACT_KIND
        );
        assert_ne!(
            TRACE_ARTIFACT_KIND,
            crate::e2e_dom_snapshot::DOM_SNAPSHOT_ARTIFACT_KIND
        );
    }
}
// CODEGEN-END
