// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
// CODEGEN-BEGIN
//! Portable on-disk evidence bundle layout shared by `jet test` and
//! `jet e2e` (#2716).
//!
//! ## Layout
//!
//! ```text
//! <bundle-root>/
//!   manifest.json           — [`BundleManifest`], schema-versioned
//!   artifacts/<rel-path>    — artifact bytes (screenshots, traces, …)
//!   events.jsonl            — optional per-event stream
//! ```
//!
//! ## Portability rule
//!
//! Every artifact in the manifest is stored as a path **relative to
//! `<bundle-root>`** — never absolute, never escaping the root. A
//! bundle may therefore be copied, moved, archived, or extracted from
//! a tarball and still resolve all of its own references. The
//! [`BundleHandle::resolve`] API enforces this: it rejects absolute
//! manifest paths and paths that climb out of the bundle with `..`.
//!
//! ## Compatibility
//!
//! This layout is intentionally minimal — schema version, run id,
//! command, project, commit, environment, and a flat list of
//! artifacts. Both `jet test` and `jet e2e` can emit the same manifest
//! shape; downstream consumers (PM report, agent post-mortem) load
//! through [`BundleHandle::load`] without invoking any runner.
//!
//! @spec #2716

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Component, Path, PathBuf};

/// Stable schema tag for the portable evidence bundle manifest. Embedded
/// in `manifest.json` so consumers can detect schema drift.
pub const BUNDLE_SCHEMA_VERSION: &str = "jet.evidence.bundle.v1";

/// Current major schema version. The `vN` suffix on
/// [`BUNDLE_SCHEMA_VERSION`] must match this. A reader rejects any
/// manifest whose major version exceeds this value because it cannot
/// safely interpret unknown future fields.
pub const BUNDLE_SCHEMA_CURRENT_MAJOR: u32 = 1;

/// Default file name for the manifest inside a bundle root.
pub const MANIFEST_FILE_NAME: &str = "manifest.json";

/// Reader-side schema compatibility verdict (#2717).
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaCompat {
    /// Manifest tag matches the current schema exactly.
    Current,
    /// Manifest tag is an older supported major. Reserved for when
    /// migration shims land; for now no older majors are supported and
    /// this variant is unused.
    OlderSupported,
    /// Manifest tag is newer than the current major and cannot be
    /// safely interpreted by this reader.
    TooNew { actual_major: u32 },
    /// Tag does not match the `jet.evidence.bundle.vN` shape or fails
    /// to parse a positive major.
    Invalid,
}

/// Validate a schema-version string and return the compatibility
/// verdict. The expected shape is `jet.evidence.bundle.v<MAJOR>` with
/// `MAJOR` a positive integer.
// @spec #2717
pub fn classify_schema_version(tag: &str) -> SchemaCompat {
    let Some(rest) = tag.strip_prefix("jet.evidence.bundle.v") else {
        return SchemaCompat::Invalid;
    };
    let Ok(major) = rest.parse::<u32>() else {
        return SchemaCompat::Invalid;
    };
    if major == 0 {
        return SchemaCompat::Invalid;
    }
    match major.cmp(&BUNDLE_SCHEMA_CURRENT_MAJOR) {
        std::cmp::Ordering::Equal => SchemaCompat::Current,
        std::cmp::Ordering::Less => SchemaCompat::OlderSupported,
        std::cmp::Ordering::Greater => SchemaCompat::TooNew {
            actual_major: major,
        },
    }
}

/// Validate that this reader can render a manifest with the given
/// schema tag. Returns `Ok(())` for current (and reserved
/// older-supported) versions; otherwise returns an actionable error
/// naming the expected version.
// @spec #2717
pub fn validate_schema_version(tag: &str) -> Result<()> {
    match classify_schema_version(tag) {
        SchemaCompat::Current | SchemaCompat::OlderSupported => Ok(()),
        SchemaCompat::TooNew { actual_major } => Err(anyhow!(
            "evidence bundle schema is too new for this jet build \
             (manifest is v{actual_major}; this reader supports up to v{current}). \
             upgrade jet to read this bundle.",
            current = BUNDLE_SCHEMA_CURRENT_MAJOR,
        )),
        SchemaCompat::Invalid => Err(anyhow!(
            "evidence bundle schema_version `{tag}` is not a recognised \
             jet.evidence.bundle.vN tag; expected `{BUNDLE_SCHEMA_VERSION}`",
        )),
    }
}

