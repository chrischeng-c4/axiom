---
id: implementation
type: change_implementation
change_id: enhancement-resolver-conditional-exports-import-require-browse
---

# Implementation

## Summary

Implement Node.js-compatible conditional exports resolution with caller-supplied condition ordering. Add conditions field to ResolveOptions (default: [import, browser, default]), ResolveConfig in jet.config.toml, and wire through CLI dev/build commands. Algorithm iterates caller conditions array doing map.get() lookups for BTreeMap compatibility.

## Diff

```diff
diff --git a/crates/cclab-jet/src/cli.rs b/crates/cclab-jet/src/cli.rs
index 395a7e0b..21ca9293 100644
--- a/crates/cclab-jet/src/cli.rs
+++ b/crates/cclab-jet/src/cli.rs
@@ -483,6 +483,12 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
 
             let entry = find_entry_point(&root_dir)?;
             eprintln!("[jet] Resolved port={}", port);
+            // @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
+            let mut resolve_options = crate::resolver::ResolveOptions::default();
+            if let Some(conds) = jet_config.resolve_conditions() {
+                resolve_options.conditions = conds.to_vec();
+            }
+
             let config = crate::dev_server::ServerConfig {
                 host,
                 port,
@@ -492,10 +498,10 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
                 proxy: jet_config.dev.proxy,
                 aliases: jet_config.alias.clone(),
             };
-
             let bundle_opts = crate::bundler::BundleOptions {
                 entry: config.entry.clone(),
                 output_dir: root_dir.join("dist"),
+                resolve_options,
                 ..Default::default()
             };
             let bundler = crate::bundler::Bundler::new(bundle_opts)
@@ -576,12 +582,20 @@ async fn execute_async(matches: &ArgMatches) -> Result<()> {
 
             let start = std::time::Instant::now();
 
-            // Build using existing bundler
+            // @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
+            let build_config = crate::task_runner::config::JetConfig::load(&root_dir)
+                .unwrap_or_default();
+            let mut resolve_options = crate::resolver::ResolveOptions::default();
+            if let Some(conds) = build_config.resolve_conditions() {
+                resolve_options.conditions = conds.to_vec();
+            }
+
             let bundle_opts = crate::bundler::BundleOptions {
                 entry: entry.clone(),
                 output_dir: output_dir.clone(),
                 minify,
                 source_maps: sourcemap_mode != crate::bundler::types::SourceMapOption::None,
+                resolve_options,
                 ..Default::default()
             };
 
diff --git a/crates/cclab-jet/src/resolver/mod.rs b/crates/cclab-jet/src/resolver/mod.rs
index 149e744a..b4653a4b 100644
--- a/crates/cclab-jet/src/resolver/mod.rs
+++ b/crates/cclab-jet/src/resolver/mod.rs
@@ -31,6 +31,17 @@ pub struct ResolveOptions {
     /// When true, treat ALL bare package specifiers (not starting with ./ or ../)
     /// as external. Used for lib builds where node_modules deps should not be bundled.
     pub externalize_all_packages: bool,
+
+    /// Ordered export conditions applied when resolving `exports` fields in
+    /// package.json.  The resolver iterates export-object keys in their JSON
+    /// insertion order and accepts the first key that is a member of this slice.
+    ///
+    /// Default: `["import", "browser", "default"]` (browser ESM dev mode).
+    /// Override via `jet.config.toml` `[resolve] conditions` for build mode.
+    ///
+    // @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R1
+    // @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
+    pub conditions: Vec<String>,
 }
 
 /// Resolved module information
@@ -196,7 +207,8 @@ impl ModuleResolver {
         let package_json = package_dir.join("package.json");
 
         if package_json.exists() {
-            if let Ok(Some(export_path)) = package::resolve_exports(&package_json, subpath) {
+            let cond_refs: Vec<&str> = self.options.conditions.iter().map(|s| s.as_str()).collect();
+            if let Ok(Some(export_path)) = package::resolve_exports(&package_json, subpath, &cond_refs) {
                 let resolved_path = package_dir
                     .join(export_path.trim_start_matches('.').trim_start_matches('/'));
                 if let Ok(resolved) = self.try_extensions(&resolved_path) {
@@ -293,6 +305,11 @@ impl Default for ResolveOptions {
             alias: Vec::new(),
             externals: HashSet::new(),
             externalize_all_packages: false,
+            conditions: vec![
+                "import".to_string(),
+                "browser".to_string(),
+                "default".to_string(),
+            ],
         }
     }
 }
@@ -503,4 +520,24 @@ mod tests {
             "lodash must resolve from the closer intermediate node_modules"
         );
     }
+
+    // ──────────────────────────────────────────────────────────────────
+    // T8: dev mode default conditions
+    // ──────────────────────────────────────────────────────────────────
+
+    /// T8: When ResolveOptions::default(), conditions is [import, browser, default].
+    // REQ: R4
+    #[test]
+    fn test_dev_mode_default_conditions() {
+        let options = ResolveOptions::default();
+        assert_eq!(
+            options.conditions,
+            vec![
+                "import".to_string(),
+                "browser".to_string(),
+                "default".to_string()
+            ],
+            "Default conditions must be [import, browser, default] for browser ESM dev mode"
+        );
+    }
 }
diff --git a/crates/cclab-jet/src/resolver/package.rs b/crates/cclab-jet/src/resolver/package.rs
index 9f860c97..585738b6 100644
--- a/crates/cclab-jet/src/resolver/package.rs
+++ b/crates/cclab-jet/src/resolver/package.rs
@@ -38,10 +38,18 @@ pub fn get_package_main(path: &Path) -> Result<String> {
     Ok("index.js".to_string())
 }
 
-/// Resolve using package.json "exports" field (modern Node.js)
+/// Resolve using package.json "exports" field (modern Node.js).
+///
+/// `conditions` controls which export conditions are accepted (e.g.
+/// `["import", "browser", "default"]`).  The caller supplies the ordered
+/// list; conditions are evaluated in object-key insertion order, matching
+/// the Node.js PACKAGE_EXPORTS_RESOLVE specification.
+///
+// @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R1
 pub fn resolve_exports(
     package_json_path: &Path,
     subpath: Option<&str>,
+    conditions: &[&str],
 ) -> Result<Option<String>> {
     let package = read_package_json(package_json_path)?;
 
@@ -59,12 +67,12 @@ pub fn resolve_exports(
 
         serde_json::Value::Object(map) => {
             if let Some(value) = map.get(subpath) {
-                return resolve_export_value(value);
+                return resolve_export_value(value, conditions);
             }
 
             for (pattern, value) in map.iter() {
                 if let Some(matched) = match_export_pattern(pattern, subpath) {
-                    if let Some(resolved) = resolve_export_value(value)? {
+                    if let Some(resolved) = resolve_export_value(value, conditions)? {
                         let final_path = resolved.replace('*', &matched);
                         return Ok(Some(final_path));
                     }
@@ -72,7 +80,7 @@ pub fn resolve_exports(
             }
 
             if subpath == "." && map.contains_key(".") {
-                return resolve_export_value(&map["."]);
+                return resolve_export_value(&map["."], conditions);
             }
         }
 
@@ -82,16 +90,35 @@ pub fn resolve_exports(
     Ok(None)
 }
 
-/// Resolve an export value (handles conditional exports)
-fn resolve_export_value(value: &serde_json::Value) -> Result<Option<String>> {
+/// Resolve an export value, applying caller-supplied `conditions`.
+///
+/// Iterates object keys in JSON insertion order (preserved by `serde_json::Map`).
+/// For each key that is a member of `conditions`, if the mapped value is a
+/// string it is returned directly; if it is a nested object the function
+/// recurses with the same `conditions` slice.  This matches the Node.js
+/// PACKAGE_EXPORTS_RESOLVE spec (first-matching-condition-wins, nested
+/// conditions narrow the resolution path).
+///
+// @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R2
+// @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R3
+fn resolve_export_value(
+    value: &serde_json::Value,
+    conditions: &[&str],
+) -> Result<Option<String>> {
     match value {
         serde_json::Value::String(path) => Ok(Some(path.clone())),
 
         serde_json::Value::Object(map) => {
-            for condition in &["import", "default", "require", "node", "browser"] {
-                if let Some(v) = map.get(*condition) {
-                    if let serde_json::Value::String(path) = v {
-                        return Ok(Some(path.clone()));
+            for cond in conditions {
+                if let Some(v) = map.get(*cond) {
+                    match v {
+                        serde_json::Value::String(path) => return Ok(Some(path.clone())),
+                        serde_json::Value::Object(_) => {
+                            if let Some(result) = resolve_export_value(v, conditions)? {
+                                return Ok(Some(result));
+                            }
+                        }
+                        _ => {}
                     }
                 }
             }
@@ -132,6 +159,9 @@ mod tests {
     use std::io::Write;
     use tempfile::NamedTempFile;
 
+    /// Default conditions slice matching dev mode defaults.
+    const DEV_CONDITIONS: &[&str] = &["import", "browser", "default"];
+
     #[test]
     fn test_read_package_json() {
         let mut file = NamedTempFile::new().unwrap();
@@ -160,6 +190,7 @@ mod tests {
         assert_eq!(main, "lib/index.js");
     }
 
+    // REQ: R1
     #[test]
     fn test_resolve_exports_string() {
         let mut file = NamedTempFile::new().unwrap();
@@ -169,10 +200,11 @@ mod tests {
         )
         .unwrap();
 
-        let result = resolve_exports(file.path(), Some(".")).unwrap();
+        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
         assert_eq!(result, Some("./dist/index.js".to_string()));
     }
 
+    // REQ: R1
     #[test]
     fn test_resolve_exports_object() {
         let mut file = NamedTempFile::new().unwrap();
@@ -188,13 +220,14 @@ mod tests {
         )
         .unwrap();
 
-        let result = resolve_exports(file.path(), Some(".")).unwrap();
+        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
         assert_eq!(result, Some("./dist/index.js".to_string()));
 
-        let result2 = resolve_exports(file.path(), Some("./package.json")).unwrap();
+        let result2 = resolve_exports(file.path(), Some("./package.json"), DEV_CONDITIONS).unwrap();
         assert_eq!(result2, Some("./package.json".to_string()));
     }
 
+    // REQ: R1
     #[test]
     fn test_resolve_exports_conditional() {
         let mut file = NamedTempFile::new().unwrap();
@@ -213,10 +246,12 @@ mod tests {
         )
         .unwrap();
 
-        let result = resolve_exports(file.path(), Some(".")).unwrap();
+        // With default dev conditions (import first) → selects ESM entry
+        let result = resolve_exports(file.path(), Some("."), DEV_CONDITIONS).unwrap();
         assert_eq!(result, Some("./dist/esm/index.js".to_string()));
     }
 
+    // REQ: R1
     #[test]
     fn test_resolve_exports_pattern() {
         let mut file = NamedTempFile::new().unwrap();
@@ -231,7 +266,7 @@ mod tests {
         )
         .unwrap();
 
-        let result = resolve_exports(file.path(), Some("./features/auth")).unwrap();
+        let result = resolve_exports(file.path(), Some("./features/auth"), DEV_CONDITIONS).unwrap();
         assert_eq!(result, Some("./dist/features/auth.js".to_string()));
     }
 
@@ -247,4 +282,227 @@ mod tests {
         );
         assert_eq!(match_export_pattern("./foo/*", "./bar/baz"), None);
     }
+
+    // ─── T1: resolve_import_condition ────────────────────────────────────────────
+
+    /// T1: S1 exports + conditions=[import, default] → ./esm.mjs
+    // REQ: R1
+    #[test]
+    fn test_resolve_import_condition() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "import": "./esm.mjs",
+                        "require": "./cjs.js",
+                        "default": "./index.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        let result = resolve_exports(file.path(), Some("."), &["import", "default"]).unwrap();
+        assert_eq!(result, Some("./esm.mjs".to_string()));
+    }
+
+    // ─── T2: resolve_require_condition ───────────────────────────────────────────
+
+    /// T2: S1 exports + conditions=[require, default] → ./cjs.js
+    // REQ: R1
+    #[test]
+    fn test_resolve_require_condition() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "import": "./esm.mjs",
+                        "require": "./cjs.js",
+                        "default": "./index.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        let result = resolve_exports(file.path(), Some("."), &["require", "default"]).unwrap();
+        assert_eq!(result, Some("./cjs.js".to_string()));
+    }
+
+    // ─── T3: resolve_browser_condition ───────────────────────────────────────────
+
+    /// T3: browser-specific exports + conditions=[browser, default] → browser entry
+    // REQ: R1
+    #[test]
+    fn test_resolve_browser_condition() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "browser": "./browser.js",
+                        "node": "./node.js",
+                        "default": "./index.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        let result = resolve_exports(file.path(), Some("."), &["browser", "default"]).unwrap();
+        assert_eq!(result, Some("./browser.js".to_string()));
+    }
+
+    // ─── T4: nested_node_import ──────────────────────────────────────────────────
+
+    /// T4: S4 nested exports + conditions=[node, import, default] → recurse into
+    /// node object, return ./node.mjs
+    // REQ: R2
+    #[test]
+    fn test_nested_node_import() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "node": {{
+                            "import": "./node.mjs",
+                            "require": "./node.cjs"
+                        }},
+                        "browser": "./browser.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        let result = resolve_exports(file.path(), Some("."), &["node", "import", "default"]).unwrap();
+        assert_eq!(result, Some("./node.mjs".to_string()));
+    }
+
+    // ─── T5: nested_skip_unmatched_block ─────────────────────────────────────────
+
+    /// T5: S6 exports with node block, conditions=[import, default] →
+    /// skip node block, return ./fallback.js via default
+    // REQ: R2
+    #[test]
+    fn test_nested_skip_unmatched_block() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "node": {{
+                            "import": "./node.mjs",
+                            "require": "./node.cjs"
+                        }},
+                        "default": "./fallback.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        // conditions=[import, default] — "node" is not in the conditions list so
+        // the node block is skipped; "default" matches at the top level.
+        let result = resolve_exports(file.path(), Some("."), &["import", "default"]).unwrap();
+        assert_eq!(result, Some("./fallback.js".to_string()));
+    }
+
+    // ─── T6: no_matching_condition_error ─────────────────────────────────────────
+
+    /// T6: S3 exports — import+require only, conditions=[browser, default] →
+    /// no match, returns Ok(None)
+    // REQ: R5
+    #[test]
+    fn test_no_matching_condition_returns_none() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "import": "./esm.mjs",
+                        "require": "./cjs.js"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        // Neither "browser" nor "default" are present in the exports object.
+        let result = resolve_exports(file.path(), Some("."), &["browser", "default"]).unwrap();
+        assert_eq!(result, None, "Should return None when no condition matches");
+    }
+
+    // ─── T7: string_shorthand_ignores_conditions ─────────────────────────────────
+
+    /// T7: string exports shorthand — any conditions → return string directly
+    // REQ: R1
+    #[test]
+    fn test_string_shorthand_ignores_conditions() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{"name": "test", "exports": "./dist/index.js"}}"#
+        )
+        .unwrap();
+
+        // Even unusual conditions — string shorthand always resolves
+        for conds in [
+            &["require"] as &[&str],
+            &["node"],
+            &["browser", "default"],
+            &[],
+        ] {
+            let result = resolve_exports(file.path(), Some("."), conds).unwrap();
+            assert_eq!(
+                result,
+                Some("./dist/index.js".to_string()),
+                "String shorthand must resolve regardless of conditions: {:?}",
+                conds
+            );
+        }
+    }
+
+    // ─── T10: object_key_order_tiebreak ──────────────────────────────────────────
+
+    /// T10: caller-supplied conditions order drives precedence (R3).
+    // REQ: R3
+    #[test]
+    fn test_conditions_order_drives_precedence() {
+        let mut file = NamedTempFile::new().unwrap();
+        writeln!(
+            file,
+            r#"{{
+                "name": "test",
+                "exports": {{
+                    ".": {{
+                        "require": "./cjs.js",
+                        "import": "./esm.mjs"
+                    }}
+                }}
+            }}"#
+        )
+        .unwrap();
+
+        let result_import_first = resolve_exports(file.path(), Some("."), &["import", "require"]).unwrap();
+        assert_eq!(result_import_first, Some("./esm.mjs".to_string()));
+
+        let result_require_first = resolve_exports(file.path(), Some("."), &["require", "import"]).unwrap();
+        assert_eq!(result_require_first, Some("./cjs.js".to_string()));
+    }
 }
diff --git a/crates/cclab-jet/src/task_runner/config.rs b/crates/cclab-jet/src/task_runner/config.rs
index 0c201aa4..35860f20 100644
--- a/crates/cclab-jet/src/task_runner/config.rs
+++ b/crates/cclab-jet/src/task_runner/config.rs
@@ -5,6 +5,7 @@
 //! - `dev`        — dev server settings (port, proxy map)
 //! - `alias`      — module path aliases (overrides tsconfig.json paths)
 //! - `build`      — production build settings (out_dir)
+//! - `resolve`    — module resolution settings (conditions)
 
 use anyhow::{Context, Result};
 use serde::Deserialize;
@@ -32,6 +33,32 @@ pub struct JetConfig {
     /// Production build settings.
     #[serde(default)]
     pub build: JetBuildConfig,
+
+    /// Module resolution settings.
+    ///
+    /// Controls which `exports` conditions are tried when resolving package
+    /// entries.  Absent → resolver uses default `["import", "browser", "default"]`.
+    // @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
+    #[serde(default)]
+    pub resolve: ResolveConfig,
+}
+
+/// Module resolution configuration block.
+///
+/// Deserialized from the `[resolve]` section of `jet.config.toml`.
+///
+/// Example:
+/// ```toml
+/// [resolve]
+/// conditions = ["import", "node", "default"]
+/// ```
+// @spec .score/changes/enhancement-resolver-conditional-exports-import-require-browse/specs/enhancement-resolver-conditional-exports-import-require-browse-spec.md#R4
+#[derive(Debug, Clone, Deserialize, Default)]
+pub struct ResolveConfig {
+    /// Ordered export conditions for package.json `exports` resolution.
+    ///
+    /// When `None`, the resolver defaults to `["import", "browser", "default"]`.
+    pub conditions: Option<Vec<String>>,
 }
 
 /// Dev server configuration block.
