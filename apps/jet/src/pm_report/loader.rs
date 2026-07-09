// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
// CODEGEN-BEGIN
//! Static evidence bundle loader for PM web reports (#2731).
//!
//! The PM web viewer renders an evidence bundle straight from static
//! hosting — no jet daemon, no live runner. This module owns the
//! load-side contract: it accepts a bundle root URL or directory,
//! parses the manifest, validates the schema version, and exposes
//! artifact references as portable *relative* URLs so the same
//! bundle can be served from `file://`, a static site, or an object
//! store without rewriting paths.
//!
//! The loader reuses [`BundleHandle::load`] for disk reads; the new
//! types here add the URL-shaped projection and the error states the
//! PM viewer needs to render a sensible empty/error UI.
//!
//! Out of scope: authenticated remote storage, live execution
//! control, and the artifact-byte stream (the viewer fetches those
//! over HTTP using the URLs this loader returns).

use crate::evidence_bundle::{
    classify_schema_version, BundleArtifact, BundleHandle, BundleManifest, SchemaCompat,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

/// Stable schema tag for [`StaticReportBundle`].
pub const PM_LOADER_SCHEMA_VERSION: &str = "jet.pm.report-loader.v1";

/// Error states the static loader exposes to the PM web shell.
/// Renderers map each variant to a dedicated empty/error panel.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Error)]
pub enum StaticReportLoadError {
    #[error("manifest not found at {path}")]
    ManifestMissing { path: String },
    #[error("manifest at {path} is not valid JSON: {source}")]
    ManifestInvalid {
        path: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("evidence bundle schema_version `{tag}` is not supported by this viewer")]
    SchemaUnsupported { tag: String },
    #[error("artifact path `{path}` is not portable (must be relative inside the bundle)")]
    NonPortableArtifact { path: String },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// One artifact reference projected into a viewer-friendly shape.
/// `url` is always a bundle-relative URL ready to be joined onto the
/// report base href (e.g. `artifacts/case-1.png`).
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedArtifact {
    pub id: String,
    pub kind: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// Loaded static bundle. `manifest` is the raw manifest as recorded
/// on disk; `artifacts` is the URL-projected view the PM shell
/// renders without further bookkeeping.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaticReportBundle {
    pub schema_version: String,
    pub manifest: BundleManifest,
    pub artifacts: Vec<ResolvedArtifact>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pm-report.md#schema
impl StaticReportBundle {
    /// Load a bundle from a directory laid out by `report_package` /
    /// `BundleHandle::write`. Returns a structured error so the
    /// viewer can render an actionable empty state on failure.
    pub fn load_from_dir(root: impl AsRef<Path>) -> Result<Self, StaticReportLoadError> {
        let root = root.as_ref();
        let manifest_path = root.join("manifest.json");
        if !manifest_path.exists() {
            return Err(StaticReportLoadError::ManifestMissing {
                path: manifest_path.display().to_string(),
            });
        }
        let handle = BundleHandle::load(root).map_err(|e| {
            // Disambiguate schema-version failures from JSON failures.
            let msg = format!("{e:#}");
            if let Some(tag) = extract_schema_tag(&msg) {
                StaticReportLoadError::SchemaUnsupported { tag }
            } else if let Some(src) = e
                .chain()
                .find_map(|err| err.downcast_ref::<serde_json::Error>())
            {
                StaticReportLoadError::ManifestInvalid {
                    path: manifest_path.display().to_string(),
                    source: serde_json::Error::io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        src.to_string(),
                    )),
                }
            } else {
                StaticReportLoadError::Io(std::io::Error::other(msg))
            }
        })?;
        Self::from_manifest(handle.manifest().clone())
    }

    /// Construct a static bundle from a pre-parsed manifest. Useful
    /// when the manifest bytes have already been fetched (e.g. from
    /// the PM shell over HTTP).
    pub fn from_manifest(manifest: BundleManifest) -> Result<Self, StaticReportLoadError> {
        if matches!(
            classify_schema_version(&manifest.schema_version),
            SchemaCompat::Invalid | SchemaCompat::TooNew { .. }
        ) {
            return Err(StaticReportLoadError::SchemaUnsupported {
                tag: manifest.schema_version.clone(),
            });
        }
        let artifacts = manifest
            .artifacts
            .iter()
            .map(resolve_artifact)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            schema_version: PM_LOADER_SCHEMA_VERSION.to_string(),
            manifest,
            artifacts,
        })
    }

    pub fn artifact(&self, id: &str) -> Option<&ResolvedArtifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    /// True when the bundle parsed but exposes no artifacts. The PM
    /// shell renders an "empty bundle" state instead of an error
    /// modal in this case.
    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

