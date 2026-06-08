---
id: sdd-tools-validate-spec-execute-logic
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools validate spec execute logic

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/tools/validate_spec.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `definition` | projects/agentic-workflow/src/tools/validate_spec.rs | function | pub | 19 | definition() -> ToolDefinition |
| `execute` | projects/agentic-workflow/src/tools/validate_spec.rs | function | pub | 62 | execute(args: &Value, project_root: &Path) -> Result<String> |
## Source
<!-- type: source lang: rust -->

````rust
/// Execute the validate_spec_completeness tool
pub fn execute(args: &Value, project_root: &Path) -> Result<String> {
    let change_id = get_required_string(args, "change_id")?;
    let spec_id = get_required_string(args, "spec_id")?;

    // Read the spec file
    let spec_path = project_root
        .join(".aw/changes")
        .join(&change_id)
        .join("specs")
        .join(format!("{}.md", spec_id));

    if !spec_path.exists() {
        anyhow::bail!("Spec file not found: {}", spec_path.display());
    }

    let content = std::fs::read_to_string(&spec_path)?;

    // Parse the spec content
    let result = validate_spec_content(&content)?;

    // Build output
    let coverage = if result.requirements_count > 0 {
        let coverage_pct =
            (result.scenarios_count as f64 / result.requirements_count as f64 * 100.0).min(100.0);
        coverage_pct
    } else {
        0.0
    };

    let output = json!({
        "is_complete": result.is_complete,
        "missing_elements": result.missing_elements,
        "warnings": result.warnings,
        "coverage": {
            "requirements_count": result.requirements_count,
            "scenarios_count": result.scenarios_count,
            "requirements_with_scenarios_percent": coverage,
            "diagrams_count": result.diagrams_count,
            "has_api_spec": result.has_api_spec
        },
        "spec_type": result.spec_type
    });

    Ok(serde_json::to_string_pretty(&output)?)
}

/// Check if content contains a specific diagram type
fn has_diagram_type(content: &str, diagram_type: &DiagramType) -> bool {
    match diagram_type {
        DiagramType::Sequence => content.contains("sequenceDiagram"),
        DiagramType::Erd => content.contains("erDiagram"),
        DiagramType::Class => content.contains("classDiagram"),
        DiagramType::Flowchart => content.contains("flowchart") || content.contains("graph "),
        DiagramType::State => content.contains("stateDiagram"),
        DiagramType::MindMap => content.contains("mindmap"),
        DiagramType::Requirement => content.contains("requirementDiagram"),
        DiagramType::Journey => content.contains("journey"),
    }
}

/// Check if content contains a specific API spec type
fn has_api_spec_type(content: &str, api_spec_type: &ApiSpecType) -> bool {
    match api_spec_type {
        ApiSpecType::OpenApi31 => {
            content.contains("```yaml\nopenapi:")
                || content.contains("openapi: 3.1")
                || content.contains("openapi: \"3.1")
                || content.contains("\"openapi\": \"3.1")
        }
        ApiSpecType::AsyncApi26 => {
            content.contains("```yaml\nasyncapi:")
                || content.contains("asyncapi: 2.6")
                || content.contains("asyncapi: \"2.6")
                || content.contains("\"asyncapi\": \"2.6")
        }
        ApiSpecType::JsonSchema => content.contains("\"$schema\"") || content.contains("$schema:"),
        ApiSpecType::OpenRpc13 => {
            content.contains("openrpc: 1.3")
                || content.contains("openrpc: \"1.3")
                || content.contains("\"openrpc\": \"1.3")
        }
        ApiSpecType::ServerlessWorkflow08 => {
            content.contains("specVersion: 0.8")
                || content.contains("specVersion: \"0.8")
                || content.contains("specVersion: '0.8")
                || content.contains("\"specVersion\": \"0.8")
        }
    }
}

