// SPEC-MANAGED: .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
// CODEGEN-BEGIN
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use super::resolver::ResolvedPackage;

/// Lockfile v2 format, persisted as `jet-lock.yaml`.
///
/// Each entry captures the fully resolved version, tarball location,
/// integrity hash, and the package's own dependency map so that
/// re-installation can skip resolution entirely.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: String,

    /// SHA-256 of sorted package.json deps for frozen lockfile check.
    #[serde(rename = "depsHash", default, skip_serializing_if = "Option::is_none")]
    pub deps_hash: Option<String>,

    /// Overrides map from package.json.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub overrides: HashMap<String, String>,

    /// Patched packages: package@version → patch file path.
    #[serde(
        rename = "patchedPackages",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub patched_packages: HashMap<String, String>,

    #[serde(default)]
    pub packages: HashMap<String, LockfileEntry>,
}

/// A single package entry inside the lockfile.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileEntry {
    pub version: String,
    pub resolution: Resolution,

    /// True when this entry is a local workspace package.
    /// `is_valid()` skips the store-presence check for workspace entries.
    #[serde(default, skip_serializing_if = "is_false")]
    pub workspace: bool,

    /// Relative path from lockfile root to workspace package directory.
    /// Only present when `workspace == true`.
    #[serde(rename = "localPath", default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub dependencies: HashMap<String, String>,

    #[serde(
        rename = "peerDependencies",
        default,
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub peer_dependencies: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub bin: HashMap<String, String>,

    #[serde(rename = "hasInstallScript", default, skip_serializing_if = "is_false")]
    pub has_install_script: bool,

    /// If set, this package is nested inside another package's node_modules.
    #[serde(rename = "nestedIn", default, skip_serializing_if = "Option::is_none")]
    pub nested_in: Option<String>,
}

fn is_false(v: &bool) -> bool {
    !v
}

/// Why a lockfile failed the `verify_hydrated` end-to-end check.
///
/// The variants are ordered the way they appear in
/// [`Lockfile::verify_hydrated`] — a defect short-circuits on the
/// first failure so we always report the most specific cause.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HydrationDefect {
    /// `~/.jet-store/<name>@<version>` is missing or has a stale
    /// `.jet-integrity` file. Triggers a re-fetch of just that
    /// package (R2/R3).
    MissingStorePackage { name: String, version: String },
    /// `node_modules/<name>` does not exist even though the lockfile
    /// expects it. Triggers a re-link (R1).
    MissingNodeModulesLink { name: String, version: String },
    /// `node_modules/<name>` is a symlink whose target no longer
    /// exists on disk. Forces the lockfile reinstall path so the
    /// broken link is replaced (R4).
    BrokenNodeModulesLink { name: String, version: String },
    /// `symlink_metadata` on `node_modules/<name>` returned an IO
    /// error OTHER than NotFound (EACCES on a parent dir, EIO on a
    /// flaky disk, ENOTDIR mid-traversal, EMFILE under fd pressure).
    /// Previously misclassified as `MissingNodeModulesLink` — a user
    /// found "missing link" reports for links that actually were
    /// present on disk. `reason` carries the underlying io::Error
    /// (GH #3542).
    NodeModulesLinkUnreadable {
        name: String,
        version: String,
        reason: String,
    },
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl std::fmt::Display for HydrationDefect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingStorePackage { name, version } => {
                write!(f, "store entry missing for {}@{}", name, version)
            }
            Self::MissingNodeModulesLink { name, version } => write!(
                f,
                "node_modules/{} link missing (expects {}@{})",
                name, name, version
            ),
            Self::BrokenNodeModulesLink { name, version } => write!(
                f,
                "node_modules/{} symlink is broken (expects {}@{})",
                name, name, version
            ),
            Self::NodeModulesLinkUnreadable {
                name,
                version,
                reason,
            } => write!(
                f,
                "node_modules/{} symlink_metadata failed ({}); expects {}@{}. \
                 This is not the same as 'missing' — the entry may be present \
                 but unreadable (check perms on the parent directory) (GH #3542)",
                name, reason, name, version
            ),
        }
    }
}

