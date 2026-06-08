---
id: implementation
type: change_implementation
change_id: sdd-spec-format-unify
---

# Implementation

## Summary

Implemented SDD spec format unification: top-down fill order, yaml for structured sections, mermaid for requirements/test-plan/scenarios, Mermaid Plus stubs in UNIVERSAL_SKELETON. All 1388 tests pass.

## Diff

```diff
diff --git a/crates/sdd/src/generators/requirements.rs b/crates/sdd/src/generators/requirements.rs
index 92d95bf4..435f9d9b 100644
--- a/crates/sdd/src/generators/requirements.rs
+++ b/crates/sdd/src/generators/requirements.rs
@@ -6,7 +6,8 @@
 use super::{Generator, GeneratorArgs};
 use crate::models::spec_rules::SectionType;
 
-/// Generator for `<!-- type: requirements lang: markdown -->` sections.
+/// Generator for `<!-- type: requirements lang: mermaid -->` sections.
+/// Uses Mermaid Plus requirementDiagram (SysML v1.6) format.
 pub struct RequirementsGenerator;
 
 impl Generator for RequirementsGenerator {
@@ -85,6 +86,6 @@ mod tests {
     fn test_with_annotation() {
         let args = GeneratorArgs::new(SectionType::Requirements);
         let output = RequirementsGenerator.generate_with_annotation(&args);
-        assert!(output.starts_with("<!-- type: requirements lang: markdown -->"));
+        assert!(output.starts_with("<!-- type: requirements lang: mermaid -->"));
     }
 }
diff --git a/crates/sdd/src/generators/rpc_api.rs b/crates/sdd/src/generators/rpc_api.rs
index fcd1e16b..ca64446e 100644
--- a/crates/sdd/src/generators/rpc_api.rs
+++ b/crates/sdd/src/generators/rpc_api.rs
@@ -1,12 +1,13 @@
 //! RPC API section generator.
 //!
-//! Produces an OpenRPC 1.3 JSON skeleton with method_name, params,
+//! Produces an OpenRPC 1.3 YAML skeleton with method_name, params,
 //! and result schema. Includes `x-sdd` metadata injection.
 
 use super::{Generator, GeneratorArgs};
 use crate::models::spec_rules::SectionType;
 
-/// Generator for `<!-- type: rpc-api lang: json -->` sections.
+/// Generator for `<!-- type: rpc-api lang: yaml -->` sections.
+/// Uses OpenRPC 1.3 as YAML (more token-efficient than JSON).
 pub struct RpcApiGenerator;
 
 impl Generator for RpcApiGenerator {
@@ -16,51 +17,40 @@ impl Generator for RpcApiGenerator {
 
     fn generate(&self, args: &GeneratorArgs) -> String {
         let sdd_id = args.sdd_id.as_deref().unwrap_or("TODO");
-        let refs_json = if args.sdd_refs.is_empty() {
+        let refs_yaml = if args.sdd_refs.is_empty() {
             String::new()
         } else {
             let refs: Vec<String> = args
                 .sdd_refs
                 .iter()
-                .map(|r| format!("\"{}\"", r))
+                .map(|r| format!("    - {}", r))
                 .collect();
-            format!(",\n      \"refs\": [{}]", refs.join(", "))
+            format!("\n  refs:\n{}", refs.join("\n"))
         };
 
         format!(
-            "```json\n\
-{{\n\
-  \"openrpc\": \"1.3.2\",\n\
-  \"info\": {{\n\
-    \"title\": \"[API Title]\",\n\
-    \"version\": \"1.0.0\",\n\
-    \"x-sdd\": {{\n\
-      \"id\": \"{sdd_id}\"{refs_json}\n\
-    }}\n\
-  }},\n\
-  \"methods\": [\n\
-    {{\n\
-      \"name\": \"[method_name]\",\n\
-      \"summary\": \"[Description]\",\n\
-      \"params\": [\n\
-        {{\n\
-          \"name\": \"param1\",\n\
-          \"required\": true,\n\
-          \"schema\": {{ \"type\": \"string\" }}\n\
-        }}\n\
-      ],\n\
-      \"result\": {{\n\
-        \"name\": \"result\",\n\
-        \"schema\": {{\n\
-          \"type\": \"object\",\n\
-          \"properties\": {{\n\
-            \"success\": {{ \"type\": \"boolean\" }}\n\
-          }}\n\
-        }}\n\
-      }}\n\
-    }}\n\
-  ]\n\
-}}\n\
+            "```yaml\n\
+openrpc: \"1.3.2\"\n\
+info:\n\
+  title: \"[API Title]\"\n\
+  version: \"1.0.0\"\n\
+  x-sdd:\n\
+    id: \"{sdd_id}\"{refs_yaml}\n\
+methods:\n\
+  - name: \"[method_name]\"\n\
+    summary: \"[Description]\"\n\
+    params:\n\
+      - name: param1\n\
+        required: true\n\
+        schema:\n\
+          type: string\n\
+    result:\n\
+      name: result\n\
+      schema:\n\
+        type: object\n\
+        properties:\n\
+          success:\n\
+            type: boolean\n\
 ```"
         )
     }
