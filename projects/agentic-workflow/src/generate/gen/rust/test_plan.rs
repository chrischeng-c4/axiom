// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#source
// CODEGEN-BEGIN
//! Unit-test documentation generator.
//!
//! Reads unit-test frontmatter (requirementDiagram elements + verifies relationships)
//! and generates `#[test]` function stubs with `assert_verifies_req!` macro.
//!
//! Cross-links with scenarios: if a test T1 verifies R3 and a scenario S1 also
//! verifies R3, the scenario's GWT comments are inlined in the test stub body.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R2
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R4

use serde_yaml::Value;

use crate::generate::marker::{emit_spec_ref, Lang};

/// A test element parsed from a unit-test frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#schema
#[derive(Debug, Clone)]
pub struct TestElement {
    /// Element identifier.
    pub id: String,
    /// Element type.
    pub element_type: String,
    /// Requirements verified by this element.
    pub verifies: Vec<String>,
}

/// A scenario entry for cross-linking (given/when/then).
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#schema
#[derive(Debug, Clone)]
pub struct ScenarioRef {
    /// Scenario identifier.
    pub id: String,
    /// Requirements verified by this scenario.
    pub verifies: Vec<String>,
    /// Given clause.
    pub given: String,
    /// When clause.
    pub when: String,
    /// Then clause.
    pub then: String,
}

/// Output from test plan code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#schema
#[derive(Debug, Clone)]
pub struct TestPlanGenOutput {
    /// Generated Rust test function stubs.
    pub code: String,
    /// Number of test stubs generated.
    pub test_count: usize,
}

/// A test row parsed from a markdown unit-test table.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#schema
#[derive(Debug, Clone)]
pub struct MarkdownTest {
    /// Test identifier.
    pub id: String,
    /// Test name.
    pub name: String,
    /// What this test covers.
    pub covers: String,
    /// The assertion expression.
    pub assertion: String,
}

/// Output from markdown-table unit-test code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#schema
#[derive(Debug, Clone)]
pub struct MarkdownTestPlanOutput {
    /// Generated Rust test code.
    pub code: String,
    /// Number of tests generated.
    pub test_count: usize,
    /// Spec refs found in the test plan.
    pub spec_refs: Vec<String>,
}