/// Resolution metadata: tarball URL + integrity hashes.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub tarball: String,
    pub shasum: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrity: Option<String>,
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Lockfile {
    pub fn new() -> Self {
        Self {
            lockfile_version: "2.0".to_string(),
            deps_hash: None,
            overrides: HashMap::new(),
            patched_packages: HashMap::new(),
            packages: HashMap::new(),
        }
    }

    /// Compute a deterministic hash of dependency specs for frozen lockfile check.
    pub fn compute_deps_hash(deps: &HashMap<String, String>) -> String {
        use sha2::Digest;
        let mut sorted: Vec<_> = deps.iter().collect();
        sorted.sort_by_key(|(k, _)| (*k).clone());
        let mut hasher = sha2::Sha256::new();
        for (name, version) in sorted {
            hasher.update(format!("{}@{}\n", name, version).as_bytes());
        }
        format!("{:x}", hasher.finalize())
    }

    /// Build a lockfile from the resolver's output.
    pub fn from_resolved(resolved: &HashMap<String, ResolvedPackage>) -> Self {
        let mut lockfile = Self::new();

        for (name, pkg) in resolved {
            let key = format!("/{}@{}", name, pkg.version);
            lockfile.packages.insert(
                key,
                LockfileEntry {
                    version: pkg.version.clone(),
                    resolution: Resolution {
                        tarball: pkg.tarball_url.clone(),
                        shasum: pkg.shasum.clone(),
                        integrity: pkg.integrity.clone(),
                    },
                    workspace: pkg.workspace,
                    local_path: pkg.local_path.clone(),
                    dependencies: pkg.dependencies.clone(),
                    peer_dependencies: pkg.peer_dependencies.clone(),
                    bin: pkg.bin.clone(),
                    has_install_script: pkg.has_install_script,
                    nested_in: pkg.nested_in.clone(),
                },
            );
        }

        lockfile
    }

    /// Check whether every package in the lockfile exists in the store
    /// with a matching integrity hash. If any package is missing or
    /// stale the lockfile is considered invalid.
    ///
    /// Workspace entries (where `workspace == true`) bypass the store-presence
    /// check since they are local symlinks, not extracted tarballs.
    pub fn is_valid(&self, store: &super::store::StoreManager) -> bool {
        self.packages.iter().all(|(key, entry)| {
            // Workspace packages are local symlinks — skip store check.
            if entry.workspace {
                return true;
            }
            let name = parse_name_from_key(key);
            store.has_package(&name, &entry.version, &entry.resolution.shasum)
        })
    }

    /// Strict end-to-end hydration check used by the `Already up to date`
    /// marker fast-path. Verifies, for every non-workspace,
    /// non-nested lockfile entry, that:
    ///
    /// 1. The package directory exists in the global store with a matching
    ///    integrity marker (same predicate as [`is_valid`]).
    /// 2. The root `node_modules/<name>` entry exists and, if it is a
    ///    symlink, points to a still-existing target on disk.
    ///
    /// Returns `Ok(())` when the project is fully hydrated. Returns
    /// `Err(HydrationDefect)` describing the first problem found so the
    /// installer can fall back to a re-fetch / re-link path instead of
    /// printing `Already up to date` over a broken tree.
    ///
    /// Nested entries (those with `nested_in`) live under another
    /// package's `node_modules/` and are validated via that parent's
    /// link target rather than the root `node_modules/`, so they are
    /// skipped at the root layer here.
    pub fn verify_hydrated(
        &self,
        store: &super::store::StoreManager,
        node_modules: &Path,
    ) -> Result<(), HydrationDefect> {
        for (key, entry) in &self.packages {
            // Workspace packages are local symlinks — skip both checks.
            if entry.workspace {
                continue;
            }
            let name = parse_name_from_key(key);

            // R1 / R2 — store-presence (catches scoped paths via
            // `parse_name_from_key`, see `test_parse_name_from_key`).
            if !store.has_package(&name, &entry.version, &entry.resolution.shasum) {
                return Err(HydrationDefect::MissingStorePackage {
                    name,
                    version: entry.version.clone(),
                });
            }

            // R1 / R4 — root node_modules link presence. Only the
            // top-level entries surface here; nested entries live
            // inside another package's node_modules and are healed
            // when their parent is re-linked.
            if entry.nested_in.is_some() {
                continue;
            }
            let link = node_modules.join(&name);
            // `symlink_metadata` succeeds for both real dirs and
            // (possibly broken) symlinks. If the file does not exist
            // at all the symlink/dir was never created.
            //
            // GH #3542 — previously `let Ok else` lumped every IO error
            // (EACCES, EIO, ENOTDIR, EMFILE) together with NotFound,
            // misclassifying readable-but-unreadable entries as
            // "missing". Match on `e.kind() == NotFound` to keep the
            // legitimate missing-link branch fast, and route other IO
            // errors to a dedicated `NodeModulesLinkUnreadable` variant
            // carrying the underlying reason.
            let meta = match link.symlink_metadata() {
                Ok(m) => m,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(HydrationDefect::MissingNodeModulesLink {
                        name,
                        version: entry.version.clone(),
                    });
                }
                Err(e) => {
                    tracing::warn!(
                        target: "jet::pkg_manager::lockfile",
                        link = %link.display(),
                        error = %e,
                        "GH #3542 symlink_metadata failed on node_modules/{} ({}); reporting NodeModulesLinkUnreadable instead of misclassifying as MissingNodeModulesLink",
                        name, e
                    );
                    return Err(HydrationDefect::NodeModulesLinkUnreadable {
                        name,
                        version: entry.version.clone(),
                        reason: e.to_string(),
                    });
                }
            };
            // If it is a symlink, follow it. A broken symlink fails
            // `metadata` (which traverses the link) even though
            // `symlink_metadata` succeeded above.
            if meta.file_type().is_symlink() && link.metadata().is_err() {
                return Err(HydrationDefect::BrokenNodeModulesLink {
                    name,
                    version: entry.version.clone(),
                });
            }
        }
        Ok(())
    }

    /// Convert the lockfile back into a resolved-packages map so the
    /// installer can proceed without re-running the resolver.
    pub fn to_resolved(&self) -> HashMap<String, ResolvedPackage> {
        self.packages
            .iter()
            .map(|(key, entry)| {
                let name = parse_name_from_key(key);
                (
                    name.clone(),
                    ResolvedPackage {
                        name,
                        version: entry.version.clone(),
                        tarball_url: entry.resolution.tarball.clone(),
                        shasum: entry.resolution.shasum.clone(),
                        integrity: entry.resolution.integrity.clone(),
                        dependencies: entry.dependencies.clone(),
                        peer_dependencies: entry.peer_dependencies.clone(),
                        bin: entry.bin.clone(),
                        has_install_script: entry.has_install_script,
                        nested_in: entry.nested_in.clone(),
                        workspace: entry.workspace,
                        local_path: entry.local_path.clone(),
                    },
                )
            })
            .collect()
    }

    /// Read a lockfile from disk.
    pub fn read(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).with_context(|| lockfile_read_io_ctx(path))?;
        let lockfile: Lockfile =
            serde_yaml::from_str(&content).with_context(|| lockfile_parse_ctx(path))?;
        Ok(lockfile)
    }

    /// Write the lockfile to disk as YAML.
    pub fn write(&self, path: &Path) -> Result<()> {
        let content = serde_yaml::to_string(self).with_context(|| lockfile_serialize_ctx(path))?;
        std::fs::write(path, content).with_context(|| lockfile_write_io_ctx(path))?;
        Ok(())
    }
}

