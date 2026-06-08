// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/validate_spec/preamble-definition.md#source
// CODEGEN-BEGIN
//! validate_spec_completeness MCP Tool
//!
//! Validates that a spec has all required elements for code generation.
#![allow(deprecated)]
//! Checks for required diagrams, API specs, and coverage of requirements.
//! Uses the central SpecType enum from models/spec_rules.rs for consistency.

use super::{get_required_string, ToolDefinition};
use crate::models::spec_rules::{ApiSpecType, DiagramType, SpecType};
use crate::Result;
use serde_json::{json, Value};
use std::path::Path;
use std::str::FromStr;

/// Get the tool definition for validate_spec_completeness
/// @spec projects/agentic-workflow/tech-design/core/tools/validate_spec/preamble-definition.md#source
pub fn definition() -> ToolDefinition {
    ToolDefinition {
        name: "sdd_validate_spec_completeness".to_string(),
        description: "Validate that a spec has all required elements for code generation. Checks for required diagrams, API specs based on spec_type, and coverage of requirements with scenarios.".to_string(),
        input_schema: json!({
            "type": "object",
            "required": ["project_path", "change_id", "spec_id"],
            "properties": {
                "project_path": {
                    "type": "string",
                    "description": "Project root path (use $PWD for current directory)"
                },
                "change_id": {
                    "type": "string",
                    "description": "The change ID containing the spec"
                },
                "spec_id": {
                    "type": "string",
                    "description": "The spec ID to validate"
                }
            }
        }),
    }
}

/// Validation result
#[derive(Debug)]
struct ValidationResult {
    is_complete: bool,
    missing_elements: Vec<String>,
    warnings: Vec<String>,
    requirements_count: usize,
    scenarios_count: usize,
    diagrams_count: usize,
    has_api_spec: bool,
    spec_type: Option<String>,
}
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/validate_spec/execute-logic.md#source
// CODEGEN-BEGIN
/// Execute the validate_spec_completeness tool
/// @spec projects/agentic-workflow/tech-design/core/tools/validate_spec/execute-logic.md#source
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
// CODEGEN-END

// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/tools/validate_spec/tests.md#source
// CODEGEN-BEGIN
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_spec(content: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();

        // Create spec directory
        let spec_dir = project_root.join(".aw/changes/test-change/specs");
        std::fs::create_dir_all(&spec_dir).unwrap();

        // Write spec file
        std::fs::write(spec_dir.join("test-spec.md"), content).unwrap();

        temp_dir
    }

    #[test]
    fn test_validate_complete_http_api_spec() {
        let content = r#"---
id: test-spec
title: Test API Spec
spec_type: http-api
---

## Overview

This is a test API spec.

## Requirements

### R1: Create User
Create a new user in the system.

## Acceptance Criteria

### Scenario: Create user successfully
**GIVEN** valid user data
**WHEN** POST /users is called
**THEN** user is created

## Flow Diagram

```mermaid
sequenceDiagram
    Client->>Server: POST /users
    Server-->>Client: 201 Created
```

## API Specification

```yaml
openapi: 3.1.0
info:
  title: Test API
  version: 1.0.0
paths:
  /users:
    post:
      summary: Create user
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(output["is_complete"].as_bool().unwrap());
        assert!(output["missing_elements"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_validate_incomplete_http_api_spec() {
        let content = r#"---
id: test-spec
title: Test API Spec
spec_type: http-api
---

## Overview

This is a test API spec.

## Requirements

### R1: Create User
Create a new user in the system.

## Acceptance Criteria

### Scenario: Create user successfully
**GIVEN** valid user data
**WHEN** POST /users is called
**THEN** user is created
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(!output["is_complete"].as_bool().unwrap());
        let missing = output["missing_elements"].as_array().unwrap();
        assert!(missing
            .iter()
            .any(|m| m.as_str().unwrap().contains("Sequence diagram")));
        assert!(missing
            .iter()
            .any(|m| m.as_str().unwrap().contains("OpenAPI")));
    }

    #[test]
    fn test_validate_data_model_spec() {
        let content = r#"---
id: test-spec
title: Data Model Spec
spec_type: data-model
---

## Overview

This is a data model spec.

## Requirements

### R1: User Entity
Define the User entity.

## Acceptance Criteria

### Scenario: User has required fields
**WHEN** User is created
**THEN** it has id, name, email

## Entity Diagram

```mermaid
erDiagram
    USER {
        int id
        string name
        string email
    }
```

## Data Model

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "User",
  "type": "object",
  "properties": {
    "id": { "type": "integer" },
    "name": { "type": "string" },
    "email": { "type": "string" }
  }
}
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(output["is_complete"].as_bool().unwrap());
    }

    #[test]
    fn test_validate_spec_coverage() {
        let content = r#"---
id: test-spec
title: Test Spec
spec_type: utility
---

## Overview

This is a test spec.

## Requirements

### R1: First requirement
Description.

### R2: Second requirement
Description.

### R3: Third requirement
Description.

## Acceptance Criteria

### Scenario: First scenario
**WHEN** something happens
**THEN** result occurs
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        // Should be complete (utility has no required diagrams)
        assert!(output["is_complete"].as_bool().unwrap());

        // But should have coverage warning
        let warnings = output["warnings"].as_array().unwrap();
        assert!(warnings.iter().any(|w| w
            .as_str()
            .unwrap()
            .contains("Only 1 scenarios for 3 requirements")));

        // Check coverage stats
        let coverage = &output["coverage"];
        assert_eq!(coverage["requirements_count"].as_i64().unwrap(), 3);
        assert_eq!(coverage["scenarios_count"].as_i64().unwrap(), 1);
    }

    #[test]
    fn test_validate_spec_without_type() {
        let content = r#"---
id: test-spec
title: Test Spec
---

## Overview

This is a test spec.

## Requirements

### R1: Requirement
Description.

## Acceptance Criteria

### Scenario: Test scenario
**WHEN** something happens
**THEN** result occurs
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        // Should be complete (no spec_type means no required elements)
        assert!(output["is_complete"].as_bool().unwrap());

        // But should have warning about missing spec_type
        let warnings = output["warnings"].as_array().unwrap();
        assert!(warnings
            .iter()
            .any(|w| w.as_str().unwrap().contains("No spec_type specified")));
    }

    #[test]
    fn test_validate_complete_rpc_api_spec() {
        let content = r#"---
id: test-spec
title: RPC API Spec
spec_type: rpc-api
---

## Overview

This is an RPC API spec.

## Requirements

### R1: Method Definition
Define the RPC methods.

## Acceptance Criteria

### Scenario: Call method successfully
**WHEN** client calls method
**THEN** server responds

## Class Diagram

```mermaid
classDiagram
    class Calculator {
        +add(a, b) int
        +subtract(a, b) int
    }
```

## API Specification

```yaml
openrpc: 1.3.0
info:
  title: Calculator API
  version: 1.0.0
methods:
  - name: add
    params: []
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(output["is_complete"].as_bool().unwrap());
        assert!(output["missing_elements"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_validate_incomplete_rpc_api_spec_wrong_api_type() {
        // rpc-api with OpenAPI instead of OpenRPC should fail
        let content = r#"---
id: test-spec
title: RPC API Spec
spec_type: rpc-api
---

## Overview

This is an RPC API spec with wrong API type.

## Requirements

### R1: Method Definition
Define the RPC methods.

## Acceptance Criteria

### Scenario: Call method
**WHEN** client calls
**THEN** server responds

## Class Diagram

```mermaid
classDiagram
    class Calculator {
        +add(a, b) int
    }
```

## API Specification (OpenAPI 3.1)

```yaml
openapi: 3.1.0
info:
  title: Wrong API Type
  version: 1.0.0
paths: {}
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        // Should NOT be complete - has OpenAPI but needs OpenRPC
        assert!(!output["is_complete"].as_bool().unwrap());
        let missing = output["missing_elements"].as_array().unwrap();
        assert!(missing
            .iter()
            .any(|m| m.as_str().unwrap().contains("OpenRPC 1.3")));
    }

    #[test]
    fn test_validate_complete_workflow_spec() {
        let content = r#"---
id: test-spec
title: Workflow Spec
spec_type: workflow
---

## Overview

This is a workflow spec.

## Requirements

### R1: State Transitions
Define the workflow states.

## Acceptance Criteria

### Scenario: Complete workflow
**WHEN** workflow starts
**THEN** transitions to end state

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Processing
    Processing --> Done
    Done --> [*]
```

## API Specification

```yaml
specVersion: 0.8.0
id: my-workflow
name: My Workflow
start: Processing
states:
  - name: Processing
    type: operation
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        assert!(output["is_complete"].as_bool().unwrap());
        assert!(output["missing_elements"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_validate_incomplete_workflow_spec() {
        // workflow without Serverless Workflow spec should fail
        let content = r#"---
id: test-spec
title: Workflow Spec
spec_type: workflow
---

## Overview

This is a workflow spec without API spec.

## Requirements

### R1: State Transitions
Define the workflow states.

## Acceptance Criteria

### Scenario: Complete workflow
**WHEN** workflow starts
**THEN** transitions to end

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Processing
    Processing --> [*]
```
"#;

        let temp_dir = create_test_spec(content);
        let args = json!({
            "project_path": temp_dir.path().to_str().unwrap(),
            "change_id": "test-change",
            "spec_id": "test-spec"
        });

        let result = execute(&args, temp_dir.path()).unwrap();
        let output: Value = serde_json::from_str(&result).unwrap();

        // Should NOT be complete - missing Serverless Workflow spec
        assert!(!output["is_complete"].as_bool().unwrap());
        let missing = output["missing_elements"].as_array().unwrap();
        assert!(missing
            .iter()
            .any(|m| m.as_str().unwrap().contains("Serverless Workflow 0.8")));
    }
}
// CODEGEN-END
