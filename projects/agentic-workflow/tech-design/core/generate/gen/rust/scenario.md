---
id: sdd-gen-rust-scenario-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Scenario Generator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/gen/rust/scenario.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ScenarioDef` | projects/agentic-workflow/src/generate/gen/rust/scenario.rs | struct | pub | 18 |  |
| `ScenarioGenOutput` | projects/agentic-workflow/src/generate/gen/rust/scenario.rs | struct | pub | 30 |  |
| `generate_scenarios` | projects/agentic-workflow/src/generate/gen/rust/scenario.rs | function | pub | 41 | generate_scenarios(scenarios_yaml: &Value, spec_path: &str) -> ScenarioGenOutput |
| `parse_scenarios` | projects/agentic-workflow/src/generate/gen/rust/scenario.rs | function | pub | 95 | parse_scenarios(yaml: &Value) -> Vec<ScenarioDef> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ScenarioDef:
    type: object
    required: [id, given, when, then, verifies]
    description: Parsed scenario from YAML frontmatter.
    properties:
      id: { type: string }
      name: { type: string }
      given: { type: string }
      when: { type: string }
      then: { type: string }
      verifies:
        type: array
        items: { type: string }
    x-rust-struct:
      derive: [Debug, Clone]

  ScenarioGenOutput:
    type: object
    required: [code, scenario_count, scenarios]
    description: Output from scenarios code generation.
    properties:
      code: { type: string }
      scenario_count:
        type: integer
        x-rust-type: usize
      scenarios:
        type: array
        items: { $ref: "#/definitions/ScenarioDef" }
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/gen/rust/scenario.rs -->
```rust
//! Scenarios documentation generator.
//!
//! Reads scenarios frontmatter YAML (given/when/then format) and generates
//! `#[tokio::test]` function stubs with structured GWT comments.
//!
//! Body is `todo!()` with full GWT context as comments — enables LLM to
//! complete the test with minimal gap.

// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R3

use serde_yaml::Value;

/// Parsed scenario from YAML frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/scenario.md#schema
#[derive(Debug, Clone)]
pub struct ScenarioDef {
    pub id: String,
    pub name: Option<String>,
    pub given: String,
    pub when: String,
    pub then: String,
    pub verifies: Vec<String>,
}

/// Output from scenarios code generation.
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/scenario.md#schema
#[derive(Debug, Clone)]
pub struct ScenarioGenOutput {
    pub code: String,
    pub scenario_count: usize,
    pub scenarios: Vec<ScenarioDef>,
}

/// Generate `#[tokio::test]` stubs from a scenarios YAML frontmatter.
///
/// Each scenario becomes an integration test with GWT structured as comments.
/// The body is `todo!()` — the LLM has full GWT context to complete it.
// @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R3
pub fn generate_scenarios(scenarios_yaml: &Value, spec_path: &str) -> ScenarioGenOutput {
    let scenarios = parse_scenarios(scenarios_yaml);
    let mut lines = Vec::new();

    // Module wrapper
    lines.push("#[cfg(test)]".to_string());
    lines.push("mod generated_scenarios {".to_string());
    lines.push("    use super::*;".to_string());
    lines.push(String::new());

    for scenario in &scenarios {
        let fn_name = to_scenario_fn_name(&scenario.id);

        // Header doc comment
        if let Some(name) = &scenario.name {
            lines.push(format!("    /// Scenario {}: {}", scenario.id, name));
        } else {
            lines.push(format!("    /// Scenario {}", scenario.id));
        }
        lines.push(format!("    /// Spec: {}#{}", spec_path, scenario.id));

        if !scenario.verifies.is_empty() {
            lines.push(format!("    /// Verifies: {:?}", scenario.verifies));
        }

        // Async test for integration scenarios
        lines.push("    #[tokio::test]".to_string());
        lines.push(format!("    async fn {}() {{", fn_name));

        // GWT structured comments
        lines.push(format!("        // GIVEN: {}", scenario.given));
        lines.push(String::new());
        lines.push(format!("        // WHEN: {}", scenario.when));
        lines.push(String::new());
        lines.push(format!("        // THEN: {}", scenario.then));
        lines.push(String::new());
        lines.push("        todo!(\"Implement scenario test\")".to_string());
        lines.push("    }".to_string());
        lines.push(String::new());
    }

    lines.push("}".to_string());

    ScenarioGenOutput {
        code: lines.join("\n"),
        scenario_count: scenarios.len(),
        scenarios,
    }
}

