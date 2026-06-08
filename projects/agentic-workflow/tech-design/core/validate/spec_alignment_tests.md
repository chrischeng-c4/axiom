---
id: projects-sdd-tests-spec-alignment-tests-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment validation TDs implement source/spec traceability closure gates."
---

# Spec Alignment Integration Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated integration tests for spec alignment parsing, format rules, logical
rules, recursive checks, output-shape assertions, and known regression fixtures.
The file is emitted through the Rust tests template using a module wrapper and
module-local helper preamble so the semantic section remains `tests` instead
of splitting Rust-specific fixtures into new section types.

## Tests
<!-- type: tests lang: yaml -->

```yaml
file_preamble: |
  //! Tests for spec alignment checking.
  //!
  //! Covers all 23 test cases from the check-alignment change spec Test Plan:
  //! - Unit tests: parser, format rules, logical rules
  //! - Integration tests: check() entry point, CLI output
  //! - Acceptance tests: real spec files
module:
  attributes:
    - "#[cfg(test)]"
  name: spec_alignment_tests
preamble: |
      use std::fs;
      use std::path::PathBuf;
  
      use agentic_workflow::spec_alignment;
      use agentic_workflow::spec_alignment::{CheckResult, SpecDocument, Violation, ViolationKind};
  
      // =========================================================================
      // Helpers
      // =========================================================================
  
      /// Create a temporary directory with helper methods for writing spec files.
      struct TestDir {
          _temp: tempfile::TempDir,
          root: PathBuf,
      }
  
      impl TestDir {
          fn new() -> Self {
              let temp = tempfile::TempDir::new().unwrap();
              let root = temp.path().to_path_buf();
              Self { _temp: temp, root }
          }
  
          /// Write a file relative to the temp root and return its path.
          fn write_file(&self, name: &str, content: &str) -> PathBuf {
              let path = self.root.join(name);
              if let Some(parent) = path.parent() {
                  fs::create_dir_all(parent).unwrap();
              }
              fs::write(&path, content).unwrap();
              path
          }
      }
  
      /// Helper: parse content into a SpecDocument.
      fn parse(content: &str) -> SpecDocument {
          spec_alignment::parser::parse("test.md", content)
      }
  
      /// Helper: run format checks on content.
      fn format_violations(content: &str) -> Vec<Violation> {
          let doc = parse(content);
          spec_alignment::format_rules::check(&doc)
      }
  
      /// Helper: run logical checks on content.
      fn logical_violations(content: &str) -> Vec<Violation> {
          let doc = parse(content);
          spec_alignment::logical_rules::check(&doc)
      }
  
      /// Find violations of a specific kind.
      fn find_kind(violations: &[Violation], kind: ViolationKind) -> Vec<&Violation> {
          violations.iter().filter(|v| v.kind == kind).collect()
      }
  
      // =========================================================================
      // Unit: Parser tests (R1)
      // =========================================================================
imports: []
tests:
  - name: test_parse_spec_document_with_frontmatter
    indent_body: false
    body: |
      let content = r#"---
      id: test-spec
      main_spec_ref: "some/path.md"
      ---
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      Some overview text.
      
      ## Config
      <!-- type: config lang: json -->
      
      ```json
      {"key": "value"}
      ```
      
      ## Logic
      <!-- type: logic lang: mermaid -->
      
      ```mermaid
      flowchart TD
          A --> B
      ```
      "#;
      
      let doc = parse(content);
      
      // Frontmatter parsed
      assert!(doc.frontmatter.is_object());
      assert_eq!(doc.frontmatter["id"], "test-spec");
      
      // 3 sections
      assert_eq!(doc.sections.len(), 3);
      
      // Annotations parsed
      let s0 = &doc.sections[0];
      assert_eq!(s0.heading, "Overview");
      let ann0 = s0.annotation.as_ref().unwrap();
      assert_eq!(ann0.section_type, "overview");
      assert_eq!(ann0.lang, "markdown");
      
      let s1 = &doc.sections[1];
      assert_eq!(s1.heading, "Config");
      let ann1 = s1.annotation.as_ref().unwrap();
      assert_eq!(ann1.section_type, "config");
      assert_eq!(ann1.lang, "json");
      
      let s2 = &doc.sections[2];
      assert_eq!(s2.heading, "Logic");
      let ann2 = s2.annotation.as_ref().unwrap();
      assert_eq!(ann2.section_type, "logic");
      assert_eq!(ann2.lang, "mermaid");
  - name: test_parse_section_without_annotation
    indent_body: false
    body: |
      let content = r#"---
      id: test
      ---
      
      ## Commands
      
      Some content without annotation.
      "#;
      
      let doc = parse(content);
      assert_eq!(doc.sections.len(), 1);
      assert_eq!(doc.sections[0].heading, "Commands");
      assert!(doc.sections[0].annotation.is_none());
  - name: test_parse_code_blocks_within_section
    indent_body: false
    body: |
      let content = r#"## Data
      <!-- type: schema lang: json -->
      
      ```json
      {"type": "object"}
      ```
      
      ```yaml
      name: test
      version: 1
      ```
      "#;
      
      let doc = parse(content);
      assert_eq!(doc.sections.len(), 1);
      let section = &doc.sections[0];
      assert_eq!(section.code_blocks.len(), 2);
      assert_eq!(section.code_blocks[0].lang, "json");
      assert_eq!(section.code_blocks[1].lang, "yaml");
      assert!(section.code_blocks[0].content.contains("\"type\""));
      assert!(section.code_blocks[1].content.contains("name: test"));
  - name: test_parse_json_code_block
    indent_body: false
    body: |
      let content = r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {"name": "test_tool", "type": "object"}
      ```
      "#;
      
      let doc = parse(content);
      let block = &doc.sections[0].code_blocks[0];
      assert_eq!(block.lang, "json");
      assert!(block.parsed_json.is_some());
      let json = block.parsed_json.as_ref().unwrap();
      assert_eq!(json["name"], "test_tool");
  - name: test_parse_invalid_json_code_block
    indent_body: false
    body: |
      let content = r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {not valid json: [}
      ```
      "#;
      
      let doc = parse(content);
      let block = &doc.sections[0].code_blocks[0];
      assert_eq!(block.lang, "json");
      assert!(block.parsed_json.is_none()); // No error, just None
  - name: test_missing_section_annotation
    indent_body: false
    body: |
      let content = r#"## Foo
      
      Some content without annotation.
      "#;
      
      let violations = format_violations(content);
      let missing = find_kind(&violations, ViolationKind::MissingSectionAnnotation);
      assert_eq!(missing.len(), 1);
      assert_eq!(missing[0].heading.as_deref(), Some("Foo"));
      assert!(missing[0].line.is_some());
  - name: test_duplicate_section_heading
    indent_body: false
    body: |
      // Build content with two ## Overview sections separated by enough lines
      let content = r#"---
      id: test
      ---
      
      Some intro text.
      More intro.
      More intro.
      More intro.
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      First overview content.
      
      Padding line 1.
      Padding line 2.
      Padding line 3.
      Padding line 4.
      Padding line 5.
      Padding line 6.
      Padding line 7.
      Padding line 8.
      Padding line 9.
      Padding line 10.
      Padding line 11.
      Padding line 12.
      Padding line 13.
      Padding line 14.
      Padding line 15.
      Padding line 16.
      Padding line 17.
      Padding line 18.
      Padding line 19.
      Padding line 20.
      Padding line 21.
      Padding line 22.
      Padding line 23.
      Padding line 24.
      Padding line 25.
      Padding line 26.
      Padding line 27.
      Padding line 28.
      Padding line 29.
      Padding line 30.
      Padding line 31.
      Padding line 32.
      Padding line 33.
      Padding line 34.
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      Second overview content.
      "#;
      
      let violations = format_violations(content);
      let dupes = find_kind(&violations, ViolationKind::DuplicateSection);
      assert_eq!(dupes.len(), 1);
      assert_eq!(dupes[0].heading.as_deref(), Some("Overview"));
      let lines = dupes[0].lines.as_ref().unwrap();
      assert_eq!(lines.len(), 2);
      // Both should be present (exact lines depend on content above)
      assert!(lines[0] < lines[1]);
  - name: test_format_priority_violation_config_no_yaml
    indent_body: false
    body: |
      // Config sections require a yaml code block (per format_rules.rs:25 —
      // yaml is more token-efficient than json).
      let content = r#"## Config
      <!-- type: config lang: yaml -->
      
      Just some prose, no code block.
      "#;
      
      let violations = format_violations(content);
      let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
      assert_eq!(fpv.len(), 1);
      assert_eq!(fpv[0].heading.as_deref(), Some("Config"));
      assert_eq!(fpv[0].expected_lang.as_deref(), Some("yaml"));
  - name: test_format_priority_violation_logic_no_mermaid
    indent_body: false
    body: |
      let content = r#"## Logic
      <!-- type: logic lang: mermaid -->
      
      Just some prose, no mermaid diagram.
      "#;
      
      let violations = format_violations(content);
      let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
      assert_eq!(fpv.len(), 1);
      assert_eq!(fpv[0].heading.as_deref(), Some("Logic"));
      assert_eq!(fpv[0].expected_lang.as_deref(), Some("mermaid"));
  - name: test_prose_only_section_exempt
    indent_body: false
    body: |
      let content = r#"## Overview
      <!-- type: overview lang: markdown -->
      
      This is just prose. No code blocks required.
      The overview section type is exempt from format_priority_violation.
      "#;
      
      let violations = format_violations(content);
      let fpv = find_kind(&violations, ViolationKind::FormatPriorityViolation);
      assert!(
          fpv.is_empty(),
          "Overview (prose-only type) should not trigger format_priority_violation"
      );
  - name: test_duplicate_definition_same_name
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {"name": "foo", "description": "First definition"}
      ```
      
      ```json
      {"name": "foo", "description": "Second definition"}
      ```
      "#;
      
      let violations = logical_violations(content);
      let dupes = find_kind(&violations, ViolationKind::DuplicateDefinition);
      assert_eq!(dupes.len(), 1);
      assert_eq!(dupes[0].name.as_deref(), Some("foo"));
      let lines = dupes[0].lines.as_ref().unwrap();
      assert_eq!(lines.len(), 2);
  - name: test_definition_conflict_required
    indent_body: false
    body: |
      let content = r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {
        "name": "sdd_workflow_create_change_merge",
        "required": ["status", "specs_merged", "audit_log"],
        "properties": {
          "status": {"type": "string"},
          "specs_merged": {"type": "boolean"},
          "audit_log": {"type": "array"}
        }
      }
      ```
      
      ```json
      {
        "name": "sdd_workflow_create_change_merge",
        "required": ["status", "merged_specs"],
        "properties": {
          "status": {"type": "string"},
          "merged_specs": {"type": "boolean"}
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let conflicts = find_kind(&violations, ViolationKind::DefinitionConflictRequired);
      assert_eq!(conflicts.len(), 1);
      assert_eq!(
          conflicts[0].name.as_deref(),
          Some("sdd_workflow_create_change_merge")
      );
      // Details should contain blocks with their required arrays
      assert!(conflicts[0].details.is_some());
  - name: test_definition_conflict_field_name_near_match
    indent_body: false
    body: |
      // Use fields with actual Levenshtein edit distance ≤ 2:
      // "statuss" vs "status" (distance = 1), "counnt" vs "count" (distance = 1)
      let content = r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "properties": {
          "statuss": {"type": "string"},
          "count": {"type": "integer"}
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "properties": {
          "status": {"type": "string"},
          "counnt": {"type": "integer"}
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let near_matches = find_kind(&violations, ViolationKind::DefinitionConflictFieldName);
      assert_eq!(near_matches.len(), 1);
      assert_eq!(near_matches[0].name.as_deref(), Some("my_tool"));
      // Details should contain the near-match pairs
      let details = near_matches[0].details.as_ref().unwrap();
      let pairs = details["pairs"].as_array().unwrap();
      assert!(!pairs.is_empty());
      // Check that pairs contain the near-match field names
      let all_pair_strs: Vec<Vec<&str>> = pairs
          .iter()
          .map(|p| {
              p.as_array()
                  .unwrap()
                  .iter()
                  .map(|v| v.as_str().unwrap())
                  .collect()
          })
          .collect();
      // Should have status/statuss and count/counnt pairs
      assert!(
          all_pair_strs
              .iter()
              .any(|p| p.contains(&"status") && p.contains(&"statuss")),
          "Should contain status vs statuss pair, got: {:?}",
          all_pair_strs
      );
      assert!(
          all_pair_strs
              .iter()
              .any(|p| p.contains(&"count") && p.contains(&"counnt")),
          "Should contain count vs counnt pair, got: {:?}",
          all_pair_strs
      );
  - name: test_definition_conflict_schema_type_mismatch
    indent_body: false
    body: |
      let content = r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "properties": {
          "status": {"type": "string"}
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "properties": {
          "status": {"type": "string", "enum": ["ok", "error"]}
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let schema_conflicts = find_kind(&violations, ViolationKind::DefinitionConflictSchema);
      assert_eq!(schema_conflicts.len(), 1);
      assert_eq!(schema_conflicts[0].field.as_deref(), Some("status"));
      assert_eq!(schema_conflicts[0].name.as_deref(), Some("my_tool"));
  - name: test_rpc_field_consistency_x_extension
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "sdd_tool",
        "x-sdd": {"phase": "init"},
        "description": "First"
      }
      ```
      
      ```json
      {
        "name": "sdd_tool",
        "x-sdd": {"phase": "complete"},
        "description": "Second"
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let rpc = find_kind(&violations, ViolationKind::RpcFieldConsistency);
      assert_eq!(rpc.len(), 1);
      assert_eq!(rpc[0].name.as_deref(), Some("sdd_tool"));
      assert_eq!(rpc[0].field.as_deref(), Some("x-sdd"));
  - name: test_check_single_file
    indent_body: false
    body: |
      let dir = TestDir::new();
      let file = dir.write_file(
          "clean.md",
          r#"---
      id: clean
      ---
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      All good here.
      
      ## Config
      <!-- type: config lang: yaml -->
      
      ```yaml
      key: value
      ```
      "#,
      );
      
      let result = spec_alignment::check(&file);
      assert!(result.passed);
      assert_eq!(result.total_violations, 0);
      assert_eq!(result.files.len(), 1);
      assert_eq!(result.files[0].status, "ok");
  - name: test_check_directory_recursive
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // File 1: clean
      dir.write_file(
          "specs/clean1.md",
          r#"## Overview
      <!-- type: overview lang: markdown -->
      
      Content.
      "#,
      );
      
      // File 2: clean
      dir.write_file(
          "specs/clean2.md",
          r#"## Requirements
      <!-- type: requirements lang: mermaid -->
      
      ```mermaid
      requirementDiagram
      requirement R1 {
        id: R1
        text: "stub"
        risk: low
        verifymethod: test
      }
      ```
      "#,
      );
      
      // File 3: has violations (missing annotation)
      dir.write_file(
          "specs/sub/bad.md",
          r#"## Commands
      
      No annotation here.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let result = spec_alignment::check(&specs_dir);
      
      assert!(!result.passed);
      assert_eq!(result.files.len(), 3);
      
      let fail_count = result.files.iter().filter(|f| f.status == "fail").count();
      let ok_count = result.files.iter().filter(|f| f.status == "ok").count();
      assert_eq!(fail_count, 1);
      assert_eq!(ok_count, 2);
  - name: test_cli_text_output_format
    indent_body: false
    body: |
      // Verify CheckResult serialization structure matches expected format.
      // We test the data model since the CLI handler uses process::exit which
      // we cannot test directly in an integration test within the same process.
      let dir = TestDir::new();
      let file = dir.write_file(
          "bad.md",
          r#"## Commands
      
      No annotation.
      
      ## Commands
      
      Duplicate heading also without annotation.
      "#,
      );
      
      let result = spec_alignment::check(&file);
      assert!(!result.passed);
      assert!(result.total_violations >= 2);
      
      // Verify the violations contain the expected kinds
      let violations = &result.files[0].violations;
      let kinds: Vec<&ViolationKind> = violations.iter().map(|v| &v.kind).collect();
      assert!(kinds.contains(&&ViolationKind::MissingSectionAnnotation));
      assert!(kinds.contains(&&ViolationKind::DuplicateSection));
      
      // Verify text format: kind Display gives snake_case
      for v in violations {
          let display = format!("{}", v.kind);
          // Must be snake_case, not PascalCase
          assert!(
              !display.contains(char::is_uppercase),
              "ViolationKind Display should be snake_case, got: {}",
              display
          );
      }
  - name: test_cli_json_output_format
    indent_body: false
    body: |
      let dir = TestDir::new();
      let file = dir.write_file(
          "bad.md",
          r#"## Commands
      
      No annotation.
      "#,
      );
      
      let result = spec_alignment::check(&file);
      
      // Serialize to JSON and verify it round-trips
      let json_str = serde_json::to_string_pretty(&result).unwrap();
      let parsed: CheckResult = serde_json::from_str(&json_str).unwrap();
      
      assert_eq!(parsed.files.len(), result.files.len());
      assert_eq!(parsed.total_violations, result.total_violations);
      assert_eq!(parsed.passed, result.passed);
      
      // Verify JSON contains snake_case kind values
      assert!(json_str.contains("\"missing_section_annotation\""));
  - name: test_cli_exit_code_clean
    indent_body: false
    body: |
      let dir = TestDir::new();
      dir.write_file(
          "specs/clean.md",
          r#"## Overview
      <!-- type: overview lang: markdown -->
      
      Clean spec content.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let result = spec_alignment::check(&specs_dir);
      assert!(result.passed, "Clean specs should pass (exit code 0)");
      assert_eq!(result.total_violations, 0);
  - name: test_cli_exit_code_violations
    indent_body: false
    body: |
      let dir = TestDir::new();
      dir.write_file(
          "specs/bad.md",
          r#"## NoAnnotation
      
      Missing annotation.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let result = spec_alignment::check(&specs_dir);
      assert!(
          !result.passed,
          "Specs with violations should fail (exit code 1)"
      );
      assert!(result.total_violations > 0);
  - name: test_zero_false_positives_on_existing_specs
    body: |
      // Run against a known clean spec file.
      // This test uses the actual project spec file — it will only pass
      // when run from the project root (cargo test from workspace).
      let spec_path = PathBuf::from("projects/agentic-workflow/tech-design/core/logic/spec-structure.md");
      if !spec_path.exists() {
          // Skip if not run from project root (e.g. in CI with different working dir)
          eprintln!(
              "SKIP test_zero_false_positives_on_existing_specs: spec file not found at {}",
              spec_path.display()
          );
          return;
      }
      
      let result = spec_alignment::check(&spec_path);
      assert!(
          result.passed,
          "Known clean spec file should have 0 violations, got {}: {:?}",
          result.total_violations,
          result
              .files
              .iter()
              .flat_map(|f| &f.violations)
              .map(|v| format!("{}: {}", v.kind, v.message))
              .collect::<Vec<_>>()
      );
  - name: test_catches_1136_violations
    indent_body: false
    body: |
      // Reproduce the kinds of violations found in #1136:
      // - 4 duplicate "Overview" headings
      // - 3 conflicting RPC definitions (same name, different fields)
      // - Missing annotations
      let dir = TestDir::new();
      let file = dir.write_file(
          "issue_1136_repro.md",
          r#"---
      id: repro-1136
      ---
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      First overview.
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      Second overview.
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      Third overview.
      
      ## Overview
      <!-- type: overview lang: markdown -->
      
      Fourth overview.
      
      ## Commands
      
      Missing annotation on this section.
      
      ## API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "sdd_workflow_merge",
        "required": ["status", "spec_merged"],
        "properties": {
          "status": {"type": "string"},
          "spec_merged": {"type": "boolean"}
        },
        "x-sdd": {"phase": "merge"}
      }
      ```
      
      ```json
      {
        "name": "sdd_workflow_merge",
        "required": ["status", "specs_merged", "audit_log"],
        "properties": {
          "status": {"type": "string", "enum": ["ok", "error"]},
          "specs_merged": {"type": "boolean"},
          "audit_log": {"type": "array"}
        },
        "x-sdd": {"phase": "init"}
      }
      ```
      
      ```json
      {
        "name": "sdd_workflow_merge",
        "required": ["result"],
        "properties": {
          "result": {"type": "object"}
        },
        "x-sdd": {"phase": "complete"}
      }
      ```
      "#,
      );
      
      let result = spec_alignment::check(&file);
      assert!(!result.passed);
      
      let violations = &result.files[0].violations;
      
      // Should catch duplicate "Overview" headings
      let dup_sections = find_kind(violations, ViolationKind::DuplicateSection);
      assert!(
          !dup_sections.is_empty(),
          "Should detect duplicate Overview sections"
      );
      // Verify the duplicate Overview has 4 lines
      let overview_dup = dup_sections
          .iter()
          .find(|v| v.heading.as_deref() == Some("Overview"))
          .expect("Should have Overview duplicate");
      assert_eq!(overview_dup.lines.as_ref().unwrap().len(), 4);
      
      // Should catch missing annotation on "Commands"
      let missing_ann = find_kind(violations, ViolationKind::MissingSectionAnnotation);
      assert!(
          !missing_ann.is_empty(),
          "Should detect missing annotation on Commands"
      );
      let commands_missing = missing_ann
          .iter()
          .find(|v| v.heading.as_deref() == Some("Commands"));
      assert!(commands_missing.is_some());
      
      // Should catch duplicate RPC definitions
      let dup_defs = find_kind(violations, ViolationKind::DuplicateDefinition);
      assert!(
          !dup_defs.is_empty(),
          "Should detect duplicate definitions for sdd_workflow_merge"
      );
      
      // Should catch conflicting required arrays
      let req_conflicts = find_kind(violations, ViolationKind::DefinitionConflictRequired);
      assert!(
          !req_conflicts.is_empty(),
          "Should detect conflicting required arrays"
      );
      
      // Should catch near-match field names (spec_merged vs specs_merged, edit distance = 1)
      let field_near = find_kind(violations, ViolationKind::DefinitionConflictFieldName);
      assert!(
          !field_near.is_empty(),
          "Should detect near-match field names (spec_merged vs specs_merged)"
      );
      
      // Should catch schema conflicts (status: string vs string+enum)
      let schema_conflicts = find_kind(violations, ViolationKind::DefinitionConflictSchema);
      assert!(
          !schema_conflicts.is_empty(),
          "Should detect schema type conflicts on status field"
      );
      
      // Should catch x-sdd extension inconsistencies
      let rpc_ext = find_kind(violations, ViolationKind::RpcFieldConsistency);
      assert!(
          !rpc_ext.is_empty(),
          "Should detect x-sdd extension mismatches"
      );
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/spec_alignment_tests.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit the spec alignment integration tests from the Rust tests template,
      including module wrapper support, module-local helpers, and raw fixture
      body preservation.
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