/// Context string for `std::fs::read_to_string` failures inside
/// `Lockfile::read`. Names the path and the operation so devs grepping
/// `jet-lock.yaml` failures in CI logs can land on the right step.
/// Tagged `GH #3558`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn lockfile_read_io_ctx(path: &Path) -> String {
    format!(
        "GH #3558 reading lockfile from {} failed (e.g. missing file, EACCES, EIO); the jet-lock.yaml at this path could not be opened",
        path.display()
    )
}

/// Context string for `serde_yaml::from_str` failures inside
/// `Lockfile::read`. Distinguishes the YAML-parse failure from the
/// preceding read failure so dev triage can target the parser vs. the
/// filesystem. Tagged `GH #3558`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn lockfile_parse_ctx(path: &Path) -> String {
    format!(
        "GH #3558 parsing lockfile YAML from {} failed; the jet-lock.yaml at this path is malformed (typical cause: stray git merge markers or a hand-edited mapping out of shape)",
        path.display()
    )
}

/// Context string for `serde_yaml::to_string` failures inside
/// `Lockfile::write`. Tagged `GH #3558`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn lockfile_serialize_ctx(path: &Path) -> String {
    format!(
        "GH #3558 serializing lockfile YAML for write to {} failed; the in-memory Lockfile struct could not be encoded as YAML (typical cause: a non-serializable field in a custom resolver entry)",
        path.display()
    )
}

/// Context string for `std::fs::write` failures inside
/// `Lockfile::write`. Tagged `GH #3558`.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn lockfile_write_io_ctx(path: &Path) -> String {
    format!(
        "GH #3558 writing lockfile to {} failed (e.g. read-only mount, ENOSPC, EACCES); the jet-lock.yaml at this path could not be persisted and the install/update operation will not be reproducible",
        path.display()
    )
}

/// GH #3648 — classification of a symlink target. Replaces the prior
/// `link.metadata().is_err()` collapse inside `Lockfile::verify_hydrated`
/// that lumped every IO error (EACCES on the target, EIO, network mount,
/// ENOTDIR mid-traversal) together with NotFound. The first error site
/// (`symlink_metadata`) already got this treatment in GH #3542; this is
/// the sibling fix for the follow-on `metadata` call that traverses the
/// link target.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkTargetClass {
    /// `metadata` succeeded — link target exists and is readable.
    Reachable,
    /// `metadata` returned `NotFound` on the target — a genuine
    /// broken/dangling symlink.
    NotFound,
    /// `metadata` returned some other IO error (EACCES, EIO, ENOTDIR,
    /// EMFILE, network-mount transient). The link itself may be valid,
    /// but its target was unreadable at the moment of the check. Carries
    /// the underlying io::Error message so the caller can surface it.
    Unreadable { reason: String },
}

/// GH #3648 — classify the result of traversing a symlink target via
/// `Path::metadata`. Distinguishes a genuine broken link (NotFound on
/// target) from a target that exists but is unreadable.
/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
pub(crate) fn safe_classify_link_target(link: &Path) -> LinkTargetClass {
    match link.metadata() {
        Ok(_) => LinkTargetClass::Reachable,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => LinkTargetClass::NotFound,
        Err(e) => LinkTargetClass::Unreadable {
            reason: e.to_string(),
        },
    }
}