/// Parse scenarios from YAML frontmatter.
///
/// Accepts either a sequence or a map (keyed by scenario ID).
/// @spec projects/agentic-workflow/tech-design/core/generate/gen/rust/scenario.md#source
pub fn parse_scenarios(yaml: &Value) -> Vec<ScenarioDef> {
    let mut result = Vec::new();

    // Try sequence format: [{id: S1, given: ..., when: ..., then: ...}]
    if let Some(seq) = yaml.get("scenarios").and_then(|v| v.as_sequence()) {
        for item in seq {
            if let Some(scenario) = parse_scenario_item(item) {
                result.push(scenario);
            }
        }
        return result;
    }

    // Try map format: {S1: {given: ..., when: ..., then: ...}}
    if let Some(map) = yaml.get("scenarios").and_then(|v| v.as_mapping()) {
        let mut ids: Vec<&str> = map.keys().filter_map(|k| k.as_str()).collect();
        ids.sort();
        for id in ids {
            if let Some(item) = map.get(id) {
                let given = item
                    .get("given")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let when = item
                    .get("when")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let then = item
                    .get("then")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let name = item
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let verifies = parse_verifies(item);

                result.push(ScenarioDef {
                    id: id.to_string(),
                    name,
                    given,
                    when,
                    then,
                    verifies,
                });
            }
        }
    }

    result
}

fn parse_scenario_item(item: &Value) -> Option<ScenarioDef> {
    let id = item.get("id")?.as_str()?.to_string();
    let given = item
        .get("given")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let when = item
        .get("when")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let then = item
        .get("then")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let name = item
        .get("name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let verifies = parse_verifies(item);

    Some(ScenarioDef {
        id,
        name,
        given,
        when,
        then,
        verifies,
    })
}

fn parse_verifies(item: &Value) -> Vec<String> {
    item.get("verifies")
        .and_then(|v| v.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn to_scenario_fn_name(id: &str) -> String {
    format!("scenario_{}", id.to_lowercase().replace('-', "_"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R3
    #[test]
    fn test_generates_async_test_stubs_from_scenarios() {
        let yaml_str = r#"
scenarios:
  S1:
    name: State machine enum generation
    given: State-machine spec with 3 nodes
    when: score gen apply runs on the spec
    then: Rust enum generated with 3 variants
    verifies: [R1, R5]
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_scenarios(&yaml, "spec.md");

        assert!(
            output.code.contains("#[tokio::test]"),
            "Should generate async test"
        );
        assert!(
            output.code.contains("async fn scenario_s1()"),
            "Should generate scenario_s1"
        );
        assert_eq!(output.scenario_count, 1, "Should generate 1 scenario");
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R3
    #[test]
    fn test_scenario_contains_gwt_comments() {
        let yaml_str = r#"
scenarios:
  S2:
    given: Interaction spec with 2 actors
    when: score gen apply runs
    then: Method signatures generated
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_scenarios(&yaml, "spec.md");

        assert!(output.code.contains("GIVEN:"), "Should have GIVEN comment");
        assert!(output.code.contains("WHEN:"), "Should have WHEN comment");
        assert!(output.code.contains("THEN:"), "Should have THEN comment");
        assert!(
            output.code.contains("2 actors"),
            "Should contain given description"
        );
    }

    // @spec projects/agentic-workflow/tech-design/core/logic/codegen-documentation.md#R3
    #[test]
    fn test_scenario_body_is_todo() {
        let yaml_str = r#"
scenarios:
  - id: S3
    given: something
    when: something happens
    then: result is expected
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let output = generate_scenarios(&yaml, "spec.md");

        assert!(output.code.contains("todo!"), "Body should be todo!");
    }

    #[test]
    fn test_parse_scenarios_sequence_format() {
        let yaml_str = r#"
scenarios:
  - id: S1
    given: A state
    when: An event
    then: A result
    verifies: [R1]
  - id: S2
    given: Another state
    when: Another event
    then: Another result
"#;
        let yaml: serde_yaml::Value = serde_yaml::from_str(yaml_str).unwrap();
        let scenarios = parse_scenarios(&yaml);

        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].id, "S1");
        assert_eq!(scenarios[0].verifies, vec!["R1"]);
        assert_eq!(scenarios[1].id, "S2");
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/gen/rust/scenario.rs
    action: modify
    section: source
    impl_mode: codegen
    description: Source template owns the complete scenario generator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- ok.
