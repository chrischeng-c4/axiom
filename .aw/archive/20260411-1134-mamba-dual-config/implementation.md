---
id: implementation
type: change_implementation
change_id: 1134-mamba-dual-config
---

# Implementation

## Summary

Add @spec traceability annotations and regression test for #1134 MambaConfig unification.

The actual unification work (merging dual MambaConfig structs) was already implemented:
- config/schema.rs: canonical MambaConfig with CrateEntry enum supporting both rich format ([project] + [crates.<key>] tables) and flat format (entry_point at root, crates as version strings)
- driver/config.rs: only contains CompilerConfig, imports MambaConfig from crate::config::MambaConfig

This change adds:
1. @spec annotations linking public methods to requirements R1-R5
2. Regression test issue_1134_conductor_toml_format_no_parse_error that specifically verifies Conductor's mamba.toml format (with [project] and [crates.cclab-schema-mamba] sub-tables) now parses correctly — this was the original bug that triggered #1134.

All 77 config tests pass.

## Diff

```diff
diff --git a/crates/mamba/src/config/schema.rs b/crates/mamba/src/config/schema.rs
index db5eda10..cc53cf45 100644
--- a/crates/mamba/src/config/schema.rs
+++ b/crates/mamba/src/config/schema.rs
@@ -195,6 +195,8 @@ impl MambaConfig {
     ///
     /// Returns `(config, config_path)` for the first file found, or `None` if
     /// the filesystem root is reached without finding one.
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R1
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn discover(start_dir: &Path) -> Option<(Self, PathBuf)> {
         let mut dir = start_dir;
         loop {
@@ -211,12 +213,16 @@ impl MambaConfig {
     }
 
     /// Parse a mamba.toml file (#251).
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn from_file(path: &Path) -> crate::error::Result<Self> {
         let content = std::fs::read_to_string(path)?;
         Self::from_str(&content)
     }
 
     /// Parse from a TOML string.
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn from_str(content: &str) -> crate::error::Result<Self> {
         let config: Self = toml::from_str(content)
             .map_err(|e| MambaError::Other(format!("config parse error: {e}")))?;
@@ -228,6 +234,7 @@ impl MambaConfig {
     ///
     /// Checks the top-level `entry_point` field first (flat format),
     /// then falls back to `project.entry_point` (richer format).
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn entry_point(&self) -> Option<&str> {
         self.entry_point
             .as_deref()
@@ -240,6 +247,8 @@ impl MambaConfig {
     /// per-crate `expose` lists (richer format).
     ///
     /// An absent or empty expose list means "expose everything".
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn is_symbol_exposed(&self, crate_name: &str, symbol: &str) -> bool {
         // Check top-level expose map first (flat format).
         if let Some(list) = self.expose.get(crate_name) {
@@ -267,6 +276,8 @@ impl MambaConfig {
     /// Returns `true` if the key has a non-empty expose list in either
     /// the top-level `[expose]` table or a `CrateEntry::Config` with
     /// a non-empty expose list.
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R2
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R3
     pub fn has_crate_expose(&self, key: &str) -> bool {
         // Top-level expose map (flat format).
         if self.expose.contains_key(key) {
@@ -291,6 +302,7 @@ impl MambaConfig {
     ///
     /// Validation is lenient for the flat format: `[project]` is optional,
     /// and plain-string crate entries skip the expose/version-or-path checks.
+    // @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R5
     pub fn validate(&self) -> crate::error::Result<()> {
         // Validate project metadata only when a [project] table was provided.
         if self.has_project() {
@@ -620,4 +632,37 @@ entry_point = "project.py"
         let config = MambaConfig::from_str(toml).unwrap();
         assert_eq!(config.entry_point(), Some("project.py"));
     }
+
+    // ── #1134 regression: Conductor mamba.toml format ────────────────────────
+
+    /// Regression test for #1134: Conductor's mamba.toml with [project] and
+    /// [crates.cclab-schema-mamba] sub-tables caused a TOML parse error when
+    /// the CLI used driver/config.rs which expected flat crates = version_string.
+    /// The unified MambaConfig in config/schema.rs now handles both formats.
+    #[test]
+    fn issue_1134_conductor_toml_format_no_parse_error() {
+        // This is the format that Conductor's mamba.toml uses.
+        // Before #1134, this caused: "invalid type: map, expected a string"
+        let toml = r#"
+[project]
+name = "conductor"
+version = "0.1.0"
+entry_point = "src/main.py"
+
+[crates.cclab-schema-mamba]
+version = "0.1.0"
+expose = ["BaseModel", "Field"]
+"#;
+        let config = MambaConfig::from_str(toml).expect("Conductor mamba.toml format should parse without error (#1134)");
+        assert_eq!(config.project.name, "conductor");
+        assert_eq!(config.entry_point(), Some("src/main.py"));
+        assert!(config.crates.contains_key("cclab-schema-mamba"));
+        match &config.crates["cclab-schema-mamba"] {
+            CrateEntry::Config(c) => {
+                assert!(c.expose.contains(&"BaseModel".to_string()));
+                assert!(c.expose.contains(&"Field".to_string()));
+            }
+            CrateEntry::Version(_) => panic!("Expected CrateEntry::Config for structured crate entry"),
+        }
+    }
 }
diff --git a/crates/mamba/src/driver/config.rs b/crates/mamba/src/driver/config.rs
index 0a3947e4..18b6bfab 100644
--- a/crates/mamba/src/driver/config.rs
+++ b/crates/mamba/src/driver/config.rs
@@ -1,5 +1,6 @@
 use crate::config::MambaConfig;
 
+// @spec .score/changes/1134-mamba-dual-config/groups/unify-mamba-config/specs/1134-mamba-dual-config-spec.md#R4
 /// Compiler configuration.
 #[derive(Debug, Clone)]
 pub struct CompilerConfig {

```

## Review: 1134-mamba-dual-config-spec

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: 1134-mamba-dual-config

**Summary**: Implementation satisfies all spec requirements. The dual MambaConfig unification is complete: a single canonical MambaConfig in config/schema.rs supports both rich and flat format; driver/config.rs only contains CompilerConfig importing MambaConfig from crate::config. All 5 requirements (R1-R5) are implemented and annotated with @spec. A regression test for #1134's original Conductor mamba.toml parse error was added and passes. All 77 config/driver tests pass.



## Alignment Warnings

13 violation(s) found across 1 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Requirements' at line 23 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Scenarios' at line 32 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Diagrams' at line 63 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'API Spec' at line 85 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Test Plan' at line 111 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Changes' at line 137 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Data Model' at line 175 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Interaction' at line 219 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | missing_section_annotation | Section 'Logic' at line 224 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/mamba/config/config-schema.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
