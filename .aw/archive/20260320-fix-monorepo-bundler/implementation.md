---
id: implementation
type: change_implementation
change_id: fix-monorepo-bundler
---

# Implementation

## Summary

Fix bundler circular dependency error and monorepo node_modules walk-up. 3 source files changed, 1 new integration test file. (1) bundler/graph.rs: add all_node_ids() — returns modules in insertion order as fallback when topological sort fails due to cycles; (2) bundler/mod.rs: replace hard error on circular deps with graceful fallback — transform_modules() now returns (Vec<CompiledModule>, bool) where bool=has_cycle; generate_bundle() checks has_cycle first and routes to generate_bundle_with_runtime() which handles circular refs natively via pre-seeded cache pattern; (3) resolver/mod.rs: add 2 unit tests — test_resolver_walks_up_to_monorepo_root_node_modules and test_resolver_skips_intermediate_node_modules_without_package — verifying the existing resolve_package walk-up loop resolves packages from workspace root node_modules; (4) tests/bundler_monorepo.rs (new): 4 integration tests — test_bundler_circular_dependency_completes (A→B→A cycle must succeed using __jet__ runtime), test_bundler_three_module_cycle_completes (A→B→C→A cycle), test_bundler_resolves_monorepo_workspace_root_package (React at workspace root bundled correctly), test_bundler_project_node_modules_takes_priority (project-level wins over workspace-root).

## Diff