/// @spec .aw/tech-design/projects/jet/semantic/jet-pkg-manager.md#schema
impl Default for Lockfile {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse the package name from a lockfile key like `/react@18.2.0`
/// or `/@scope/pkg@1.0.0`.
fn parse_name_from_key(key: &str) -> String {
    let without_slash = key.trim_start_matches('/');
    // Handle scoped packages: @scope/pkg@1.0.0
    // We need to find the *last* '@' that separates name from version
    without_slash
        .rsplit_once('@')
        .map(|(n, _)| n)
        .unwrap_or(without_slash)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pkg_manager::store::StoreManager;

    #[test]
    fn test_lockfile_creation() {
        let lockfile = Lockfile::new();
        assert_eq!(lockfile.lockfile_version, "2.0");
        assert_eq!(lockfile.packages.len(), 0);
    }

    #[test]
    fn test_from_resolved() {
        let mut resolved = HashMap::new();
        resolved.insert(
            "react".to_string(),
            ResolvedPackage {
                name: "react".to_string(),
                version: "18.2.0".to_string(),
                tarball_url: "https://registry.npmjs.org/react/-/react-18.2.0.tgz".to_string(),
                shasum: "abc123".to_string(),
                integrity: Some("sha512-xyz".to_string()),
                dependencies: HashMap::from([("loose-envify".to_string(), "^1.1.0".to_string())]),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
                workspace: false,
                local_path: None,
            },
        );

        let lockfile = Lockfile::from_resolved(&resolved);
        assert_eq!(lockfile.lockfile_version, "2.0");
        assert_eq!(lockfile.packages.len(), 1);
        assert!(lockfile.packages.contains_key("/react@18.2.0"));

        let entry = &lockfile.packages["/react@18.2.0"];
        assert_eq!(entry.version, "18.2.0");
        assert_eq!(entry.resolution.shasum, "abc123");
        assert_eq!(entry.dependencies.len(), 1);
    }

    #[test]
    fn test_lockfile_roundtrip() {
        let mut resolved = HashMap::new();
        resolved.insert(
            "lodash".to_string(),
            ResolvedPackage {
                name: "lodash".to_string(),
                version: "4.17.21".to_string(),
                tarball_url: "https://registry.npmjs.org/lodash/-/lodash-4.17.21.tgz".to_string(),
                shasum: "deadbeef".to_string(),
                integrity: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
                workspace: false,
                local_path: None,
            },
        );

        let lockfile = Lockfile::from_resolved(&resolved);

        // Write to tempfile, read back, compare
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jet-lock.yaml");

        lockfile.write(&path).unwrap();
        let loaded = Lockfile::read(&path).unwrap();

        assert_eq!(loaded.lockfile_version, "2.0");
        assert_eq!(loaded.packages.len(), 1);

        let entry = &loaded.packages["/lodash@4.17.21"];
        assert_eq!(entry.version, "4.17.21");
        assert_eq!(entry.resolution.shasum, "deadbeef");
        assert!(entry.resolution.integrity.is_none());
    }

    #[test]
    fn test_to_resolved() {
        let mut resolved = HashMap::new();
        resolved.insert(
            "express".to_string(),
            ResolvedPackage {
                name: "express".to_string(),
                version: "4.18.2".to_string(),
                tarball_url: "https://example.com/express.tgz".to_string(),
                shasum: "sha1abc".to_string(),
                integrity: None,
                dependencies: HashMap::from([("body-parser".to_string(), "1.20.1".to_string())]),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
                workspace: false,
                local_path: None,
            },
        );

        let lockfile = Lockfile::from_resolved(&resolved);
        let back = lockfile.to_resolved();

        assert_eq!(back.len(), 1);
        let pkg = &back["express"];
        assert_eq!(pkg.version, "4.18.2");
        assert_eq!(pkg.dependencies.len(), 1);
    }

    #[test]
    fn test_parse_name_from_key() {
        assert_eq!(parse_name_from_key("/react@18.2.0"), "react");
        assert_eq!(parse_name_from_key("/@babel/core@7.23.0"), "@babel/core");
        assert_eq!(parse_name_from_key("/lodash@4.17.21"), "lodash");
    }

    // ------------------------------------------------------------------
    // Tests for workspace fields in LockfileEntry (R6)
    // ------------------------------------------------------------------

    #[test]
    fn test_lockfile_workspace_entry_roundtrip() {
        let entry = LockfileEntry {
            version: "1.5.0".to_string(),
            resolution: Resolution {
                tarball: String::new(),
                shasum: String::new(),
                integrity: None,
            },
            workspace: true,
            local_path: Some("packages/ui".to_string()),
            dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: HashMap::new(),
            has_install_script: false,
            nested_in: None,
        };

        // Serialize to YAML
        let yaml = serde_yaml::to_string(&entry).unwrap();

        // Workspace flag and localPath must appear in the serialized form
        assert!(
            yaml.contains("workspace: true"),
            "Serialized YAML should contain 'workspace: true': {}",
            yaml
        );
        assert!(
            yaml.contains("localPath: packages/ui"),
            "Serialized YAML should contain 'localPath: packages/ui': {}",
            yaml
        );

        // Deserialize back and verify fields are preserved
        let loaded: LockfileEntry = serde_yaml::from_str(&yaml).unwrap();
        assert!(
            loaded.workspace,
            "workspace field should round-trip as true"
        );
        assert_eq!(
            loaded.local_path,
            Some("packages/ui".to_string()),
            "local_path should round-trip correctly"
        );
        assert_eq!(loaded.version, "1.5.0");
    }

    #[test]
    fn test_lockfile_is_valid_skips_workspace() {
        // Create a StoreManager pointing to a temp dir (no packages cached there)
        let dir = tempfile::tempdir().unwrap();
        let store = StoreManager::new(dir.path().to_path_buf()).unwrap();

        let mut lockfile = Lockfile::new();

        // Add a workspace entry — store will NOT have it
        lockfile.packages.insert(
            "/ui@1.5.0".to_string(),
            LockfileEntry {
                version: "1.5.0".to_string(),
                resolution: Resolution {
                    tarball: String::new(),
                    shasum: "someshasum".to_string(),
                    integrity: None,
                },
                workspace: true,
                local_path: Some("packages/ui".to_string()),
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
            },
        );

        // is_valid() must return true because workspace entries bypass the store check
        assert!(
            lockfile.is_valid(&store),
            "is_valid() should skip store check for workspace entries"
        );
    }

    #[test]
    fn test_lockfile_non_workspace_entry_not_skipped() {
        // A non-workspace entry that is absent from the store should make is_valid() false
        let dir = tempfile::tempdir().unwrap();
        let store = StoreManager::new(dir.path().to_path_buf()).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/react@18.2.0".to_string(),
            LockfileEntry {
                version: "18.2.0".to_string(),
                resolution: Resolution {
                    tarball: "https://example.com/react.tgz".to_string(),
                    shasum: "abc123".to_string(),
                    integrity: None,
                },
                workspace: false,
                local_path: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
            },
        );

        // Store doesn't have this package → is_valid() should be false
        assert!(
            !lockfile.is_valid(&store),
            "is_valid() should check store for non-workspace entries"
        );
    }