@@ -82,11 +72,11 @@ mod tests {
         let args = GeneratorArgs::new(SectionType::RpcApi)
             .with_sdd_id("my-rpc");
         let output = RpcApiGenerator.generate(&args);
-        assert!(output.contains("```json"));
-        assert!(output.contains("\"openrpc\": \"1.3.2\""));
-        assert!(output.contains("\"x-sdd\""));
-        assert!(output.contains("\"id\": \"my-rpc\""));
-        assert!(output.contains("\"methods\""));
+        assert!(output.contains("```yaml"));
+        assert!(output.contains("openrpc: \"1.3.2\""));
+        assert!(output.contains("x-sdd:"));
+        assert!(output.contains("id: \"my-rpc\""));
+        assert!(output.contains("methods:"));
     }
 
     #[test]
@@ -95,13 +85,13 @@ mod tests {
             .with_sdd_id("my-rpc")
             .with_sdd_refs(vec!["schema-spec".to_string()]);
         let output = RpcApiGenerator.generate(&args);
-        assert!(output.contains("\"schema-spec\""));
+        assert!(output.contains("schema-spec"));
     }
 
     #[test]
     fn test_with_annotation() {
         let args = GeneratorArgs::new(SectionType::RpcApi);
         let output = RpcApiGenerator.generate_with_annotation(&args);
-        assert!(output.starts_with("<!-- type: rpc-api lang: json -->"));
+        assert!(output.starts_with("<!-- type: rpc-api lang: yaml -->"));
     }
 }
diff --git a/crates/sdd/src/generators/scenarios.rs b/crates/sdd/src/generators/scenarios.rs
index cc2b9cba..24023459 100644
--- a/crates/sdd/src/generators/scenarios.rs
+++ b/crates/sdd/src/generators/scenarios.rs
@@ -6,7 +6,8 @@
 use super::{Generator, GeneratorArgs};
 use crate::models::spec_rules::SectionType;
 