/// Which command produced this bundle.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BundleCommand {
    Test,
    E2e,
}

/// Reproducibility context for a bundle — pulled at write-time from the
/// host environment so the consumer can match a bundle back to a commit
/// and a CI host.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleEnvironment {
    pub os: String,
    pub runner_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ci: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node_version: Option<String>,
}

/// One artifact in the bundle — a screenshot, trace, log, JSON
/// report, anything captured by a test/e2e step. The `path` is
/// always relative to the bundle root.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleArtifact {
    pub id: String,
    pub kind: String,
    pub path: PathBuf,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// On-disk manifest for an evidence bundle. Lives at
/// `<bundle-root>/manifest.json`. The first field is the schema version
/// so a consumer can route by it without parsing the rest.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleManifest {
    pub schema_version: String,
    pub run_id: String,
    pub command: BundleCommand,
    pub project: String,
    pub commit: String,
    pub environment: BundleEnvironment,
    pub artifacts: Vec<BundleArtifact>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
impl BundleManifest {
    /// Construct an empty manifest tagged with the current schema version.
    pub fn new(
        run_id: impl Into<String>,
        command: BundleCommand,
        project: impl Into<String>,
        commit: impl Into<String>,
        environment: BundleEnvironment,
    ) -> Self {
        Self {
            schema_version: BUNDLE_SCHEMA_VERSION.to_string(),
            run_id: run_id.into(),
            command,
            project: project.into(),
            commit: commit.into(),
            environment,
            artifacts: Vec::new(),
        }
    }
}

/// Reader handle that ties a manifest to its bundle root so callers can
/// resolve artifact references into absolute paths.
/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
#[derive(Debug, Clone)]
pub struct BundleHandle {
    root: PathBuf,
    manifest: BundleManifest,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-evidence.md#schema
impl BundleHandle {
    /// Load a bundle from `root/manifest.json`.
    ///
    /// The reader validates [`BundleManifest::schema_version`] before
    /// returning the handle — an unrecognised or too-new tag fails loud
    /// with an actionable error, so a stale jet build cannot silently
    /// half-render a future-shaped bundle.
    // @spec #2717
    pub fn load(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let manifest_path = root.join(MANIFEST_FILE_NAME);
        let bytes = std::fs::read(&manifest_path)
            .with_context(|| format!("reading manifest from {}", manifest_path.display()))?;
        let manifest: BundleManifest = serde_json::from_slice(&bytes)
            .with_context(|| format!("parsing manifest at {}", manifest_path.display()))?;
        validate_schema_version(&manifest.schema_version)
            .with_context(|| format!("rejecting evidence bundle at {}", manifest_path.display()))?;
        Ok(Self {
            root: root.to_path_buf(),
            manifest,
        })
    }