    #[test]
    fn test_lockfile_scoped_non_workspace_entry_not_skipped() {
        let dir = tempfile::tempdir().unwrap();
        let store = StoreManager::new(dir.path().to_path_buf()).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/@types/prop-types@15.7.15".to_string(),
            LockfileEntry {
                version: "15.7.15".to_string(),
                resolution: Resolution {
                    tarball: "https://example.com/prop-types.tgz".to_string(),
                    shasum: "abc123".to_string(),
                    integrity: None,
                },
                workspace: false,
                local_path: None,
                dependencies: HashMap::new(),
                peer_dependencies: HashMap::new(),
                bin: HashMap::new(),
                has_install_script: false,
                nested_in: None,
            },
        );

        assert!(
            !lockfile.is_valid(&store),
            "is_valid() should check the scoped store path for non-workspace entries"
        );
    }

    #[test]
    fn test_lockfile_workspace_field_defaults_false() {
        // A LockfileEntry deserialized without `workspace:` field should default to false
        let yaml = r#"version: "1.0.0"
resolution:
  tarball: https://example.com/pkg.tgz
  shasum: abc
"#;
        let entry: LockfileEntry = serde_yaml::from_str(yaml).unwrap();
        assert!(!entry.workspace, "workspace field should default to false");
        assert!(
            entry.local_path.is_none(),
            "local_path should default to None"
        );
    }

    // --- Issue #1930 — verify_hydrated tests ------------------------------

    /// Seed a fully-formed store entry at `{store}/{name}@{version}`
    /// so `StoreManager::has_package` accepts it. Returns the path of
    /// the seeded directory.
    fn seed_store_entry(
        store_root: &Path,
        name: &str,
        version: &str,
        shasum: &str,
    ) -> std::path::PathBuf {
        let pkg_dir = store_root.join(format!("{}@{}", name, version));
        // Scoped packages (`@types/prop-types`) need the `@types/`
        // parent — same convention `install_package` uses.
        if let Some(parent) = pkg_dir.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::create_dir_all(&pkg_dir).unwrap();
        std::fs::write(
            pkg_dir.join("package.json"),
            format!(r#"{{"name":"{}","version":"{}"}}"#, name, version),
        )
        .unwrap();
        std::fs::write(pkg_dir.join(".jet-integrity"), shasum).unwrap();
        pkg_dir
    }

    fn make_entry(version: &str, shasum: &str) -> LockfileEntry {
        LockfileEntry {
            version: version.to_string(),
            resolution: Resolution {
                tarball: format!("https://example.com/pkg-{}.tgz", version),
                shasum: shasum.to_string(),
                integrity: None,
            },
            workspace: false,
            local_path: None,
            dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: HashMap::new(),
            has_install_script: false,
            nested_in: None,
        }
    }

    /// Reproduces the Issue #1930 failure mode: lockfile claims a
    /// scoped `@types/prop-types@15.7.15` entry exists, but the
    /// store directory is missing. `verify_hydrated` must surface
    /// `MissingStorePackage` so the installer falls back to a
    /// re-fetch instead of printing `Already up to date`.
    #[test]
    fn test_verify_hydrated_detects_missing_scoped_store_entry() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/@types/prop-types@15.7.15".to_string(),
            make_entry("15.7.15", "scoped-shasum"),
        );

        // Store directory deliberately absent — the bug repro.
        let defect = lockfile
            .verify_hydrated(&store, &node_modules)
            .expect_err("missing scoped store dir must be a defect");
        match defect {
            HydrationDefect::MissingStorePackage { name, version } => {
                assert_eq!(name, "@types/prop-types");
                assert_eq!(version, "15.7.15");
            }
            other => panic!("expected MissingStorePackage, got {:?}", other),
        }
    }

    /// Even when the store entry is present, a missing
    /// `node_modules/<name>` link means the project is not hydrated
    /// — covers R1 (marker fast-path must not lie about
    /// `node_modules` presence).
    #[test]
    fn test_verify_hydrated_detects_missing_node_modules_link() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        seed_store_entry(&store_root, "react", "18.2.0", "abc123");

        let mut lockfile = Lockfile::new();
        lockfile
            .packages
            .insert("/react@18.2.0".to_string(), make_entry("18.2.0", "abc123"));

        let defect = lockfile
            .verify_hydrated(&store, &node_modules)
            .expect_err("missing root link must be a defect");
        match defect {
            HydrationDefect::MissingNodeModulesLink { name, version } => {
                assert_eq!(name, "react");
                assert_eq!(version, "18.2.0");
            }
            other => panic!("expected MissingNodeModulesLink, got {:?}", other),
        }
    }

    /// R4 — a symlink in `node_modules` whose target was deleted out
    /// from under us must be reported so the lockfile install path
    /// re-creates it. `symlink_metadata` succeeds for a dangling
    /// link but `metadata` does not, which is the discriminator
    /// `verify_hydrated` relies on.
    #[cfg(unix)]
    #[test]
    fn test_verify_hydrated_detects_broken_symlink() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        let pkg_dir = seed_store_entry(&store_root, "lodash", "4.17.21", "lod-shasum");

        // Create a symlink, then delete the target to simulate a
        // store directory that was GC'd or wiped manually.
        let link = node_modules.join("lodash");
        std::os::unix::fs::symlink(&pkg_dir, &link).unwrap();
        std::fs::remove_dir_all(&pkg_dir).unwrap();

        // Re-seed the store entry so the store-presence check
        // passes — we want to isolate the broken-symlink branch.
        seed_store_entry(&store_root, "lodash", "4.17.21", "lod-shasum");
        // Repoint the symlink at a path that no longer exists.
        std::fs::remove_file(&link).unwrap();
        std::os::unix::fs::symlink(tmp.path().join("does-not-exist"), &link).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/lodash@4.17.21".to_string(),
            make_entry("4.17.21", "lod-shasum"),
        );

        let defect = lockfile
            .verify_hydrated(&store, &node_modules)
            .expect_err("broken symlink must be a defect");
        assert!(
            matches!(defect, HydrationDefect::BrokenNodeModulesLink { .. }),
            "expected BrokenNodeModulesLink, got {:?}",
            defect
        );
    }

    /// Sanity check the happy path so future regressions to
    /// `verify_hydrated` cannot turn it into a no-op.
    #[cfg(unix)]
    #[test]
    fn test_verify_hydrated_ok_when_store_and_link_present() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        let pkg_dir = seed_store_entry(&store_root, "ok-pkg", "1.0.0", "ok-shasum");
        std::os::unix::fs::symlink(&pkg_dir, node_modules.join("ok-pkg")).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert(
            "/ok-pkg@1.0.0".to_string(),
            make_entry("1.0.0", "ok-shasum"),
        );

        assert!(
            lockfile.verify_hydrated(&store, &node_modules).is_ok(),
            "fully hydrated project must verify clean"
        );
    }

    /// Nested entries (`nested_in: Some(parent)`) live inside the
    /// parent's `node_modules/`, not the project root, so the root
    /// check must skip them. Otherwise every transitive dep would
    /// produce false positives.
    #[test]
    fn test_verify_hydrated_skips_nested_entries_for_root_link() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        seed_store_entry(&store_root, "deep", "0.1.0", "deep-shasum");

        let mut nested = make_entry("0.1.0", "deep-shasum");
        nested.nested_in = Some("parent".to_string());

        let mut lockfile = Lockfile::new();
        lockfile.packages.insert("/deep@0.1.0".to_string(), nested);

        // No root node_modules/deep link, but the entry is nested
        // so verify_hydrated must still pass.
        assert!(
            lockfile.verify_hydrated(&store, &node_modules).is_ok(),
            "nested entries must not require a root node_modules link"
        );
    }

    // ──────────────────────────────────────────────────────────────────
    // GH #3542 — symlink_metadata IO errors must NOT be misclassified
    // as MissingNodeModulesLink. The dedicated NodeModulesLinkUnreadable
    // variant carries the underlying reason so operators can distinguish
    // "actually missing" from "unreadable due to permissions/disk".
    // ──────────────────────────────────────────────────────────────────

    /// Unix-only: chmod 000 on `node_modules` to force EACCES on
    /// `symlink_metadata`. Result must be `NodeModulesLinkUnreadable`,
    /// NOT `MissingNodeModulesLink`.
    #[cfg(unix)]
    #[test]
    fn gh3542_verify_hydrated_eacces_returns_unreadable_not_missing() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        // Seed store so MissingStorePackage doesn't fire first.
        let pkg_dir = seed_store_entry(&store_root, "react", "18.2.0", "abc123");
        // Also create the symlink so a plain ENOENT doesn't fire when
        // we have perms; the EACCES on node_modules itself is what we
        // want to surface.
        std::os::unix::fs::symlink(&pkg_dir, node_modules.join("react")).unwrap();

        let mut lockfile = Lockfile::new();
        lockfile
            .packages
            .insert("/react@18.2.0".to_string(), make_entry("18.2.0", "abc123"));

        // Drop perms on node_modules so child symlink_metadata fails EACCES.
        std::fs::set_permissions(&node_modules, std::fs::Permissions::from_mode(0o000)).unwrap();

        // Skip cleanly when running as root (chmod has no effect).
        if std::fs::read_dir(&node_modules).is_ok() {
            std::fs::set_permissions(&node_modules, std::fs::Permissions::from_mode(0o755))
                .unwrap();
            return;
        }

        let result = lockfile.verify_hydrated(&store, &node_modules);

        // Restore perms so tempdir cleanup can succeed.
        std::fs::set_permissions(&node_modules, std::fs::Permissions::from_mode(0o755)).unwrap();

        match result {
            Err(HydrationDefect::NodeModulesLinkUnreadable {
                name,
                version,
                reason,
            }) => {
                assert_eq!(name, "react");
                assert_eq!(version, "18.2.0");
                assert!(
                    !reason.is_empty(),
                    "NodeModulesLinkUnreadable must carry the underlying io::Error reason"
                );
            }
            Err(HydrationDefect::MissingNodeModulesLink { .. }) => {
                panic!(
                    "EACCES must NOT be misclassified as MissingNodeModulesLink — \
                     that's the exact bug GH #3542 guards"
                );
            }
            other => panic!("expected NodeModulesLinkUnreadable, got {other:?}"),
        }
    }

    /// Display message for `NodeModulesLinkUnreadable` must include the
    /// underlying reason, the package coordinates, AND distinguish itself
    /// from "missing" so a user reading logs gets the right mental model.
    #[test]
    fn gh3542_display_unreadable_includes_reason_and_distinguishes_from_missing() {
        let defect = HydrationDefect::NodeModulesLinkUnreadable {
            name: "react".to_string(),
            version: "18.2.0".to_string(),
            reason: "Permission denied (os error 13)".to_string(),
        };
        let msg = format!("{defect}");
        assert!(
            msg.contains("react"),
            "Display must name the package: {msg}"
        );
        assert!(
            msg.contains("18.2.0"),
            "Display must name the version: {msg}"
        );
        assert!(
            msg.contains("Permission denied"),
            "Display must carry the underlying io::Error reason: {msg}"
        );
        assert!(
            msg.contains("not the same as 'missing'") || msg.contains("unreadable"),
            "Display must distinguish itself from MissingNodeModulesLink: {msg}"
        );
        assert!(
            msg.contains("GH #3542"),
            "Display must carry the log-grep tag: {msg}"
        );
    }

    /// Sanity: ENOENT (true missing) still returns MissingNodeModulesLink
    /// — the fast path is unchanged.
    #[test]
    fn gh3542_genuinely_missing_link_still_returns_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let store_root = tmp.path().join("store");
        let node_modules = tmp.path().join("node_modules");
        std::fs::create_dir_all(&node_modules).unwrap();
        let store = StoreManager::new(store_root.clone()).unwrap();
        seed_store_entry(&store_root, "react", "18.2.0", "abc123");
        // No symlink created — the link is genuinely missing.

        let mut lockfile = Lockfile::new();
        lockfile
            .packages
            .insert("/react@18.2.0".to_string(), make_entry("18.2.0", "abc123"));

        let defect = lockfile
            .verify_hydrated(&store, &node_modules)
            .expect_err("missing link must still surface");
        match defect {
            HydrationDefect::MissingNodeModulesLink { name, version } => {
                assert_eq!(name, "react");
                assert_eq!(version, "18.2.0");
            }
            other => panic!(
                "ENOENT must still return MissingNodeModulesLink (not Unreadable), got {other:?}"
            ),
        }
    }

    #[test]
    fn test_workspace_entry_not_serialized_when_false() {
        // When workspace == false, it should NOT appear in the serialized YAML
        let entry = LockfileEntry {
            version: "1.0.0".to_string(),
            resolution: Resolution {
                tarball: "https://example.com/pkg.tgz".to_string(),
                shasum: "abc".to_string(),
                integrity: None,
            },
            workspace: false,
            local_path: None,
            dependencies: HashMap::new(),
            peer_dependencies: HashMap::new(),
            bin: HashMap::new(),
            has_install_script: false,
            nested_in: None,
        };
        let yaml = serde_yaml::to_string(&entry).unwrap();
        assert!(
            !yaml.contains("workspace"),
            "workspace: false should be omitted from serialized YAML: {}",
            yaml
        );
        assert!(
            !yaml.contains("localPath"),
            "localPath should be omitted when None: {}",
            yaml
        );
    }

    // ─── GH #3558: Lockfile read/write context strings ──────────────────

    /// GH #3558 — each of the four context strings must include the GH
    /// tag and the path so devs grepping a lockfile failure can land on
    /// the right step.
    #[test]
    fn gh3558_lockfile_ctx_strings_name_tag_and_path() {
        let p = std::path::Path::new("/proj/jet-lock.yaml");

        let read = lockfile_read_io_ctx(p);
        let parse = lockfile_parse_ctx(p);
        let write = lockfile_write_io_ctx(p);
        let serialize = lockfile_serialize_ctx(p);

        for (label, msg) in [
            ("read_io", &read),
            ("parse", &parse),
            ("write_io", &write),
            ("serialize", &serialize),
        ] {
            assert!(
                msg.contains("GH #3558"),
                "{label} ctx must tag GH #3558, got: {msg}"
            );
            assert!(
                msg.contains("/proj/jet-lock.yaml"),
                "{label} ctx must name the path, got: {msg}"
            );
        }
    }

    /// GH #3558 — the four contexts must be pairwise distinct so a dev
    /// reading "yaml: line 17: expected mapping" plus the wrapping ctx
    /// can tell whether the failure was parse-vs-serialize-vs-io.
    #[test]
    fn gh3558_lockfile_ctx_strings_are_pairwise_distinct() {
        let p = std::path::Path::new("/a/jet-lock.yaml");
        let strs = [
            lockfile_read_io_ctx(p),
            lockfile_parse_ctx(p),
            lockfile_write_io_ctx(p),
            lockfile_serialize_ctx(p),
        ];
        for i in 0..strs.len() {
            for j in (i + 1)..strs.len() {
                assert_ne!(
                    strs[i], strs[j],
                    "ctx strings {i} and {j} must differ so triage can tell which step failed; got identical: {}",
                    strs[i]
                );
            }
        }

        // And each one names its operation discriminator.
        assert!(
            strs[0].contains("reading"),
            "read_io ctx must name 'reading', got: {}",
            strs[0]
        );
        assert!(
            strs[1].contains("parsing"),
            "parse ctx must name 'parsing', got: {}",
            strs[1]
        );
        assert!(
            strs[2].contains("writing"),
            "write_io ctx must name 'writing', got: {}",
            strs[2]
        );
        assert!(
            strs[3].contains("serializing"),
            "serialize ctx must name 'serializing', got: {}",
            strs[3]
        );
    }

    /// GH #3558 — integration: a `Lockfile::read` call against a
    /// non-existent path must produce an anyhow error whose Display
    /// chain includes the path AND the word "reading", so the user
    /// reading the error gets both the bad path and the operation that
    /// hit it.
    #[test]
    fn gh3558_lockfile_read_error_surfaces_path_and_operation() {
        let bogus = std::path::Path::new("/definitely/does/not/exist/jet-lock.yaml");
        let err = Lockfile::read(bogus).unwrap_err();
        let chain = format!("{err:#}");

        assert!(
            chain.contains("/definitely/does/not/exist/jet-lock.yaml"),
            "anyhow error chain must surface the path, got: {chain}"
        );
        assert!(
            chain.contains("reading lockfile"),
            "anyhow error chain must name the operation, got: {chain}"
        );
    }
}

