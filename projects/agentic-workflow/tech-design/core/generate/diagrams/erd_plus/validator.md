---
id: sdd-erd-plus-validator-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# ERD Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `ERDSeverity` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | enum | pub | 11 |  |
| `ERDValidationError` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | struct | pub | 18 |  |
| `ERDValidationResult` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | struct | pub | 27 |  |
| `ERDValidator` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | struct | pub | 55 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 61 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 34 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 65 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 70 | validate(&self, erd: &ERDDef) -> ERDValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 42 | with_error(mut self, error: ERDValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs | function | pub | 48 | with_warning(mut self, warning: ERDValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  ERDSeverity:
    type: string
    enum: [Error, Warning]
    x-rust-enum:
      derive: [Debug, Clone, "serde::Serialize", PartialEq]
      serde_rename_all: lowercase

  ERDValidationError:
    type: object
    required: [code, message, path, severity]
    properties:
      code: { type: string }
      message: { type: string }
      path: { type: string }
      severity: { $ref: "#/definitions/ERDSeverity" }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  ERDValidationResult:
    type: object
    required: [valid, errors, warnings]
    properties:
      valid: { type: boolean }
      errors:
        type: array
        items: { $ref: "#/definitions/ERDValidationError" }
      warnings:
        type: array
        items: { $ref: "#/definitions/ERDValidationError" }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", Default]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs -->
```rust
//! ERD+ semantic validator

use super::schema::{ERDDef, KeyType};
use std::collections::HashSet;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ERDSeverity {
    Error,
    Warning,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct ERDValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: ERDSeverity,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct ERDValidationResult {
    pub valid: bool,
    pub errors: Vec<ERDValidationError>,
    pub warnings: Vec<ERDValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#source
impl ERDValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: ERDValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: ERDValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#source
pub struct ERDValidator {
    strict: bool,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#source
impl ERDValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, erd: &ERDDef) -> ERDValidationResult {
        let mut result = ERDValidationResult::ok();
        let entity_names: HashSet<String> = erd.entities.keys().cloned().collect();

        // 1. Check for empty diagram
        if erd.entities.is_empty() {
            result = result.with_error(ERDValidationError {
                code: "EMPTY_ERD".to_string(),
                message: "ERD must have at least one entity".to_string(),
                path: "entities".to_string(),
                severity: ERDSeverity::Error,
            });
        }

        // 2. Validate relationship endpoints
        for (idx, rel) in erd.relationships.iter().enumerate() {
            if !entity_names.contains(&rel.from) {
                result = result.with_error(ERDValidationError {
                    code: "INVALID_RELATIONSHIP_FROM".to_string(),
                    message: format!("Relationship source '{}' not found", rel.from),
                    path: format!("relationships[{}].from", idx),
                    severity: ERDSeverity::Error,
                });
            }
            if !entity_names.contains(&rel.to) {
                result = result.with_error(ERDValidationError {
                    code: "INVALID_RELATIONSHIP_TO".to_string(),
                    message: format!("Relationship target '{}' not found", rel.to),
                    path: format!("relationships[{}].to", idx),
                    severity: ERDSeverity::Error,
                });
            }
        }

        // 3. Validate FK references
        for (entity_name, entity_def) in &erd.entities {
            for (attr_idx, attr) in entity_def.attributes.iter().enumerate() {
                if let Some(ref reference) = attr.references {
                    // Parse "Entity.attribute" format
                    if let Some((ref_entity, _ref_attr)) = reference.split_once('.') {
                        if !entity_names.contains(ref_entity) {
                            result = result.with_error(ERDValidationError {
                                code: "INVALID_FK_REFERENCE".to_string(),
                                message: format!(
                                    "FK reference '{}' points to non-existent entity '{}'",
                                    reference, ref_entity
                                ),
                                path: format!(
                                    "entities.{}.attributes[{}].references",
                                    entity_name, attr_idx
                                ),
                                severity: ERDSeverity::Error,
                            });
                        }
                    } else {
                        result = result.with_warning(ERDValidationError {
                            code: "INVALID_REFERENCE_FORMAT".to_string(),
                            message: format!(
                                "Reference '{}' should be in 'Entity.attribute' format",
                                reference
                            ),
                            path: format!(
                                "entities.{}.attributes[{}].references",
                                entity_name, attr_idx
                            ),
                            severity: ERDSeverity::Warning,
                        });
                    }
                }

                // Warn if FK without references
                if attr.key == Some(KeyType::FK) && attr.references.is_none() {
                    result = result.with_warning(ERDValidationError {
                        code: "FK_WITHOUT_REFERENCE".to_string(),
                        message: format!(
                            "FK attribute '{}' in '{}' has no references",
                            attr.name, entity_name
                        ),
                        path: format!("entities.{}.attributes[{}]", entity_name, attr_idx),
                        severity: ERDSeverity::Warning,
                    });
                }
            }
        }

        // 4. Check for entities without PK (warning)
        for (entity_name, entity_def) in &erd.entities {
            let has_pk = entity_def
                .attributes
                .iter()
                .any(|a| a.key == Some(KeyType::PK));
            if !has_pk && !entity_def.attributes.is_empty() {
                result = result.with_warning(ERDValidationError {
                    code: "NO_PRIMARY_KEY".to_string(),
                    message: format!("Entity '{}' has no primary key defined", entity_name),
                    path: format!("entities.{}", entity_name),
                    severity: ERDSeverity::Warning,
                });
            }
        }

        if self.strict {
            let strict_codes = ["FK_WITHOUT_REFERENCE", "NO_PRIMARY_KEY"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = ERDSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/erd_plus/validator.md#source
impl Default for ERDValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_erd(json: serde_json::Value) -> ERDDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_erd() {
        let erd = parse_erd(json!({
            "id": "test",
            "entities": {
                "User": {
                    "attributes": [
                        { "name": "id", "type": "UUID", "key": "PK" }
                    ]
                }
            }
        }));

        let result = ERDValidator::new().validate(&erd);
        assert!(result.valid);
    }

    #[test]
    fn test_invalid_fk_reference() {
        let erd = parse_erd(json!({
            "id": "test",
            "entities": {
                "Order": {
                    "attributes": [
                        { "name": "user_id", "type": "UUID", "key": "FK", "references": "NonExistent.id" }
                    ]
                }
            }
        }));

        let result = ERDValidator::new().validate(&erd);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "INVALID_FK_REFERENCE"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/erd_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete ERD+ validator module.
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