```diff
diff --git a/crates/cclab-jet/src/bundler/graph.rs b/crates/cclab-jet/src/bundler/graph.rs
index f3d503a9..44c00d56 100644
--- a/crates/cclab-jet/src/bundler/graph.rs
+++ b/crates/cclab-jet/src/bundler/graph.rs
@@ -155,6 +155,15 @@ impl ModuleGraph {
         self.graph.node_count()
     }
 
+    /// Get all module IDs in graph insertion order.
+    ///
+    /// Used as a fallback ordering when topological sort fails due to cycles.
+    /// Insertion order guarantees the entry point (first module added) is at
+    /// index 0, which is required by `generate_bundle_with_runtime`.
+    pub fn all_node_ids(&self) -> Vec<ModuleId> {
+        self.graph.node_indices().collect()
+    }
+
     pub fn clear(&mut self) {
         self.graph.clear();
         self.path_to_id.clear();
diff --git a/crates/cclab-jet/src/bundler/mod.rs b/crates/cclab-jet/src/bundler/mod.rs
index 38c7ae69..4e15979f 100644
--- a/crates/cclab-jet/src/bundler/mod.rs
+++ b/crates/cclab-jet/src/bundler/mod.rs
@@ -189,8 +189,8 @@ impl Bundler {
         tracing::info!("Starting bundle from entry: {:?}", entry);
 
         self.build_graph(&entry).await?;
-        let modules = self.transform_modules().await?;
-        let output = self.generate_bundle(modules)?;
+        let (modules, has_cycle) = self.transform_modules().await?;
+        let output = self.generate_bundle(modules, has_cycle)?;
 
         Ok(output)
     }
@@ -351,17 +351,10 @@ impl Bundler {
         let module_count = graph.module_count();
 
         if graph.has_cycle() {
-            tracing::warn!("Circular dependencies detected in module graph");
-            if let Err(cycle_paths) = graph.topological_sort() {
-                tracing::error!("Dependency cycle:");
-                for (i, path) in cycle_paths.iter().enumerate() {
-                    tracing::error!("  {} -> {:?}", i + 1, path);
-                }
-                return Err(anyhow::anyhow!(
-                    "Circular dependency detected: {} modules in cycle",
-                    cycle_paths.len()
-                ));
-            }
+            tracing::warn!(
+                "Circular dependencies detected in module graph — \
+                 will use runtime module system (generate_bundle_with_runtime)"
+            );
         }
 
         tracing::info!("Module graph built: {} modules", module_count);
@@ -380,17 +373,26 @@ impl Bundler {
         Ok(std::fs::canonicalize(&resolved.path)?)
     }
 
-    async fn transform_modules(&self) -> Result<Vec<CompiledModule>> {
+    async fn transform_modules(&self) -> Result<(Vec<CompiledModule>, bool)> {
         tracing::debug!("Transforming modules");
 
         let graph = self.graph.read();
 
-        let sorted_ids = graph.topological_sort().map_err(|cycle_paths| {
-            anyhow::anyhow!(
-                "Cannot transform modules with circular dependencies: {:?}",
-                cycle_paths
-            )
-        })?;
+        let (sorted_ids, has_cycle) = match graph.topological_sort() {
+            Ok(ids) => (ids, false),
+            Err(cycle_paths) => {
+                tracing::warn!(
+                    "Circular dependency cycle detected ({} modules): {:?}",
+                    cycle_paths.len(),
+                    cycle_paths
+                );
+                tracing::warn!(
+                    "Using graph insertion order as module ID assignment; \
+                     bundle will use runtime module system"
+                );
+                (graph.all_node_ids(), true)
+            }
+        };
 
         let module_map: std::collections::HashMap<PathBuf, usize> = sorted_ids
             .iter()
@@ -470,10 +472,10 @@ impl Bundler {
 
         tracing::info!("Transformed {} modules", modules.len());
 
-        Ok(modules)
+        Ok((modules, has_cycle))
     }
 
-    fn generate_bundle(&self, modules: Vec<CompiledModule>) -> Result<BundleOutput> {
+    fn generate_bundle(&self, modules: Vec<CompiledModule>, has_cycle: bool) -> Result<BundleOutput> {
         tracing::debug!("Generating bundle from {} modules", modules.len());
 
         if modules.is_empty() {
@@ -484,11 +486,14 @@ impl Bundler {
             });
         }
 
-        // Scope-hoisted bundle: eliminates the full `__jet__` module runtime
-        // and gives the minifier cross-module visibility for DCE and constant
-        // folding.
+        // Bundle format selection:
         //
-        // Phase selection:
+        //   Runtime (`generate_bundle_with_runtime`) — used when:
+        //     • circular dependencies are present (cycles prevent topo-sort;
+        //       the `__jet__.require` runtime handles circular refs natively
+        //       via the pre-seeded `cache[id] = { exports: {} }` pattern)
+        //     • dynamic import() calls are present (async chunks need the
+        //       module registry at runtime)
         //
         //   Phase 2 (true flattening) — used when `minify=true` and safe:
         //     `generate_flattened_bundle` merges all module bodies into a
@@ -504,10 +509,12 @@ impl Bundler {
         //   Phase 1 (per-module IIFE wrappers) — used when:
         //     • minify=false (dev builds; prefixed names would enlarge output)
         //     • any module uses eval/with/arguments[ (unsafe to merge scopes)
-        //
-        //   Runtime fallback — used when dynamic import() is present (async
-        //   chunks need the module registry at runtime).
-        let bundle = if scope_hoist::is_scope_hoist_safe(&modules) {
+        let bundle = if has_cycle {
+            tracing::debug!(
+                "Using runtime module system (circular dependencies present)"
+            );
+            generate_bundle_with_runtime(&modules)
+        } else if scope_hoist::is_scope_hoist_safe(&modules) {
             if self.minify && scope_hoist::is_flatten_safe(&modules) {
                 tracing::debug!(
                     "Using Phase 2 true module flattening \
diff --git a/crates/cclab-jet/src/resolver/mod.rs b/crates/cclab-jet/src/resolver/mod.rs
index 6b23a444..ac2c05e3 100644
--- a/crates/cclab-jet/src/resolver/mod.rs
+++ b/crates/cclab-jet/src/resolver/mod.rs
@@ -341,4 +341,151 @@ mod tests {
             ("@org/pkg".to_string(), Some("./a/b/c".to_string()))
         );
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // Monorepo walk-up: resolver finds node_modules at workspace root
+    // ──────────────────────────────────────────────────────────────────
+
+    /// Simulate an Nx monorepo layout:
+    ///
+    ///   workspace_root/                ← contains nx.json
+    ///     node_modules/react/          ← packages installed here
+    ///       package.json
+    ///       index.js
+    ///     apps/demo/src/App.tsx        ← importing file
+    ///
+    /// The resolver must walk up from `apps/demo/src/` and find
+    /// `node_modules/react` at the workspace root.
+    #[test]
+    fn test_resolver_walks_up_to_monorepo_root_node_modules() {
+        use tempfile::tempdir;
+        use std::io::Write;
+
+        let workspace = tempdir().unwrap();
+        let ws_root = workspace.path();
+
+        // Create workspace root marker
+        std::fs::write(ws_root.join("nx.json"), r#"{"affected":{}}"#).unwrap();
+
+        // Create react package at workspace root node_modules
+        let react_dir = ws_root.join("node_modules").join("react");
+        std::fs::create_dir_all(&react_dir).unwrap();
+        std::fs::write(
+            react_dir.join("package.json"),
+            r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
+        )
+        .unwrap();
+        std::fs::write(
+            react_dir.join("index.js"),
+            "exports.createElement = function() {}; exports.useState = function() {};",
+        )
+        .unwrap();
+
+        // Create deeply nested app source file
+        let src_dir = ws_root.join("apps").join("demo").join("src");
+        std::fs::create_dir_all(&src_dir).unwrap();
+        let entry_file = src_dir.join("App.tsx");
+        std::fs::write(
+            &entry_file,
+            r#"import React from 'react'; export default function App() { return null; }"#,
+        )
+        .unwrap();
+
+        let resolver = ModuleResolver::new(ResolveOptions {
+            extensions: vec![
+                "js".to_string(),
+                "jsx".to_string(),
+                "ts".to_string(),
+                "tsx".to_string(),
+            ],
+            resolve_index: true,
+            ..Default::default()
+        })
+        .unwrap();
+
+        // Resolve 'react' from the nested app file
+        let result = resolver.resolve("react", &entry_file);
+
+        assert!(
+            result.is_ok(),
+            "Should resolve 'react' from workspace root node_modules, got: {:?}",
+            result.err()
+        );
+        let resolved = result.unwrap();
+        assert!(!resolved.is_external, "react must NOT be treated as external");
+        assert!(
+            resolved.path.to_string_lossy().contains("node_modules/react"),
+            "Resolved path must be inside node_modules/react, got: {:?}",
+            resolved.path
+        );
+    }
+
+    /// Verify that the resolver correctly skips directories that don't have
+    /// the target package in their node_modules and keeps walking up.
+    #[test]
+    fn test_resolver_skips_intermediate_node_modules_without_package() {
+        use tempfile::tempdir;
+
+        let workspace = tempdir().unwrap();
+        let ws_root = workspace.path();
+
+        // Intermediate node_modules WITHOUT react (has a different package)
+        let intermediate_nm = ws_root.join("apps").join("demo").join("node_modules");
+        std::fs::create_dir_all(intermediate_nm.join("lodash")).unwrap();
+        std::fs::write(
+            intermediate_nm.join("lodash").join("package.json"),
+            r#"{"name":"lodash","version":"4.0.0","main":"lodash.js"}"#,
+        )
+        .unwrap();
+        std::fs::write(
+            intermediate_nm.join("lodash").join("lodash.js"),
+            "exports.merge = function() {};",
+        )
+        .unwrap();
+
+        // React only at workspace root
+        let react_dir = ws_root.join("node_modules").join("react");
+        std::fs::create_dir_all(&react_dir).unwrap();
+        std::fs::write(
+            react_dir.join("package.json"),
+            r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
+        )
+        .unwrap();
+        std::fs::write(react_dir.join("index.js"), "exports.createElement = function() {};")
+            .unwrap();
+
+        // Source file nested inside apps/demo/src/
+        let src_dir = ws_root.join("apps").join("demo").join("src");
+        std::fs::create_dir_all(&src_dir).unwrap();
+        let entry_file = src_dir.join("index.tsx");
+        std::fs::write(&entry_file, "import React from 'react';").unwrap();
+
+        let resolver = ModuleResolver::new(ResolveOptions {
+            extensions: vec!["js".to_string(), "ts".to_string(), "tsx".to_string()],
+            resolve_index: true,
+            ..Default::default()
+        })
+        .unwrap();
+
+        // 'react' must be found at workspace root even though intermediate
+        // node_modules exists (it only has lodash, not react)
+        let result = resolver.resolve("react", &entry_file);
+        assert!(
+            result.is_ok(),
+            "react should be found at workspace root despite intermediate node_modules: {:?}",
+            result.err()
+        );
+
+        // 'lodash' must be found at the intermediate level (closer wins)
+        let lodash_result = resolver.resolve("lodash", &entry_file);
+        assert!(
+            lodash_result.is_ok(),
+            "lodash should be found at intermediate node_modules: {:?}",
+            lodash_result.err()
+        );
+        assert!(
+            lodash_result.unwrap().path.to_string_lossy().contains("apps/demo/node_modules"),
+            "lodash must resolve from the closer intermediate node_modules"
+        );
+    }
 }

diff --git a/crates/cclab-jet/tests/bundler_monorepo.rs b/crates/cclab-jet/tests/bundler_monorepo.rs
new file mode 100644
index 00000000..xxxxxxxx
--- /dev/null
+++ b/crates/cclab-jet/tests/bundler_monorepo.rs
+//! Integration tests for bundler fixes:
+//!   1. Circular dependency handling — bundle completes instead of erroring
+//!   2. Monorepo node_modules walk-up — packages at workspace root are bundled
+
+use cclab_jet::bundler::{Bundler, BundleOptions};
+use cclab_jet::resolver::ResolveOptions;
+use std::collections::HashSet;
+use tempfile::tempdir;
+
+// Helper: write a file, creating parent dirs as needed.
+fn write_file(base: &std::path::Path, rel: &str, content: &str) {
+    let path = base.join(rel);
+    if let Some(parent) = path.parent() {
+        std::fs::create_dir_all(parent).unwrap();
+    }
+    std::fs::write(&path, content).unwrap();
+}
+
+// ──────────────────────────────────────────────────────────────────────────
+// Test 1: Circular dependency — bundle must complete without error
+// ──────────────────────────────────────────────────────────────────────────
+
+/// A → B → A forms a cycle (shared-ui-form-inputs pattern).
+/// Previously the bundler bailed with "Circular dependency detected" error.
+/// After the fix, the bundle must succeed using the runtime module system.
+#[tokio::test]
+async fn test_bundler_circular_dependency_completes() {
+    let dir = tempdir().unwrap();
+    let root = dir.path();
+
+    // a.js → imports b.js
+    write_file(
+        root,
+        "src/a.js",
+        r#"var b = require('./b');
+exports.hello = function() { return 'hello from a, b says: ' + b.world(); };
+"#,
+    );
+
+    // b.js → imports a.js (creates cycle a → b → a)
+    write_file(
+        root,
+        "src/b.js",
+        r#"var a = require('./a');
+exports.world = function() { return 'world from b'; };
+"#,
+    );
+
+    let entry = root.join("src/a.js");
+    let options = BundleOptions {
+        entry: entry.clone(),
+        output_dir: root.join("dist"),
+        resolve_options: ResolveOptions {
+            extensions: vec!["js".to_string()],
+            resolve_index: true,
+            ..Default::default()
+        },
+        ..Default::default()
+    };
+
+    let bundler = Bundler::new(options).unwrap();
+    let result = bundler.bundle(entry).await;
+
+    assert!(
+        result.is_ok(),
+        "Bundler must not bail on circular dependencies, got error: {:?}",
+        result.err()
+    );
+
+    let output = result.unwrap();
+    assert!(
+        !output.code.is_empty(),
+        "Bundle output must not be empty for circular dependency graph"
+    );
+
+    // The bundle must contain both modules
+    assert!(
+        output.code.contains("hello from a"),
+        "Module a content must be present in bundle"
+    );
+    assert!(
+        output.code.contains("world from b"),
+        "Module b content must be present in bundle"
+    );
+
+    // The runtime module system must be used (has __jet__.define and __jet__.require)
+    assert!(
+        output.code.contains("__jet__"),
+        "Cyclic bundle must use the __jet__ runtime module system"
+    );
+}
+
+/// Three-module cycle: A → B → C → A.
+/// Verifies the fallback handles larger cycles correctly.
+#[tokio::test]
+async fn test_bundler_three_module_cycle_completes() {
+    let dir = tempdir().unwrap();
+    let root = dir.path();
+
+    write_file(root, "src/a.js", "var b = require('./b'); exports.a = 1;");
+    write_file(root, "src/b.js", "var c = require('./c'); exports.b = 2;");
+    write_file(root, "src/c.js", "var a = require('./a'); exports.c = 3;");
+
+    let entry = root.join("src/a.js");
+    let options = BundleOptions {
+        entry: entry.clone(),
+        output_dir: root.join("dist"),
+        resolve_options: ResolveOptions {
+            extensions: vec!["js".to_string()],
+            resolve_index: true,
+            ..Default::default()
+        },
+        ..Default::default()
+    };
+
+    let bundler = Bundler::new(options).unwrap();
+    let result = bundler.bundle(entry).await;
+
+    assert!(
+        result.is_ok(),
+        "Three-module cycle must not cause an error: {:?}",
+        result.err()
+    );
+}
+
+// ──────────────────────────────────────────────────────────────────────────
+// Test 2: Monorepo node_modules walk-up — package at workspace root bundled
+// ──────────────────────────────────────────────────────────────────────────
+
+/// Simulates an Nx monorepo where React is installed at the workspace root
+/// but the app is in a subdirectory apps/demo/src/.
+///
+/// Layout:
+///   workspace_root/
+///     node_modules/react/     ← React installed here
+///     apps/demo/src/index.js  ← entry point, imports react
+///
+/// The bundler must resolve 'react' by walking up from apps/demo/src/
+/// and find it at workspace_root/node_modules/react.
+/// Acceptance: bundle output > 100 bytes (React code is included).
+#[tokio::test]
+async fn test_bundler_resolves_monorepo_workspace_root_package() {
+    let dir = tempdir().unwrap();
+    let root = dir.path();
+
+    // Create a minimal React-like package at workspace root node_modules.
+    // Use a trimmed fixture (not real React) to keep the test fast.
+    write_file(
+        root,
+        "node_modules/react/package.json",
+        r#"{"name":"react","version":"18.0.0","main":"index.js"}"#,
+    );
+    write_file(
+        root,
+        "node_modules/react/index.js",
+        r#"// React trimmed fixture
+exports.createElement = function(type, props) { return { type: type, props: props }; };
+exports.useState = function(initial) { return [initial, function() {}]; };
+exports.useEffect = function(fn) {};
+exports.Component = function Component() {};
+exports.version = '18.0.0';
+"#,
+    );
+
+    // Entry app in a deeply nested project directory
+    write_file(
+        root,
+        "apps/demo/src/index.js",
+        r#"var React = require('react');
+exports.render = function(el) {
+    return React.createElement(el, {});
+};
+exports.version = React.version;
+"#,
+    );
+
+    let entry = root.join("apps/demo/src/index.js");
+    let options = BundleOptions {
+        entry: entry.clone(),
+        output_dir: root.join("dist"),
+        resolve_options: ResolveOptions {
+            extensions: vec!["js".to_string()],
+            resolve_index: true,
+            externals: HashSet::new(), // React must NOT be external
+            ..Default::default()
+        },
+        ..Default::default()
+    };
+
+    let bundler = Bundler::new(options).unwrap();
+    let result = bundler.bundle(entry).await;
+
+    assert!(
+        result.is_ok(),
+        "Bundler must resolve React from workspace root node_modules: {:?}",
+        result.err()
+    );
+
+    let output = result.unwrap();
+
+    // React code must be inlined (not treated as external)
+    assert!(
+        output.code.contains("createElement"),
+        "React.createElement must be present in the bundle (not treated as external)"
+    );
+    assert!(
+        output.code.contains("useState"),
+        "React.useState must be present in the bundle"
+    );
+    assert!(
+        output.code.contains("18.0.0"),
+        "React version string from fixture must appear in bundle"
+    );
+
+    // Bundle must be substantially larger than just the app code (React is included)
+    // The fixture React is ~200 bytes; combined bundle must exceed 100 bytes.
+    assert!(
+        output.code.len() > 100,
+        "Bundle ({} bytes) must include React code from workspace root (expected > 100 bytes)",
+        output.code.len()
+    );
+}
+
+/// Packages in project-level node_modules take priority over workspace root.
+/// This mirrors Node.js resolution: closest node_modules wins.
+#[tokio::test]
+async fn test_bundler_project_node_modules_takes_priority() {
+    let dir = tempdir().unwrap();
+    let root = dir.path();
+
+    // Workspace root has lodash v4
+    write_file(
+        root,
+        "node_modules/lodash/package.json",
+        r#"{"name":"lodash","version":"4.0.0","main":"lodash.js"}"#,
+    );
+    write_file(
+        root,
+        "node_modules/lodash/lodash.js",
+        "exports.version = '4.0.0'; exports.merge = function() {};",
+    );
+
+    // Project-level node_modules has lodash v3 (closer, must win)
+    write_file(
+        root,
+        "apps/demo/node_modules/lodash/package.json",
+        r#"{"name":"lodash","version":"3.0.0","main":"lodash.js"}"#,
+    );
+    write_file(
+        root,
+        "apps/demo/node_modules/lodash/lodash.js",
+        "exports.version = '3.0.0'; exports.merge = function() {};",
+    );
+
+    write_file(
+        root,
+        "apps/demo/src/index.js",
+        "var _ = require('lodash'); exports.v = _.version;",
+    );
+
+    let entry = root.join("apps/demo/src/index.js");
+    let options = BundleOptions {
+        entry: entry.clone(),
+        output_dir: root.join("dist"),
+        resolve_options: ResolveOptions {
+            extensions: vec!["js".to_string()],
+            resolve_index: true,
+            ..Default::default()
+        },
+        ..Default::default()
+    };
+
+    let bundler = Bundler::new(options).unwrap();
+    let result = bundler.bundle(entry).await.unwrap();
+
+    // Project-level lodash v3 must be picked (it's closer to the importing file)
+    assert!(
+        result.code.contains("3.0.0"),
+        "Project-level lodash v3 must take priority over workspace-root v4: {}",
+        result.code
+    );
+    assert!(
+        !result.code.contains("4.0.0"),
+        "Workspace-root lodash v4 must NOT appear when project-level v3 is present"
+    );
+}
```