/// Generate `#[test]` function stubs from a unit-test YAML frontmatter value.
///
/// Each test element becomes a `#[test] fn` stub with `assert_verifies_req!` macro.
/// Scenarios verifying the same requirements are inlined as GWT comments.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R2
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R4
pub fn generate_test_plan(
    test_plan_yaml: &Value,
    scenarios: &[ScenarioRef],
    spec_path: &str,
) -> TestPlanGenOutput {
    let elements = parse_test_elements(test_plan_yaml);
    let mut lines = Vec::new();

    // Module wrapper for tests
    lines.push("#[cfg(test)]".to_string());
    lines.push("mod generated_tests {".to_string());
    lines.push("    use super::*;".to_string());
    lines.push(String::new());

    for element in &elements {
        let fn_name = to_test_fn_name(&element.id);

        // Find scenarios that cross-link with this test's requirements
        let linked_scenarios: Vec<&ScenarioRef> = scenarios
            .iter()
            .filter(|s| s.verifies.iter().any(|r| element.verifies.contains(r)))
            .collect();

        // Doc comment listing verified requirements
        lines.push(format!(
            "    /// Test: {} — verifies {:?}",
            element.id, element.verifies
        ));
        lines.push(format!("    /// Spec: {}#{}", spec_path, element.id));

        lines.push("    #[test]".to_string());
        lines.push(format!("    fn {}() {{", fn_name));

        // assert_verifies_req! macro for each verified requirement
        for req_id in &element.verifies {
            lines.push(format!("        // assert_verifies_req!({});", req_id));
        }

        // Inline linked scenario GWT comments (R4)
        for scenario in &linked_scenarios {
            lines.push(format!(
                "        // Scenario {}: {}",
                scenario.id,
                scenario.verifies.join(", ")
            ));
            lines.push(format!("        // Given: {}", scenario.given));
            lines.push(format!("        // When: {}", scenario.when));
            lines.push(format!("        // Then: {}", scenario.then));
        }

        lines.push("        todo!(\"Implement test\")".to_string());
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    lines.push("}".to_string());

    TestPlanGenOutput {
        code: lines.join("\n"),
        test_count: elements.len(),
    }
}

fn parse_test_elements(yaml: &Value) -> Vec<TestElement> {
    let mut elements = Vec::new();

    // Look for elements in unit-test frontmatter
    if let Some(elems) = yaml.get("elements").and_then(|v| v.as_mapping()) {
        let mut elem_ids: Vec<&str> = elems.keys().filter_map(|k| k.as_str()).collect();
        elem_ids.sort();

        // Build a map of which requirements each element verifies
        let verifies_map = build_verifies_map(yaml);

        for elem_id in elem_ids {
            if let Some(elem) = elems.get(elem_id) {
                let element_type = elem
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Test")
                    .to_string();
                let verifies = verifies_map.get(elem_id).cloned().unwrap_or_default();
                elements.push(TestElement {
                    id: elem_id.to_string(),
                    element_type,
                    verifies,
                });
            }
        }
    }

    elements
}

fn build_verifies_map(yaml: &Value) -> std::collections::HashMap<String, Vec<String>> {
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    if let Some(rels) = yaml.get("relationships").and_then(|v| v.as_sequence()) {
        for rel in rels {
            let from = rel.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = rel.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let kind = rel
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("verifies");
            if kind == "verifies" {
                map.entry(from.to_string())
                    .or_default()
                    .push(to.to_string());
            }
        }
    }

    map
}

fn to_test_fn_name(id: &str) -> String {
    format!("test_{}", id.to_lowercase().replace('-', "_"))
}

// ---------------------------------------------------------------------------
// Markdown-table unit-test generator
// ---------------------------------------------------------------------------

/// Parse all `## Unit Test` tables and generate `#[cfg(test)] mod tests { ... }`.
/// Legacy `## Test Plan` headings are accepted during migration.
///
/// Table format:
/// ```markdown
/// | ID | Test | Covers | Assertion |
/// |----|------|--------|-----------|
/// | T1 | `name` | ... | assertion text |
/// ```
///
/// Each row becomes a `#[test] fn name() { /* SPEC-REF */ todo!(assertion) }`.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#source
pub fn generate_test_plan_from_markdown(
    spec_content: &str,
    spec_path: &str,
) -> Option<MarkdownTestPlanOutput> {
    let tests = parse_markdown_test_plan(spec_content);
    if tests.is_empty() {
        return None;
    }

    let mut lines = Vec::new();
    let mut spec_refs = Vec::new();

    lines.push("#[cfg(test)]".to_string());
    lines.push("mod tests {".to_string());
    lines.push("    use super::*;".to_string());
    lines.push(String::new());

    for t in &tests {
        let section_id = format!("unit-test-{}", t.id.to_lowercase());
        let marker = emit_spec_ref(spec_path, &section_id, &t.assertion, Lang::Rust);
        spec_refs.push(format!("{}#{}", spec_path, section_id));

        lines.push(format!("    /// {} — {}", t.id, t.covers));
        lines.push("    #[test]".to_string());
        lines.push(format!("    fn {}() {{", t.name));
        for marker_line in marker.lines() {
            lines.push(format!("        {}", marker_line));
        }
        lines.push(format!("        todo!({:?})", t.assertion));
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    lines.push("}".to_string());

    Some(MarkdownTestPlanOutput {
        code: lines.join("\n"),
        test_count: tests.len(),
        spec_refs,
    })
}

/// Parse Mermaid Plus `unit-test` requirementDiagram frontmatter and generate
/// deterministic Rust test stubs. Legacy `test-plan` annotations are accepted.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/test_plan.md#source
pub fn generate_unit_tests_from_mermaid(
    spec_content: &str,
    spec_path: &str,
) -> Option<TestPlanGenOutput> {
    let block = crate::generate::frontmatter::extract_mermaid_plus_blocks(spec_content)
        .into_iter()
        .find(|block| {
            matches!(
                block.section_type.as_deref(),
                Some("unit-test" | "test-plan")
            ) || block
                .section_heading
                .as_deref()
                .map(|heading| {
                    let h = heading.to_ascii_lowercase();
                    h.starts_with("unit test") || h.starts_with("test plan")
                })
                .unwrap_or(false)
        })?;
    let output = generate_test_plan(&block.frontmatter, &[], spec_path);
    if output.test_count == 0 {
        None
    } else {
        Some(output)
    }
}

/// Parse pipe-tables under the `## Unit Test` or legacy `## Test Plan` section.
fn parse_markdown_test_plan(spec_content: &str) -> Vec<MarkdownTest> {
    let mut tests = Vec::new();
    let mut in_unit_test = false;

    for line in spec_content.lines() {
        let trimmed = line.trim();

        if let Some(rest) = trimmed.strip_prefix("## ") {
            let heading = rest.trim().to_ascii_lowercase();
            in_unit_test = heading.starts_with("unit test") || heading.starts_with("test plan");
            continue;
        }

        if !in_unit_test || !trimmed.starts_with('|') {
            continue;
        }

        let cells: Vec<&str> = trimmed
            .trim_start_matches('|')
            .trim_end_matches('|')
            .split('|')
            .map(|s| s.trim())
            .collect();
        if cells.len() != 4 {
            continue;
        }

        // Skip separator row (e.g., |---|---|---|---|)
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':' || ch == ' '))
        {
            continue;
        }

        // Skip header row
        let is_header =
            cells[0].eq_ignore_ascii_case("ID") && cells[1].eq_ignore_ascii_case("Test");
        if is_header {
            continue;
        }

        let id = cells[0].to_string();
        // ID should start with alphanumeric (e.g. T1, T20)
        if !id
            .chars()
            .next()
            .map_or(false, |c| c.is_ascii_alphanumeric())
        {
            continue;
        }

        // Test name: strip surrounding backticks
        let name = cells[1].trim_matches('`').to_string();
        let first_char = name.chars().next();
        let is_valid_ident = first_char.map_or(false, |c| c.is_ascii_alphabetic() || c == '_')
            && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
        if !is_valid_ident {
            continue;
        }

        tests.push(MarkdownTest {
            id,
            name,
            covers: cells[2].to_string(),
            assertion: cells[3].to_string(),
        });
    }

    tests
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R2
    #[test]
    fn test_generates_test_stubs_from_unit_test() {
        let yaml_str = r#"
elements:
  T1:
    type: Test
  T2:
    type: Test
relationships:
  - from: T1
    to: R1
    kind: verifies
  - from: T2
    to: R2
    kind: verifies
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_test_plan(&yaml, &[], "spec.md");

        assert!(
            output.code.contains("#[test]"),
            "Should generate test functions"
        );
        assert!(
            output.code.contains("fn test_t1()"),
            "Should generate test_t1"
        );
        assert!(
            output.code.contains("fn test_t2()"),
            "Should generate test_t2"
        );
        assert_eq!(output.test_count, 2, "Should generate 2 tests");
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R2
    #[test]
    fn test_test_stub_contains_req_assertion() {
        let yaml_str = r#"
elements:
  T1:
    type: Test
relationships:
  - from: T1
    to: R1
    kind: verifies
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_test_plan(&yaml, &[], "spec.md");

        assert!(
            output.code.contains("assert_verifies_req!"),
            "Should have assert macro"
        );
        assert!(output.code.contains("R1"), "Should reference R1");
    }

    #[test]
    fn test_markdown_table_unit_test_parses_rows() {
        let spec = r#"
## Unit Test

<!-- type: unit-test lang: markdown -->

Intro paragraph.

### Enum

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| T1 | `default_is_pending` | Default impl | `TaskState::default() == Pending` |
| T2 | `serde_round_trip` | serde | round-trip all variants |

### More

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| T3 | `terminal_states` | is_terminal | Success.is_terminal() == true |
"#;
        let output = generate_test_plan_from_markdown(spec, "spec.md").unwrap();
        assert_eq!(output.test_count, 3, "should parse 3 test rows");
        assert!(
            output.code.contains("fn default_is_pending"),
            "fn name from cell 2"
        );
        assert!(output.code.contains("fn serde_round_trip"));
        assert!(output.code.contains("fn terminal_states"));
        assert!(
            output.code.contains("SPEC-REF"),
            "each test should have SPEC-REF"
        );
        assert!(output.code.contains("#[cfg(test)]"));
        assert!(output.code.contains("mod tests"));
    }

    #[test]
    fn test_markdown_table_skips_non_unit_test_sections() {
        let spec = r#"
## Changes

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| X1 | `should_not_emit` | n/a | should be ignored |

## State Machine

Just a heading, no tests here.
"#;
        assert!(generate_test_plan_from_markdown(spec, "spec.md").is_none());
    }

    #[test]
    fn test_markdown_table_rejects_invalid_idents() {
        let spec = r#"
## Unit Test

| ID | Test | Covers | Assertion |
|----|------|--------|-----------|
| T1 | `123_bad` | invalid ident starting with digit | skipped |
| T2 | `good_one` | valid | kept |
"#;
        let out = generate_test_plan_from_markdown(spec, "spec.md").unwrap();
        assert_eq!(out.test_count, 1, "only good_one should emit");
        assert!(out.code.contains("fn good_one"));
        assert!(!out.code.contains("fn 123_bad"));
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R4
    #[test]
    fn test_inlines_gwt_from_linked_scenario() {
        let yaml_str = r#"
elements:
  T1:
    type: Test
relationships:
  - from: T1
    to: R1
    kind: verifies
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();

        let scenarios = vec![ScenarioRef {
            id: "S1".to_string(),
            verifies: vec!["R1".to_string()],
            given: "A state-machine spec with 3 states".to_string(),
            when: "score gen apply runs on the spec".to_string(),
            then: "Rust enum is generated with 3 variants".to_string(),
        }];

        let output = generate_test_plan(&yaml, &scenarios, "spec.md");

        // R4: GWT from S1 should be inlined since both T1 and S1 verify R1
        assert!(
            output.code.contains("Scenario S1"),
            "Should inline S1 scenario"
        );
        assert!(output.code.contains("Given:"), "Should inline Given");
        assert!(output.code.contains("When:"), "Should inline When");
        assert!(output.code.contains("Then:"), "Should inline Then");
    }
}

// CODEGEN-END