/// Format the warn string emitted when an artifact path contains
/// non-UTF-8 bytes. Pinned in tests so a rename without grep breaks
/// the suite.
// @spec gh3793 — silent lossy URL substitution for non-UTF-8 artifact paths
pub(crate) fn format_pm_report_loader_non_utf8_artifact_warn(
    artifact_id: &str,
    path: &Path,
) -> String {
    format!(
        "gh3793: PM report loader artifact id={artifact_id:?} has non-UTF-8 \
         path {path}; rendered URL will be U+FFFD-substituted and will NOT \
         match the on-disk filename — operator should rename the artifact",
        path = path.display(),
    )
}

/// Render the bundle-relative URL string for an artifact path.
///
/// UTF-8 paths pass through unchanged. Non-UTF-8 paths still produce
/// a string (via `to_string_lossy`) so the report HTML still renders,
/// but emit a `tracing::warn!` tagged `gh3793` so operators can chase
/// the U+FFFD-substituted link that no longer resolves on disk.
// @spec gh3793
pub(crate) fn resolve_artifact_path_or_warn(path: &Path, artifact_id: &str) -> String {
    match path.to_str() {
        Some(s) => s.to_string(),
        None => {
            tracing::warn!(
                target: "jet::pm::report_loader",
                artifact_id = %artifact_id,
                path = %path.display(),
                "{}",
                format_pm_report_loader_non_utf8_artifact_warn(artifact_id, path),
            );
            path.to_string_lossy().into_owned()
        }
    }
}

fn resolve_artifact(a: &BundleArtifact) -> Result<ResolvedArtifact, StaticReportLoadError> {
    let raw = resolve_artifact_path_or_warn(&a.path, &a.id);
    if a.path.is_absolute() || raw.contains("..") {
        return Err(StaticReportLoadError::NonPortableArtifact { path: raw });
    }
    // Normalise Windows-style separators so the URL is uniform.
    let url = raw.replace('\\', "/");
    Ok(ResolvedArtifact {
        id: a.id.clone(),
        kind: a.kind.clone(),
        url,
        content_type: a.content_type.clone(),
    })
}