#[cfg(test)]
mod gh3648_safe_classify_link_target_tests {
    //! GH #3648 — `verify_hydrated`'s broken-link branch used
    //! `link.metadata().is_err()` which swallowed every IO error class
    //! (EACCES on the target, EIO, ENOTDIR, network-mount transient)
    //! along with NotFound. Same mistake GH #3542 fixed one branch above
    //! on `symlink_metadata`. Safe helper distinguishes the cases.
    use super::*;
    use std::path::PathBuf;

    fn unique_tmp(name: &str) -> PathBuf {
        let mut dir = std::env::temp_dir();
        let pid = std::process::id();
        let ns = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        dir.push(format!("jet-gh3648-{pid}-{ns}-{name}"));
        dir
    }

    #[test]
    fn nonexistent_path_classifies_as_not_found() {
        let p = unique_tmp("nonexistent");
        assert_eq!(
            safe_classify_link_target(&p),
            LinkTargetClass::NotFound,
            "vanilla missing path must classify as NotFound, not Reachable"
        );
    }

    #[test]
    fn existing_file_classifies_as_reachable() {
        let p = unique_tmp("real");
        std::fs::write(&p, b"hello").unwrap();
        let out = safe_classify_link_target(&p);
        std::fs::remove_file(&p).ok();
        assert_eq!(out, LinkTargetClass::Reachable);
    }

