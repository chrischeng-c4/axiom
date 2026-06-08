---
id: projects-sdd-tests-spec-alignment-phase2-tests-rs
fill_sections: [overview, tests, changes]
capability_refs:
  - id: existing-project-standardization
    role: primary
    gap: traceability-closure-gate
    claim: traceability-closure-gate
    coverage: full
    rationale: "Spec alignment validation TDs implement source/spec traceability closure gates."
---

# Spec Alignment Phase 2 Tests

## Overview
<!-- type: overview lang: markdown -->

Codegenerated phase-2 integration tests for @spec annotation parsing, coverage
analysis, requirement/scenario relationships, nested schema conflict detection,
implementation prompt checks, and CLI coverage integration. The file is emitted
through the Rust tests template with module wrapper, leading section banners,
ignored tests, and raw Markdown fixture preservation.

## Tests
<!-- type: tests lang: yaml -->

```yaml
file_preamble: |
  //! Phase 2 tests for spec alignment checking.
  //!
  //! Covers test cases from the check-alignment-phase2 change spec Test Plan:
  //! - R14: @spec annotation parsing (7 tests)
  //! - R15: Coverage analysis (4 tests)
  //! - R16: Unspecced function detection stubs (1 test)
  //! - R17: Schema↔struct validation stubs (2 tests)
  //! - R18: Requirement↔scenario cross-reference (3 tests)
  //! - R19: Nested schema conflict detection (5 tests)
  //! - R20: Implementation prompt @spec instruction (1 test)
  //! - R21: CLI / check_with_coverage integration (3 tests)
module:
  attributes:
    - "#[cfg(test)]"
  name: spec_alignment_phase2_tests
preamble: |
      use std::fs;
      use std::path::PathBuf;
  
      use agentic_workflow::spec_alignment;
      use agentic_workflow::spec_alignment::{Violation, ViolationKind};
  
      // =========================================================================
      // Helpers
      // =========================================================================
  
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
  
          fn write_file(&self, name: &str, content: &str) -> PathBuf {
              let path = self.root.join(name);
              if let Some(parent) = path.parent() {
                  fs::create_dir_all(parent).unwrap();
              }
              fs::write(&path, content).unwrap();
              path
          }
      }
  
      fn parse(content: &str) -> spec_alignment::SpecDocument {
          spec_alignment::parser::parse("test.md", content)
      }
  
      fn logical_violations(content: &str) -> Vec<Violation> {
          let doc = parse(content);
          spec_alignment::logical_rules::check(&doc)
      }
  
      fn find_kind(violations: &[Violation], kind: ViolationKind) -> Vec<&Violation> {
          violations.iter().filter(|v| v.kind == kind).collect()
      }
  
      // =========================================================================
      // R14: @spec annotation parsing
      // =========================================================================
imports: []
tests:
  - name: test_parse_annotation_rust_comment
    body: |
      let content = "// @spec projects/agentic-workflow/logic/check-alignment.md#R1\n";
      let annotations = spec_alignment::annotations::scan_file("test.rs", content);
      
      assert_eq!(annotations.len(), 1);
      assert_eq!(
          annotations[0].spec_path,
          "projects/agentic-workflow/logic/check-alignment.md"
      );
      assert_eq!(annotations[0].requirement_id, "R1");
      assert_eq!(annotations[0].source_file, "test.rs");
      assert_eq!(annotations[0].line, 1);
      assert_eq!(annotations[0].comment_syntax, "//");
  - name: test_parse_annotation_python_comment
    body: |
      let content = "# @spec specs/my-spec.md#R5\n";
      let annotations = spec_alignment::annotations::scan_file("test.py", content);
      
      assert_eq!(annotations.len(), 1);
      assert_eq!(annotations[0].spec_path, "specs/my-spec.md");
      assert_eq!(annotations[0].requirement_id, "R5");
      assert_eq!(annotations[0].comment_syntax, "#");
  - name: test_parse_annotation_html_comment
    body: |
      let content = "<!-- @spec docs/api-spec.md#R12 -->\n";
      let annotations = spec_alignment::annotations::scan_file("test.html", content);
      
      assert_eq!(annotations.len(), 1);
      assert_eq!(annotations[0].spec_path, "docs/api-spec.md");
      assert_eq!(annotations[0].requirement_id, "R12");
      assert_eq!(annotations[0].comment_syntax, "<!--");
  - name: test_parse_annotation_no_match
    body: |
      // No @spec annotation present
      let content = "// This is a regular comment\nfn main() {}\n";
      let annotations = spec_alignment::annotations::scan_file("test.rs", content);
      assert!(annotations.is_empty());
      
      // @spec without proper path format
      let content2 = "// @spec no-extension#R1\n";
      let annotations2 = spec_alignment::annotations::scan_file("test.rs", content2);
      assert!(annotations2.is_empty());
  - name: test_parse_annotation_multiple_per_file
    indent_body: false
    body: |
      let content = r#"// @spec specs/a.md#R1
      fn foo() {}
      
      // @spec specs/a.md#R2
      fn bar() {}
      
      // @spec specs/b.md#R3
      fn baz() {}
      "#;
      let annotations = spec_alignment::annotations::scan_file("test.rs", content);
      
      assert_eq!(annotations.len(), 3);
      assert_eq!(annotations[0].requirement_id, "R1");
      assert_eq!(annotations[0].line, 1);
      assert_eq!(annotations[1].requirement_id, "R2");
      assert_eq!(annotations[1].line, 4);
      assert_eq!(annotations[2].spec_path, "specs/b.md");
      assert_eq!(annotations[2].requirement_id, "R3");
      assert_eq!(annotations[2].line, 7);
  - name: test_parse_annotation_sql_comment
    body: |
      let content = "-- @spec db/schema-spec.md#R7\n";
      let annotations = spec_alignment::annotations::scan_file("test.sql", content);
      
      assert_eq!(annotations.len(), 1);
      assert_eq!(annotations[0].spec_path, "db/schema-spec.md");
      assert_eq!(annotations[0].requirement_id, "R7");
      assert_eq!(annotations[0].comment_syntax, "--");
  - name: test_parse_annotation_c_block_comment
    body: |
      let content = "/* @spec logic/rules.md#R3 */\n";
      let annotations = spec_alignment::annotations::scan_file("test.js", content);
      
      assert_eq!(annotations.len(), 1);
      assert_eq!(annotations[0].spec_path, "logic/rules.md");
      assert_eq!(annotations[0].requirement_id, "R3");
      assert_eq!(annotations[0].comment_syntax, "/*");
  - name: test_scan_directories
    leading: |
          // =========================================================================
          // R14: Directory scanning
          // =========================================================================
    body: |
      let dir = TestDir::new();
      dir.write_file("src/main.rs", "// @spec specs/main.md#R1\nfn main() {}\n");
      dir.write_file(
          "src/lib.rs",
          "// @spec specs/lib.md#R2\npub fn lib_fn() {}\n",
      );
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let annotations = spec_alignment::annotations::scan_directories(&source_dirs);
      
      assert_eq!(annotations.len(), 2);
      let req_ids: Vec<&str> = annotations
          .iter()
          .map(|a| a.requirement_id.as_str())
          .collect();
      assert!(req_ids.contains(&"R1"));
      assert!(req_ids.contains(&"R2"));
  - name: test_coverage_all_requirements_covered
    leading: |
          // =========================================================================
          // R15: Coverage analysis
          // =========================================================================
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with 2 requirements
      let spec_path = dir.write_file(
          "specs/my-spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | First req   |
      | R2 | Second req  |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1 functionality.
      
      ### Scenario: Test R2
      
      Tests R2 functionality.
      "#,
      );
      
      // Source files with @spec annotations covering both requirements
      let specs_full_path = spec_path.display().to_string();
      dir.write_file(
          "src/main.rs",
          &format!(
              "// @spec {}#R1\nfn foo() {{}}\n// @spec {}#R2\nfn bar() {{}}\n",
              specs_full_path, specs_full_path
          ),
      );
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let specs_dir = dir.root.join("specs");
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result
          .coverage
          .as_ref()
          .expect("Coverage report should be present");
      assert_eq!(report.covered.len(), 2);
      assert!(report.uncovered_requirements.is_empty());
      assert_eq!(report.coverage_ratio, 1.0);
  - name: test_coverage_uncovered_requirement
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with 2 requirements
      dir.write_file(
          "specs/my-spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | First req   |
      | R2 | Second req  |
      
      ## Scenarios
      
      ### Scenario: Test both
      
      Tests R1 and R2 functionality.
      "#,
      );
      
      // Source files with annotation covering only R1 — R2 is uncovered
      // Use a path that won't match the spec file
      dir.write_file(
          "src/main.rs",
          "// @spec projects/agentic-workflow/tech-design/core/specs/nonexistent-spec.md#R1\nfn foo() {}\n",
      );
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let specs_dir = dir.root.join("specs");
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result
          .coverage
          .as_ref()
          .expect("Coverage report should be present");
      // Both R1 and R2 should be uncovered since the annotation path doesn't match
      assert!(!report.uncovered_requirements.is_empty());
      assert!(report.coverage_ratio < 1.0);
  - name: test_coverage_stale_annotation
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with 1 requirement
      dir.write_file(
          "specs/my-spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Only req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      
      // Source file references a non-existent spec path — should be stale
      dir.write_file(
          "src/main.rs",
          "// @spec specs/does-not-exist.md#R99\nfn foo() {}\n",
      );
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let specs_dir = dir.root.join("specs");
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result
          .coverage
          .as_ref()
          .expect("Coverage report should be present");
      assert!(
          !report.stale_annotations.is_empty(),
          "Should detect stale annotation pointing to non-existent spec"
      );
  - name: test_requirement_scenario_all_covered
    leading: |
          // =========================================================================
          // R18: Requirement↔Scenario cross-reference
          // =========================================================================
    indent_body: false
    body: |
      let dir = TestDir::new();
      let file = dir.write_file(
          "spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | First req   |
      | R2 | Second req  |
      
      ## Scenarios
      
      ### Scenario: Test both
      
      Tests R1 and R2 are both referenced here.
      "#,
      );
      
      let content = fs::read_to_string(&file).unwrap();
      let doc = spec_alignment::parser::parse(&file.display().to_string(), &content);
      let (violations, orphans) = spec_alignment::requirement_coverage::check(&doc);
      
      assert!(violations.is_empty(), "All requirements should be covered");
      assert!(orphans.is_empty());
  - name: test_requirement_scenario_orphan
    indent_body: false
    body: |
      let dir = TestDir::new();
      let file = dir.write_file(
          "spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description  |
      |----|--------------|
      | R1 | Covered req  |
      | R2 | Orphan req   |
      | R3 | Also orphan  |
      
      ## Scenarios
      
      ### Scenario: Test R1 only
      
      Only R1 is referenced.
      "#,
      );
      
      let content = fs::read_to_string(&file).unwrap();
      let doc = spec_alignment::parser::parse(&file.display().to_string(), &content);
      let (violations, orphans) = spec_alignment::requirement_coverage::check(&doc);
      
      // R2 and R3 are not referenced by any scenario
      let orphan_violations = find_kind(&violations, ViolationKind::OrphanRequirement);
      assert_eq!(orphan_violations.len(), 2);
      assert_eq!(orphans.len(), 2);
      
      let orphan_ids: Vec<&str> = orphans.iter().map(|o| o.requirement_id.as_str()).collect();
      assert!(orphan_ids.contains(&"R2"));
      assert!(orphan_ids.contains(&"R3"));
  - name: test_requirement_test_plan_covers_column
    indent_body: false
    body: |
      let dir = TestDir::new();
      let file = dir.write_file(
          "spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description  |
      |----|--------------|
      | R1 | First req    |
      | R2 | Second req   |
      
      ## Scenarios
      
      ### Scenario: only R1
      
      References R1 only in scenarios.
      
      ## Test Plan
      
      | Test | Covers | Description |
      |------|--------|-------------|
      | T1   | R2     | Tests R2    |
      "#,
      );
      
      let content = fs::read_to_string(&file).unwrap();
      let doc = spec_alignment::parser::parse(&file.display().to_string(), &content);
      let (violations, orphans) = spec_alignment::requirement_coverage::check(&doc);
      
      // R1 is referenced in scenarios, R2 in test plan — both should be covered
      assert!(
          violations.is_empty(),
          "Both R1 and R2 should be covered via scenarios and test plan respectively. Got: {:?}",
          violations
      );
      assert!(orphans.is_empty());
  - name: test_nested_schema_required_conflict
    leading: |
          // =========================================================================
          // R19: Nested schema conflict detection
          // =========================================================================
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "required": ["status", "data"],
            "properties": {
      "status": {"type": "string"},
      "data": {"type": "object"}
            }
          }
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "required": ["status"],
            "properties": {
      "status": {"type": "string"}
            }
          }
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let nested_req = find_kind(&violations, ViolationKind::NestedSchemaConflictRequired);
      assert_eq!(
          nested_req.len(),
          1,
          "Should detect conflicting nested required arrays"
      );
      assert_eq!(nested_req[0].name.as_deref(), Some("my_tool"));
  - name: test_nested_schema_properties_conflict
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "properties": {
      "status": {"type": "string"}
            }
          }
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "properties": {
      "status": {"type": "integer"}
            }
          }
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let nested_schema = find_kind(&violations, ViolationKind::NestedSchemaConflictSchema);
      assert_eq!(
          nested_schema.len(),
          1,
          "Should detect conflicting nested schema types"
      );
      assert_eq!(nested_schema[0].field.as_deref(), Some("status"));
  - name: test_nested_schema_field_name_near_match
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "properties": {
      "statuss": {"type": "string"},
      "count": {"type": "integer"}
            }
          }
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "properties": {
      "status": {"type": "string"},
      "counnt": {"type": "integer"}
            }
          }
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let nested_field = find_kind(&violations, ViolationKind::NestedSchemaConflictFieldName);
      assert_eq!(
          nested_field.len(),
          1,
          "Should detect near-match nested property names"
      );
      assert_eq!(nested_field[0].name.as_deref(), Some("my_tool"));
      
      // Check details contain the near-match pairs
      let details = nested_field[0].details.as_ref().unwrap();
      let pairs = details["pairs"].as_array().unwrap();
      assert!(!pairs.is_empty());
  - name: test_nested_schema_params_conflict
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "params": [
          {
            "name": "input",
            "schema": {
      "required": ["path"],
      "properties": {
        "path": {"type": "string"}
      }
            }
          }
        ]
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "params": [
          {
            "name": "input",
            "schema": {
      "required": ["path", "format"],
      "properties": {
        "path": {"type": "string"},
        "format": {"type": "string"}
      }
            }
          }
        ]
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let nested_req = find_kind(&violations, ViolationKind::NestedSchemaConflictRequired);
      assert_eq!(
          nested_req.len(),
          1,
          "Should detect conflicting nested required arrays in params[*].schema"
      );
  - name: test_nested_schema_no_conflict_when_identical
    leading: |
          // =========================================================================
          // R19: No false positives for consistent nested schemas
          // =========================================================================
    indent_body: false
    body: |
      let content = r#"## RPC API
      <!-- type: rpc-api lang: json -->
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "required": ["status"],
            "properties": {
      "status": {"type": "string"}
            }
          }
        }
      }
      ```
      
      ```json
      {
        "name": "my_tool",
        "result": {
          "schema": {
            "required": ["status"],
            "properties": {
      "status": {"type": "string"}
            }
          }
        }
      }
      ```
      "#;
      
      let violations = logical_violations(content);
      let nested_req = find_kind(&violations, ViolationKind::NestedSchemaConflictRequired);
      let nested_schema = find_kind(&violations, ViolationKind::NestedSchemaConflictSchema);
      let nested_field = find_kind(&violations, ViolationKind::NestedSchemaConflictFieldName);
      assert!(
          nested_req.is_empty(),
          "Identical nested required arrays should not conflict"
      );
      assert!(
          nested_schema.is_empty(),
          "Identical nested schemas should not conflict"
      );
      assert!(
          nested_field.is_empty(),
          "Identical field names should not trigger near-match"
      );
  - name: test_check_with_coverage_produces_report
    leading: |
          // =========================================================================
          // Integration: check_with_coverage entry point
          // =========================================================================
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
      let src_dir = dir.root.join("src");
      fs::create_dir_all(&src_dir).unwrap();
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      assert!(
          result.coverage.is_some(),
          "check_with_coverage should always produce a coverage report"
      );
  - name: test_check_with_coverage_detects_orphan_requirements
    indent_body: false
    body: |
      let dir = TestDir::new();
      dir.write_file(
          "specs/orphan.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description  |
      |----|--------------|
      | R1 | Covered req  |
      | R2 | Orphan req   |
      
      ## Scenarios
      
      ### Scenario: Test R1 only
      
      Only R1 is referenced here.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      fs::create_dir_all(&src_dir).unwrap();
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result.coverage.as_ref().unwrap();
      assert!(
          !report.orphan_requirements.is_empty(),
          "Should detect orphan requirements in the coverage report"
      );
      assert!(
          report
              .orphan_requirements
              .iter()
              .any(|o| o.requirement_id == "R2"),
          "R2 should be detected as orphan"
      );
  - name: test_check_with_coverage_passed_includes_coverage
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with requirement but no scenario (orphan) and no annotation (uncovered)
      dir.write_file(
          "specs/incomplete.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description  |
      |----|--------------|
      | R1 | Uncovered    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      References R1.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      fs::create_dir_all(&src_dir).unwrap();
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      // R1 has no @spec annotation in source, so uncovered_requirements is non-empty
      // Therefore passed should be false
      let report = result.coverage.as_ref().unwrap();
      if !report.uncovered_requirements.is_empty() {
          assert!(
              !result.passed,
              "passed should be false when uncovered requirements exist"
          );
      }
  - name: test_coverage_stale_requirement_id
    leading: |
          // =========================================================================
          // R15: Stale annotation — requirement ID does not exist
          // =========================================================================
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with R1 only
      let spec_path = dir.write_file(
          "specs/my-spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Only req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      
      // Source file references R99 which does not exist in the spec
      let specs_full_path = spec_path.display().to_string();
      dir.write_file(
          "src/main.rs",
          &format!("// @spec {}#R99\nfn foo() {{}}\n", specs_full_path),
      );
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let specs_dir = dir.root.join("specs");
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result
          .coverage
          .as_ref()
          .expect("Coverage report should be present");
      assert!(
          !report.stale_annotations.is_empty(),
          "Should detect stale annotation pointing to non-existent requirement ID R99"
      );
      assert_eq!(report.stale_annotations[0].requirement_id, "R99");
  - name: test_unspecced_function_detection_skipped_when_daemon_not_ready
    leading: |
          // =========================================================================
          // R16: Unspecced function detection (daemon stub)
          // =========================================================================
    indent_body: false
    body: |
      let dir = TestDir::new();
      
      // Spec with a requirement
      dir.write_file(
          "specs/my-spec.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Some req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      
      // Source file with a public function but no @spec annotation
      dir.write_file("src/main.rs", "pub fn process_data() {}\n");
      
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      let specs_dir = dir.root.join("specs");
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      let report = result
          .coverage
          .as_ref()
          .expect("Coverage report should be present");
      // Daemon is not ready in test env, so unspecced_functions should be empty (skipped)
      assert!(
          report.unspecced_functions.is_empty(),
          "unspecced_functions should be empty when daemon is not ready"
      );
  - name: test_schema_struct_check_returns_empty_when_daemon_not_ready
    leading: |
          // =========================================================================
          // R17: Schema↔struct validation stubs
          // =========================================================================
    indent_body: false
    body: |
      let dir = TestDir::new();
      dir.write_file(
          "specs/schema.md",
          r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {"name": "MyStruct", "properties": {"field": {"type": "string"}}}
      ```
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let (violations, mismatches) = spec_alignment::schema_struct::check(&specs_dir, false);
      
      assert!(violations.is_empty());
      assert!(mismatches.is_empty());
  - name: test_schema_struct_type_mapping
    body: |
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("string", None),
          "String"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("integer", None),
          "i64"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("boolean", None),
          "bool"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("string", Some("date-time")),
          "DateTime"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("string", Some("uuid")),
          "Uuid"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("array", None),
          "Vec"
      );
      assert_eq!(
          spec_alignment::schema_struct::json_schema_type_to_rust("object", None),
          "HashMap"
      );
  - name: test_implementation_prompt_has_spec_instruction
    leading: |
          // =========================================================================
          // R20: Implementation prompt contains @spec annotation instruction
          // =========================================================================
    body: |
      // Verify the create_change_impl.rs prompt template contains @spec annotation
      // instruction. We read the source file directly to verify it includes the text.
      let source = include_str!("../src/tools/create_change_impl.rs");
      assert!(
          source.contains("@spec"),
          "create_change_impl.rs must contain @spec annotation instruction"
      );
      assert!(
          source.contains("Spec Annotations"),
          "create_change_impl.rs must contain 'Spec Annotations' section header"
      );
      assert!(
          source.contains("public functions that implement spec requirements"),
          "create_change_impl.rs must instruct agents to annotate public functions"
      );
  - name: test_cli_coverage_flag_runs_phase2
    leading: |
          // =========================================================================
          // R21: CLI / check_with_coverage integration tests
          // =========================================================================
    indent_body: false
    body: |
      // Verifies that check_with_coverage produces a coverage report section.
      // This is the library-level equivalent of `aw check-alignment --coverage`.
      let dir = TestDir::new();
      dir.write_file(
          "specs/api.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Some req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      dir.write_file("src/lib.rs", "// no annotations\nfn main() {}\n");
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      fs::create_dir_all(&src_dir).unwrap();
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      
      // Coverage report must be present (Phase 2 was invoked)
      assert!(
          result.coverage.is_some(),
          "--coverage must produce a coverage report"
      );
      let report = result.coverage.as_ref().unwrap();
      // R1 should appear as uncovered since no @spec annotations exist
      assert!(
          !report.uncovered_requirements.is_empty(),
          "R1 should be uncovered when no @spec annotations exist"
      );
  - name: test_cli_coverage_json_includes_report
    indent_body: false
    body: |
      // Verifies that JSON serialization of check_with_coverage result includes
      // the 'coverage' object with all Phase 2 fields.
      let dir = TestDir::new();
      dir.write_file(
          "specs/schema.md",
          r#"## Overview
      <!-- type: overview lang: markdown -->
      
      Clean spec.
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      fs::create_dir_all(&src_dir).unwrap();
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      let json_str = serde_json::to_string_pretty(&result).unwrap();
      let json_val: serde_json::Value = serde_json::from_str(&json_str).unwrap();
      
      // JSON output must have 'coverage' top-level key with all Phase 2 fields
      let coverage = json_val
          .get("coverage")
          .expect("JSON output must have 'coverage' key");
      assert!(
          coverage.get("covered").is_some(),
          "coverage must have 'covered' field"
      );
      assert!(
          coverage.get("uncovered_requirements").is_some(),
          "coverage must have 'uncovered_requirements' field"
      );
      assert!(
          coverage.get("unspecced_functions").is_some(),
          "coverage must have 'unspecced_functions' field"
      );
      assert!(
          coverage.get("stale_annotations").is_some(),
          "coverage must have 'stale_annotations' field"
      );
      assert!(
          coverage.get("orphan_requirements").is_some(),
          "coverage must have 'orphan_requirements' field"
      );
      assert!(
          coverage.get("coverage_ratio").is_some(),
          "coverage must have 'coverage_ratio' field"
      );
  - name: test_cli_coverage_source_dir_flag
    indent_body: false
    body: |
      // Verifies that --source-dir limits annotation scanning scope.
      // Only the specified directory should be scanned for @spec annotations.
      let dir = TestDir::new();
      
      let spec_path = dir.write_file(
          "specs/api.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Some req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      
      let specs_full_path = spec_path.display().to_string();
      
      // Source in dir_a has the @spec annotation
      dir.write_file(
          "dir_a/lib.rs",
          &format!("// @spec {}#R1\npub fn foo() {{}}\n", specs_full_path),
      );
      // Source in dir_b has a different annotation
      dir.write_file(
          "dir_b/lib.rs",
          "// @spec projects/agentic-workflow/tech-design/core/specs/nonexistent.md#R99\npub fn bar() {}\n",
      );
      
      let specs_dir = dir.root.join("specs");
      
      // Scan only dir_a — should find the R1 annotation, making it covered
      let dir_a = dir.root.join("dir_a");
      let source_dirs_a: Vec<&std::path::Path> = vec![dir_a.as_path()];
      let result_a = spec_alignment::check_with_coverage(&specs_dir, &source_dirs_a);
      let report_a = result_a.coverage.as_ref().unwrap();
      assert_eq!(
          report_a.covered.len(),
          1,
          "R1 should be covered when scanning dir_a"
      );
      assert!(
          report_a.stale_annotations.is_empty(),
          "No stale annotations when scanning only dir_a"
      );
      
      // Scan only dir_b — should NOT find R1 annotation, R1 uncovered, and stale R99
      let dir_b = dir.root.join("dir_b");
      let source_dirs_b: Vec<&std::path::Path> = vec![dir_b.as_path()];
      let result_b = spec_alignment::check_with_coverage(&specs_dir, &source_dirs_b);
      let report_b = result_b.coverage.as_ref().unwrap();
      assert!(
          !report_b.uncovered_requirements.is_empty(),
          "R1 should be uncovered when scanning only dir_b"
      );
      assert!(
          !report_b.stale_annotations.is_empty(),
          "R99 annotation should be stale"
      );
  - name: test_cli_coverage_daemon_not_ready
    indent_body: false
    body: |
      // Verifies graceful degradation when daemon is not ready:
      // - unspecced_functions should be empty (skipped)
      // - schema_struct_mismatches should be empty (skipped)
      // - annotation coverage and requirement↔scenario checks still run
      let dir = TestDir::new();
      dir.write_file(
          "specs/api.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Some req    |
      | R2 | Orphan req  |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Only R1 referenced.
      "#,
      );
      
      // Public function with no @spec annotation
      dir.write_file("src/lib.rs", "pub fn unspecced_fn() {}\n");
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      let report = result.coverage.as_ref().unwrap();
      
      // Daemon is not running in test env — unspecced_functions should be empty (skipped)
      assert!(
          report.unspecced_functions.is_empty(),
          "unspecced_functions should be empty when daemon is not ready"
      );
      
      // Schema↔struct mismatches should be empty (skipped)
      assert!(
          report.schema_struct_mismatches.is_empty(),
          "schema_struct_mismatches should be empty when daemon is not ready"
      );
      
      // Annotation coverage still runs — R1 and R2 should be uncovered (no @spec)
      assert!(
          !report.uncovered_requirements.is_empty(),
          "Annotation coverage should still identify uncovered requirements"
      );
      
      // Requirement↔scenario still runs — R2 is orphan (not referenced by scenarios)
      assert!(
          !report.orphan_requirements.is_empty(),
          "Requirement↔scenario check should still detect orphan R2"
      );
      assert!(
          report
              .orphan_requirements
              .iter()
              .any(|o| o.requirement_id == "R2"),
          "R2 should be detected as orphan"
      );
  - name: test_unspecced_function_detected
    leading: |
          // =========================================================================
          // R16: Unspecced function detection — daemon-dependent stubs
          // =========================================================================
    attributes:
      - "#[test]"
      - "#[ignore = \"Requires daemon symbol index — stub returns empty until daemon integration\"]"
    indent_body: false
    body: |
      // When daemon is available, a public function `pub fn process_data()` without
      // a @spec annotation should appear in unspecced_functions.
      let dir = TestDir::new();
      dir.write_file(
          "specs/api.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Process req |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      dir.write_file("src/main.rs", "pub fn process_data() {}\n");
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      let report = result.coverage.as_ref().unwrap();
      
      assert!(
          report
              .unspecced_functions
              .iter()
              .any(|f| f.name == "process_data" && f.kind == "fn_item"),
          "process_data should be reported as unspecced function"
      );
  - name: test_unspecced_private_fn_ignored
    attributes:
      - "#[test]"
      - "#[ignore = \"Requires daemon symbol index — stub returns empty until daemon integration\"]"
    indent_body: false
    body: |
      // Private functions should NOT appear in unspecced_functions
      // (daemon index only reports public symbols).
      let dir = TestDir::new();
      dir.write_file(
          "specs/api.md",
          r#"## Requirements
      <!-- type: requirements lang: markdown -->
      
      | ID | Description |
      |----|-------------|
      | R1 | Some req    |
      
      ## Scenarios
      
      ### Scenario: Test R1
      
      Tests R1.
      "#,
      );
      dir.write_file("src/main.rs", "fn helper() {}\n");
      
      let specs_dir = dir.root.join("specs");
      let src_dir = dir.root.join("src");
      let source_dirs: Vec<&std::path::Path> = vec![src_dir.as_path()];
      
      let result = spec_alignment::check_with_coverage(&specs_dir, &source_dirs);
      let report = result.coverage.as_ref().unwrap();
      
      assert!(
          !report
              .unspecced_functions
              .iter()
              .any(|f| f.name == "helper"),
          "Private function 'helper' should not be in unspecced_functions"
      );
  - name: test_schema_struct_match
    leading: |
          // =========================================================================
          // R17: Schema↔struct validation — daemon-dependent stubs
          // =========================================================================
    attributes:
      - "#[test]"
      - "#[ignore = \"Requires daemon symbol index — stub returns empty until daemon integration\"]"
    indent_body: false
    body: |
      // When daemon is available, matching JSON Schema properties and Rust struct
      // fields should produce no violations.
      let dir = TestDir::new();
      dir.write_file(
          "specs/model.md",
          r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {
        "name": "MyModel",
        "properties": {
          "name": {"type": "string"},
          "count": {"type": "integer"}
        }
      }
      ```
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      // daemon_ready=true simulates daemon being available
      let (violations, mismatches) = spec_alignment::schema_struct::check(&specs_dir, true);
      
      // With daemon providing struct data, matching fields should produce no violations
      assert!(
          violations.is_empty(),
          "Matching schema and struct should produce no violations"
      );
      assert!(
          mismatches.is_empty(),
          "Matching schema and struct should produce no mismatches"
      );
  - name: test_schema_struct_missing_field
    attributes:
      - "#[test]"
      - "#[ignore = \"Requires daemon symbol index — stub returns empty until daemon integration\"]"
    indent_body: false
    body: |
      // When daemon is available, a JSON Schema with a property not present in the
      // Rust struct should produce a schema_struct_mismatch violation.
      let dir = TestDir::new();
      dir.write_file(
          "specs/model.md",
          r#"## Schema
      <!-- type: schema lang: json -->
      
      ```json
      {
        "name": "MyModel",
        "properties": {
          "name": {"type": "string"},
          "status": {"type": "string"}
        }
      }
      ```
      "#,
      );
      
      let specs_dir = dir.root.join("specs");
      let (violations, mismatches) = spec_alignment::schema_struct::check(&specs_dir, true);
      
      // With daemon: if struct lacks 'status' field, should produce mismatch
      assert!(
          mismatches
              .iter()
              .any(|m| m.field == "status" && m.kind == "missing_in_struct"),
          "Missing 'status' in struct should be reported as schema_struct_mismatch"
      );
      assert!(
          violations
              .iter()
              .any(|v| v.kind == ViolationKind::SchemaStructMismatch),
          "Should produce SchemaStructMismatch violation"
      );
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/tests/spec_alignment_phase2_tests.rs
    action: modify
    section: tests
    impl_mode: codegen
    generator: rust.tests
    description: |
      Emit the spec-alignment phase-2 test suite from the Rust tests template
      with module wrapper, ignored-test attributes, and raw fixture bodies.
  - action: annotate
    section: unit-test
    impl_mode: hand-written
    description: "Traceability metadata edge for the unit-test section."

```