/// Validate spec content using the central SpecType enum for rules
fn validate_spec_content(content: &str) -> Result<ValidationResult> {
    let mut missing_elements = Vec::new();
    let mut warnings = Vec::new();
    let mut requirements_count = 0;
    let mut scenarios_count = 0;
    let mut spec_type_str: Option<String> = None;

    // Parse frontmatter
    if let Some(fm_end) = content
        .find("---\n")
        .and_then(|start| content[start + 4..].find("---").map(|end| start + 4 + end))
    {
        let frontmatter = &content[4..fm_end];

        // Extract spec_type from frontmatter
        for line in frontmatter.lines() {
            if line.starts_with("spec_type:") {
                spec_type_str = Some(line.trim_start_matches("spec_type:").trim().to_string());
            }
        }
    }

    // Count requirements (lines starting with ### R followed by a number)
    for line in content.lines() {
        if line.starts_with("### R")
            && line
                .chars()
                .nth(5)
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
        {
            requirements_count += 1;
        }
    }

    // Count scenarios (### Scenario: headers)
    for line in content.lines() {
        if line.starts_with("### Scenario:") {
            scenarios_count += 1;
        }
    }

    // Count diagrams (```mermaid blocks)
    let diagrams_count = content.matches("```mermaid").count();

    // Check for any API spec in content (for has_api_spec field)
    let has_any_api_spec = has_api_spec_type(content, &ApiSpecType::OpenApi31)
        || has_api_spec_type(content, &ApiSpecType::AsyncApi26)
        || has_api_spec_type(content, &ApiSpecType::OpenRpc13)
        || has_api_spec_type(content, &ApiSpecType::ServerlessWorkflow08)
        || has_api_spec_type(content, &ApiSpecType::JsonSchema)
        || content.contains("## API Specification");

    // Validate based on spec_type using the central SpecType enum
    if let Some(ref st_str) = spec_type_str {
        match SpecType::from_str(st_str) {
            Ok(spec_type) => {
                // Get required diagrams from SpecType
                let required_diagrams = spec_type.required_diagrams();
                if !required_diagrams.is_empty() {
                    // For types with multiple options (data-model, algorithm, workflow), any one is OK
                    let has_any_required = required_diagrams
                        .iter()
                        .any(|dt| has_diagram_type(content, dt));

                    if !has_any_required {
                        // Format diagram name for user-friendly message
                        let format_diagram_name = |dt: &DiagramType| -> &str {
                            match dt {
                                DiagramType::Sequence => "Sequence",
                                DiagramType::Erd => "ERD",
                                DiagramType::Class => "class",
                                DiagramType::Flowchart => "Flowchart",
                                DiagramType::State => "state",
                                DiagramType::MindMap => "mind map",
                                DiagramType::Requirement => "requirement",
                                DiagramType::Journey => "journey",
                            }
                        };

                        let diagram_names: Vec<&str> =
                            required_diagrams.iter().map(format_diagram_name).collect();
                        let msg = if required_diagrams.len() > 1 {
                            format!(
                                "{} diagram required for {} spec",
                                diagram_names.join(" or "),
                                st_str
                            )
                        } else {
                            format!("{} diagram required for {} spec", diagram_names[0], st_str)
                        };
                        missing_elements.push(msg);
                    }
                }

                // Get required API spec from SpecType
                if let Some(required_api) = spec_type.required_api_spec() {
                    if !has_api_spec_type(content, &required_api) {
                        // Format API spec name for user-friendly message
                        let api_name = match required_api {
                            ApiSpecType::OpenApi31 => "OpenAPI 3.1",
                            ApiSpecType::AsyncApi26 => "AsyncAPI 2.6",
                            ApiSpecType::JsonSchema => "JSON Schema",
                            ApiSpecType::OpenRpc13 => "OpenRPC 1.3",
                            ApiSpecType::ServerlessWorkflow08 => "Serverless Workflow 0.8",
                        };
                        missing_elements.push(format!(
                            "{} specification required for {} spec",
                            api_name, st_str
                        ));
                    }
                }

                // Note: Integration's sequence diagram requirement is handled by
                // required_diagrams() above, same as other spec types
            }
            Err(_) => {
                warnings.push(format!("Unknown spec_type: {}", st_str));
            }
        }
    } else {
        warnings.push("No spec_type specified - unable to validate required elements".to_string());
    }

    // General validations
    if requirements_count == 0 {
        missing_elements.push("At least one requirement is required".to_string());
    }

    if scenarios_count == 0 {
        missing_elements.push("At least one acceptance scenario is required".to_string());
    }

    // Coverage warnings
    if requirements_count > 0 && scenarios_count < requirements_count {
        warnings.push(format!(
            "Only {} scenarios for {} requirements - consider adding more scenarios",
            scenarios_count, requirements_count
        ));
    }

    // Diagram warnings
    if diagrams_count == 0 {
        warnings.push("No diagrams found - consider adding visual documentation".to_string());
    }

    // Check for semantic annotations in flowcharts (for code generation readiness)
    let has_flowchart = content.contains("flowchart") || content.contains("graph ");
    if has_flowchart && !content.contains("semantic:") && !content.contains("\"semantic\"") {
        warnings.push("Flowchart lacks semantic annotations for code generation".to_string());
    }

    let is_complete = missing_elements.is_empty();

    Ok(ValidationResult {
        is_complete,
        missing_elements,
        warnings,
        requirements_count,
        scenarios_count,
        diagrams_count,
        has_api_spec: has_any_api_spec,
        spec_type: spec_type_str,
    })
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/tools/validate_spec.rs
    action: modify
    section: source
    impl_mode: codegen
    replaces:
      - "execute"
      - "has_diagram_type"
      - "has_api_spec_type"
      - "validate_spec_content"
    description: "Validate-spec execution and content validation helper logic."
```