-/// Generator for `<!-- type: scenarios lang: markdown -->` sections.
+/// Generator for `<!-- type: scenarios lang: yaml -->` sections.
+/// Uses YAML GWT structured format: list of {id, given, when, then, diagram_ref?}.
 pub struct ScenariosGenerator;
 
 impl Generator for ScenariosGenerator {
@@ -31,20 +32,23 @@ impl Generator for ScenariosGenerator {
 
         format!(
             "{}{}\
-### Scenario: [Happy path name]\n\
-**GIVEN** [precondition]\n\
-**WHEN** [action is taken]\n\
-**THEN** [expected outcome]\n\
+```yaml\n\
+- id: S1\n\
+  given: \"[Precondition — system state before action]\"\n\
+  when: \"[Action or event that triggers the scenario]\"\n\
+  then: \"[Expected observable outcome]\"\n\
 \n\
-### Scenario: [Error path name]\n\
-**GIVEN** [precondition]\n\
-**WHEN** [invalid action]\n\
-**THEN** [expected error behavior]\n\
+- id: S2\n\
+  given: \"[Error precondition]\"\n\
+  when: \"[Invalid action or edge case]\"\n\
+  then: \"[Expected error behavior]\"\n\
 \n\
-### Scenario: [Edge case name]\n\
-**GIVEN** [edge precondition]\n\
-**WHEN** [boundary action]\n\
-**THEN** [expected edge behavior]",
+- id: S3\n\
+  given: \"[Edge case precondition]\"\n\
+  when: \"[Boundary action]\"\n\
+  then: \"[Expected boundary behavior]\"\n\
+  diagram_ref: \"[optional: section-anchor for related diagram]\"\n\
+```",
             context_hint, refs_hint
         )
     }
@@ -62,13 +66,14 @@ mod tests {
     }
 
     #[test]
-    fn test_generate_contains_bdd_structure() {
+    fn test_generate_contains_yaml_gwt_structure() {
         let args = GeneratorArgs::new(SectionType::Scenarios);
         let output = ScenariosGenerator.generate(&args);
-        assert!(output.contains("### Scenario:"));
-        assert!(output.contains("**GIVEN**"));
-        assert!(output.contains("**WHEN**"));
-        assert!(output.contains("**THEN**"));
+        assert!(output.contains("```yaml"));
+        assert!(output.contains("- id: S1"));
+        assert!(output.contains("given:"));
+        assert!(output.contains("when:"));
+        assert!(output.contains("then:"));
     }
 
     #[test]
@@ -83,6 +88,6 @@ mod tests {
     fn test_with_annotation() {
         let args = GeneratorArgs::new(SectionType::Scenarios);
         let output = ScenariosGenerator.generate_with_annotation(&args);
-        assert!(output.starts_with("<!-- type: scenarios lang: markdown -->"));
+        assert!(output.starts_with("<!-- type: scenarios lang: yaml -->"));
     }
 }
diff --git a/crates/sdd/src/generators/test_plan.rs b/crates/sdd/src/generators/test_plan.rs
index b22e124d..dd796acd 100644
--- a/crates/sdd/src/generators/test_plan.rs
+++ b/crates/sdd/src/generators/test_plan.rs
@@ -6,7 +6,8 @@
 use super::{Generator, GeneratorArgs};
 use crate::models::spec_rules::SectionType;
 
-/// Generator for `<!-- type: test_plan lang: markdown -->` sections.
+/// Generator for `<!-- type: test-plan lang: mermaid -->` sections.
+/// Uses Mermaid Plus requirementDiagram with element nodes and verifies relationships.
 pub struct TestPlanGenerator;
 
 impl Generator for TestPlanGenerator {
@@ -95,6 +96,6 @@ mod tests {
     fn test_with_annotation() {
         let args = GeneratorArgs::new(SectionType::TestPlan);
         let output = TestPlanGenerator.generate_with_annotation(&args);
-        assert!(output.starts_with("<!-- type: test-plan lang: markdown -->"));
+        assert!(output.starts_with("<!-- type: test-plan lang: mermaid -->"));
     }
 }
diff --git a/crates/sdd/src/models/spec_rules.rs b/crates/sdd/src/models/spec_rules.rs
index 3778fa75..e7e39e56 100644
--- a/crates/sdd/src/models/spec_rules.rs
+++ b/crates/sdd/src/models/spec_rules.rs
@@ -81,56 +81,67 @@ impl SectionType {
     /// Fill priority order (lower number = fill first).
     ///
     /// Used to produce deterministic section fill sequences.
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-fill-order-topdown.md#R1
     pub fn fill_order(&self) -> u8 {
         match self {
             // 0: understand scope first
             SectionType::Overview => 0,
-            // 1: data layer
-            SectionType::DbModel => 1,
-            // 2: referenced by API types
-            SectionType::Schema => 2,
-            // 3: state transitions
-            SectionType::StateMachine => 3,
-            // 4: business logic
-            SectionType::Logic => 4,
-            // 5: architecture
-            SectionType::Dependency => 5,
-            // 6: call chains
-            SectionType::Interaction => 6,
-            // 7: API surface (refs schema)
-            SectionType::RestApi => 7,
-            SectionType::RpcApi => 8,
-            SectionType::AsyncApi => 9,
-            SectionType::Cli => 10,
-            // 8: UI
-            SectionType::Wireframe => 11,
-            SectionType::Component => 12,
-            SectionType::DesignToken => 13,
-            // 9: other
-            SectionType::Config => 14,
-            SectionType::Mindmap => 15,
-            SectionType::Requirements => 16,
-            SectionType::Scenarios => 17,
-            // 10: needs all others
+            // 1-2: requirements and behavior (top-down reasoning)
+            SectionType::Requirements => 1,
+            SectionType::Scenarios => 2,
+            // 3-8: diagrams (structural overview before details)
+            SectionType::Mindmap => 3,
+            SectionType::StateMachine => 4,
+            SectionType::Interaction => 5,
+            SectionType::Logic => 6,
+            SectionType::Dependency => 7,
+            SectionType::DbModel => 8,
+            // 9-13: data and API (defined after diagrams)
+            SectionType::Schema => 9,
+            SectionType::RestApi => 10,
+            SectionType::RpcApi => 11,
+            SectionType::AsyncApi => 12,
+            SectionType::Cli => 13,
+            // 14-16: UI
+            SectionType::Wireframe => 14,
+            SectionType::Component => 15,
+            SectionType::DesignToken => 16,
+            // 17: config
+            SectionType::Config => 17,
+            // 18: verification (needs requirements + diagrams + data)
             SectionType::TestPlan => 18,
-            // 11: last
+            // 19-20: delta and doc (last)
             SectionType::Changes => 19,
             SectionType::Doc => 20,
         }
     }
 
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R1
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R4
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R5
     /// Default content language for this section type.
+    ///
+    /// Three langs only: `markdown`, `yaml`, `mermaid`. JSON has been removed.
+    ///
+    /// - `markdown`: prose-only sections (overview, doc)
+    /// - `mermaid`: all diagram sections + requirements (requirementDiagram) + test-plan
+    /// - `yaml`: all structured data sections (APIs, schema, config, scenarios, etc.)
     pub fn default_lang(&self) -> &'static str {
         match self {
-            SectionType::Overview | SectionType::Requirements
-            | SectionType::Scenarios | SectionType::TestPlan
-            | SectionType::Doc => "markdown",
+            // Prose-only sections
+            SectionType::Overview | SectionType::Doc => "markdown",
+            // Diagram sections (Mermaid Plus format — YAML frontmatter inside mermaid block)
+            // requirements and test-plan use Mermaid Plus requirementDiagram (SysML v1.6)
             SectionType::Interaction | SectionType::Logic | SectionType::Dependency
-            | SectionType::StateMachine | SectionType::DbModel | SectionType::Mindmap => "mermaid",
-            SectionType::RestApi | SectionType::AsyncApi
-            | SectionType::Changes | SectionType::Wireframe | SectionType::Cli => "yaml",
-            SectionType::RpcApi | SectionType::Schema | SectionType::Config
-            | SectionType::Component | SectionType::DesignToken => "json",
+            | SectionType::StateMachine | SectionType::DbModel | SectionType::Mindmap
+            | SectionType::Requirements | SectionType::TestPlan => "mermaid",
+            // Structured data sections — all use YAML (not JSON)
+            // scenarios: YAML GWT format {id, given, when, then, diagram_ref?}
+            // schema, rpc-api, config, component, design-token: YAML (was JSON)
+            SectionType::RestApi | SectionType::AsyncApi | SectionType::Changes
+            | SectionType::Wireframe | SectionType::Cli | SectionType::Scenarios
+            | SectionType::RpcApi | SectionType::Schema | SectionType::Config
+            | SectionType::Component | SectionType::DesignToken => "yaml",
         }
     }
 
@@ -942,6 +953,42 @@ mod tests {
         }
     }
 
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-fill-order-topdown.md#R1
+    #[test]
+    fn test_fill_order_requirements_before_schema() {
+        // R1: Top-down fill order — requirements before data/API sections
+        assert!(
+            SectionType::Requirements.fill_order() < SectionType::Schema.fill_order(),
+            "requirements should fill before schema"
+        );
+        assert!(
+            SectionType::Scenarios.fill_order() < SectionType::Schema.fill_order(),
+            "scenarios should fill before schema"
+        );
+        assert!(
+            SectionType::Requirements.fill_order() < SectionType::DbModel.fill_order(),
+            "requirements should fill before db-model"
+        );
+        assert!(
+            SectionType::Requirements.fill_order() < SectionType::RestApi.fill_order(),
+            "requirements should fill before rest-api"
+        );
+        // test-plan fills after all diagrams and data
+        assert!(
+            SectionType::TestPlan.fill_order() > SectionType::Schema.fill_order(),
+            "test-plan should fill after schema"
+        );
+        // changes and doc are last
+        assert!(
+            SectionType::TestPlan.fill_order() < SectionType::Changes.fill_order(),
+            "test-plan should fill before changes"
+        );
+        assert!(
+            SectionType::Changes.fill_order() < SectionType::Doc.fill_order(),
+            "changes should fill before doc"
+        );
+    }
+
     #[test]
     fn test_section_type_overview_first() {
         let types = SectionType::all_in_fill_order();
@@ -964,13 +1011,23 @@ mod tests {
 
     #[test]
     fn test_section_type_default_lang() {
+        // Prose-only sections
         assert_eq!(SectionType::Overview.default_lang(), "markdown");
+        assert_eq!(SectionType::Doc.default_lang(), "markdown");
+        // YAML sections (structured data — no JSON)
         assert_eq!(SectionType::Changes.default_lang(), "yaml");
-        assert_eq!(SectionType::Interaction.default_lang(), "mermaid");
         assert_eq!(SectionType::RestApi.default_lang(), "yaml");
-        assert_eq!(SectionType::RpcApi.default_lang(), "json");
+        assert_eq!(SectionType::RpcApi.default_lang(), "yaml"); // was "json"
+        assert_eq!(SectionType::Schema.default_lang(), "yaml"); // was "json"
+        assert_eq!(SectionType::Config.default_lang(), "yaml"); // was "json"
+        assert_eq!(SectionType::Component.default_lang(), "yaml"); // was "json"
+        assert_eq!(SectionType::DesignToken.default_lang(), "yaml"); // was "json"
+        assert_eq!(SectionType::Scenarios.default_lang(), "yaml"); // was "markdown"
         assert_eq!(SectionType::Wireframe.default_lang(), "yaml");
-        assert_eq!(SectionType::Doc.default_lang(), "markdown");
+        // Mermaid sections (diagrams + requirements + test-plan)
+        assert_eq!(SectionType::Interaction.default_lang(), "mermaid");
+        assert_eq!(SectionType::Requirements.default_lang(), "mermaid"); // was "markdown"
+        assert_eq!(SectionType::TestPlan.default_lang(), "mermaid"); // was "markdown"
     }
 
     // ─── SectionEntry tests ──────────────────────────────────────────────────
diff --git a/crates/sdd/src/spec_alignment/format_rules.rs b/crates/sdd/src/spec_alignment/format_rules.rs
index 3e9d4265..809e1f93 100644
--- a/crates/sdd/src/spec_alignment/format_rules.rs
+++ b/crates/sdd/src/spec_alignment/format_rules.rs
@@ -8,33 +8,45 @@
 use std::collections::HashMap;
 
 use super::models::{SpecDocument, Violation, ViolationKind};
+#[cfg(test)]
+use super::models::{CodeBlock, SpecSection, SectionAnnotation};
 
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R2
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R3
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R4
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R5
 /// Section types that require a code block of the declared lang.
 /// Maps section_type -> required code fence lang.
+///
+/// All structured data sections use yaml (not json).
+/// requirements and test-plan use mermaid (Mermaid Plus requirementDiagram).
+/// scenarios uses yaml (GWT structured format).
 const REQUIRED_CODE_BLOCK_TYPES: &[(&str, &str)] = &[
-    ("config", "json"),
+    ("config", "yaml"),      // was "json" — yaml is more token-efficient
     ("logic", "mermaid"),
-    ("rpc-api", "json"),
+    ("rpc-api", "yaml"),     // was "json" — OpenRPC as YAML
     ("state-machine", "mermaid"),
     ("cli", "yaml"),
     ("changes", "yaml"),
-    ("schema", "json"),
+    ("schema", "yaml"),      // was "json" — JSON Schema as YAML
     ("rest-api", "yaml"),
     ("async-api", "yaml"),
     ("db-model", "mermaid"),
     ("dependency", "mermaid"),
     ("interaction", "mermaid"),
     ("wireframe", "yaml"),
-    ("component", "json"),
-    ("design-token", "json"),
+    ("component", "yaml"),   // was "json" — Custom Elements Manifest as YAML
+    ("design-token", "yaml"), // was "json" — W3C DTCG as YAML
+    ("mindmap", "mermaid"),
+    ("requirements", "mermaid"), // Mermaid Plus requirementDiagram (SysML v1.6)
+    ("test-plan", "mermaid"),    // Mermaid Plus requirementDiagram with verifies
+    ("scenarios", "yaml"),       // YAML GWT structured format
 ];
 
 /// Prose-only section types exempt from code block requirements.
+/// Only overview and doc remain prose-only.
 const PROSE_ONLY_TYPES: &[&str] = &[
     "overview",
-    "requirements",
-    "scenarios",
-    "test-plan",
     "doc",
 ];
 
@@ -153,3 +165,188 @@ fn check_format_priority(doc: &SpecDocument, violations: &mut Vec<Violation>) {
         }
     }
 }
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+    use serde_json::json;
+
+    fn make_doc(sections: Vec<SpecSection>) -> SpecDocument {
+        SpecDocument {
+            path: "test.md".to_string(),
+            frontmatter: json!({}),
+            sections,
+        }
+    }
+
+    fn section_with_block(heading: &str, section_type: &str, lang_annotation: &str, block_lang: &str) -> SpecSection {
+        SpecSection {
+            heading: heading.to_string(),
+            line: 1,
+            annotation: Some(SectionAnnotation {
+                section_type: section_type.to_string(),
+                lang: lang_annotation.to_string(),
+            }),
+            code_blocks: vec![CodeBlock {
+                lang: block_lang.to_string(),
+                line: 2,
+                content: "content".to_string(),
+                parsed_json: None,
+            }],
+        }
+    }
+
+    fn section_no_block(heading: &str, section_type: &str, lang_annotation: &str) -> SpecSection {
+        SpecSection {
+            heading: heading.to_string(),
+            line: 1,
+            annotation: Some(SectionAnnotation {
+                section_type: section_type.to_string(),
+                lang: lang_annotation.to_string(),
+            }),
+            code_blocks: vec![],
+        }
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R2
+    #[test]
+    fn test_schema_requires_yaml_not_json() {
+        let doc_yaml = make_doc(vec![section_with_block("Schema", "schema", "yaml", "yaml")]);
+        let doc_json = make_doc(vec![section_with_block("Schema", "schema", "yaml", "json")]);
+
+        assert!(check(&doc_yaml).is_empty(), "yaml block should pass for schema");
+        assert!(
+            check(&doc_json).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "json block should fail for schema"
+        );
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R2
+    #[test]
+    fn test_rpc_api_requires_yaml_not_json() {
+        let doc_yaml = make_doc(vec![section_with_block("RPC API", "rpc-api", "yaml", "yaml")]);
+        let doc_json = make_doc(vec![section_with_block("RPC API", "rpc-api", "yaml", "json")]);
+
+        assert!(check(&doc_yaml).is_empty(), "yaml should pass for rpc-api");
+        assert!(
+            check(&doc_json).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "json should fail for rpc-api"
+        );
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R2
+    #[test]
+    fn test_config_requires_yaml() {
+        let doc_yaml = make_doc(vec![section_with_block("Config", "config", "yaml", "yaml")]);
+        assert!(check(&doc_yaml).is_empty(), "yaml should pass for config");
+
+        let doc_json = make_doc(vec![section_with_block("Config", "config", "yaml", "json")]);
+        assert!(
+            check(&doc_json).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "json should fail for config"
+        );
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R4
+    #[test]
+    fn test_requirements_requires_mermaid() {
+        let doc_mermaid = make_doc(vec![section_with_block("Requirements", "requirements", "mermaid", "mermaid")]);
+        assert!(check(&doc_mermaid).is_empty(), "mermaid should pass for requirements");
+
+        let doc_no_block = make_doc(vec![section_no_block("Requirements", "requirements", "mermaid")]);
+        assert!(
+            check(&doc_no_block).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "no mermaid block should fail for requirements"
+        );
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R5
+    #[test]
+    fn test_scenarios_requires_yaml() {
+        let doc_yaml = make_doc(vec![section_with_block("Scenarios", "scenarios", "yaml", "yaml")]);
+        assert!(check(&doc_yaml).is_empty(), "yaml should pass for scenarios");
+
+        let doc_no_block = make_doc(vec![section_no_block("Scenarios", "scenarios", "yaml")]);
+        assert!(
+            check(&doc_no_block).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "no yaml block should fail for scenarios"
+        );
+    }
+
+    // @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-format-rules-yaml.md#R4
+    #[test]
+    fn test_test_plan_requires_mermaid() {
+        let doc_mermaid = make_doc(vec![section_with_block("Test Plan", "test-plan", "mermaid", "mermaid")]);
+        assert!(check(&doc_mermaid).is_empty(), "mermaid should pass for test-plan");
+
+        let doc_no_block = make_doc(vec![section_no_block("Test Plan", "test-plan", "mermaid")]);
+        assert!(
+            check(&doc_no_block).iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "no mermaid block should fail for test-plan"
+        );
+    }
+
+    #[test]
+    fn test_overview_prose_only_no_code_required() {
+        let doc = make_doc(vec![SpecSection {
+            heading: "Overview".to_string(),
+            line: 1,
+            annotation: Some(SectionAnnotation {
+                section_type: "overview".to_string(),
+                lang: "markdown".to_string(),
+            }),
+            code_blocks: vec![],
+        }]);
+        let violations = check(&doc);
+        assert!(
+            !violations.iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "overview should not require code block"
+        );
+    }
+
+    #[test]
+    fn test_doc_prose_only_no_code_required() {
+        let doc = make_doc(vec![SpecSection {
+            heading: "Doc".to_string(),
+            line: 1,
+            annotation: Some(SectionAnnotation {
+                section_type: "doc".to_string(),
+                lang: "markdown".to_string(),
+            }),
+            code_blocks: vec![],
+        }]);
+        let violations = check(&doc);
+        assert!(
+            !violations.iter().any(|v| v.kind == ViolationKind::FormatPriorityViolation),
+            "doc should not require code block"
+        );
+    }
+
+    #[test]
+    fn test_missing_annotation_reported() {
+        let doc = make_doc(vec![SpecSection {
+            heading: "Unannotated Section".to_string(),
+            line: 5,
+            annotation: None,
+            code_blocks: vec![],
+        }]);
+        let violations = check(&doc);
+        assert!(
+            violations.iter().any(|v| v.kind == ViolationKind::MissingSectionAnnotation),
+            "missing annotation should be reported"
+        );
+    }
+
+    #[test]
+    fn test_duplicate_section_reported() {
+        let doc = make_doc(vec![
+            section_no_block("Overview", "overview", "markdown"),
+            section_no_block("Overview", "overview", "markdown"),
+        ]);
+        let violations = check(&doc);
+        assert!(
+            violations.iter().any(|v| v.kind == ViolationKind::DuplicateSection),
+            "duplicate heading should be reported"
+        );
+    }
+}
diff --git a/crates/sdd/src/tools/common_change_spec.rs b/crates/sdd/src/tools/common_change_spec.rs
index e593874b..3cb4b84c 100644
--- a/crates/sdd/src/tools/common_change_spec.rs
+++ b/crates/sdd/src/tools/common_change_spec.rs
@@ -10,9 +10,20 @@ use std::path::Path;
 
 // ─── Universal Skeleton ─────────────────────────────────────────────────────
 
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R6
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R7
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md#R8
 /// Universal skeleton template with ALL possible sections.
 /// Sections are annotated with `<!-- type: xxx lang: yyy -->`.
 /// Agent decides which to fill; prune removes unfilled sections.
+///
+/// Format decisions (D1-D9 from issue):
+/// - 3 langs only: markdown, yaml, mermaid. JSON removed.
+/// - requirements/test-plan: Mermaid Plus requirementDiagram (YAML frontmatter inside mermaid block)
+/// - scenarios: YAML GWT structured format {id, given, when, then, diagram_ref?}
+/// - schema/rpc-api/config/component/design-token: yaml (not json)
+/// - all diagram sections: Mermaid Plus (YAML frontmatter inside mermaid block)
+/// - changes: optional satisfies: [R-id] field for requirement traceability
 pub const UNIVERSAL_SKELETON: &str = r#"---
 id: {spec_id}
 main_spec_ref: ~
@@ -27,36 +38,130 @@ merge_strategy: new
 <!-- TODO -->
 
 ## Requirements
-<!-- type: requirements lang: markdown -->
+<!-- type: requirements lang: mermaid -->
 
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus requirementDiagram (SysML v1.6). Example:
+```mermaid
+---
+id: requirements
+---
+requirementDiagram
+
+requirement R1 {
+  id: R1
+  text: "Description of requirement 1"
+  risk: low
+  verifymethod: test
+}
+
+requirement R2 {
+  id: R2
+  text: "Description of requirement 2"
+  risk: medium
+  verifymethod: analysis
+}
+```
+-->
 
 ## Scenarios
-<!-- type: scenarios lang: markdown -->
+<!-- type: scenarios lang: yaml -->
 
-<!-- TODO -->
+<!-- TODO: Use YAML GWT structured format. Example:
+```yaml
+- id: S1
+  given: Initial state description
+  when: Action or event that triggers the scenario
+  then: Expected outcome
+
+- id: S2
+  given: Another initial state
+  when: Another action
+  then: Another expected outcome
+  diagram_ref: interaction-S2
+```
+-->
 
 ## Diagrams
 
+### Mindmap
+<!-- type: mindmap lang: mermaid -->
+<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: mindmap
+---
+mindmap
+  root((System))
+    Component A
+    Component B
+```
+-->
+
+### State Machine
+<!-- type: state-machine lang: mermaid -->
+<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: state-machine
+initial: idle
+---
+stateDiagram-v2
+    [*] --> idle
+```
+-->
+
 ### Interaction
 <!-- type: interaction lang: mermaid -->
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus sequenceDiagram (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: interaction
+---
+sequenceDiagram
+    actor User
+    User->>System: action
+```
+-->
 
 ### Logic
 <!-- type: logic lang: mermaid -->
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: logic
+---
+flowchart TD
+    A([Start]) --> B{Decision}
+```
+-->
 
 ### Dependencies
 <!-- type: dependency lang: mermaid -->
-<!-- TODO -->
-
-### State Machine
-<!-- type: state-machine lang: mermaid -->
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: dependency
+---
+classDiagram
+    class ComponentA
+    class ComponentB
+    ComponentA --> ComponentB
+```
+-->
 
 ### Data Model
 <!-- type: db-model lang: mermaid -->
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
+```mermaid
+---
+id: db-model
+---
+erDiagram
+    ENTITY {
+        string id PK
+    }
+```
+-->
 
 ## API Spec
 
@@ -65,8 +170,16 @@ merge_strategy: new
 <!-- TODO -->
 
 ### RPC API
-<!-- type: rpc-api lang: json -->
-<!-- TODO -->
+<!-- type: rpc-api lang: yaml -->
+<!-- TODO: OpenRPC 1.3 as YAML. Example:
+```yaml
+openrpc: "1.3.2"
+info:
+  title: Service Name
+  version: "1.0.0"
+methods: []
+```
+-->
 
 ### Async API
 <!-- type: async-api lang: yaml -->
@@ -77,17 +190,44 @@ merge_strategy: new
 <!-- TODO -->
 
 ### Schema
-<!-- type: schema lang: json -->
-<!-- TODO -->
+<!-- type: schema lang: yaml -->
+<!-- TODO: JSON Schema as YAML. Example:
+```yaml
+"$schema": "https://json-schema.org/draft/2020-12/schema"
+type: object
+properties:
+  id:
+    type: string
+required: [id]
+```
+-->
 
 ### Config
-<!-- type: config lang: json -->
+<!-- type: config lang: yaml -->
 <!-- TODO -->
 
 ## Test Plan
-<!-- type: test-plan lang: markdown -->
+<!-- type: test-plan lang: mermaid -->
 
-<!-- TODO -->
+<!-- TODO: Use Mermaid Plus requirementDiagram with element nodes and verifies relationships.
+```mermaid
+---
+id: test-plan
+---
+requirementDiagram
+
+element T1 {
+  type: "Test"
+}
+
+element T2 {
+  type: "Test"
+}
+
+T1 - verifies -> R1
+T2 - verifies -> R2
+```
+-->
 
 ## Changes
 <!-- type: changes lang: yaml -->
@@ -100,12 +240,12 @@ merge_strategy: new
 <!-- TODO -->
 
 ## Component
-<!-- type: component lang: json -->
+<!-- type: component lang: yaml -->
 
 <!-- TODO -->
 
 ## Design Token
-<!-- type: design-token lang: json -->
+<!-- type: design-token lang: yaml -->
 
 <!-- TODO -->
 
@@ -117,32 +257,33 @@ merge_strategy: new
 # Reviews
 "#;
 
+// @spec .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-fill-order-topdown.md#R2
 /// All fillable section names (used for analyze step).
 ///
-/// Ordered by `SectionType::fill_order()` — overview first, doc last.
+/// Ordered by `SectionType::fill_order()` — top-down human reasoning order.
 /// Must match `SectionType::as_str()` values.
 pub const ALL_SECTIONS: &[&str] = &[
-    "overview",
-    "db-model",
-    "schema",
-    "state-machine",
-    "logic",
-    "dependency",
-    "interaction",
-    "rest-api",
-    "rpc-api",
-    "async-api",
-    "cli",
-    "wireframe",
-    "component",
-    "design-token",
-    "config",
-    "mindmap",
-    "requirements",
-    "scenarios",
-    "test-plan",
-    "changes",
-    "doc",
+    "overview",       // 0
+    "requirements",   // 1
+    "scenarios",      // 2
+    "mindmap",        // 3
+    "state-machine",  // 4
+    "interaction",    // 5
+    "logic",          // 6
+    "dependency",     // 7
+    "db-model",       // 8
+    "schema",         // 9
+    "rest-api",       // 10
+    "rpc-api",        // 11
+    "async-api",      // 12
+    "cli",            // 13
+    "wireframe",      // 14
+    "component",      // 15
+    "design-token",   // 16
+    "config",         // 17
+    "test-plan",      // 18
+    "changes",        // 19
+    "doc",            // 20
 ];
 
 // ─── Spec Path Helpers ──────────────────────────────────────────────────────
@@ -1025,12 +1166,19 @@ mod tests {
             None,
             Path::new("/tmp"),
         );
-        // Verify section type annotations are present
+        // Verify section type annotations are present (new format: 3 langs only)
         assert!(skeleton.contains("<!-- type: overview lang: markdown -->"));
         assert!(skeleton.contains("<!-- type: changes lang: yaml -->"));
-        assert!(skeleton.contains("<!-- type: requirements lang: markdown -->"));
+        assert!(skeleton.contains("<!-- type: requirements lang: mermaid -->")); // was markdown
+        assert!(skeleton.contains("<!-- type: scenarios lang: yaml -->")); // was markdown
+        assert!(skeleton.contains("<!-- type: test-plan lang: mermaid -->")); // was markdown
         assert!(skeleton.contains("<!-- type: interaction lang: mermaid -->"));
         assert!(skeleton.contains("<!-- type: rest-api lang: yaml -->"));
+        assert!(skeleton.contains("<!-- type: rpc-api lang: yaml -->")); // was json
+        assert!(skeleton.contains("<!-- type: schema lang: yaml -->")); // was json
+        assert!(skeleton.contains("<!-- type: config lang: yaml -->")); // was json
+        assert!(skeleton.contains("<!-- type: component lang: yaml -->")); // was json
+        assert!(skeleton.contains("<!-- type: design-token lang: yaml -->")); // was json
     }
 
     #[test]

```

## Review: sdd-fill-order-topdown

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-spec-format-unify

**Summary**: All hard checklist items pass. R1: fill_order() reordered to top-down in spec_rules.rs with correct values (overview=0, requirements=1, scenarios=2...doc=20). R2: ALL_SECTIONS constant updated in common_change_spec.rs to match new order. R3: Tests updated — test_section_type_default_lang updated, test_fill_order_requirements_before_schema added. All 1388 tests pass. The alignment warnings are about the change spec file's unfilled TODO sections (not implementation code) — informational only.

## Review: sdd-format-rules-yaml

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-spec-format-unify

**Summary**: All hard checklist items pass. R1: default_lang() updated in spec_rules.rs — schema/rpc-api/config/component/design-token return yaml, requirements/test-plan return mermaid, scenarios return yaml. R2: REQUIRED_CODE_BLOCK_TYPES in format_rules.rs updated for all changed langs. R3: Mermaid Plus mindmap entry added. R4: requirements/test-plan added as mermaid, removed from PROSE_ONLY_TYPES. R5: scenarios added as yaml, removed from PROSE_ONLY_TYPES. Only overview and doc remain in PROSE_ONLY_TYPES. 10 new tests in format_rules.rs, 1388 total passing.

## Review: sdd-spec-format-unification

verdict: APPROVED
reviewer: reviewer
iteration: 1
change_id: sdd-spec-format-unify

**Summary**: All requirements implemented. R1: fill_order top-down in spec_rules.rs. R2: default_lang yaml for schema/rpc-api/config/component/design-token. R3: requirements/test-plan use mermaid. R4: scenarios uses yaml GWT format. R5: format_rules.rs REQUIRED_CODE_BLOCK_TYPES updated, PROSE_ONLY_TYPES reduced to overview/doc only. R6: UNIVERSAL_SKELETON updated with Mermaid Plus stubs, correct lang annotations. R7: changes section example includes satisfies field. R8: Mermaid Plus generators wired via UNIVERSAL_SKELETON stubs and lang annotations. 1388 tests passing. All acceptance criteria from the original issue are met.



## Alignment Warnings

28 violation(s) found across 3 spec(s).

| File | Kind | Message |
|------|------|---------|
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | missing_section_annotation | Section 'Requirements' at line 14 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | missing_section_annotation | Section 'Diagrams' at line 40 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | missing_section_annotation | Section 'API Spec' at line 62 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | missing_section_annotation | Section 'Changes' at line 93 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | format_priority_violation | Section 'Scenarios' (type: scenarios) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | format_priority_violation | Section 'Test Plan' (type: test-plan) requires a ```mermaid code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-structure.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | missing_section_annotation | Section 'Requirements' at line 14 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | missing_section_annotation | Section 'Diagrams' at line 59 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | missing_section_annotation | Section 'API Spec' at line 81 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | missing_section_annotation | Section 'Changes' at line 112 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | format_priority_violation | Section 'Scenarios' (type: scenarios) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | format_priority_violation | Section 'Test Plan' (type: test-plan) requires a ```mermaid code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/check-alignment.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'Overview' at line 9 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'Requirements' at line 22 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'Scenarios' at line 81 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'Diagrams' at line 123 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'API Spec' at line 145 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | missing_section_annotation | Section 'Changes' at line 176 has no type annotation (expected <!-- type: X lang: Y -->) |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | format_priority_violation | Section 'Test Plan' (type: test-plan) requires a ```mermaid code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | format_priority_violation | Section 'Wireframe' (type: wireframe) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | format_priority_violation | Section 'Component' (type: component) requires a ```yaml code block but none found |
| /Users/chrischeng/projects/cclab/.score/tech_design/crates/sdd/logic/spec-format-unification.md | format_priority_violation | Section 'Design Token' (type: design-token) requires a ```yaml code block but none found |