    /// Write a manifest to `root/manifest.json`, creating `root` if
    /// needed. Returns a handle that resolves against the new location.
    pub fn write(root: impl AsRef<Path>, manifest: &BundleManifest) -> Result<Self> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)
            .with_context(|| format!("creating bundle root at {}", root.display()))?;
        let body = serde_json::to_vec_pretty(manifest).context("serialising manifest")?;
        std::fs::write(root.join(MANIFEST_FILE_NAME), body)
            .with_context(|| format!("writing manifest at {}", root.display()))?;
        Ok(Self {
            root: root.to_path_buf(),
            manifest: manifest.clone(),
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn manifest(&self) -> &BundleManifest {
        &self.manifest
    }

    /// Resolve an artifact reference to an absolute on-disk path under
    /// the current bundle root. Rejects absolute manifest paths and
    /// paths that climb out of the root via `..` — both are bundle
    /// integrity violations.
    pub fn resolve(&self, artifact_id: &str) -> Result<PathBuf> {
        let artifact = self
            .manifest
            .artifacts
            .iter()
            .find(|a| a.id == artifact_id)
            .ok_or_else(|| anyhow!("artifact `{artifact_id}` not present in manifest"))?;
        ensure_relative_inside_root(&artifact.path)?;
        Ok(self.root.join(&artifact.path))
    }
}

/// Returns Ok if `path` is a portable, root-anchored reference (no
/// absolute prefix, no `..` traversal). Used by [`BundleHandle::resolve`]
/// so bundles that violate the portability rule fail loud instead of
/// silently leaking outside the root.
fn ensure_relative_inside_root(path: &Path) -> Result<()> {
    if path.is_absolute() {
        return Err(anyhow!(
            "bundle artifact path must be relative; got absolute {}",
            path.display()
        ));
    }
    let mut depth: i32 = 0;
    for comp in path.components() {
        match comp {
            Component::Normal(_) => depth += 1,
            Component::CurDir => continue,
            Component::ParentDir => {
                depth -= 1;
                if depth < 0 {
                    return Err(anyhow!(
                        "bundle artifact path escapes root: {}",
                        path.display()
                    ));
                }
            }
            Component::Prefix(_) | Component::RootDir => {
                return Err(anyhow!(
                    "bundle artifact path must be relative; got rooted {}",
                    path.display()
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_manifest() -> BundleManifest {
        let mut m = BundleManifest::new(
            "run-abc",
            BundleCommand::Test,
            "jet",
            "deadbeef",
            BundleEnvironment {
                os: "darwin".into(),
                runner_version: "0.3.48".into(),
                ci: None,
                node_version: Some("v20.10.0".into()),
            },
        );
        m.artifacts.push(BundleArtifact {
            id: "summary".into(),
            kind: "test_summary".into(),
            path: PathBuf::from("artifacts/summary.json"),
            content_type: Some("application/json".into()),
        });
        m.artifacts.push(BundleArtifact {
            id: "screenshot-1".into(),
            kind: "screenshot".into(),
            path: PathBuf::from("artifacts/case-1/page-1.png"),
            content_type: Some("image/png".into()),
        });
        m
    }

    fn seed_bundle(root: &Path) {
        let m = sample_manifest();
        BundleHandle::write(root, &m).unwrap();
        let art_dir = root.join("artifacts/case-1");
        std::fs::create_dir_all(&art_dir).unwrap();
        std::fs::write(root.join("artifacts/summary.json"), b"{\"ok\":true}").unwrap();
        std::fs::write(art_dir.join("page-1.png"), b"PNGDATA").unwrap();
    }

    #[test]
    fn manifest_serialization_carries_schema_version_first() {
        let m = sample_manifest();
        let json = serde_json::to_string(&m).unwrap();
        assert!(json.starts_with("{\"schema_version\":\"jet.evidence.bundle.v1\""));
    }

    #[test]
    fn round_trips_through_disk() {
        let tmp = TempDir::new().unwrap();
        seed_bundle(tmp.path());
        let h = BundleHandle::load(tmp.path()).unwrap();
        assert_eq!(h.manifest(), &sample_manifest());
    }

    // ── Portability: copy bundle to a new directory, refs still resolve ──
    #[test]
    fn bundle_resolves_refs_after_moving_directories() {
        let src = TempDir::new().unwrap();
        seed_bundle(src.path());

        // Move the entire bundle into a second temp dir by directory copy.
        let dst = TempDir::new().unwrap();
        let moved_root = dst.path().join("moved-bundle");
        std::fs::create_dir_all(&moved_root).unwrap();
        copy_tree(src.path(), &moved_root);
        // Wipe the original to prove resolution uses the new root only.
        std::fs::remove_dir_all(src.path()).unwrap();

        let h = BundleHandle::load(&moved_root).unwrap();
        let resolved = h.resolve("screenshot-1").unwrap();
        assert!(resolved.starts_with(&moved_root), "{resolved:?}");
        assert!(resolved.exists(), "moved artifact must still resolve");
        let bytes = std::fs::read(&resolved).unwrap();
        assert_eq!(&bytes, b"PNGDATA");
    }

    #[test]
    fn resolve_rejects_absolute_artifact_paths() {
        let tmp = TempDir::new().unwrap();
        let mut m = sample_manifest();
        m.artifacts.push(BundleArtifact {
            id: "bad-abs".into(),
            kind: "log".into(),
            path: PathBuf::from("/etc/passwd"),
            content_type: None,
        });
        BundleHandle::write(tmp.path(), &m).unwrap();
        let h = BundleHandle::load(tmp.path()).unwrap();
        let err = h.resolve("bad-abs").unwrap_err().to_string();
        assert!(err.contains("must be relative"), "{err}");
    }

    #[test]
    fn resolve_rejects_path_escaping_root_with_parent_dir() {
        let tmp = TempDir::new().unwrap();
        let mut m = sample_manifest();
        m.artifacts.push(BundleArtifact {
            id: "bad-escape".into(),
            kind: "log".into(),
            path: PathBuf::from("../outside.txt"),
            content_type: None,
        });
        BundleHandle::write(tmp.path(), &m).unwrap();
        let h = BundleHandle::load(tmp.path()).unwrap();
        let err = h.resolve("bad-escape").unwrap_err().to_string();
        assert!(err.contains("escapes root"), "{err}");
    }

    #[test]
    fn resolve_unknown_artifact_id_is_an_error() {
        let tmp = TempDir::new().unwrap();
        seed_bundle(tmp.path());
        let h = BundleHandle::load(tmp.path()).unwrap();
        let err = h.resolve("does-not-exist").unwrap_err().to_string();
        assert!(err.contains("not present in manifest"), "{err}");
    }

    #[test]
    fn test_and_e2e_can_share_the_same_manifest_shape() {
        let m_test = BundleManifest::new(
            "r1",
            BundleCommand::Test,
            "jet",
            "c1",
            BundleEnvironment {
                os: "linux".into(),
                runner_version: "0.3.48".into(),
                ci: None,
                node_version: None,
            },
        );
        let m_e2e = BundleManifest::new(
            "r2",
            BundleCommand::E2e,
            "jet",
            "c1",
            BundleEnvironment {
                os: "linux".into(),
                runner_version: "0.3.48".into(),
                ci: None,
                node_version: None,
            },
        );
        // Schema tag is the same; only the `command` discriminator differs.
        assert_eq!(m_test.schema_version, m_e2e.schema_version);
        assert_ne!(m_test.command, m_e2e.command);
    }

    fn copy_tree(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for entry in std::fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let src = entry.path();
            let dst = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy_tree(&src, &dst);
            } else {
                std::fs::copy(&src, &dst).unwrap();
            }
        }
    }

    // ── Schema versioning + reader compatibility (#2717) ────────────────

    #[test]
    fn classify_schema_version_recognises_current() {
        assert_eq!(
            classify_schema_version(BUNDLE_SCHEMA_VERSION),
            SchemaCompat::Current,
        );
    }

    #[test]
    fn classify_schema_version_flags_future_majors_as_too_new() {
        let actual = classify_schema_version("jet.evidence.bundle.v2");
        assert!(matches!(actual, SchemaCompat::TooNew { actual_major: 2 }));
    }

    #[test]
    fn classify_schema_version_rejects_invalid_tags() {
        for bad in [
            "",
            "jet.evidence.bundle",
            "jet.evidence.bundle.v",
            "jet.evidence.bundle.vNaN",
            "jet.evidence.bundle.v0",
            "some.other.schema.v1",
        ] {
            assert_eq!(
                classify_schema_version(bad),
                SchemaCompat::Invalid,
                "expected Invalid for {bad:?}",
            );
        }
    }

    #[test]
    fn validate_schema_version_too_new_names_versions_actionably() {
        let err = validate_schema_version("jet.evidence.bundle.v9")
            .unwrap_err()
            .to_string();
        assert!(err.contains("v9"), "{err}");
        assert!(
            err.contains("v1"),
            "expected reader to name supported version: {err}"
        );
        assert!(err.to_lowercase().contains("upgrade"), "{err}");
    }

    #[test]
    fn load_rejects_too_new_bundle_with_actionable_error() {
        let tmp = TempDir::new().unwrap();
        let mut m = sample_manifest();
        m.schema_version = "jet.evidence.bundle.v99".into();
        let body = serde_json::to_vec_pretty(&m).unwrap();
        std::fs::create_dir_all(tmp.path()).unwrap();
        std::fs::write(tmp.path().join(MANIFEST_FILE_NAME), body).unwrap();

        let err = format!("{:#}", BundleHandle::load(tmp.path()).unwrap_err());
        assert!(err.to_lowercase().contains("too new"), "{err}");
    }

    #[test]
    fn load_rejects_invalid_schema_tag() {
        let tmp = TempDir::new().unwrap();
        let mut m = sample_manifest();
        m.schema_version = "not.a.schema".into();
        let body = serde_json::to_vec_pretty(&m).unwrap();
        std::fs::write(tmp.path().join(MANIFEST_FILE_NAME), body).unwrap();

        let err = format!("{:#}", BundleHandle::load(tmp.path()).unwrap_err());
        assert!(err.contains("not a recognised"), "{err}");
    }
}
// CODEGEN-END
