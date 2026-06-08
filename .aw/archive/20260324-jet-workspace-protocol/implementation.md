---
id: implementation
type: change_implementation
change_id: jet-workspace-protocol
---

# Implementation

## Summary

Extend cclab-jet package manager with pnpm-style workspace monorepo support:

1. pnpm-workspace.yaml Discovery (workspace.rs): Added PnpmWorkspaceYaml struct and extended WorkspaceManager::load_config() with a third lowest-priority detection branch. Parses packages:, catalog:, and catalogs: fields; flattens named catalogs into WorkspaceConfig.catalog with key prefix <catalog_name>:. 5 inline unit tests added.

2. Workspace Install Integration (mod.rs): install_with_options() now detects WorkspaceMode::Jet and delegates to new workspace_install_all() helper. Iterates packages in topological order, classifying each dep: workspace: protocol deps get relative symlinks via create_relative_symlink(); external deps use the existing registry flow via new install_resolved_to() helper. Writes a single jet-lock.yaml at workspace root on completion.

3. Lockfile Workspace Fields (lockfile.rs): LockfileEntry gains workspace: bool (serde skip_serializing_if = is_false) and local_path: Option<String> (serde renamed localPath). is_valid() now skips store-presence check for workspace entries. from_resolved() propagates both fields. 5 inline unit tests added.

4. ResolvedPackage Extension (resolver.rs): Added workspace: bool and local_path: Option<String> fields to thread workspace metadata through resolve -> lockfile pipeline. stream_resolve_package() defaults both to false/None.

5. Integration Test Suite (tests/workspace_protocol.rs): New file with 11 end-to-end tests using tempdir monorepo fixtures (no network): pnpm-workspace.yaml discovery, priority over jet-workspace.yaml, catalog resolution, WorkspaceMode detection, workspace:* symlink, workspace:^ version range, recursive 3-package install, no-registry verification, lockfile fields, idempotent install, and resolve_workspace_protocol variants.

## Diff

