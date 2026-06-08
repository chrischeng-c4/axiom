---
id: sdd-requirement-plus-validator-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Requirement Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `RequirementSeverity` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | enum | pub | 12 |  |
| `RequirementValidationError` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | struct | pub | 20 |  |
| `RequirementValidationResult` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | struct | pub | 30 |  |
| `RequirementValidator` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | struct | pub | 58 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 64 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 37 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 68 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 73 | validate(&self, diagram: &RequirementDiagramDef) -> RequirementValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 45 | with_error(mut self, error: RequirementValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs | function | pub | 51 | with_warning(mut self, warning: RequirementValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  RequirementSeverity:
    type: string
    enum: [Error, Warning]
    description: Severity of a requirement validation finding.
    x-rust-enum:
      derive: [Debug, Clone, "serde::Serialize", PartialEq]
      serde_rename_all: lowercase

  RequirementValidationError:
    type: object
    required: [code, message, path, severity]
    description: A single requirement validation finding.
    properties:
      code:
        type: string
      message:
        type: string
      path:
        type: string
      severity:
        $ref: "#/definitions/RequirementSeverity"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  RequirementValidationResult:
    type: object
    required: [valid, errors, warnings]
    description: Aggregate validation outcome.
    properties:
      valid:
        type: boolean
      errors:
        type: array
        items:
          $ref: "#/definitions/RequirementValidationError"
      warnings:
        type: array
        items:
          $ref: "#/definitions/RequirementValidationError"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", Default]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs -->
```rust
//! Requirement+ semantic validator

use super::schema::RequirementDiagramDef;
use std::collections::HashSet;

/// Severity of a requirement validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RequirementSeverity {
    Error,
    Warning,
}

/// A single requirement validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequirementValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: RequirementSeverity,
}

/// Aggregate validation outcome.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct RequirementValidationResult {
    pub valid: bool,
    pub errors: Vec<RequirementValidationError>,
    pub warnings: Vec<RequirementValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#source
impl RequirementValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: RequirementValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: RequirementValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#source
pub struct RequirementValidator {
    strict: bool,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#source
impl RequirementValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, diagram: &RequirementDiagramDef) -> RequirementValidationResult {
        let mut result = RequirementValidationResult::ok();

        // Collect all IDs
        let req_ids: HashSet<String> = diagram.requirements.keys().cloned().collect();
        let elem_ids: HashSet<String> = diagram.elements.keys().cloned().collect();
        let all_ids: HashSet<String> = req_ids.union(&elem_ids).cloned().collect();

        // 1. Check for empty diagram
        if diagram.requirements.is_empty() {
            result = result.with_error(RequirementValidationError {
                code: "EMPTY_DIAGRAM".to_string(),
                message: "Requirement diagram must have at least one requirement".to_string(),
                path: "requirements".to_string(),
                severity: RequirementSeverity::Error,
            });
        }

        // 2. Validate relationship endpoints
        for (idx, rel) in diagram.relationships.iter().enumerate() {
            if !all_ids.contains(&rel.from) {
                result = result.with_error(RequirementValidationError {
                    code: "INVALID_RELATIONSHIP_FROM".to_string(),
                    message: format!(
                        "Relationship source '{}' not found in requirements or elements",
                        rel.from
                    ),
                    path: format!("relationships[{}].from", idx),
                    severity: RequirementSeverity::Error,
                });
            }
            if !all_ids.contains(&rel.to) {
                result = result.with_error(RequirementValidationError {
                    code: "INVALID_RELATIONSHIP_TO".to_string(),
                    message: format!(
                        "Relationship target '{}' not found in requirements or elements",
                        rel.to
                    ),
                    path: format!("relationships[{}].to", idx),
                    severity: RequirementSeverity::Error,
                });
            }
        }

        // 3. Check for orphan requirements (not connected to any element)
        let connected_reqs: HashSet<String> = diagram
            .relationships
            .iter()
            .flat_map(|r| vec![r.from.clone(), r.to.clone()])
            .filter(|id| req_ids.contains(id))
            .collect();

        for req_id in &req_ids {
            if !connected_reqs.contains(req_id) {
                result = result.with_warning(RequirementValidationError {
                    code: "ORPHAN_REQUIREMENT".to_string(),
                    message: format!("Requirement '{}' has no relationships", req_id),
                    path: format!("requirements.{}", req_id),
                    severity: RequirementSeverity::Warning,
                });
            }
        }

        // 4. Check for empty text
        for (req_id, req) in &diagram.requirements {
            if req.text.trim().is_empty() {
                result = result.with_error(RequirementValidationError {
                    code: "EMPTY_REQUIREMENT_TEXT".to_string(),
                    message: format!("Requirement '{}' has empty text", req_id),
                    path: format!("requirements.{}.text", req_id),
                    severity: RequirementSeverity::Error,
                });
            }
        }

        for (elem_id, elem) in &diagram.elements {
            if elem.text.trim().is_empty() {
                result = result.with_error(RequirementValidationError {
                    code: "EMPTY_ELEMENT_TEXT".to_string(),
                    message: format!("Element '{}' has empty text", elem_id),
                    path: format!("elements.{}.text", elem_id),
                    severity: RequirementSeverity::Error,
                });
            }
        }

        if self.strict {
            let strict_codes = ["ORPHAN_REQUIREMENT"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = RequirementSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/requirement_plus/validator.md#source
impl Default for RequirementValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> RequirementDiagramDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "requirements": {
                "R1": { "text": "Test requirement", "risk": "Low", "verification": "Test" }
            },
            "elements": {
                "E1": { "text": "Test element", "type": "module" }
            },
            "relationships": [
                { "from": "E1", "to": "R1", "type": "satisfies" }
            ]
        }));

        let result = RequirementValidator::new().validate(&diagram);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_relationship() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "requirements": {
                "R1": { "text": "Test", "risk": "Low", "verification": "Test" }
            },
            "relationships": [
                { "from": "NonExistent", "to": "R1", "type": "satisfies" }
            ]
        }));

        let result = RequirementValidator::new().validate(&diagram);
        assert!(!result.valid);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/requirement_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Requirement+ validator module.
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