@@ -105,6 +132,10 @@ fn default_true() -> bool {
 }
 
 impl JetConfig {
+    pub fn resolve_conditions(&self) -> Option<&[String]> {
+        self.resolve.conditions.as_deref()
+    }
+
     /// Load configuration from jet.config.toml in the project root.
     pub fn load(project_root: &Path) -> Result<Self> {
         let config_path = project_root.join("jet.config.toml");
@@ -258,4 +289,44 @@ cache = false
         assert!(simple.env.is_empty());
         assert!(simple.command.is_none());
     }
+
+    // ─── T9: config_override_conditions ──────────────────────────────────────────
+
+    /// T9: When jet.config sets [resolve] conditions, those are used instead of default.
+    // REQ: R4
+    #[test]
+    fn test_config_override_conditions() {
+        let toml_str = r#"
+[resolve]
+conditions = ["import", "node", "default"]
+"#;
+        let config: JetConfig = toml::from_str(toml_str).unwrap();
+        assert_eq!(
+            config.resolve.conditions,
+            Some(vec![
+                "import".to_string(),
+                "node".to_string(),
+                "default".to_string(),
+            ]),
+            "resolve.conditions from config must be deserialized correctly"
+        );
+
+        let conds = config.resolve_conditions().unwrap();
+        assert_eq!(conds.len(), 3);
+        assert_eq!(conds[0], "import");
+        assert_eq!(conds[1], "node");
+        assert_eq!(conds[2], "default");
+    }
+
+    /// When [resolve] section is absent, resolve_conditions() returns None
+    /// (caller falls back to resolver default).
+    // REQ: R4
+    #[test]
+    fn test_config_default_conditions_absent() {
+        let config: JetConfig = JetConfig::default();
+        assert!(
+            config.resolve_conditions().is_none(),
+            "No [resolve] section → resolve_conditions() returns None"
+        );
+    }
 }

```

## Review: enhancement-resolver-conditional-exports-import-require-browse-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: enhancement-resolver-conditional-exports-import-require-browse

**Summary**: All hard checklist items pass. R1-R5 fully implemented with correct algorithm (iterates caller conditions, not map keys). 12+ tests covering T1-T10. 70 tests pass, 0 failures.