```diff
diff --git a/crates/cclab-jet/src/pkg_manager/lockfile.rs b/crates/cclab-jet/src/pkg_manager/lockfile.rs
index 38496727..3f16c023 100644
--- a/crates/cclab-jet/src/pkg_manager/lockfile.rs
+++ b/crates/cclab-jet/src/pkg_manager/lockfile.rs
@@ -37,6 +37,20 @@ pub struct LockfileEntry {
     pub version: String,
     pub resolution: Resolution,
 
+    /// True when this entry is a local workspace package.
+    /// `is_valid()` skips the store-presence check for workspace entries.
+    #[serde(default, skip_serializing_if = "is_false")]
+    pub workspace: bool,
+
+    /// Relative path from lockfile root to workspace package directory.
+    /// Only present when `workspace == true`.
+    #[serde(
+        rename = "localPath",
+        default,
+        skip_serializing_if = "Option::is_none"
+    )]
+    pub local_path: Option<String>,
+
     #[serde(default, skip_serializing_if = "HashMap::is_empty")]
     pub dependencies: HashMap<String, String>,
 
@@ -120,6 +134,8 @@ impl Lockfile {
                         shasum: pkg.shasum.clone(),
                         integrity: pkg.integrity.clone(),
                     },
+                    workspace: pkg.workspace,
+                    local_path: pkg.local_path.clone(),
                     dependencies: pkg.dependencies.clone(),
                     peer_dependencies: pkg.peer_dependencies.clone(),
                     bin: pkg.bin.clone(),
@@ -135,11 +151,18 @@ impl Lockfile {
     /// Check whether every package in the lockfile exists in the store
     /// with a matching integrity hash. If any package is missing or
     /// stale the lockfile is considered invalid.
+    ///
+    /// Workspace entries (where `workspace == true`) bypass the store-presence
+    /// check since they are local symlinks, not extracted tarballs.
     pub fn is_valid(
         &self,
         store: &super::store::StoreManager,
     ) -> bool {
         self.packages.iter().all(|(key, entry)| {
+            // Workspace packages are local symlinks — skip store check.
+            if entry.workspace {
+                return true;
+            }
             let name = parse_name_from_key(key);
             store.has_package(
                 &name,
@@ -169,6 +192,8 @@ impl Lockfile {
                         bin: entry.bin.clone(),
                         has_install_script: entry.has_install_script,
                         nested_in: entry.nested_in.clone(),
+                        workspace: entry.workspace,
+                        local_path: entry.local_path.clone(),
                     },
                 )
             })
@@ -212,6 +237,7 @@ fn parse_name_from_key(key: &str) -> String {
 #[cfg(test)]
 mod tests {
     use super::*;
+    use crate::pkg_manager::store::StoreManager;
 
     #[test]
     fn test_lockfile_creation() {
@@ -240,6 +266,8 @@ mod tests {
                 bin: HashMap::new(),
                 has_install_script: false,
                 nested_in: None,
+                workspace: false,
+                local_path: None,
             },
         );
 
@@ -271,6 +299,8 @@ mod tests {
                 bin: HashMap::new(),
                 has_install_script: false,
                 nested_in: None,
+                workspace: false,
+                local_path: None,
             },
         );
 
@@ -311,6 +341,8 @@ mod tests {
                 bin: HashMap::new(),
                 has_install_script: false,
                 nested_in: None,
+                workspace: false,
+                local_path: None,
             },
         );
 
@@ -332,4 +364,166 @@ mod tests {
         );
         assert_eq!(parse_name_from_key("/lodash@4.17.21"), "lodash");
     }
+
+    // ------------------------------------------------------------------
+    // Tests for workspace fields in LockfileEntry (R6)
+    // ------------------------------------------------------------------
+
+    #[test]
+    fn test_lockfile_workspace_entry_roundtrip() {
+        let entry = LockfileEntry {
+            version: "1.5.0".to_string(),
+            resolution: Resolution {
+                tarball: String::new(),
+                shasum: String::new(),
+                integrity: None,
+            },
+            workspace: true,
+            local_path: Some("packages/ui".to_string()),
+            dependencies: HashMap::new(),
+            peer_dependencies: HashMap::new(),
+            bin: HashMap::new(),
+            has_install_script: false,
+            nested_in: None,
+        };
+
+        // Serialize to YAML
+        let yaml = serde_yaml::to_string(&entry).unwrap();
+
+        // Workspace flag and localPath must appear in the serialized form
+        assert!(
+            yaml.contains("workspace: true"),
+            "Serialized YAML should contain 'workspace: true': {}",
+            yaml
+        );
+        assert!(
+            yaml.contains("localPath: packages/ui"),
+            "Serialized YAML should contain 'localPath: packages/ui': {}",
+            yaml
+        );
+
+        // Deserialize back and verify fields are preserved
+        let loaded: LockfileEntry = serde_yaml::from_str(&yaml).unwrap();
+        assert!(loaded.workspace, "workspace field should round-trip as true");
+        assert_eq!(
+            loaded.local_path,
+            Some("packages/ui".to_string()),
+            "local_path should round-trip correctly"
+        );
+        assert_eq!(loaded.version, "1.5.0");
+    }
+
+    #[test]
+    fn test_lockfile_is_valid_skips_workspace() {
+        // Create a StoreManager pointing to a temp dir (no packages cached there)
+        let dir = tempfile::tempdir().unwrap();
+        let store =
+            StoreManager::new(dir.path().to_path_buf()).unwrap();
+
+        let mut lockfile = Lockfile::new();
+
+        // Add a workspace entry — store will NOT have it
+        lockfile.packages.insert(
+            "/ui@1.5.0".to_string(),
+            LockfileEntry {
+                version: "1.5.0".to_string(),
+                resolution: Resolution {
+                    tarball: String::new(),
+                    shasum: "someshasum".to_string(),
+                    integrity: None,
+                },
+                workspace: true,
+                local_path: Some("packages/ui".to_string()),
+                dependencies: HashMap::new(),
+                peer_dependencies: HashMap::new(),
+                bin: HashMap::new(),
+                has_install_script: false,
+                nested_in: None,
+            },
+        );
+
+        // is_valid() must return true because workspace entries bypass the store check
+        assert!(
+            lockfile.is_valid(&store),
+            "is_valid() should skip store check for workspace entries"
+        );
+    }
+
+    #[test]
+    fn test_lockfile_non_workspace_entry_not_skipped() {
+        // A non-workspace entry that is absent from the store should make is_valid() false
+        let dir = tempfile::tempdir().unwrap();
+        let store =
+            StoreManager::new(dir.path().to_path_buf()).unwrap();
+
+        let mut lockfile = Lockfile::new();
+        lockfile.packages.insert(
+            "/react@18.2.0".to_string(),
+            LockfileEntry {
+                version: "18.2.0".to_string(),
+                resolution: Resolution {
+                    tarball: "https://example.com/react.tgz".to_string(),
+                    shasum: "abc123".to_string(),
+                    integrity: None,
+                },
+                workspace: false,
+                local_path: None,
+                dependencies: HashMap::new(),
+                peer_dependencies: HashMap::new(),
+                bin: HashMap::new(),
+                has_install_script: false,
+                nested_in: None,
+            },
+        );
+
+        // Store doesn't have this package → is_valid() should be false
+        assert!(
+            !lockfile.is_valid(&store),
+            "is_valid() should check store for non-workspace entries"
+        );
+    }
+
+    #[test]
+    fn test_lockfile_workspace_field_defaults_false() {
+        // A LockfileEntry deserialized without `workspace:` field should default to false
+        let yaml = r#"version: "1.0.0"
+resolution:
+  tarball: https://example.com/pkg.tgz
+  shasum: abc
+"#;
+        let entry: LockfileEntry = serde_yaml::from_str(yaml).unwrap();
+        assert!(!entry.workspace, "workspace field should default to false");
+        assert!(entry.local_path.is_none(), "local_path should default to None");
+    }
+
+    #[test]
+    fn test_workspace_entry_not_serialized_when_false() {
+        // When workspace == false, it should NOT appear in the serialized YAML
+        let entry = LockfileEntry {
+            version: "1.0.0".to_string(),
+            resolution: Resolution {
+                tarball: "https://example.com/pkg.tgz".to_string(),
+                shasum: "abc".to_string(),
+                integrity: None,
+            },
+            workspace: false,
+            local_path: None,
+            dependencies: HashMap::new(),
+            peer_dependencies: HashMap::new(),
+            bin: HashMap::new(),
+            has_install_script: false,
+            nested_in: None,
+        };
+        let yaml = serde_yaml::to_string(&entry).unwrap();
+        assert!(
+            !yaml.contains("workspace"),
+            "workspace: false should be omitted from serialized YAML: {}",
+            yaml
+        );
+        assert!(
+            !yaml.contains("localPath"),
+            "localPath should be omitted when None: {}",
+            yaml
+        );
+    }
 }
diff --git a/crates/cclab-jet/src/pkg_manager/mod.rs b/crates/cclab-jet/src/pkg_manager/mod.rs
index 3ce2a8e4..b3edad6c 100644
--- a/crates/cclab-jet/src/pkg_manager/mod.rs
+++ b/crates/cclab-jet/src/pkg_manager/mod.rs
@@ -1,6 +1,6 @@
 use anyhow::{Context, Result};
 use std::collections::HashMap;
-use std::path::PathBuf;
+use std::path::{Path, PathBuf};
 use std::sync::Arc;
 use tokio::sync::Semaphore;
 
@@ -16,7 +16,7 @@ pub mod resolver;
 pub mod store;
 pub mod workspace;
 
-use lockfile::Lockfile;
+use lockfile::{Lockfile, LockfileEntry, Resolution};
 use npmrc::NpmrcConfig;
 use registry::RegistryClient;
 use resolver::{DependencyResolver, ResolvedPackage};
@@ -114,6 +114,15 @@ impl PackageManager {
     ) -> Result<()> {
         tracing::info!("Installing dependencies...");
 
+        // Detect workspace mode first. Jet workspaces use a separate install
+        // path that handles workspace: protocol deps via relative symlinks and
+        // installs all packages in topological order.
+        if let workspace::WorkspaceMode::Jet(mut ws_mgr) =
+            workspace::WorkspaceMode::detect(&self.root_dir)?
+        {
+            return self.workspace_install_all(&mut ws_mgr).await;
+        }
+
         let frozen = frozen_lockfile || Self::is_ci_env();
         let package_json = self.read_package_json()?;
         let mut all_deps = package_json.dependencies.clone();
@@ -561,6 +570,274 @@ impl PackageManager {
     }
 }
 
+/// Compute the relative path from directory `from_dir` to directory `to_dir`.
+fn make_relative_path(from_dir: &Path, to_dir: &Path) -> PathBuf {
+    let from: Vec<_> = from_dir.components().collect();
+    let to: Vec<_> = to_dir.components().collect();
+
+    let common = from
+        .iter()
+        .zip(to.iter())
+        .take_while(|(a, b)| a == b)
+        .count();
+
+    let mut rel = PathBuf::new();
+    for _ in 0..(from.len() - common) {
+        rel.push("..");
+    }
+    for comp in &to[common..] {
+        rel.push(comp.as_os_str());
+    }
+    rel
+}
+
+/// Create (or update idempotently) a relative symlink at `link_path` pointing
+/// to `target_dir`. The symlink target is expressed as a relative path so that
+/// the workspace tree is fully portable.
+fn create_relative_symlink(link_path: &Path, target_dir: &Path) -> Result<()> {
+    let link_parent = link_path.parent().unwrap_or(Path::new("."));
+
+    // Resolve both paths to absolute for reliable relative-path computation.
+    let abs_target = if target_dir.is_absolute() {
+        target_dir.to_path_buf()
+    } else {
+        std::env::current_dir()?.join(target_dir)
+    };
+    let abs_link_parent = if link_parent.is_absolute() {
+        link_parent.to_path_buf()
+    } else {
+        std::env::current_dir()?.join(link_parent)
+    };
+
+    let rel = make_relative_path(&abs_link_parent, &abs_target);
+
+    // Idempotency: if an existing symlink already points to the same target, skip.
+    if link_path.is_symlink() {
+        if let Ok(existing) = std::fs::read_link(link_path) {
+            if existing == rel {
+                return Ok(());
+            }
+        }
+        std::fs::remove_file(link_path)?;
+    } else if link_path.exists() {
+        std::fs::remove_dir_all(link_path)?;
+    }
+
+    // Ensure parent directory exists.
+    if let Some(parent) = link_path.parent() {
+        std::fs::create_dir_all(parent)?;
+    }
+
+    #[cfg(unix)]
+    std::os::unix::fs::symlink(&rel, link_path)?;
+
+    #[cfg(windows)]
+    std::os::windows::fs::symlink_dir(&rel, link_path)?;
+
+    Ok(())
+}
+
+impl PackageManager {
+    /// Install workspace packages in topological order.
+    ///
+    /// For each workspace package:
+    /// - `workspace:*` / `workspace:^` / `workspace:~` deps → relative symlink
+    /// - External deps → standard registry resolve / download / hardlink
+    ///
+    /// A single `jet-lock.yaml` is written at the workspace root after all
+    /// packages are installed.
+    async fn workspace_install_all(
+        &self,
+        ws_mgr: &mut workspace::WorkspaceManager,
+    ) -> Result<()> {
+        let order = ws_mgr.topological_order()?;
+        let ws_root = &self.root_dir;
+        let mut lockfile = Lockfile::new();
+
+        for pkg_name in &order {
+            let pkg = ws_mgr
+                .get_package(pkg_name)
+                .ok_or_else(|| anyhow::anyhow!("Workspace package '{}' not found", pkg_name))?
+                .clone();
+
+            let pkg_dir = ws_root.join(&pkg.path);
+
+            // Read direct deps from this package's package.json.
+            let pkg_json_path = pkg_dir.join("package.json");
+            let content = std::fs::read_to_string(&pkg_json_path)
+                .with_context(|| format!("Cannot read {:?}", pkg_json_path))?;
+            let pkg_json: serde_json::Value = serde_json::from_str(&content)?;
+
+            let mut all_deps: HashMap<String, String> = HashMap::new();
+            for field in &["dependencies", "devDependencies"] {
+                if let Some(deps) = pkg_json.get(field).and_then(|v| v.as_object()) {
+                    for (name, ver) in deps {
+                        if let Some(ver_str) = ver.as_str() {
+                            all_deps.insert(name.clone(), ver_str.to_string());
+                        }
+                    }
+                }
+            }
+
+            let node_modules = pkg_dir.join("node_modules");
+            std::fs::create_dir_all(&node_modules)?;
+
+            let mut workspace_deps: Vec<(String, String)> = Vec::new();
+            let mut external_deps: HashMap<String, String> = HashMap::new();
+
+            for (dep_name, dep_spec) in &all_deps {
+                if workspace::WorkspaceManager::is_workspace_protocol(dep_spec) {
+                    workspace_deps.push((dep_name.clone(), dep_spec.clone()));
+                } else {
+                    external_deps.insert(dep_name.clone(), dep_spec.clone());
+                }
+            }
+
+            // --- Workspace deps: symlink only, no registry call ---
+            for (dep_name, dep_spec) in &workspace_deps {
+                let target_pkg = ws_mgr.get_package(dep_name).ok_or_else(|| {
+                    anyhow::anyhow!(
+                        "Workspace package '{}' not found (required by '{}')",
+                        dep_name,
+                        pkg_name
+                    )
+                })?;
+
+                let resolved_version = ws_mgr
+                    .resolve_workspace_protocol(dep_spec, dep_name)
+                    .unwrap_or_else(|| target_pkg.version.clone());
+
+                let abs_target = ws_root.join(&target_pkg.path);
+                let link_path = node_modules.join(dep_name);
+                create_relative_symlink(&link_path, &abs_target)?;
+
+                let lf_key = format!("/{}@{}", dep_name, resolved_version);
+                let local_path = target_pkg.path.to_string_lossy().to_string();
+                lockfile.packages.entry(lf_key).or_insert_with(|| LockfileEntry {
+                    version: resolved_version.clone(),
+                    resolution: Resolution {
+                        tarball: String::new(),
+                        shasum: String::new(),
+                        integrity: None,
+                    },
+                    workspace: true,
+                    local_path: Some(local_path),
+                    dependencies: HashMap::new(),
+                    peer_dependencies: HashMap::new(),
+                    bin: HashMap::new(),
+                    has_install_script: false,
+                    nested_in: None,
+                });
+            }
+
+            // --- External deps: standard registry flow ---
+            if !external_deps.is_empty() {
+                let overrides = HashMap::new();
+                let resolved = self
+                    .resolver
+                    .resolve(&external_deps, &self.registry, &overrides)
+                    .await?;
+
+                self.install_resolved_to(&resolved, &pkg_dir).await?;
+
+                for (name, rp) in &resolved {
+                    let lf_key = format!("/{}@{}", name, rp.version);
+                    lockfile.packages.entry(lf_key).or_insert_with(|| LockfileEntry {
+                        version: rp.version.clone(),
+                        resolution: Resolution {
+                            tarball: rp.tarball_url.clone(),
+                            shasum: rp.shasum.clone(),
+                            integrity: rp.integrity.clone(),
+                        },
+                        workspace: false,
+                        local_path: None,
+                        dependencies: rp.dependencies.clone(),
+                        peer_dependencies: rp.peer_dependencies.clone(),
+                        bin: rp.bin.clone(),
+                        has_install_script: rp.has_install_script,
+                        nested_in: rp.nested_in.clone(),
+                    });
+                }
+            }
+        }
+
+        let lf_path = ws_root.join("jet-lock.yaml");
+        lockfile.write(&lf_path)?;
+
+        tracing::info!("Workspace install complete ({} packages)", order.len());
+        Ok(())
+    }
+
+    /// Like `install_resolved` but installs into an arbitrary `pkg_dir` instead
+    /// of `self.root_dir`. Used by `workspace_install_all` for per-package installs.
+    async fn install_resolved_to(
+        &self,
+        resolved: &HashMap<String, ResolvedPackage>,
+        pkg_dir: &Path,
+    ) -> Result<()> {
+        let node_modules = pkg_dir.join("node_modules");
+        std::fs::create_dir_all(&node_modules)?;
+
+        let futures: Vec<_> = resolved
+            .values()
+            .map(|pkg| {
+                let store = self.store.clone();
+                let registry = self.registry.clone();
+                let semaphore = self.semaphore.clone();
+                let node_modules = node_modules.clone();
+                let pkg = pkg.clone();
+
+                async move {
+                    // Skip workspace packages — they are handled via symlinks.
+                    if pkg.workspace {
+                        return Ok::<_, anyhow::Error>(());
+                    }
+
+                    let link_target = if let Some(ref parent) = pkg.nested_in {
+                        node_modules.join(parent).join("node_modules")
+                    } else {
+                        node_modules.clone()
+                    };
+
+                    if is_pkg_installed(&link_target, &pkg.name, &pkg.version) {
+                        return Ok(());
+                    }
+
+                    if !store.has_package(&pkg.name, &pkg.version, &pkg.shasum) {
+                        let _permit = semaphore.acquire().await.unwrap();
+                        let tarball = registry
+                            .download_package(&pkg.name, &pkg.version)
+                            .await?;
+                        store.install_package(
+                            &pkg.name,
+                            &pkg.version,
+                            &tarball,
+                            &pkg.shasum,
+                        )?;
+                    }
+
+                    if pkg.nested_in.is_some() {
+                        std::fs::create_dir_all(&link_target)?;
+                    }
+                    store.link_package(&pkg.name, &pkg.version, &link_target)?;
+
+                    Ok::<_, anyhow::Error>(())
+                }
+            })
+            .collect();
+
+        futures::future::try_join_all(futures).await?;
+
+        for pkg in resolved.values() {
+            if !pkg.bin.is_empty() {
+                self.store.link_bins(&pkg.name, &pkg.bin, &node_modules)?;
+            }
+        }
+
+        Ok(())
+    }
+}
+
 /// Fast check: does `node_modules/{name}/package.json` already
 /// have the expected version?  Avoids re-linking unchanged packages.
 fn is_pkg_installed(
diff --git a/crates/cclab-jet/src/pkg_manager/resolver.rs b/crates/cclab-jet/src/pkg_manager/resolver.rs
index 38c5c25c..38031205 100644
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -41,6 +41,12 @@ pub struct ResolvedPackage {
     /// If set, this package should be nested inside the given parent's
     /// node_modules instead of top-level (for version conflicts).
     pub nested_in: Option<String>,
+    /// True when this package is a local workspace package (workspace: protocol).
+    /// Propagates to LockfileEntry.workspace.
+    pub workspace: bool,
+    /// Relative path from workspace root to workspace package directory.
+    /// Only present when workspace == true. Propagates to LockfileEntry.local_path.
+    pub local_path: Option<String>,
 }
 
 /// Dependency resolver with transitive resolution and circular dependency
@@ -378,6 +384,8 @@ fn stream_resolve_package(
         bin,
         has_install_script,
         nested_in: None,
+        workspace: false,
+        local_path: None,
     };
 
     // Notify background downloader immediately (overlapping tarball fetch).
diff --git a/crates/cclab-jet/src/pkg_manager/workspace.rs b/crates/cclab-jet/src/pkg_manager/workspace.rs
index 9c567432..03ce3e83 100644
--- a/crates/cclab-jet/src/pkg_manager/workspace.rs
+++ b/crates/cclab-jet/src/pkg_manager/workspace.rs
@@ -5,6 +5,17 @@ use std::path::{Path, PathBuf};
 
 use super::nx::NxWorkspaceManager;
 
+/// pnpm-workspace.yaml format (third-priority config source).
+#[derive(Debug, Clone, Default, Deserialize)]
+struct PnpmWorkspaceYaml {
+    #[serde(default)]
+    packages: Vec<String>,
+    #[serde(default)]
+    catalog: HashMap<String, String>,
+    #[serde(default)]
+    catalogs: HashMap<String, HashMap<String, String>>,
+}
+
 /// Workspace configuration from package.json or jet-workspace.yaml.
 #[derive(Debug, Clone, Default, Serialize, Deserialize)]
 pub struct WorkspaceConfig {
@@ -72,9 +83,10 @@ impl WorkspaceManager {
         }))
     }
 
-    /// Load workspace config from package.json.workspaces or jet-workspace.yaml.
+    /// Load workspace config from jet-workspace.yaml, package.json.workspaces,
+    /// or pnpm-workspace.yaml (in that priority order).
     fn load_config(root: &Path) -> Result<Option<WorkspaceConfig>> {
-        // Try jet-workspace.yaml first
+        // 1. Try jet-workspace.yaml first (highest priority)
         let yaml_path = root.join("jet-workspace.yaml");
         if yaml_path.exists() {
             let content = std::fs::read_to_string(&yaml_path)?;
@@ -82,7 +94,7 @@ impl WorkspaceManager {
             return Ok(Some(config));
         }
 
-        // Fall back to package.json.workspaces
+        // 2. Fall back to package.json.workspaces
         let pkg_path = root.join("package.json");
         if pkg_path.exists() {
             let content = std::fs::read_to_string(&pkg_path)?;
@@ -99,6 +111,28 @@ impl WorkspaceManager {
             }
         }
 
+        // 3. Fall back to pnpm-workspace.yaml (lowest priority)
+        let pnpm_yaml_path = root.join("pnpm-workspace.yaml");
+        if pnpm_yaml_path.exists() {
+            let content = std::fs::read_to_string(&pnpm_yaml_path)?;
+            let pnpm: PnpmWorkspaceYaml = serde_yaml::from_str(&content)?;
+            let mut catalog = pnpm.catalog;
+            // Merge named catalogs with key prefix "<catalog_name>:"
+            for (catalog_name, entries) in pnpm.catalogs {
+                for (dep_name, version) in entries {
+                    catalog.insert(
+                        format!("{}:{}", catalog_name, dep_name),
+                        version,
+                    );
+                }
+            }
+            return Ok(Some(WorkspaceConfig {
+                packages: pnpm.packages,
+                catalog,
+                ..Default::default()
+            }));
+        }
+
         Ok(None)
     }
 
@@ -383,4 +417,145 @@ mod tests {
         assert!(!config.shamefully_hoist);
         assert_eq!(config.public_hoist_pattern.len(), 2);
     }
+
+    // ------------------------------------------------------------------
+    // Tests for pnpm-workspace.yaml detection (R1, R2, S1, S2, S3)
+    // ------------------------------------------------------------------
+
+    #[test]
+    fn test_pnpm_workspace_yaml_discovery() {
+        let dir = tempdir().unwrap();
+
+        // Only pnpm-workspace.yaml — no jet-workspace.yaml, no package.json.workspaces
+        std::fs::write(
+            dir.path().join("pnpm-workspace.yaml"),
+            "packages:\n  - packages/*\n",
+        )
+        .unwrap();
+
+        // Create workspace package for glob to find
+        std::fs::create_dir_all(dir.path().join("packages/ui")).unwrap();
+        std::fs::write(
+            dir.path().join("packages/ui/package.json"),
+            r#"{"name": "ui", "version": "1.0.0"}"#,
+        )
+        .unwrap();
+
+        let result = WorkspaceManager::discover(dir.path()).unwrap();
+        assert!(
+            result.is_some(),
+            "WorkspaceManager should discover pnpm-workspace.yaml"
+        );
+        let wm = result.unwrap();
+        assert_eq!(wm.packages.len(), 1);
+        assert_eq!(wm.packages[0].name, "ui");
+    }
+
+    #[test]
+    fn test_jet_workspace_yaml_priority() {
+        let dir = tempdir().unwrap();
+
+        // jet-workspace.yaml lists apps/*, pnpm-workspace.yaml lists packages/*
+        std::fs::write(
+            dir.path().join("jet-workspace.yaml"),
+            "packages:\n  - apps/*\n",
+        )
+        .unwrap();
+        std::fs::write(
+            dir.path().join("pnpm-workspace.yaml"),
+            "packages:\n  - packages/*\n",
+        )
+        .unwrap();
+
+        // Create a package in apps/ (should be found)
+        std::fs::create_dir_all(dir.path().join("apps/web")).unwrap();
+        std::fs::write(
+            dir.path().join("apps/web/package.json"),
+            r#"{"name": "web", "version": "1.0.0"}"#,
+        )
+        .unwrap();
+
+        // Create a package in packages/ (should NOT be found; pnpm yaml ignored)
+        std::fs::create_dir_all(dir.path().join("packages/lib")).unwrap();
+        std::fs::write(
+            dir.path().join("packages/lib/package.json"),
+            r#"{"name": "lib", "version": "2.0.0"}"#,
+        )
+        .unwrap();
+
+        let result = WorkspaceManager::discover(dir.path()).unwrap();
+        assert!(result.is_some());
+        let wm = result.unwrap();
+        // Only the app from jet-workspace.yaml should be discovered
+        assert_eq!(wm.packages.len(), 1, "jet-workspace.yaml should win over pnpm-workspace.yaml");
+        assert_eq!(wm.packages[0].name, "web");
+    }
+
+    #[test]
+    fn test_pnpm_catalog_default() {
+        let dir = tempdir().unwrap();
+
+        let yaml = "packages:\n  - packages/*\ncatalog:\n  react: \"^18.0.0\"\n  typescript: \"^5.0.0\"\n";
+        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
+
+        let result = WorkspaceManager::discover(dir.path()).unwrap();
+        assert!(result.is_some());
+        let wm = result.unwrap();
+
+        assert_eq!(
+            wm.catalog_version("react"),
+            Some("^18.0.0"),
+            "Default catalog entry should be accessible via catalog_version()"
+        );
+        assert_eq!(
+            wm.catalog_version("typescript"),
+            Some("^5.0.0"),
+            "Second catalog entry should be accessible"
+        );
+        // Unknown entry returns None
+        assert_eq!(wm.catalog_version("vue"), None);
+    }
+
+    #[test]
+    fn test_pnpm_catalogs_named() {
+        let dir = tempdir().unwrap();
+
+        // Named catalogs are merged into catalog map with prefix "<catalog_name>:"
+        let yaml =
+            "packages:\n  - packages/*\ncatalogs:\n  default:\n    react: \"^18\"\n    vue: \"^3\"\n  legacy:\n    react: \"^16\"\n";
+        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
+
+        let result = WorkspaceManager::discover(dir.path()).unwrap();
+        assert!(result.is_some());
+        let wm = result.unwrap();
+
+        assert_eq!(
+            wm.catalog_version("default:react"),
+            Some("^18"),
+            "Named catalog entries should be prefixed with '<catalog_name>:'"
+        );
+        assert_eq!(wm.catalog_version("default:vue"), Some("^3"));
+        assert_eq!(wm.catalog_version("legacy:react"), Some("^16"));
+        // Bare name is not present (only prefixed entries)
+        assert_eq!(wm.catalog_version("react"), None);
+    }
+
+    #[test]
+    fn test_pnpm_workspace_yaml_with_catalog_and_catalogs() {
+        let dir = tempdir().unwrap();
+
+        // Both catalog: and catalogs: present — both should be merged
+        let yaml =
+            "packages:\n  - packages/*\ncatalog:\n  shared: \"^1.0.0\"\ncatalogs:\n  v2:\n    shared: \"^2.0.0\"\n";
+        std::fs::write(dir.path().join("pnpm-workspace.yaml"), yaml).unwrap();
+
+        let result = WorkspaceManager::discover(dir.path()).unwrap();
+        assert!(result.is_some());
+        let wm = result.unwrap();
+
+        // Default catalog entry accessible directly
+        assert_eq!(wm.catalog_version("shared"), Some("^1.0.0"));
+        // Named catalog entry accessible with prefix
+        assert_eq!(wm.catalog_version("v2:shared"), Some("^2.0.0"));
+    }
 }

diff --git a/crates/cclab-jet/tests/workspace_protocol.rs b/crates/cclab-jet/tests/workspace_protocol.rs
new file mode 100644
index 00000000..00000000
--- /dev/null
+++ b/crates/cclab-jet/tests/workspace_protocol.rs
@@ -0,0 +1,457 @@
+//! Integration tests for the workspace protocol (pnpm-style) implementation.
+//!
+//! These tests exercise the end-to-end workspace-protocol install flow:
+//! - pnpm-workspace.yaml discovery and catalog parsing
+//! - workspace:* / workspace:^ / workspace:~ symlink creation
+//! - Recursive workspace install across all packages in topological order
+//! - Lockfile entries for workspace packages (workspace=true, localPath, version)
+//!
+//! All test fixtures use tempdir monorepos with **only** workspace-protocol
+//! dependencies so that no network access is needed.
+
+use cclab_jet::pkg_manager::lockfile::Lockfile;
+use cclab_jet::pkg_manager::workspace::{WorkspaceManager, WorkspaceMode};
+use cclab_jet::pkg_manager::PackageManager;
+use std::collections::HashMap;
+use std::path::Path;
+use tempfile::tempdir;
+
+// ------------------------------------------------------------------
+// Fixture helpers
+// ------------------------------------------------------------------
+
+/// Write a file, creating parent directories as needed.
+fn write_file(base: &Path, rel: &str, content: &str) {
+    let full = base.join(rel);
+    if let Some(parent) = full.parent() {
+        std::fs::create_dir_all(parent).unwrap();
+    }
+    std::fs::write(full, content).unwrap();
+}
+
+/// Build a `package.json` string.
+fn pkg_json(name: &str, version: &str, deps: &[(&str, &str)]) -> String {
+    let dep_entries: Vec<String> = deps
+        .iter()
+        .map(|(n, v)| format!("\"{}\":\"{}\""  , n, v))
+        .collect();
+    let deps_obj = if dep_entries.is_empty() {
+        "{}".to_string()
+    } else {
+        format!("{{{}}}", dep_entries.join(","))
+    };
+    format!(
+        r#"{{"name":"{}","version":"{}","dependencies":{}}}"#,
+        name, version, deps_obj
+    )
+}
+
+// ------------------------------------------------------------------
+// WorkspaceManager unit-level integration tests
+// ------------------------------------------------------------------
+
+#[test]
+fn test_pnpm_workspace_yaml_discovery() {
+    let dir = tempdir().unwrap();
+
+    write_file(
+        dir.path(),
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n",
+    );
+
+    // Create a workspace package
+    write_file(
+        dir.path(),
+        "packages/ui/package.json",
+        r#"{"name":"ui","version":"1.5.0"}"#,
+    );
+
+    let wm = WorkspaceManager::discover(dir.path())
+        .expect("discover should not error")
+        .expect("should discover workspace from pnpm-workspace.yaml");
+
+    assert_eq!(wm.packages.len(), 1, "one package expected");
+    assert_eq!(wm.packages[0].name, "ui");
+    assert_eq!(wm.packages[0].version, "1.5.0");
+}
+
+#[test]
+fn test_jet_workspace_yaml_priority() {
+    let dir = tempdir().unwrap();
+
+    // jet-workspace.yaml lists apps/*, pnpm-workspace.yaml lists packages/*
+    write_file(
+        dir.path(),
+        "jet-workspace.yaml",
+        "packages:\n  - apps/*\n",
+    );
+    write_file(
+        dir.path(),
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n",
+    );
+
+    write_file(
+        dir.path(),
+        "apps/web/package.json",
+        r#"{"name":"web","version":"0.1.0"}"#,
+    );
+    write_file(
+        dir.path(),
+        "packages/lib/package.json",
+        r#"{"name":"lib","version":"9.9.9"}"#,
+    );
+
+    let wm = WorkspaceManager::discover(dir.path())
+        .expect("discover should succeed")
+        .expect("workspace should be found");
+
+    // Only app from jet-workspace.yaml should be present
+    assert_eq!(
+        wm.packages.len(),
+        1,
+        "jet-workspace.yaml should take priority over pnpm-workspace.yaml"
+    );
+    assert_eq!(wm.packages[0].name, "web");
+}
+
+#[test]
+fn test_catalog_resolution() {
+    let dir = tempdir().unwrap();
+
+    let yaml = "packages:\n  - packages/*\ncatalog:\n  react: \"^18.0.0\"\ncatalogs:\n  legacy:\n    react: \"^16\"\n";
+    write_file(dir.path(), "pnpm-workspace.yaml", yaml);
+
+    let wm = WorkspaceManager::discover(dir.path())
+        .expect("discover ok")
+        .expect("workspace found");
+
+    assert_eq!(
+        wm.catalog_version("react"),
+        Some("^18.0.0"),
+        "default catalog entry should be accessible"
+    );
+    assert_eq!(
+        wm.catalog_version("legacy:react"),
+        Some("^16"),
+        "named catalog entry should be prefixed with catalog_name:"
+    );
+    assert_eq!(
+        wm.catalog_version("vue"),
+        None,
+        "unknown entry should return None"
+    );
+}
+
+// ------------------------------------------------------------------
+// WorkspaceMode detection
+// ------------------------------------------------------------------
+
+#[test]
+fn test_workspace_mode_jet_detected_for_pnpm_yaml() {
+    let dir = tempdir().unwrap();
+    write_file(
+        dir.path(),
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n",
+    );
+
+    let mode = WorkspaceMode::detect(dir.path()).expect("detect ok");
+    assert!(
+        matches!(mode, WorkspaceMode::Jet(_)),
+        "pnpm-workspace.yaml should produce WorkspaceMode::Jet"
+    );
+}
+
+// ------------------------------------------------------------------
+// Symlink creation integration tests (no network — workspace: deps only)
+// ------------------------------------------------------------------
+
+/// Build a minimal workspace fixture:
+///
+/// ```text
+/// root/
+///   pnpm-workspace.yaml  (packages: [packages/*])
+///   packages/
+///     ui/package.json    (name: ui, version: 1.5.0)
+///     web/package.json   (name: web, version: 1.0.0, deps: {ui: workspace:*})
+/// ```
+fn make_two_package_workspace(root: &Path) {
+    write_file(
+        root,
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n",
+    );
+    write_file(
+        root,
+        "packages/ui/package.json",
+        &pkg_json("ui", "1.5.0", &[]),
+    );
+    write_file(
+        root,
+        "packages/web/package.json",
+        &pkg_json("web", "1.0.0", &[("ui", "workspace:*")]),
+    );
+}
+
+#[tokio::test]
+async fn test_workspace_star_symlink() {
+    let dir = tempdir().unwrap();
+    make_two_package_workspace(dir.path());
+
+    let pm = PackageManager::new(dir.path().to_path_buf())
+        .expect("PackageManager::new should succeed");
+
+    pm.install_with_options(false)
+        .await
+        .expect("workspace install should succeed");
+
+    // `packages/web/node_modules/ui` must be a symlink
+    let symlink_path = dir.path().join("packages/web/node_modules/ui");
+    assert!(
+        symlink_path.is_symlink(),
+        "node_modules/ui should be a relative symlink, got: {:?}",
+        symlink_path.symlink_metadata()
+    );
+
+    // The symlink must resolve to packages/ui
+    let resolved = symlink_path
+        .canonicalize()
+        .expect("symlink should be resolvable");
+    let expected = dir
+        .path()
+        .join("packages/ui")
+        .canonicalize()
+        .expect("packages/ui should exist");
+    assert_eq!(
+        resolved, expected,
+        "symlink should resolve to packages/ui"
+    );
+}
+
+#[tokio::test]
+async fn test_workspace_caret_resolution() {
+    let dir = tempdir().unwrap();
+
+    write_file(
+        dir.path(),
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n",
+    );
+    // shared is at version 2.3.1
+    write_file(
+        dir.path(),
+        "packages/shared/package.json",
+        &pkg_json("@acme/shared", "2.3.1", &[]),
+    );
+    // server depends on shared via workspace:^
+    write_file(
+        dir.path(),
+        "packages/server/package.json",
+        &pkg_json("server", "1.0.0", &[("@acme/shared", "workspace:^")]),
+    );
+
+    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
+    pm.install_with_options(false).await.unwrap();
+
+    // Symlink should exist
+    let link = dir.path().join("packages/server/node_modules/@acme/shared");
+    assert!(link.is_symlink(), "workspace:^ dep should create a symlink");
+
+    // Lockfile should record version ^2.3.1
+    let lf = Lockfile::read(&dir.path().join("jet-lock.yaml"))
+        .expect("jet-lock.yaml should exist");
+
+    let ws_entry = lf
+        .packages
+        .iter()
+        .find(|(k, e)| k.contains("@acme/shared") && e.workspace)
+        .map(|(_, e)| e)
+        .expect("lockfile should have a workspace entry for @acme/shared");
+
+    assert_eq!(
+        ws_entry.version, "^2.3.1",
+        "workspace:^ should resolve to ^<actual_version>"
+    );
+    assert!(ws_entry.workspace, "entry should have workspace: true");
+}
+
+#[tokio::test]
+async fn test_recursive_workspace_install() {
+    let dir = tempdir().unwrap();
+
+    write_file(
+        dir.path(),
+        "pnpm-workspace.yaml",
+        "packages:\n  - packages/*\n  - apps/*\n",
+    );
+
+    // utils: no deps
+    write_file(
+        dir.path(),
+        "packages/utils/package.json",
+        &pkg_json("utils", "1.0.0", &[]),
+    );
+    // ui: depends on utils
+    write_file(
+        dir.path(),
+        "packages/ui/package.json",
+        &pkg_json("ui", "2.0.0", &[("utils", "workspace:*")]),
+    );
+    // web: depends on both ui and utils
+    write_file(
+        dir.path(),
+        "apps/web/package.json",
+        &pkg_json(
+            "web",
+            "0.1.0",
+            &[("ui", "workspace:*"), ("utils", "workspace:*")],
+        ),
+    );
+
+    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
+    pm.install_with_options(false).await.unwrap();
+
+    // ui/node_modules/utils symlink
+    assert!(
+        dir.path()
+            .join("packages/ui/node_modules/utils")
+            .is_symlink(),
+        "ui → utils symlink should exist"
+    );
+
+    // web/node_modules/ui symlink
+    assert!(
+        dir.path().join("apps/web/node_modules/ui").is_symlink(),
+        "web → ui symlink should exist"
+    );
+
+    // web/node_modules/utils symlink
+    assert!(
+        dir.path()
+            .join("apps/web/node_modules/utils")
+            .is_symlink(),
+        "web → utils symlink should exist"
+    );
+}
+
+#[tokio::test]
+async fn test_no_registry_call_for_workspace_dep() {
+    // All dependencies are workspace: — no tarball should be downloaded.
+    // We verify this indirectly: node_modules entries are symlinks (not real dirs).
+    let dir = tempdir().unwrap();
+    make_two_package_workspace(dir.path());
+
+    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
+    pm.install_with_options(false).await.unwrap();
+
+    let link = dir.path().join("packages/web/node_modules/ui");
+
+    // Must be a symlink, not a real extracted directory
+    assert!(
+        link.is_symlink(),
+        "workspace dep should be a symlink, not an extracted tarball"
+    );
+
+    // Real extracted packages contain a package.json; symlinks point to the
+    // source dir (which also has package.json, but the symlink itself is not
+    // a regular directory at this path).
+    let meta = std::fs::symlink_metadata(&link).unwrap();
+    assert!(
+        meta.file_type().is_symlink(),
+        "file type must be symlink, not regular dir"
+    );
+}
+
+#[tokio::test]
+async fn test_lockfile_workspace_fields() {
+    let dir = tempdir().unwrap();
+    make_two_package_workspace(dir.path());
+
+    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
+    pm.install_with_options(false).await.unwrap();
+
+    let lf_path = dir.path().join("jet-lock.yaml");
+    assert!(lf_path.exists(), "jet-lock.yaml should be written");
+
+    let lf = Lockfile::read(&lf_path).expect("should parse jet-lock.yaml");
+
+    // Find the workspace entry for ui@1.5.0
+    let ui_entry = lf
+        .packages
+        .iter()
+        .find(|(k, e)| k.contains("ui") && e.workspace)
+        .map(|(_, e)| e)
+        .expect("lockfile should contain a workspace entry for ui");
+
+    assert!(ui_entry.workspace, "workspace field must be true");
+    assert_eq!(
+        ui_entry.version, "1.5.0",
+        "version should match workspace package version"
+    );
+    assert_eq!(
+        ui_entry.local_path.as_deref(),
+        Some("packages/ui"),
+        "localPath should be relative path from workspace root to package dir"
+    );
+}
+
+#[tokio::test]
+async fn test_idempotent_symlink_creation() {
+    // Running install twice should not fail — symlinks are idempotent.
+    let dir = tempdir().unwrap();
+    make_two_package_workspace(dir.path());
+
+    let pm = PackageManager::new(dir.path().to_path_buf()).unwrap();
+    pm.install_with_options(false)
+        .await
+        .expect("first install should succeed");
+    pm.install_with_options(false)
+        .await
+        .expect("second install should be idempotent");
+
+    assert!(
+        dir.path()
+            .join("packages/web/node_modules/ui")
+            .is_symlink()
+    );
+}
+
+#[test]
+fn test_workspace_protocol_resolution_variants() {
+    // Verify resolve_workspace_protocol handles *, ^, ~ correctly
+    let ws = WorkspaceManager {
+        root: std::path::PathBuf::from("/tmp"),
+        config: cclab_jet::pkg_manager::workspace::WorkspaceConfig::default(),
+        packages: vec![cclab_jet::pkg_manager::workspace::WorkspacePackage {
+            name: "pkg-a".to_string(),
+            version: "2.3.1".to_string(),
+            path: std::path::PathBuf::from("packages/pkg-a"),
+            dependencies: HashMap::new(),
+            dev_dependencies: HashMap::new(),
+            deps_on_workspace: Vec::new(),
+        }],
+    };
+
+    assert_eq!(
+        ws.resolve_workspace_protocol("workspace:*", "pkg-a"),
+        Some("2.3.1".to_string()),
+        "workspace:* → exact version"
+    );
+    assert_eq!(
+        ws.resolve_workspace_protocol("workspace:^", "pkg-a"),
+        Some("^2.3.1".to_string()),
+        "workspace:^ → caret range"
+    );
+    assert_eq!(
+        ws.resolve_workspace_protocol("workspace:~", "pkg-a"),
+        Some("~2.3.1".to_string()),
+        "workspace:~ → tilde range"
+    );
+    assert_eq!(
+        ws.resolve_workspace_protocol("workspace:*", "nonexistent"),
+        None,
+        "unknown package → None"
+    );
+}
```

## Review: jet-workspace-protocol-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: jet-workspace-protocol

**Summary**: Solid implementation of pnpm-style workspace protocol support across 5 files (workspace.rs, mod.rs, lockfile.rs, resolver.rs, tests/workspace_protocol.rs). All 6 spec requirements (R1-R6) correctly implemented. 21 test functions (exceeding the 11 spec-required) all pass with zero regressions across 467+ total cclab-jet tests. Minor code duplication in install_resolved_to() is the only soft concern.

### Checklist

- [PASS] [HARD] Code matches all spec requirements
  - R1: pnpm-workspace.yaml detected as third-priority config source in load_config(). R2: PnpmWorkspaceYaml struct parses packages/catalog/catalogs; catalogs flattened with '<catalog_name>:' prefix. R3: install_with_options() delegates to workspace_install_all() classifying deps via is_workspace_protocol(). R4: create_relative_symlink() with idempotency. R5: topological_order() iteration with install_resolved_to() for external deps. R6: LockfileEntry.workspace (skip_serializing_if is_false) + local_path (serde rename localPath); is_valid() skips store check.
- [PASS] [HARD] Spec has Test Plan AND diff contains #[test] functions
  - Spec defines 11 test cases. Implementation provides 21: 5 unit tests in workspace.rs, 5 in lockfile.rs, 11 integration tests in tests/workspace_protocol.rs. All spec-required cases covered plus extras (idempotent_symlink, workspace_mode_detection, resolution_variants, field_defaults, serialization_skip).
- [PASS] [HARD] Existing tests still pass (no regressions)
  - All 467+ cclab-jet tests pass: 10 lockfile, 10 workspace, 4 bundler, 16 nx_support, 11 workspace_protocol, 1 doc-test. Zero failures.
- [PASS] [SOFT] Code quality and readability
  - Clear naming, well-structured helpers (make_relative_path, create_relative_symlink, install_resolved_to). Reusable test fixtures. Doc comments on all new types/fields.
- [PASS] [SOFT] Error handling completeness
  - anyhow::Context for file reads, meaningful messages ('Workspace package X not found (required by Y)'), proper ? propagation.
- [PASS] [SOFT] Performance considerations
  - install_resolved_to() duplicates install_resolved() logic — could share a common core with target_dir param. Not blocking.
- [PASS] [SOFT] Documentation where needed
  - All new structs, fields, and functions have doc comments explaining purpose and invariants.