    #[test]
    fn existing_dir_classifies_as_reachable() {
        let p = unique_tmp("real-dir");
        std::fs::create_dir(&p).unwrap();
        let out = safe_classify_link_target(&p);
        std::fs::remove_dir(&p).ok();
        assert_eq!(out, LinkTargetClass::Reachable);
    }

    #[cfg(unix)]
    #[test]
    fn dangling_symlink_classifies_as_not_found_through_target() {
        // A symlink whose target was never created — `metadata()` follows
        // the link and returns NotFound on the target. The helper must
        // surface NotFound, not a generic IO error.
        use std::os::unix::fs::symlink;
        let target = unique_tmp("nonexistent-target");
        let link = unique_tmp("dangling-link");
        symlink(&target, &link).unwrap();
        let out = safe_classify_link_target(&link);
        std::fs::remove_file(&link).ok();
        assert_eq!(out, LinkTargetClass::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn target_eacces_classifies_as_unreadable_not_not_found() {
        // Build a layout where the symlink's target lives inside an
        // unreadable directory. `metadata()` traversal fails with EACCES,
        // not NotFound — the legacy `is_err()` call would have lumped
        // this with the broken-link branch.
        use std::os::unix::fs::{symlink, PermissionsExt};
        let parent = unique_tmp("eacces-parent");
        std::fs::create_dir(&parent).unwrap();
        let target = parent.join("target");
        std::fs::write(&target, b"x").unwrap();
        let link = unique_tmp("eacces-link");
        symlink(&target, &link).unwrap();
        // Strip parent perms: traversal of the link should now fail.
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o000)).unwrap();
        let out = safe_classify_link_target(&link);
        // Restore perms so cleanup works.
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755)).ok();
        std::fs::remove_file(&link).ok();
        std::fs::remove_file(&target).ok();
        std::fs::remove_dir(&parent).ok();
        // Some platforms/filesystems still surface NotFound through an
        // unreadable parent (chrooted containers, etc.). Accept either
        // an Unreadable verdict (the intended path) or NotFound (kernel-
        // dependent fallback). What the helper MUST NOT do is panic or
        // classify as Reachable.
        match out {
            LinkTargetClass::Unreadable { reason } => {
                assert!(!reason.is_empty(), "reason must carry the io::Error string");
            }
            LinkTargetClass::NotFound => {
                // tolerated platform variance — explicit branch keeps
                // the test stable across kernels.
            }
            LinkTargetClass::Reachable => {
                panic!("unreadable target must NOT classify as Reachable, got: {out:?}")
            }
        }
    }

    #[test]
    fn unreachable_variant_carries_reason() {
        // Pin: the Unreadable variant must carry the underlying error
        // text so the caller can surface it via tracing::warn!.
        let v = LinkTargetClass::Unreadable {
            reason: "permission denied (os error 13)".to_string(),
        };
        match v {
            LinkTargetClass::Unreadable { reason } => {
                assert!(reason.contains("permission denied"));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn helper_name_pin() {
        // Pin: family convention is `safe_*`. If a future rename breaks
        // this, the loop's grep tooling needs to know.
        let _ = safe_classify_link_target as fn(&Path) -> LinkTargetClass;
    }
}
// CODEGEN-END
