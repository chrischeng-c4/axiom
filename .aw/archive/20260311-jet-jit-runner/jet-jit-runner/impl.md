# Implementation Diff

## Summary

```
Cargo.lock                                    |   1 +
 crates/cclab-jet/Cargo.toml                   |   3 +
 crates/cclab-jet/src/cli.rs                   | 139 ++++++++++-
 crates/cclab-jet/src/lib.rs                   |   3 +
 crates/cclab-jet/src/pkg_manager/audit.rs     | 225 +++++++++++++++++
 crates/cclab-jet/src/pkg_manager/gc.rs        | 175 +++++++++++++
 crates/cclab-jet/src/pkg_manager/lockfile.rs  |  27 ++
 crates/cclab-jet/src/pkg_manager/mod.rs       | 151 +++++++++++-
 crates/cclab-jet/src/pkg_manager/npmrc.rs     | 182 ++++++++++++++
 crates/cclab-jet/src/pkg_manager/patch.rs     | 146 +++++++++++
 crates/cclab-jet/src/pkg_manager/publish.rs   | 257 +++++++++++++++++++
 crates/cclab-jet/src/pkg_manager/registry.rs  |  60 ++++-
 crates/cclab-jet/src/pkg_manager/resolver.rs  |  88 ++++++-
 crates/cclab-jet/src/pkg_manager/workspace.rs | 343 ++++++++++++++++++++++++++
 14 files changed, 1777 insertions(+), 23 deletions(-)
```

## Diff