fn extract_schema_tag(msg: &str) -> Option<String> {
    let prefix = "schema_version `";
    let start = msg.find(prefix)? + prefix.len();
    let rest = &msg[start..];
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_bundle::{BundleCommand, BundleEnvironment, BUNDLE_SCHEMA_VERSION};
    use std::path::PathBuf;

    fn manifest_with_artifacts(artifacts: Vec<BundleArtifact>) -> BundleManifest {
        let mut m = BundleManifest::new(
            "run-1",
            BundleCommand::E2e,
            "demo",
            "abc123",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.60".into(),
                ci: None,
                node_version: None,
            },
        );
        m.artifacts = artifacts;
        m
    }

    fn write_bundle(root: &Path, m: &BundleManifest) {
        std::fs::create_dir_all(root).unwrap();
        let body = serde_json::to_vec_pretty(m).unwrap();
        std::fs::write(root.join("manifest.json"), body).unwrap();
    }

    #[test]
    fn static_bundle_loads_and_resolves_relative_urls() {
        // Stop condition (#2731): one packaged bundle loads and
        // exposes artifact URLs.
        let tmp = tempfile::tempdir().unwrap();
        let m = manifest_with_artifacts(vec![
            BundleArtifact {
                id: "shot-1".into(),
                kind: "step-screenshot".into(),
                path: PathBuf::from("artifacts/shot-1.png"),
                content_type: Some("image/png".into()),
            },
            BundleArtifact {
                id: "trace-1".into(),
                kind: "trace".into(),
                path: PathBuf::from("artifacts/trace-1.zip"),
                content_type: None,
            },
        ]);
        write_bundle(tmp.path(), &m);

        let bundle = StaticReportBundle::load_from_dir(tmp.path()).unwrap();
        assert_eq!(bundle.schema_version, PM_LOADER_SCHEMA_VERSION);
        assert_eq!(bundle.manifest.schema_version, BUNDLE_SCHEMA_VERSION);
        assert_eq!(bundle.artifacts.len(), 2);
        let shot = bundle.artifact("shot-1").unwrap();
        assert_eq!(shot.url, "artifacts/shot-1.png");
        assert_eq!(shot.content_type.as_deref(), Some("image/png"));
        assert!(!bundle.is_empty());
    }

    #[test]
    fn missing_manifest_returns_structured_error() {
        let tmp = tempfile::tempdir().unwrap();
        let err = StaticReportBundle::load_from_dir(tmp.path()).unwrap_err();
        assert!(matches!(err, StaticReportLoadError::ManifestMissing { .. }));
    }

    #[test]
    fn unsupported_schema_version_fails_loud() {
        let mut m = manifest_with_artifacts(vec![]);
        m.schema_version = "jet.evidence.bundle.v99".into();
        let err = StaticReportBundle::from_manifest(m).unwrap_err();
        assert!(matches!(
            err,
            StaticReportLoadError::SchemaUnsupported { ref tag } if tag == "jet.evidence.bundle.v99"
        ));
    }

    #[test]
    fn absolute_artifact_path_is_rejected_as_non_portable() {
        let m = manifest_with_artifacts(vec![BundleArtifact {
            id: "boom".into(),
            kind: "screenshot".into(),
            path: PathBuf::from("/etc/passwd"),
            content_type: None,
        }]);
        let err = StaticReportBundle::from_manifest(m).unwrap_err();
        assert!(matches!(
            err,
            StaticReportLoadError::NonPortableArtifact { ref path } if path.contains("/etc/passwd")
        ));
    }

    #[test]
    fn parent_dir_traversal_is_rejected_as_non_portable() {
        let m = manifest_with_artifacts(vec![BundleArtifact {
            id: "boom".into(),
            kind: "screenshot".into(),
            path: PathBuf::from("../escape.png"),
            content_type: None,
        }]);
        let err = StaticReportBundle::from_manifest(m).unwrap_err();
        assert!(matches!(
            err,
            StaticReportLoadError::NonPortableArtifact { .. }
        ));
    }

    #[test]
    fn empty_bundle_loads_with_empty_view() {
        let tmp = tempfile::tempdir().unwrap();
        let m = manifest_with_artifacts(vec![]);
        write_bundle(tmp.path(), &m);
        let bundle = StaticReportBundle::load_from_dir(tmp.path()).unwrap();
        assert!(bundle.is_empty());
    }

    #[test]
    fn resolved_bundle_round_trips_through_json() {
        let m = manifest_with_artifacts(vec![BundleArtifact {
            id: "shot".into(),
            kind: "step-screenshot".into(),
            path: PathBuf::from("a/b.png"),
            content_type: Some("image/png".into()),
        }]);
        let bundle = StaticReportBundle::from_manifest(m).unwrap();
        let json = serde_json::to_string(&bundle).unwrap();
        let back: StaticReportBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(back, bundle);
        assert!(json.contains("\"url\":\"a/b.png\""), "{json}");
    }

    mod gh3793_non_utf8_artifact_warn_tests {
        //! GH #3793 — `resolve_artifact` previously rendered every
        //! artifact URL through `to_string_lossy()`, silently
        //! substituting U+FFFD for non-UTF-8 bytes. The rendered URL
        //! no longer pointed at the on-disk file, and the operator
        //! had no signal.
        use super::*;

        #[test]
        fn utf8_path_returns_unchanged_string() {
            // 1) UTF-8 path round-trips through `to_str()` — no lossy
            // substitution, no warn.
            let p = PathBuf::from("artifacts/clean.png");
            let s = super::resolve_artifact_path_or_warn(&p, "clean-1");
            assert_eq!(s, "artifacts/clean.png");
        }

        #[cfg(unix)]
        #[test]
        fn non_utf8_path_returns_lossy_form() {
            // 2-3) Non-UTF-8 bytes survive into the lossy form as
            // U+FFFD. The helper returns a string, the report still
            // renders, but a U+FFFD marker is now visible.
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;
            let bytes: &[u8] = b"artifacts/bad-\xFF.png";
            let p = PathBuf::from(OsStr::from_bytes(bytes));
            let s = super::resolve_artifact_path_or_warn(&p, "bad-1");
            assert!(
                s.contains('\u{FFFD}'),
                "expected U+FFFD substitution: {s:?}"
            );
            assert!(s.starts_with("artifacts/bad-"), "prefix preserved: {s:?}");
        }

        #[test]
        fn warn_helper_name_pinned_for_discoverability() {
            // 4) Pin the helper name so a rename without grep breaks
            // tests.
            let w =
                super::format_pm_report_loader_non_utf8_artifact_warn("art-1", Path::new("x.png"));
            assert!(!w.is_empty());
        }

        #[test]
        fn warn_string_includes_gh3793_tag_and_artifact_id() {
            // 5-6) Issue tag + artifact id preserved for log triage.
            let w =
                super::format_pm_report_loader_non_utf8_artifact_warn("my-art", Path::new("x.png"));
            assert!(w.contains("gh3793"), "missing gh3793 tag: {w}");
            assert!(w.contains("my-art"), "missing artifact id: {w}");
        }

        #[test]
        fn warn_string_distinct_from_prior_silent_fallback_families() {
            // 7) Sibling-distinctness vs every prior warn family.
            let w =
                super::format_pm_report_loader_non_utf8_artifact_warn("art", Path::new("x.png"));
            for prior in [
                "gh3763", "gh3765", "gh3768", "gh3770", "gh3772", "gh3774", "gh3776", "gh3787",
                "gh3789", "gh3791",
            ] {
                assert!(!w.contains(prior), "must not overlap {prior}: {w}");
            }
        }

        #[cfg(unix)]
        #[test]
        fn non_utf8_path_resolves_through_full_pipeline() {
            // 8) `resolve_artifact` end-to-end with a non-UTF-8 path
            // produces an Ok result whose `url` carries the lossy form.
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;
            let bytes: &[u8] = b"artifacts/lossy-\xFE\xFF.png";
            let p = PathBuf::from(OsStr::from_bytes(bytes));
            let a = BundleArtifact {
                id: "lossy".into(),
                kind: "screenshot".into(),
                path: p,
                content_type: None,
            };
            let r = super::resolve_artifact(&a).expect("non-UTF-8 path still resolves");
            assert!(r.url.contains('\u{FFFD}'), "lossy form in url: {}", r.url);
            assert!(r.url.starts_with("artifacts/lossy-"), "url: {}", r.url);
        }

        #[cfg(unix)]
        #[test]
        fn non_utf8_dotdot_still_rejected() {
            // 9) `..` rejection still fires for non-UTF-8 paths whose
            // bytes contain literal `..`.
            use std::ffi::OsStr;
            use std::os::unix::ffi::OsStrExt;
            let bytes: &[u8] = b"../sneaky-\xFF.png";
            let p = PathBuf::from(OsStr::from_bytes(bytes));
            let a = BundleArtifact {
                id: "sneaky".into(),
                kind: "screenshot".into(),
                path: p,
                content_type: None,
            };
            let err = super::resolve_artifact(&a).unwrap_err();
            assert!(matches!(
                err,
                StaticReportLoadError::NonPortableArtifact { .. }
            ));
        }

        #[test]
        fn backslash_normalisation_still_applies() {
            // 10) Windows-style `\` → `/` normalisation still fires
            // even after the helper interposed.
            let a = BundleArtifact {
                id: "win".into(),
                kind: "screenshot".into(),
                path: PathBuf::from(r"artifacts\sub\shot.png"),
                content_type: None,
            };
            let r = super::resolve_artifact(&a).expect("backslash path resolves");
            assert_eq!(r.url, "artifacts/sub/shot.png");
        }
    }
}
// CODEGEN-END
