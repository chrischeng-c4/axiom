---
id: sdd-tools-validate-spec-preamble-definition
fill_sections: [overview, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: td-lifecycle-dispatch
    claim: td-lifecycle-dispatch
    coverage: full
    rationale: "Tool TDs implement TD/CB lifecycle artifact authoring, review, revision, merge, and validation commands."
---

# sdd tools validate spec preamble definition

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
      - "<module-preamble>"
      - "definition"
      - "ValidationResult"
    description: "Module preamble, validate-spec tool definition, and validation result type."
```