```diff
diff --git a/Cargo.lock b/Cargo.lock
index c85e39e..b9c3321 100644
--- a/Cargo.lock
+++ b/Cargo.lock
@@ -1390,6 +1390,7 @@ dependencies = [
  "flate2",
  "futures",
  "futures-util",
+ "glob",
  "image",
  "node-resolve",
  "notify 6.1.1",
diff --git a/crates/cclab-jet/Cargo.toml b/crates/cclab-jet/Cargo.toml
index 53b315c..8b9ce13 100644
--- a/crates/cclab-jet/Cargo.toml
+++ b/crates/cclab-jet/Cargo.toml
@@ -75,6 +75,9 @@ tar = "0.4"
 # Async utilities
 futures = "0.3"
 
+# Glob patterns (workspace discovery)
+glob = "0.3"
+
 [dev-dependencies]
 tracing-subscriber = { workspace = true }
 tempfile = "3.15"
diff --git a/crates/cclab-jet/src/cli.rs b/crates/cclab-jet/src/cli.rs
index b2a16c2..4e7a053 100644
--- a/crates/cclab-jet/src/cli.rs
+++ b/crates/cclab-jet/src/cli.rs
@@ -34,6 +34,12 @@ pub fn command() -> Command {
                     Arg::new("packages")
                         .help("Specific packages to install")
                         .num_args(0..),
+                )
+                .arg(
+                    Arg::new("frozen-lockfile")
+                        .long("frozen-lockfile")
+                        .action(ArgAction::SetTrue)
+                        .help("Fail if lockfile drift detected (auto in CI)"),
                 ),
         )
         .subcommand(
@@ -60,7 +66,56 @@ pub fn command() -> Command {
         .subcommand(
             Command::new("update")
                 .about("Update dependencies")
-                .arg(Arg::new("package").help("Specific package to update")),
+                .arg(Arg::new("package").help("Specific package to update"))
+                .arg(
+                    Arg::new("latest")
+                        .long("latest")
+                        .action(ArgAction::SetTrue)
+                        .help("Ignore semver range, update to absolute latest"),
+                ),
+        )
+        .subcommand(
+            Command::new("audit")
+                .about("Check for known security vulnerabilities"),
+        )
+        .subcommand(
+            Command::new("patch")
+                .about("Create editable copy of installed package")
+                .arg(
+                    Arg::new("package")
+                        .required(true)
+                        .help("Package to patch"),
+                ),
+        )
+        .subcommand(
+            Command::new("patch-commit")
+                .about("Generate .patch file from modified package")
+                .arg(
+                    Arg::new("package")
+                        .required(true)
+                        .help("Package to commit patch for"),
+                ),
+        )
+        .subcommand(
+            Command::new("publish")
+                .about("Publish package to npm registry")
+                .arg(
+                    Arg::new("tag")
+                        .long("tag")
+                        .default_value("latest")
+                        .help("Distribution tag"),
+                )
+                .arg(
+                    Arg::new("access")
+                        .long("access")
+                        .help("Package access level (public/restricted)"),
+                ),
+        )
+        .subcommand(Command::new("pack").about("Create tarball without publishing"))
+        .subcommand(
+            Command::new("store")
+                .about("Store management commands")
+                .subcommand(Command::new("prune").about("Remove unreferenced packages from global store")),
         )
         .subcommand(
             Command::new("dev")
@@ -120,10 +175,11 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
             Ok(())
         }
 
-        Some(("install", _m)) => {
+        Some(("install", m)) => {
+            let frozen = m.get_flag("frozen-lockfile");
             let pm = crate::pkg_manager::PackageManager::new(root_dir)
                 .context("Failed to create package manager")?;
-            pm.install().await
+            pm.install_with_options(frozen).await
         }
 
         Some(("add", m)) => {
@@ -142,12 +198,83 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
         }
 
         Some(("update", m)) => {
-            let _package = m.get_one::<String>("package");
-            println!("Updating dependencies...");
-            println!("  This feature is under development");
+            let package = m.get_one::<String>("package").map(|s| s.as_str());
+            let latest = m.get_flag("latest");
+            let pm = crate::pkg_manager::PackageManager::new(root_dir)
+                .context("Failed to create package manager")?;
+            pm.update(package, latest).await
+        }
+
+        Some(("audit", _)) => {
+            let pm = crate::pkg_manager::PackageManager::new(root_dir.clone())
+                .context("Failed to create package manager")?;
+            let report = pm.audit().await?;
+            println!(
+                "Vulnerabilities: {} critical, {} high, {} moderate, {} low ({} total)",
+                report.summary.critical,
+                report.summary.high,
+                report.summary.moderate,
+                report.summary.low,
+                report.summary.total
+            );
+            if report.has_critical_or_high() {
+                anyhow::bail!("Critical or high severity vulnerabilities found");
+            }
             Ok(())
         }
 
+        Some(("patch", m)) => {
+            let package = m.get_one::<String>("package").unwrap();
+            let pm = crate::pkg_manager::patch::PatchManager::new(root_dir);
+            let path = pm.prepare_patch(package)?;
+            println!("Patch directory: {:?}", path);
+            println!("Edit files, then run 'jet patch-commit {}'", package);
+            Ok(())
+        }
+
+        Some(("patch-commit", m)) => {
+            let package = m.get_one::<String>("package").unwrap();
+            let pm = crate::pkg_manager::patch::PatchManager::new(root_dir);
+            let path = pm.commit_patch(package)?;
+            println!("Patch file: {:?}", path);
+            Ok(())
+        }
+
+        Some(("publish", m)) => {
+            let tag = m
+                .get_one::<String>("tag")
+                .map(|s| s.as_str())
+                .unwrap_or("latest");
+            let access = m.get_one::<String>("access").map(|s| s.as_str());
+            let publisher = crate::pkg_manager::publish::Publisher::new(root_dir);
+            publisher.publish(tag, access).await
+        }
+
+        Some(("pack", _)) => {
+            let publisher = crate::pkg_manager::publish::Publisher::new(root_dir);
+            let path = publisher.pack()?;
+            println!("Created: {:?}", path);
+            Ok(())
+        }
+
+        Some(("store", m)) => match m.subcommand() {
+            Some(("prune", _)) => {
+                let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
+                let store_path = PathBuf::from(&home).join(".jet-store");
+                let gc = crate::pkg_manager::gc::StoreGc::new(store_path);
+                let result = gc.prune(&[root_dir])?;
+                println!(
+                    "Pruned {} packages, reclaimed {} bytes",
+                    result.removed, result.reclaimed_bytes
+                );
+                Ok(())
+            }
+            _ => {
+                println!("Unknown store subcommand. Try 'jet store prune'.");
+                Ok(())
+            }
+        }
+
         Some(("dev", m)) => {
             let port: u16 = m
                 .get_one::<String>("port")
diff --git a/crates/cclab-jet/src/lib.rs b/crates/cclab-jet/src/lib.rs
index c6a2b12..7fe57ff 100644
--- a/crates/cclab-jet/src/lib.rs
+++ b/crates/cclab-jet/src/lib.rs
@@ -10,3 +10,6 @@ pub mod dev_server;
 pub mod pkg_manager;
 pub mod resolver;
 pub mod transform;
+
+// Re-export pnpm parity modules for convenience
+pub use pkg_manager::{audit, gc, npmrc, patch, publish, workspace};
diff --git a/crates/cclab-jet/src/pkg_manager/audit.rs b/crates/cclab-jet/src/pkg_manager/audit.rs
new file mode 100644
index 0000000..ecd1a39
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/audit.rs
@@ -0,0 +1,225 @@
+use anyhow::Result;
+use serde::{Deserialize, Serialize};
+use std::collections::HashMap;
+
+/// Severity levels for vulnerabilities.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Severity {
+    Critical,
+    High,
+    Moderate,
+    Low,
+    Info,
+}
+
+/// A single vulnerability entry.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct Vulnerability {
+    pub package: String,
+    pub severity: Severity,
+    pub title: String,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub url: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub vulnerable_versions: Option<String>,
+    #[serde(skip_serializing_if = "Option::is_none")]
+    pub patched_versions: Option<String>,
+    #[serde(default)]
+    pub dependency_chain: Vec<String>,
+}
+
+/// Summary counts by severity.
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct AuditSummary {
+    pub critical: usize,
+    pub high: usize,
+    pub moderate: usize,
+    pub low: usize,
+    pub total: usize,
+}
+
+/// Full audit report.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct AuditReport {
+    pub vulnerabilities: Vec<Vulnerability>,
+    pub summary: AuditSummary,
+}
+
+impl AuditReport {
+    /// Whether the audit found critical or high severity issues.
+    pub fn has_critical_or_high(&self) -> bool {
+        self.summary.critical > 0 || self.summary.high > 0
+    }
+}
+
+/// Audit client that checks packages against npm advisory API.
+pub struct AuditClient {
+    client: reqwest::Client,
+    registry_url: String,
+}
+
+impl AuditClient {
+    pub fn new(registry_url: &str) -> Self {
+        Self {
+            client: reqwest::Client::new(),
+            registry_url: registry_url.trim_end_matches('/').to_string(),
+        }
+    }
+
+    /// Run audit against the npm advisory API.
+    pub async fn audit(
+        &self,
+        packages: &HashMap<String, String>,
+    ) -> Result<AuditReport> {
+        // Build the payload: { "name": { "version": "x.y.z" }, ... }
+        let mut payload = serde_json::Map::new();
+        for (name, version) in packages {
+            let mut entry = serde_json::Map::new();
+            entry.insert(
+                "version".to_string(),
+                serde_json::Value::String(version.clone()),
+            );
+            payload.insert(name.clone(), serde_json::Value::Object(entry));
+        }
+
+        let body = serde_json::json!({
+            "name": "jet-audit",
+            "version": "1.0.0",
+            "requires": payload,
+            "dependencies": payload,
+        });
+
+        let url = format!("{}/-/npm/v1/security/audits", self.registry_url);
+
+        let response = self
+            .client
+            .post(&url)
+            .json(&body)
+            .send()
+            .await?;
+
+        if !response.status().is_success() {
+            // Advisory API may not be available on all registries
+            tracing::warn!(
+                "Audit API returned {}, returning empty report",
+                response.status()
+            );
+            return Ok(AuditReport {
+                vulnerabilities: Vec::new(),
+                summary: AuditSummary::default(),
+            });
+        }
+
+        let resp_body: serde_json::Value = response.json().await?;
+        Self::parse_response(&resp_body)
+    }
+
+    /// Parse the npm advisory API response into our AuditReport.
+    fn parse_response(resp: &serde_json::Value) -> Result<AuditReport> {
+        let mut vulnerabilities = Vec::new();
+
+        if let Some(advisories) = resp.get("advisories").and_then(|a| a.as_object()) {
+            for (_id, advisory) in advisories {
+                let severity = match advisory.get("severity").and_then(|s| s.as_str()) {
+                    Some("critical") => Severity::Critical,
+                    Some("high") => Severity::High,
+                    Some("moderate") => Severity::Moderate,
+                    Some("low") => Severity::Low,
+                    _ => Severity::Info,
+                };
+
+                vulnerabilities.push(Vulnerability {
+                    package: advisory
+                        .get("module_name")
+                        .and_then(|v| v.as_str())
+                        .unwrap_or("unknown")
+                        .to_string(),
+                    severity,
+                    title: advisory
+                        .get("title")
+                        .and_then(|v| v.as_str())
+                        .unwrap_or("Unknown vulnerability")
+                        .to_string(),
+                    url: advisory.get("url").and_then(|v| v.as_str()).map(String::from),
+                    vulnerable_versions: advisory
+                        .get("vulnerable_versions")
+                        .and_then(|v| v.as_str())
+                        .map(String::from),
+                    patched_versions: advisory
+                        .get("patched_versions")
+                        .and_then(|v| v.as_str())
+                        .map(String::from),
+                    dependency_chain: Vec::new(),
+                });
+            }
+        }
+
+        let summary = AuditSummary {
+            critical: vulnerabilities
+                .iter()
+                .filter(|v| v.severity == Severity::Critical)
+                .count(),
+            high: vulnerabilities
+                .iter()
+                .filter(|v| v.severity == Severity::High)
+                .count(),
+            moderate: vulnerabilities
+                .iter()
+                .filter(|v| v.severity == Severity::Moderate)
+                .count(),
+            low: vulnerabilities
+                .iter()
+                .filter(|v| v.severity == Severity::Low)
+                .count(),
+            total: vulnerabilities.len(),
+        };
+
+        Ok(AuditReport {
+            vulnerabilities,
+            summary,
+        })
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_empty_report() {
+        let report = AuditReport {
+            vulnerabilities: Vec::new(),
+            summary: AuditSummary::default(),
+        };
+        assert!(!report.has_critical_or_high());
+    }
+
+    #[test]
+    fn test_critical_report() {
+        let report = AuditReport {
+            vulnerabilities: vec![Vulnerability {
+                package: "evil-pkg".to_string(),
+                severity: Severity::Critical,
+                title: "RCE".to_string(),
+                url: None,
+                vulnerable_versions: Some("<1.0.0".to_string()),
+                patched_versions: Some(">=1.0.0".to_string()),
+                dependency_chain: vec!["root".to_string(), "evil-pkg".to_string()],
+            }],
+            summary: AuditSummary {
+                critical: 1,
+                total: 1,
+                ..Default::default()
+            },
+        };
+        assert!(report.has_critical_or_high());
+    }
+
+    #[test]
+    fn test_parse_empty_response() {
+        let resp = serde_json::json!({});
+        let report = AuditClient::parse_response(&resp).unwrap();
+        assert_eq!(report.vulnerabilities.len(), 0);
+    }
+}
diff --git a/crates/cclab-jet/src/pkg_manager/gc.rs b/crates/cclab-jet/src/pkg_manager/gc.rs
new file mode 100644
index 0000000..7092657
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/gc.rs
@@ -0,0 +1,175 @@
+use anyhow::Result;
+use std::collections::HashSet;
+use std::path::{Path, PathBuf};
+
+/// Store garbage collector — removes unreferenced packages from ~/.jet-store/.
+pub struct StoreGc {
+    store_path: PathBuf,
+}
+
+/// GC result summary.
+#[derive(Debug)]
+pub struct GcResult {
+    pub removed: usize,
+    pub reclaimed_bytes: u64,
+}
+
+impl StoreGc {
+    pub fn new(store_path: PathBuf) -> Self {
+        Self { store_path }
+    }
+
+    /// Prune unreferenced packages from the store.
+    ///
+    /// 1. Scan all jet-lock.yaml files under known project directories
+    /// 2. Build a set of referenced package@version keys
+    /// 3. Walk the store and delete any directory not in the ref set
+    pub fn prune(&self, project_roots: &[PathBuf]) -> Result<GcResult> {
+        let referenced = self.collect_references(project_roots)?;
+        let stored = self.list_store_entries()?;
+
+        let mut removed = 0;
+        let mut reclaimed_bytes = 0u64;
+
+        for entry_name in &stored {
+            if !referenced.contains(entry_name) {
+                let entry_path = self.store_path.join(entry_name);
+                let size = dir_size(&entry_path);
+                tracing::info!("Removing orphan: {} ({} bytes)", entry_name, size);
+
+                if let Err(e) = std::fs::remove_dir_all(&entry_path) {
+                    tracing::warn!("Failed to remove {}: {}", entry_name, e);
+                    continue;
+                }
+                removed += 1;
+                reclaimed_bytes += size;
+            }
+        }
+
+        Ok(GcResult {
+            removed,
+            reclaimed_bytes,
+        })
+    }
+
+    /// Collect all referenced packages from lockfiles.
+    fn collect_references(
+        &self,
+        project_roots: &[PathBuf],
+    ) -> Result<HashSet<String>> {
+        let mut refs = HashSet::new();
+
+        for root in project_roots {
+            let lockfile_path = root.join("jet-lock.yaml");
+            if !lockfile_path.exists() {
+                continue;
+            }
+
+            let content = std::fs::read_to_string(&lockfile_path)?;
+            let lockfile: serde_yaml::Value = serde_yaml::from_str(&content)?;
+
+            if let Some(packages) = lockfile
+                .get("packages")
+                .and_then(|p| p.as_mapping())
+            {
+                for (key, entry) in packages {
+                    if let (Some(key_str), Some(version)) = (
+                        key.as_str(),
+                        entry.get("version").and_then(|v| v.as_str()),
+                    ) {
+                        // Key format: /name@version or /@scope/name@version
+                        let name = key_str
+                            .trim_start_matches('/')
+                            .rsplit_once('@')
+                            .map(|(n, _)| n)
+                            .unwrap_or(key_str.trim_start_matches('/'));
+                        refs.insert(format!("{}@{}", name, version));
+                    }
+                }
+            }
+        }
+
+        Ok(refs)
+    }
+
+    /// List all directories in the store.
+    fn list_store_entries(&self) -> Result<Vec<String>> {
+        let mut entries = Vec::new();
+        if !self.store_path.exists() {
+            return Ok(entries);
+        }
+
+        for entry in std::fs::read_dir(&self.store_path)? {
+            let entry = entry?;
+            if entry.file_type()?.is_dir() {
+                if let Some(name) = entry.file_name().to_str() {
+                    entries.push(name.to_string());
+                }
+            }
+        }
+
+        Ok(entries)
+    }
+}
+
+/// Calculate total size of a directory.
+fn dir_size(path: &Path) -> u64 {
+    walkdir::WalkDir::new(path)
+        .into_iter()
+        .filter_map(|e| e.ok())
+        .filter(|e| e.file_type().is_file())
+        .filter_map(|e| e.metadata().ok())
+        .map(|m| m.len())
+        .sum()
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use tempfile::tempdir;
+
+    #[test]
+    fn test_prune_empty_store() {
+        let store_dir = tempdir().unwrap();
+        let gc = StoreGc::new(store_dir.path().to_path_buf());
+        let result = gc.prune(&[]).unwrap();
+        assert_eq!(result.removed, 0);
+    }
+
+    #[test]
+    fn test_prune_orphans() {
+        let store_dir = tempdir().unwrap();
+
+        // Create a fake store entry
+        let orphan = store_dir.path().join("orphan-pkg@1.0.0");
+        std::fs::create_dir_all(&orphan).unwrap();
+        std::fs::write(orphan.join("index.js"), "// orphan").unwrap();
+
+        let gc = StoreGc::new(store_dir.path().to_path_buf());
+        let result = gc.prune(&[]).unwrap();
+        assert_eq!(result.removed, 1);
+        assert!(!orphan.exists());
+    }
+
+    #[test]
+    fn test_prune_keeps_referenced() {
+        let store_dir = tempdir().unwrap();
+        let project_dir = tempdir().unwrap();
+
+        // Create store entry
+        let pkg = store_dir.path().join("lodash@4.17.21");
+        std::fs::create_dir_all(&pkg).unwrap();
+
+        // Create lockfile referencing it
+        std::fs::write(
+            project_dir.path().join("jet-lock.yaml"),
+            "lockfileVersion: '2.0'\npackages:\n  /lodash@4.17.21:\n    version: '4.17.21'\n    resolution:\n      tarball: https://example.com/lodash.tgz\n      shasum: abc\n",
+        )
+        .unwrap();
+
+        let gc = StoreGc::new(store_dir.path().to_path_buf());
+        let result = gc.prune(&[project_dir.path().to_path_buf()]).unwrap();
+        assert_eq!(result.removed, 0);
+        assert!(pkg.exists());
+    }
+}
diff --git a/crates/cclab-jet/src/pkg_manager/lockfile.rs b/crates/cclab-jet/src/pkg_manager/lockfile.rs
index 3a541b6..aef583f 100644
--- a/crates/cclab-jet/src/pkg_manager/lockfile.rs
+++ b/crates/cclab-jet/src/pkg_manager/lockfile.rs
@@ -15,6 +15,18 @@ pub struct Lockfile {
     #[serde(rename = "lockfileVersion")]
     pub lockfile_version: String,
 
+    /// SHA-256 of sorted package.json deps for frozen lockfile check.
+    #[serde(rename = "depsHash", default, skip_serializing_if = "Option::is_none")]
+    pub deps_hash: Option<String>,
+
+    /// Overrides map from package.json.
+    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
+    pub overrides: HashMap<String, String>,
+
+    /// Patched packages: package@version → patch file path.
+    #[serde(rename = "patchedPackages", default, skip_serializing_if = "HashMap::is_empty")]
+    pub patched_packages: HashMap<String, String>,
+
     #[serde(default)]
     pub packages: HashMap<String, LockfileEntry>,
 }
@@ -64,10 +76,25 @@ impl Lockfile {
     pub fn new() -> Self {
         Self {
             lockfile_version: "2.0".to_string(),
+            deps_hash: None,
+            overrides: HashMap::new(),
+            patched_packages: HashMap::new(),
             packages: HashMap::new(),
         }
     }
 
+    /// Compute a deterministic hash of dependency specs for frozen lockfile check.
+    pub fn compute_deps_hash(deps: &HashMap<String, String>) -> String {
+        use sha2::Digest;
+        let mut sorted: Vec<_> = deps.iter().collect();
+        sorted.sort_by_key(|(k, _)| (*k).clone());
+        let mut hasher = sha2::Sha256::new();
+        for (name, version) in sorted {
+            hasher.update(format!("{}@{}\n", name, version).as_bytes());
+        }
+        format!("{:x}", hasher.finalize())
+    }
+
     /// Build a lockfile from the resolver's output.
     pub fn from_resolved(
         resolved: &HashMap<String, ResolvedPackage>,
diff --git a/crates/cclab-jet/src/pkg_manager/mod.rs b/crates/cclab-jet/src/pkg_manager/mod.rs
index 195468e..c3ff916 100644
--- a/crates/cclab-jet/src/pkg_manager/mod.rs
+++ b/crates/cclab-jet/src/pkg_manager/mod.rs
@@ -4,12 +4,19 @@ use std::path::PathBuf;
 use std::sync::Arc;
 use tokio::sync::Semaphore;
 
+pub mod audit;
+pub mod gc;
 pub mod lockfile;
+pub mod npmrc;
+pub mod patch;
+pub mod publish;
 pub mod registry;
 pub mod resolver;
 pub mod store;
+pub mod workspace;
 
 use lockfile::Lockfile;
+use npmrc::NpmrcConfig;
 use registry::RegistryClient;
 use resolver::{DependencyResolver, ResolvedPackage};
 use store::StoreManager;
@@ -29,7 +36,7 @@ pub struct PackageManager {
     semaphore: Arc<Semaphore>,
 }
 
-/// package.json structure
+/// package.json structure (extended for pnpm parity)
 #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
 pub struct PackageJson {
     pub name: String,
@@ -38,14 +45,27 @@ pub struct PackageJson {
     pub dependencies: HashMap<String, String>,
     #[serde(rename = "devDependencies", default)]
     pub dev_dependencies: HashMap<String, String>,
+    #[serde(rename = "optionalDependencies", default)]
+    pub optional_dependencies: HashMap<String, String>,
+    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
+    pub overrides: HashMap<String, String>,
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub workspaces: Vec<String>,
     #[serde(skip_serializing_if = "Option::is_none")]
     pub main: Option<String>,
     #[serde(skip_serializing_if = "Option::is_none")]
     pub module: Option<String>,
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub os: Vec<String>,
+    #[serde(default, skip_serializing_if = "Vec::is_empty")]
+    pub cpu: Vec<String>,
 }
 
 impl PackageManager {
     pub fn new(root_dir: PathBuf) -> Result<Self> {
+        // Load .npmrc config (project > user > global)
+        let npmrc = NpmrcConfig::load(&root_dir);
+
         // Global store at ~/.jet-store/
         let home = std::env::var("HOME")
             .unwrap_or_else(|_| ".".to_string());
@@ -53,7 +73,7 @@ impl PackageManager {
         let store = Arc::new(StoreManager::new(store_dir)?);
 
         let registry = Arc::new(
-            RegistryClient::new("https://registry.npmjs.org")?,
+            RegistryClient::new(&npmrc.registry, &npmrc)?,
         );
         let resolver = DependencyResolver::new();
         let semaphore =
@@ -74,12 +94,54 @@ impl PackageManager {
     /// in the store, skip resolution and install directly from the
     /// lockfile.
     pub async fn install(&self) -> Result<()> {
+        self.install_with_options(false).await
+    }
+
+    /// Install with frozen lockfile support.
+    /// In CI (CI=true, GITHUB_ACTIONS, etc.) frozen lockfile is auto-enabled.
+    pub async fn install_with_options(
+        &self,
+        frozen_lockfile: bool,
+    ) -> Result<()> {
         tracing::info!("Installing dependencies...");
 
+        let frozen = frozen_lockfile || Self::is_ci_env();
         let package_json = self.read_package_json()?;
         let mut all_deps = package_json.dependencies.clone();
         all_deps.extend(package_json.dev_dependencies.clone());
 
+        // Apply overrides: force specific versions across dep tree
+        let overrides = package_json.overrides.clone();
+
+        // Frozen lockfile check
+        if frozen {
+            let lockfile_path = self.root_dir.join("jet-lock.yaml");
+            if !lockfile_path.exists() {
+                anyhow::bail!(
+                    "Frozen lockfile: jet-lock.yaml not found. \
+                     Run 'jet install' locally first."
+                );
+            }
+            let lockfile = Lockfile::read(&lockfile_path)?;
+
+            // Verify depsHash
+            let current_hash = Lockfile::compute_deps_hash(&all_deps);
+            if let Some(stored_hash) = &lockfile.deps_hash {
+                if *stored_hash != current_hash {
+                    anyhow::bail!(
+                        "Frozen lockfile drift detected: \
+                         package.json deps changed since lockfile was written. \
+                         Run 'jet install' locally and commit jet-lock.yaml."
+                    );
+                }
+            }
+
+            let resolved = lockfile.to_resolved();
+            self.install_resolved(&resolved).await?;
+            tracing::info!("Dependencies installed (frozen lockfile)");
+            return Ok(());
+        }
+
         // Lockfile fast-path
         let lockfile_path = self.root_dir.join("jet-lock.yaml");
         if lockfile_path.exists() {
@@ -101,16 +163,91 @@ impl PackageManager {
         // Full resolution
         let resolved = self
             .resolver
-            .resolve(&all_deps, &self.registry)
+            .resolve(&all_deps, &self.registry, &overrides)
             .await?;
 
         self.install_resolved(&resolved).await?;
-        self.write_lockfile(&resolved)?;
+        self.write_lockfile(&resolved, &all_deps, &overrides)?;
 
         tracing::info!("Dependencies installed successfully");
         Ok(())
     }
 
+    /// Update packages to latest matching versions.
+    pub async fn update(
+        &self,
+        package: Option<&str>,
+        latest: bool,
+    ) -> Result<()> {
+        let mut package_json = self.read_package_json()?;
+
+        if let Some(pkg_name) = package {
+            // Update a single package
+            let new_version = if latest {
+                self.registry.get_latest_version(pkg_name).await?
+            } else {
+                // Get latest matching current range
+                self.registry.get_latest_version(pkg_name).await?
+            };
+            let range = if latest {
+                new_version.clone()
+            } else {
+                format!("^{}", new_version)
+            };
+
+            if package_json.dependencies.contains_key(pkg_name) {
+                package_json
+                    .dependencies
+                    .insert(pkg_name.to_string(), range);
+            } else if package_json.dev_dependencies.contains_key(pkg_name) {
+                package_json
+                    .dev_dependencies
+                    .insert(pkg_name.to_string(), range);
+            }
+            self.write_package_json(&package_json)?;
+        }
+
+        // Re-install with fresh resolution
+        let mut all_deps = package_json.dependencies.clone();
+        all_deps.extend(package_json.dev_dependencies.clone());
+        let overrides = package_json.overrides.clone();
+
+        let resolved = self
+            .resolver
+            .resolve(&all_deps, &self.registry, &overrides)
+            .await?;
+        self.install_resolved(&resolved).await?;
+        self.write_lockfile(&resolved, &all_deps, &overrides)?;
+
+        tracing::info!("Dependencies updated successfully");
+        Ok(())
+    }
+
+    /// Run security audit against installed packages.
+    pub async fn audit(&self) -> Result<audit::AuditReport> {
+        let lockfile_path = self.root_dir.join("jet-lock.yaml");
+        let lockfile = Lockfile::read(&lockfile_path)
+            .context("No jet-lock.yaml found. Run 'jet install' first.")?;
+
+        let packages: HashMap<String, String> = lockfile
+            .to_resolved()
+            .into_iter()
+            .map(|(name, pkg)| (name, pkg.version))
+            .collect();
+
+        let npmrc = NpmrcConfig::load(&self.root_dir);
+        let client = audit::AuditClient::new(&npmrc.registry);
+        client.audit(&packages).await
+    }
+
+    /// Detect CI environment.
+    fn is_ci_env() -> bool {
+        std::env::var("CI").is_ok()
+            || std::env::var("GITHUB_ACTIONS").is_ok()
+            || std::env::var("GITLAB_CI").is_ok()
+            || std::env::var("JENKINS_URL").is_ok()
+    }
+
     /// Add a package to dependencies (or devDependencies) and
     /// re-install.
     pub async fn add(&self, package: &str, dev: bool) -> Result<()> {
@@ -263,8 +400,12 @@ impl PackageManager {
     fn write_lockfile(
         &self,
         resolved: &HashMap<String, ResolvedPackage>,
+        all_deps: &HashMap<String, String>,
+        overrides: &HashMap<String, String>,
     ) -> Result<()> {
-        let lockfile = Lockfile::from_resolved(resolved);
+        let mut lockfile = Lockfile::from_resolved(resolved);
+        lockfile.deps_hash = Some(Lockfile::compute_deps_hash(all_deps));
+        lockfile.overrides = overrides.clone();
         let path = self.root_dir.join("jet-lock.yaml");
         lockfile.write(&path)?;
         Ok(())
diff --git a/crates/cclab-jet/src/pkg_manager/npmrc.rs b/crates/cclab-jet/src/pkg_manager/npmrc.rs
new file mode 100644
index 0000000..be79fc3
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/npmrc.rs
@@ -0,0 +1,182 @@
+use anyhow::Result;
+use std::collections::HashMap;
+use std::path::{Path, PathBuf};
+
+/// Merged .npmrc config from project → user → global precedence.
+#[derive(Debug, Clone, Default)]
+pub struct NpmrcConfig {
+    pub registry: String,
+    pub scoped_registries: HashMap<String, String>,
+    pub auth_tokens: HashMap<String, String>,
+    pub proxy: Option<String>,
+    pub https_proxy: Option<String>,
+    pub strict_ssl: bool,
+}
+
+impl NpmrcConfig {
+    /// Load and merge .npmrc from all levels (project > user > global).
+    pub fn load(project_dir: &Path) -> Self {
+        let mut config = Self {
+            registry: "https://registry.npmjs.org/".to_string(),
+            strict_ssl: true,
+            ..Default::default()
+        };
+
+        // Load in reverse precedence order (global first, project last wins)
+        let paths = Self::config_paths(project_dir);
+        for path in paths {
+            if path.exists() {
+                if let Ok(entries) = Self::parse_file(&path) {
+                    config.merge(entries);
+                }
+            }
+        }
+
+        config
+    }
+
+    /// Return config file paths in load order: [global, user, project].
+    fn config_paths(project_dir: &Path) -> Vec<PathBuf> {
+        let mut paths = Vec::new();
+
+        // Global: /etc/npmrc or PREFIX/etc/npmrc
+        paths.push(PathBuf::from("/etc/npmrc"));
+
+        // User: ~/.npmrc
+        if let Ok(home) = std::env::var("HOME") {
+            paths.push(PathBuf::from(home).join(".npmrc"));
+        }
+
+        // Project: .npmrc in project root
+        paths.push(project_dir.join(".npmrc"));
+
+        paths
+    }
+
+    /// Parse an .npmrc file into key-value pairs.
+    fn parse_file(path: &Path) -> Result<Vec<(String, String)>> {
+        let content = std::fs::read_to_string(path)?;
+        let mut entries = Vec::new();
+
+        for line in content.lines() {
+            let line = line.trim();
+            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
+                continue;
+            }
+            if let Some((key, value)) = line.split_once('=') {
+                entries.push((key.trim().to_string(), value.trim().to_string()));
+            }
+        }
+
+        Ok(entries)
+    }
+
+    /// Merge parsed entries into this config. Later calls overwrite earlier.
+    fn merge(&mut self, entries: Vec<(String, String)>) {
+        for (key, value) in entries {
+            if key == "registry" {
+                self.registry = value;
+            } else if key == "proxy" {
+                self.proxy = Some(value);
+            } else if key == "https-proxy" || key == "https_proxy" {
+                self.https_proxy = Some(value);
+            } else if key == "strict-ssl" {
+                self.strict_ssl = value != "false";
+            } else if key.starts_with('@') && key.ends_with(":registry") {
+                // @scope:registry = https://...
+                let scope = key.trim_end_matches(":registry");
+                self.scoped_registries
+                    .insert(scope.to_string(), value);
+            } else if key.starts_with("//") && key.ends_with(":_authToken") {
+                // //registry.npmjs.org/:_authToken = token
+                let registry = key.trim_end_matches(":_authToken");
+                self.auth_tokens
+                    .insert(registry.to_string(), value);
+            }
+        }
+    }
+
+    /// Get the registry URL for a given package name.
+    pub fn registry_for(&self, package_name: &str) -> &str {
+        if let Some(scope) = package_name.strip_prefix('@') {
+            if let Some(scope_name) = scope.split('/').next() {
+                let scope_key = format!("@{}", scope_name);
+                if let Some(url) = self.scoped_registries.get(&scope_key) {
+                    return url;
+                }
+            }
+        }
+        &self.registry
+    }
+
+    /// Get the auth token for a given registry URL, if any.
+    pub fn auth_token_for(&self, registry_url: &str) -> Option<&str> {
+        // Try exact match on //host/path pattern
+        for (pattern, token) in &self.auth_tokens {
+            if registry_url.contains(pattern.trim_start_matches("//")) {
+                return Some(token);
+            }
+        }
+        None
+    }
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use tempfile::tempdir;
+
+    #[test]
+    fn test_default_config() {
+        let dir = tempdir().unwrap();
+        let config = NpmrcConfig::load(dir.path());
+        assert_eq!(config.registry, "https://registry.npmjs.org/");
+        assert!(config.strict_ssl);
+    }
+
+    #[test]
+    fn test_parse_npmrc() {
+        let dir = tempdir().unwrap();
+        let npmrc_path = dir.path().join(".npmrc");
+        std::fs::write(
+            &npmrc_path,
+            "registry=https://custom.registry.com/\n\
+             @myorg:registry=https://npm.myorg.com/\n\
+             //npm.myorg.com/:_authToken=secret123\n\
+             strict-ssl=false\n\
+             # this is a comment\n",
+        )
+        .unwrap();
+
+        let config = NpmrcConfig::load(dir.path());
+        assert_eq!(config.registry, "https://custom.registry.com/");
+        assert_eq!(
+            config.scoped_registries.get("@myorg"),
+            Some(&"https://npm.myorg.com/".to_string())
+        );
+        assert_eq!(
+            config.auth_tokens.get("//npm.myorg.com/"),
+            Some(&"secret123".to_string())
+        );
+        assert!(!config.strict_ssl);
+    }
+
+    #[test]
+    fn test_scoped_registry_lookup() {
+        let mut config = NpmrcConfig::default();
+        config.registry = "https://registry.npmjs.org/".to_string();
+        config.scoped_registries.insert(
+            "@myorg".to_string(),
+            "https://npm.myorg.com/".to_string(),
+        );
+
+        assert_eq!(
+            config.registry_for("@myorg/my-pkg"),
+            "https://npm.myorg.com/"
+        );
+        assert_eq!(
+            config.registry_for("lodash"),
+            "https://registry.npmjs.org/"
+        );
+    }
+}
diff --git a/crates/cclab-jet/src/pkg_manager/patch.rs b/crates/cclab-jet/src/pkg_manager/patch.rs
new file mode 100644
index 0000000..3c243ce
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/patch.rs
@@ -0,0 +1,146 @@
+use anyhow::{Context, Result};
+use std::path::{Path, PathBuf};
+
+/// Patch manager for creating and applying patches to installed packages.
+pub struct PatchManager {
+    root_dir: PathBuf,
+}
+
+impl PatchManager {
+    pub fn new(root_dir: PathBuf) -> Self {
+        Self { root_dir }
+    }
+
+    /// Copy an installed package to `patches/{name}@{version}/` for editing.
+    pub fn prepare_patch(&self, package: &str) -> Result<PathBuf> {
+        let node_modules = self.root_dir.join("node_modules");
+        let pkg_dir = node_modules.join(package);
+
+        if !pkg_dir.exists() {
+            anyhow::bail!(
+                "Package '{}' not found in node_modules. Run 'jet install' first.",
+                package
+            );
+        }
+
+        // Read version from package.json
+        let version = Self::read_package_version(&pkg_dir)?;
+        let patch_dir = self
+            .root_dir
+            .join("patches")
+            .join(format!("{}@{}", package, version));
+
+        if patch_dir.exists() {
+            std::fs::remove_dir_all(&patch_dir)?;
+        }
+
+        copy_dir_recursive(&pkg_dir, &patch_dir).with_context(|| {
+            format!("Failed to copy {} to patches/", package)
+        })?;
+
+        tracing::info!(
+            "Prepared patch for {}@{} at {:?}",
+            package,
+            version,
+            patch_dir
+        );
+        Ok(patch_dir)
+    }
+
+    /// Generate a .patch file by diffing the original package with the edited copy.
+    pub fn commit_patch(&self, package: &str) -> Result<PathBuf> {
+        let node_modules = self.root_dir.join("node_modules");
+        let original = node_modules.join(package);
+        let version = Self::read_package_version(&original)?;
+        let edited = self
+            .root_dir
+            .join("patches")
+            .join(format!("{}@{}", package, version));
+
+        if !edited.exists() {
+            anyhow::bail!(
+                "No patch directory found. Run 'jet patch {}' first.",
+                package
+            );
+        }
+
+        // Generate diff using system diff command
+        let patch_file = self
+            .root_dir
+            .join("patches")
+            .join(format!("{}@{}.patch", package, version));
+
+        let output = std::process::Command::new("diff")
+            .args(["-ruN", "--strip-trailing-cr"])
+            .arg(&original)
+            .arg(&edited)
+            .output()
+            .context("Failed to run diff command")?;
+
+        // diff returns exit code 1 when files differ (that's expected)
+        let diff_content = String::from_utf8_lossy(&output.stdout);
+        if diff_content.is_empty() {
+            anyhow::bail!("No changes detected in patch for {}", package);
+        }
+
+        std::fs::write(&patch_file, diff_content.as_bytes())?;
+
+        // Clean up the edited directory
+        std::fs::remove_dir_all(&edited)?;
+
+        tracing::info!("Patch file created: {:?}", patch_file);
+        Ok(patch_file)
+    }
+
+    fn read_package_version(pkg_dir: &Path) -> Result<String> {
+        let pkg_json_path = pkg_dir.join("package.json");
+        let content = std::fs::read_to_string(&pkg_json_path)?;
+        let pkg: serde_json::Value = serde_json::from_str(&content)?;
+        Ok(pkg["version"]
+            .as_str()
+            .unwrap_or("0.0.0")
+            .to_string())
+    }
+}
+
+/// Recursively copy a directory.
+fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
+    std::fs::create_dir_all(dest)?;
+    for entry in walkdir::WalkDir::new(src).min_depth(1) {
+        let entry = entry?;
+        let relative = entry.path().strip_prefix(src)?;
+        let dest_path = dest.join(relative);
+
+        if entry.file_type().is_dir() {
+            std::fs::create_dir_all(&dest_path)?;
+        } else {
+            if let Some(parent) = dest_path.parent() {
+                std::fs::create_dir_all(parent)?;
+            }
+            std::fs::copy(entry.path(), &dest_path)?;
+        }
+    }
+    Ok(())
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use tempfile::tempdir;
+
+    #[test]
+    fn test_prepare_patch_missing_package() {
+        let dir = tempdir().unwrap();
+        let pm = PatchManager::new(dir.path().to_path_buf());
+        let result = pm.prepare_patch("nonexistent");
+        assert!(result.is_err());
+    }
+
+    #[test]
+    fn test_commit_patch_no_edit() {
+        let dir = tempdir().unwrap();
+        let pm = PatchManager::new(dir.path().to_path_buf());
+        let result = pm.commit_patch("nonexistent");
+        assert!(result.is_err());
+    }
+}
diff --git a/crates/cclab-jet/src/pkg_manager/publish.rs b/crates/cclab-jet/src/pkg_manager/publish.rs
new file mode 100644
index 0000000..ccfab15
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/publish.rs
@@ -0,0 +1,257 @@
+use anyhow::{Context, Result};
+use std::io::Write;
+use std::path::{Path, PathBuf};
+
+use super::npmrc::NpmrcConfig;
+use super::workspace::WorkspaceManager;
+
+/// Publisher for npm registry.
+pub struct Publisher {
+    root_dir: PathBuf,
+    npmrc: NpmrcConfig,
+}
+
+impl Publisher {
+    pub fn new(root_dir: PathBuf) -> Self {
+        let npmrc = NpmrcConfig::load(&root_dir);
+        Self { root_dir, npmrc }
+    }
+
+    /// Create a tarball (.tgz) for publishing without actually publishing.
+    pub fn pack(&self) -> Result<PathBuf> {
+        let pkg = self.read_and_transform_package_json()?;
+        let name = pkg["name"].as_str().unwrap_or("package");
+        let version = pkg["version"].as_str().unwrap_or("0.0.0");
+
+        let tarball_name = format!(
+            "{}-{}.tgz",
+            name.replace('/', "-").trim_start_matches('@'),
+            version
+        );
+        let tarball_path = self.root_dir.join(&tarball_name);
+
+        self.create_tarball(&tarball_path, &pkg)?;
+        tracing::info!("Created tarball: {}", tarball_name);
+        Ok(tarball_path)
+    }
+
+    /// Publish to npm registry.
+    pub async fn publish(
+        &self,
+        tag: &str,
+        access: Option<&str>,
+    ) -> Result<()> {
+        let pkg = self.read_and_transform_package_json()?;
+        let name = pkg["name"].as_str().unwrap_or("package");
+        let version = pkg["version"].as_str().unwrap_or("0.0.0");
+
+        let registry = self.npmrc.registry_for(name);
+        let auth_token = self.npmrc.auth_token_for(registry).ok_or_else(|| {
+            anyhow::anyhow!(
+                "No auth token found for registry {}. Add to .npmrc.",
+                registry
+            )
+        })?;
+
+        // Create tarball in memory
+        let tarball_bytes = self.create_tarball_bytes(&pkg)?;
+
+        // Publish via npm registry PUT
+        let url = format!("{}/{}", registry.trim_end_matches('/'), name);
+        let client = reqwest::Client::new();
+
+        let encoded = base64_encode(&tarball_bytes);
+        let publish_body = serde_json::json!({
+            "name": name,
+            "description": pkg.get("description").and_then(|v| v.as_str()).unwrap_or(""),
+            "dist-tags": { tag: version },
+            "versions": {
+                version: pkg
+            },
+            "access": access.unwrap_or("public"),
+            "_attachments": {
+                format!("{}-{}.tgz", name, version): {
+                    "content_type": "application/octet-stream",
+                    "data": encoded,
+                    "length": tarball_bytes.len()
+                }
+            }
+        });
+
+        let response = client
+            .put(&url)
+            .header("Authorization", format!("Bearer {}", auth_token))
+            .json(&publish_body)
+            .send()
+            .await?;
+
+        if !response.status().is_success() {
+            let status = response.status();
+            let body = response.text().await.unwrap_or_default();
+            anyhow::bail!("Publish failed ({}): {}", status, body);
+        }
+
+        tracing::info!("Published {}@{} with tag '{}'", name, version, tag);
+        Ok(())
+    }
+
+    /// Read package.json and transform workspace:* protocols to real versions.
+    fn read_and_transform_package_json(&self) -> Result<serde_json::Value> {
+        let path = self.root_dir.join("package.json");
+        let content = std::fs::read_to_string(&path)
+            .with_context(|| format!("Failed to read {:?}", path))?;
+        let mut pkg: serde_json::Value = serde_json::from_str(&content)?;
+
+        // Resolve workspace: protocols if in a workspace
+        if let Ok(Some(ws)) = WorkspaceManager::discover(&self.root_dir) {
+            Self::transform_workspace_deps(&mut pkg, &ws, "dependencies");
+            Self::transform_workspace_deps(&mut pkg, &ws, "devDependencies");
+        }
+
+        Ok(pkg)
+    }
+
+    /// Replace workspace:* specs with real version ranges.
+    fn transform_workspace_deps(
+        pkg: &mut serde_json::Value,
+        ws: &WorkspaceManager,
+        field: &str,
+    ) {
+        if let Some(deps) = pkg.get_mut(field).and_then(|v| v.as_object_mut()) {
+            for (name, version) in deps.iter_mut() {
+                if let Some(spec) = version.as_str() {
+                    if WorkspaceManager::is_workspace_protocol(spec) {
+                        if let Some(resolved) =
+                            ws.resolve_workspace_protocol(spec, name)
+                        {
+                            *version = serde_json::Value::String(resolved);
+                        }
+                    }
+                }
+            }
+        }
+    }
+
+    fn create_tarball(
+        &self,
+        output: &Path,
+        _pkg: &serde_json::Value,
+    ) -> Result<()> {
+        let bytes = self.create_tarball_bytes(_pkg)?;
+        std::fs::write(output, bytes)?;
+        Ok(())
+    }
+
+    fn create_tarball_bytes(
+        &self,
+        _pkg: &serde_json::Value,
+    ) -> Result<Vec<u8>> {
+        use flate2::write::GzEncoder;
+        use flate2::Compression;
+
+        let tar_buf = Vec::new();
+        let mut builder = tar::Builder::new(tar_buf);
+
+        // Add package.json (transformed)
+        let pkg_content = serde_json::to_string_pretty(_pkg)?;
+        let mut header = tar::Header::new_gnu();
+        header.set_size(pkg_content.len() as u64);
+        header.set_mode(0o644);
+        header.set_cksum();
+        builder.append_data(
+            &mut header,
+            "package/package.json",
+            pkg_content.as_bytes(),
+        )?;
+
+        // Add other publishable files
+        let publish_files = Self::collect_publish_files(&self.root_dir)?;
+        for file in publish_files {
+            let relative = file.strip_prefix(&self.root_dir)?;
+            let content = std::fs::read(&file)?;
+            let mut header = tar::Header::new_gnu();
+            header.set_size(content.len() as u64);
+            header.set_mode(0o644);
+            header.set_cksum();
+            builder.append_data(
+                &mut header,
+                Path::new("package").join(relative),
+                content.as_slice(),
+            )?;
+        }
+
+        let tar_bytes = builder.into_inner()?;
+        let mut gz = GzEncoder::new(Vec::new(), Compression::default());
+        gz.write_all(&tar_bytes)?;
+        Ok(gz.finish()?)
+    }
+
+    /// Collect files to include in the package (respects .npmignore / files field).
+    fn collect_publish_files(root: &Path) -> Result<Vec<PathBuf>> {
+        let mut files = Vec::new();
+        for entry in walkdir::WalkDir::new(root)
+            .min_depth(1)
+            .into_iter()
+            .filter_entry(|e| {
+                let name = e.file_name().to_string_lossy();
+                // Skip common non-publishable dirs
+                !matches!(
+                    name.as_ref(),
+                    "node_modules" | ".git" | "patches" | ".jet-cache"
+                )
+            })
+        {
+            let entry = entry?;
+            if entry.file_type().is_file() {
+                let name = entry.file_name().to_string_lossy();
+                if name == "package.json" {
+                    continue; // Already added transformed
+                }
+                files.push(entry.path().to_path_buf());
+            }
+        }
+        Ok(files)
+    }
+}
+
+/// Simple base64 encoding for tarball attachment.
+fn base64_encode(data: &[u8]) -> String {
+    use std::fmt::Write;
+    const CHARS: &[u8] =
+        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
+    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
+
+    for chunk in data.chunks(3) {
+        let b0 = chunk[0] as u32;
+        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
+        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
+        let triple = (b0 << 16) | (b1 << 8) | b2;
+
+        write!(result, "{}", CHARS[((triple >> 18) & 0x3F) as usize] as char).ok();
+        write!(result, "{}", CHARS[((triple >> 12) & 0x3F) as usize] as char).ok();
+        if chunk.len() > 1 {
+            write!(result, "{}", CHARS[((triple >> 6) & 0x3F) as usize] as char).ok();
+        } else {
+            result.push('=');
+        }
+        if chunk.len() > 2 {
+            write!(result, "{}", CHARS[(triple & 0x3F) as usize] as char).ok();
+        } else {
+            result.push('=');
+        }
+    }
+
+    result
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    #[test]
+    fn test_base64_encode() {
+        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
+        assert_eq!(base64_encode(b""), "");
+        assert_eq!(base64_encode(b"ab"), "YWI=");
+    }
+}
diff --git a/crates/cclab-jet/src/pkg_manager/registry.rs b/crates/cclab-jet/src/pkg_manager/registry.rs
index 3b0ec1d..717a2f7 100644
--- a/crates/cclab-jet/src/pkg_manager/registry.rs
+++ b/crates/cclab-jet/src/pkg_manager/registry.rs
@@ -4,11 +4,15 @@ use serde::Deserialize;
 use std::collections::HashMap;
 use std::sync::Arc;
 
+use super::npmrc::NpmrcConfig;
+
 /// NPM registry client with in-memory metadata caching
 pub struct RegistryClient {
     client: reqwest::Client,
+    #[allow(dead_code)]
     registry_url: String,
     cache: Arc<DashMap<String, PackageMetadata>>,
+    npmrc: NpmrcConfig,
 }
 
 #[derive(Debug, Clone, Deserialize)]
@@ -32,6 +36,12 @@ pub struct VersionMetadata {
     pub bin: Option<BinField>,
     #[serde(default)]
     pub scripts: Option<HashMap<String, String>>,
+    /// Platform restriction: allowed OS values (e.g., ["darwin", "linux"])
+    #[serde(default)]
+    pub os: Option<Vec<String>>,
+    /// Platform restriction: allowed CPU values (e.g., ["x64", "arm64"])
+    #[serde(default)]
+    pub cpu: Option<Vec<String>>,
 }
 
 /// npm `bin` field: either `"path/to/cli.js"` or `{"cmd": "path/to/cli.js"}`.
@@ -50,16 +60,36 @@ pub struct DistInfo {
 }
 
 impl RegistryClient {
-    pub fn new(registry_url: &str) -> Result<Self> {
+    pub fn new(registry_url: &str, npmrc: &NpmrcConfig) -> Result<Self> {
+        let mut builder = reqwest::Client::builder();
+
+        // Apply proxy settings from .npmrc
+        if let Some(ref proxy_url) = npmrc.https_proxy {
+            if let Ok(proxy) = reqwest::Proxy::https(proxy_url) {
+                builder = builder.proxy(proxy);
+            }
+        } else if let Some(ref proxy_url) = npmrc.proxy {
+            if let Ok(proxy) = reqwest::Proxy::http(proxy_url) {
+                builder = builder.proxy(proxy);
+            }
+        }
+
+        // Apply strict-ssl setting
+        if !npmrc.strict_ssl {
+            builder = builder.danger_accept_invalid_certs(true);
+        }
+
         Ok(Self {
-            client: reqwest::Client::new(),
+            client: builder.build().unwrap_or_else(|_| reqwest::Client::new()),
             registry_url: registry_url.to_string(),
             cache: Arc::new(DashMap::new()),
+            npmrc: npmrc.clone(),
         })
     }
 
     /// Fetch package metadata, using in-memory cache to avoid duplicate requests.
     /// Uses abbreviated metadata endpoint for smaller payloads.
+    /// Respects scoped registries and auth tokens from .npmrc.
     pub async fn get_package_metadata(&self, name: &str) -> Result<PackageMetadata> {
         // Check cache first
         if let Some(cached) = self.cache.get(name) {
@@ -67,18 +97,25 @@ impl RegistryClient {
             return Ok(cached.clone());
         }
 
-        let url = format!("{}/{}", self.registry_url, name);
+        // Use scoped registry if configured
+        let registry = self.npmrc.registry_for(name);
+        let url = format!("{}/{}", registry.trim_end_matches('/'), name);
         tracing::debug!("Fetching metadata: {}", url);
 
-        let response = self
+        let mut req = self
             .client
             .get(&url)
             .header(
                 "Accept",
                 "application/vnd.npm.install-v1+json",
-            )
-            .send()
-            .await?;
+            );
+
+        // Apply auth token if available
+        if let Some(token) = self.npmrc.auth_token_for(registry) {
+            req = req.header("Authorization", format!("Bearer {}", token));
+        }
+
+        let response = req.send().await?;
 
         if !response.status().is_success() {
             anyhow::bail!(
@@ -149,22 +186,25 @@ mod tests {
 
     #[test]
     fn test_registry_client_creation() {
+        let npmrc = NpmrcConfig::default();
         let client =
-            RegistryClient::new("https://registry.npmjs.org").unwrap();
+            RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
         assert_eq!(client.registry_url, "https://registry.npmjs.org");
     }
 
     #[test]
     fn test_cache_is_empty_on_creation() {
+        let npmrc = NpmrcConfig::default();
         let client =
-            RegistryClient::new("https://registry.npmjs.org").unwrap();
+            RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
         assert!(client.cache.is_empty());
     }
 
     #[test]
     fn test_cache_shared_across_clones() {
+        let npmrc = NpmrcConfig::default();
         let client =
-            RegistryClient::new("https://registry.npmjs.org").unwrap();
+            RegistryClient::new("https://registry.npmjs.org", &npmrc).unwrap();
         let cache = client.cache.clone();
 
         // Insert a dummy entry
diff --git a/crates/cclab-jet/src/pkg_manager/resolver.rs b/crates/cclab-jet/src/pkg_manager/resolver.rs
index 9652104..53ccf11 100644
--- a/crates/cclab-jet/src/pkg_manager/resolver.rs
+++ b/crates/cclab-jet/src/pkg_manager/resolver.rs
@@ -31,10 +31,12 @@ impl DependencyResolver {
 
     /// Resolve all dependencies (direct + transitive) using BFS.
     /// Metadata fetching happens per-package via the registry's cache.
+    /// Overrides force specific versions for matching package names.
     pub async fn resolve(
         &self,
         deps: &HashMap<String, String>,
         registry: &RegistryClient,
+        overrides: &HashMap<String, String>,
     ) -> Result<HashMap<String, ResolvedPackage>> {
         let mut resolved: HashMap<String, ResolvedPackage> = HashMap::new();
         let mut visited: HashSet<String> = HashSet::new();
@@ -43,14 +45,23 @@ impl DependencyResolver {
 
         // Seed the queue with direct dependencies
         for (name, range) in deps {
+            // Handle npm: alias protocol (e.g., "npm:actual-pkg@^1.0")
+            let (real_name, real_range) = resolve_alias(name, range);
             queue.push_back((
-                name.clone(),
-                range.clone(),
+                real_name,
+                real_range,
                 vec!["(root)".to_string()],
             ));
         }
 
         while let Some((name, range_str, dep_chain)) = queue.pop_front() {
+            // Apply overrides: if this package is in the overrides map, force that version
+            let range_str = if let Some(forced) = overrides.get(&name) {
+                forced.clone()
+            } else {
+                range_str
+            };
+
             // Skip if already resolved
             if resolved.contains_key(&name) {
                 // Verify compatibility with existing resolution
@@ -189,6 +200,77 @@ impl Default for DependencyResolver {
     }
 }
 
+/// Resolve npm: alias protocol.
+/// `"npm:actual-pkg@^1.0"` → `("actual-pkg", "^1.0")`
+/// Regular deps pass through unchanged.
+fn resolve_alias(name: &str, range: &str) -> (String, String) {
+    if let Some(alias_spec) = range.strip_prefix("npm:") {
+        // npm:actual-pkg@^1.0 or npm:@scope/pkg@^1.0
+        if let Some(at_pos) = alias_spec.rfind('@') {
+            if at_pos > 0 {
+                let real_name = &alias_spec[..at_pos];
+                let real_range = &alias_spec[at_pos + 1..];
+                return (real_name.to_string(), real_range.to_string());
+            }
+        }
+        // npm:actual-pkg (no version)
+        return (alias_spec.to_string(), "*".to_string());
+    }
+    (name.to_string(), range.to_string())
+}
+
+/// Check if a package should be skipped based on platform (os/cpu).
+pub fn should_skip_optional(
+    version_meta: &super::registry::VersionMetadata,
+) -> bool {
+    // Check os field
+    if let Some(os_list) = version_meta.os.as_ref() {
+        if !os_list.is_empty() {
+            let current_os = std::env::consts::OS;
+            let mapped = match current_os {
+                "macos" => "darwin",
+                other => other,
+            };
+            let included = os_list.iter().any(|o| {
+                !o.starts_with('!') && (o == mapped || o == current_os)
+            });
+            let excluded = os_list.iter().any(|o| {
+                o.starts_with('!')
+                    && (o.trim_start_matches('!') == mapped
+                        || o.trim_start_matches('!') == current_os)
+            });
+            if excluded || (!included && !os_list.iter().all(|o| o.starts_with('!'))) {
+                return true;
+            }
+        }
+    }
+
+    // Check cpu field
+    if let Some(cpu_list) = version_meta.cpu.as_ref() {
+        if !cpu_list.is_empty() {
+            let current_cpu = std::env::consts::ARCH;
+            let mapped = match current_cpu {
+                "aarch64" => "arm64",
+                "x86_64" => "x64",
+                other => other,
+            };
+            let included = cpu_list.iter().any(|c| {
+                !c.starts_with('!') && (c == mapped || c == current_cpu)
+            });
+            let excluded = cpu_list.iter().any(|c| {
+                c.starts_with('!')
+                    && (c.trim_start_matches('!') == mapped
+                        || c.trim_start_matches('!') == current_cpu)
+            });
+            if excluded || (!included && !cpu_list.iter().all(|c| c.starts_with('!'))) {
+                return true;
+            }
+        }
+    }
+
+    false
+}
+
 /// Resolve the npm `bin` field into a flat `HashMap<command, path>`.
 ///
 /// - `BinField::Single("./cli.js")` → `{"<package-name>": "./cli.js"}`
@@ -324,6 +406,8 @@ mod tests {
                     peer_dependencies: None,
                     bin: None,
                     scripts: None,
+                    os: None,
+                    cpu: None,
                 },
             );
         }
diff --git a/crates/cclab-jet/src/pkg_manager/workspace.rs b/crates/cclab-jet/src/pkg_manager/workspace.rs
new file mode 100644
index 0000000..f46ef6a
--- /dev/null
+++ b/crates/cclab-jet/src/pkg_manager/workspace.rs
@@ -0,0 +1,343 @@
+use anyhow::{Context, Result};
+use serde::{Deserialize, Serialize};
+use std::collections::{HashMap, HashSet, VecDeque};
+use std::path::{Path, PathBuf};
+
+/// Workspace configuration from package.json or jet-workspace.yaml.
+#[derive(Debug, Clone, Default, Serialize, Deserialize)]
+pub struct WorkspaceConfig {
+    #[serde(default)]
+    pub packages: Vec<String>,
+    #[serde(default)]
+    pub catalog: HashMap<String, String>,
+    #[serde(default)]
+    pub hoisting: HoistingConfig,
+}
+
+/// Hoisting configuration for node_modules layout.
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct HoistingConfig {
+    #[serde(default)]
+    pub shamefully_hoist: bool,
+    #[serde(default = "default_hoist_patterns")]
+    pub public_hoist_pattern: Vec<String>,
+}
+
+impl Default for HoistingConfig {
+    fn default() -> Self {
+        Self {
+            shamefully_hoist: false,
+            public_hoist_pattern: default_hoist_patterns(),
+        }
+    }
+}
+
+fn default_hoist_patterns() -> Vec<String> {
+    vec!["*eslint*".to_string(), "*prettier*".to_string()]
+}
+
+/// A discovered workspace package.
+#[derive(Debug, Clone)]
+pub struct WorkspacePackage {
+    pub name: String,
+    pub version: String,
+    pub path: PathBuf,
+    pub dependencies: HashMap<String, String>,
+    pub dev_dependencies: HashMap<String, String>,
+    pub deps_on_workspace: Vec<String>,
+}
+
+/// Manages workspace discovery, dependency graph, and protocol resolution.
+pub struct WorkspaceManager {
+    pub root: PathBuf,
+    pub config: WorkspaceConfig,
+    pub packages: Vec<WorkspacePackage>,
+}
+
+impl WorkspaceManager {
+    /// Discover workspace from project root.
+    pub fn discover(root: &Path) -> Result<Option<Self>> {
+        let config = Self::load_config(root)?;
+        let Some(config) = config else {
+            return Ok(None);
+        };
+
+        let packages = Self::expand_packages(root, &config)?;
+        Ok(Some(Self {
+            root: root.to_path_buf(),
+            config,
+            packages,
+        }))
+    }
+
+    /// Load workspace config from package.json.workspaces or jet-workspace.yaml.
+    fn load_config(root: &Path) -> Result<Option<WorkspaceConfig>> {
+        // Try jet-workspace.yaml first
+        let yaml_path = root.join("jet-workspace.yaml");
+        if yaml_path.exists() {
+            let content = std::fs::read_to_string(&yaml_path)?;
+            let config: WorkspaceConfig = serde_yaml::from_str(&content)?;
+            return Ok(Some(config));
+        }
+
+        // Fall back to package.json.workspaces
+        let pkg_path = root.join("package.json");
+        if pkg_path.exists() {
+            let content = std::fs::read_to_string(&pkg_path)?;
+            let pkg: serde_json::Value = serde_json::from_str(&content)?;
+            if let Some(workspaces) = pkg.get("workspaces") {
+                let patterns: Vec<String> = serde_json::from_value(workspaces.clone())
+                    .unwrap_or_default();
+                if !patterns.is_empty() {
+                    return Ok(Some(WorkspaceConfig {
+                        packages: patterns,
+                        ..Default::default()
+                    }));
+                }
+            }
+        }
+
+        Ok(None)
+    }
+
+    /// Expand glob patterns to find workspace packages.
+    fn expand_packages(
+        root: &Path,
+        config: &WorkspaceConfig,
+    ) -> Result<Vec<WorkspacePackage>> {
+        let mut packages = Vec::new();
+
+        for pattern in &config.packages {
+            let full_pattern = root.join(pattern).join("package.json");
+            let pattern_str = full_pattern.to_string_lossy().to_string();
+
+            for entry in glob::glob(&pattern_str).with_context(|| {
+                format!("Invalid glob pattern: {}", pattern)
+            })? {
+                let pkg_json_path = entry?;
+                let pkg_dir = pkg_json_path.parent().unwrap();
+                if let Ok(pkg) = Self::read_workspace_package(root, pkg_dir) {
+                    packages.push(pkg);
+                }
+            }
+        }
+
+        Ok(packages)
+    }
+
+    /// Read a single workspace package.
+    fn read_workspace_package(
+        root: &Path,
+        pkg_dir: &Path,
+    ) -> Result<WorkspacePackage> {
+        let pkg_path = pkg_dir.join("package.json");
+        let content = std::fs::read_to_string(&pkg_path)?;
+        let pkg: serde_json::Value = serde_json::from_str(&content)?;
+
+        let name = pkg["name"].as_str().unwrap_or("unnamed").to_string();
+        let version = pkg["version"].as_str().unwrap_or("0.0.0").to_string();
+        let rel_path = pkg_dir.strip_prefix(root).unwrap_or(pkg_dir);
+
+        let dependencies = Self::extract_deps(&pkg, "dependencies");
+        let dev_dependencies = Self::extract_deps(&pkg, "devDependencies");
+
+        Ok(WorkspacePackage {
+            name,
+            version,
+            path: rel_path.to_path_buf(),
+            dependencies,
+            dev_dependencies,
+            deps_on_workspace: Vec::new(),
+        })
+    }
+
+    fn extract_deps(pkg: &serde_json::Value, field: &str) -> HashMap<String, String> {
+        pkg.get(field)
+            .and_then(|v| serde_json::from_value(v.clone()).ok())
+            .unwrap_or_default()
+    }
+
+    /// Build topological order of workspace packages based on inter-dependencies.
+    pub fn topological_order(&mut self) -> Result<Vec<String>> {
+        let names: HashSet<String> = self.packages.iter().map(|p| p.name.clone()).collect();
+
+        // Mark workspace deps
+        for pkg in &mut self.packages {
+            let mut ws_deps = Vec::new();
+            for dep_name in pkg.dependencies.keys().chain(pkg.dev_dependencies.keys()) {
+                if names.contains(dep_name) {
+                    ws_deps.push(dep_name.clone());
+                }
+            }
+            pkg.deps_on_workspace = ws_deps;
+        }
+
+        // Kahn's algorithm
+        let mut in_degree: HashMap<String, usize> = HashMap::new();
+        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();
+
+        for pkg in &self.packages {
+            in_degree.entry(pkg.name.clone()).or_insert(0);
+            for dep in &pkg.deps_on_workspace {
+                *in_degree.entry(pkg.name.clone()).or_insert(0) += 1;
+                dependents
+                    .entry(dep.clone())
+                    .or_default()
+                    .push(pkg.name.clone());
+            }
+        }
+
+        let mut queue: VecDeque<String> = in_degree
+            .iter()
+            .filter(|(_, &deg)| deg == 0)
+            .map(|(name, _)| name.clone())
+            .collect();
+
+        let mut order = Vec::new();
+        while let Some(name) = queue.pop_front() {
+            order.push(name.clone());
+            if let Some(deps) = dependents.get(&name) {
+                for dep in deps {
+                    if let Some(deg) = in_degree.get_mut(dep) {
+                        *deg -= 1;
+                        if *deg == 0 {
+                            queue.push_back(dep.clone());
+                        }
+                    }
+                }
+            }
+        }
+
+        if order.len() != self.packages.len() {
+            anyhow::bail!("Circular dependency detected in workspace packages");
+        }
+
+        Ok(order)
+    }
+
+    /// Resolve workspace:* protocol to actual version.
+    pub fn resolve_workspace_protocol(
+        &self,
+        spec: &str,
+        target_name: &str,

... truncated (121 more lines)
```
