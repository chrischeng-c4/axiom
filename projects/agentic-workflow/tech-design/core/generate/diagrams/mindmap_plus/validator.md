---
id: sdd-mindmap-plus-validator-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Mindmap Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `MindmapSeverity` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | enum | pub | 12 |  |
| `MindmapValidationError` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | struct | pub | 20 |  |
| `MindmapValidationResult` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | struct | pub | 30 |  |
| `MindmapValidator` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | struct | pub | 58 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 65 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 37 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 72 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 77 | validate(&self, mindmap: &MindmapDef) -> MindmapValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 45 | with_error(mut self, error: MindmapValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs | function | pub | 51 | with_warning(mut self, warning: MindmapValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  MindmapSeverity:
    type: string
    enum: [Error, Warning]
    description: Severity of a mindmap validation finding.
    x-rust-enum:
      derive: [Debug, Clone, "serde::Serialize", PartialEq]
      serde_rename_all: lowercase

  MindmapValidationError:
    type: object
    required: [code, message, path, severity]
    description: A single mindmap validation finding.
    properties:
      code:
        type: string
      message:
        type: string
      path:
        type: string
      severity:
        $ref: "#/definitions/MindmapSeverity"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  MindmapValidationResult:
    type: object
    required: [valid, errors, warnings]
    description: Aggregate validation outcome.
    properties:
      valid:
        type: boolean
      errors:
        type: array
        items:
          $ref: "#/definitions/MindmapValidationError"
      warnings:
        type: array
        items:
          $ref: "#/definitions/MindmapValidationError"
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", Default]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs -->
```rust
//! Mindmap+ semantic validator

use super::schema::{MindmapDef, MindmapNodeDef};
use std::collections::HashSet;

/// Severity of a mindmap validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MindmapSeverity {
    Error,
    Warning,
}

/// A single mindmap validation finding.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct MindmapValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: MindmapSeverity,
}

/// Aggregate validation outcome.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct MindmapValidationResult {
    pub valid: bool,
    pub errors: Vec<MindmapValidationError>,
    pub warnings: Vec<MindmapValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#source
impl MindmapValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: MindmapValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: MindmapValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#source
pub struct MindmapValidator {
    strict: bool,
    max_depth: usize,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#source
impl MindmapValidator {
    pub fn new() -> Self {
        Self {
            strict: false,
            max_depth: 10,
        }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, mindmap: &MindmapDef) -> MindmapValidationResult {
        let mut result = MindmapValidationResult::ok();

        // 1. Check root label
        if mindmap.root.label.trim().is_empty() {
            result = result.with_error(MindmapValidationError {
                code: "EMPTY_ROOT_LABEL".to_string(),
                message: "Root node must have a non-empty label".to_string(),
                path: "root.label".to_string(),
                severity: MindmapSeverity::Error,
            });
        }

        // 2. Validate children recursively
        let mut labels_seen: HashSet<String> = HashSet::new();
        labels_seen.insert(mindmap.root.label.clone());
        self.validate_children(
            &mindmap.root.children,
            "root",
            1,
            &mut labels_seen,
            &mut result,
        );

        // 3. Check for empty mindmap (only root, no children) - warning
        if mindmap.root.children.is_empty() {
            result = result.with_warning(MindmapValidationError {
                code: "EMPTY_MINDMAP".to_string(),
                message: "Mindmap has no children, only root node".to_string(),
                path: "root.children".to_string(),
                severity: MindmapSeverity::Warning,
            });
        }

        if self.strict {
            let strict_codes = ["EMPTY_MINDMAP"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = MindmapSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }

    fn validate_children(
        &self,
        children: &[MindmapNodeDef],
        parent_path: &str,
        depth: usize,
        labels_seen: &mut HashSet<String>,
        result: &mut MindmapValidationResult,
    ) {
        // Check depth limit
        if depth > self.max_depth {
            *result = std::mem::take(result).with_error(MindmapValidationError {
                code: "MAX_DEPTH_EXCEEDED".to_string(),
                message: format!("Mindmap depth exceeds maximum of {}", self.max_depth),
                path: parent_path.to_string(),
                severity: MindmapSeverity::Error,
            });
            return;
        }

        for (idx, child) in children.iter().enumerate() {
            let child_path = format!("{}.children[{}]", parent_path, idx);

            // Check empty label
            if child.label.trim().is_empty() {
                *result = std::mem::take(result).with_error(MindmapValidationError {
                    code: "EMPTY_NODE_LABEL".to_string(),
                    message: "Node must have a non-empty label".to_string(),
                    path: format!("{}.label", child_path),
                    severity: MindmapSeverity::Error,
                });
            }

            // Check for duplicate labels (warning)
            if labels_seen.contains(&child.label) {
                *result = std::mem::take(result).with_warning(MindmapValidationError {
                    code: "DUPLICATE_LABEL".to_string(),
                    message: format!("Duplicate label '{}' found in mindmap", child.label),
                    path: child_path.clone(),
                    severity: MindmapSeverity::Warning,
                });
            }
            labels_seen.insert(child.label.clone());

            // Recurse into children
            self.validate_children(&child.children, &child_path, depth + 1, labels_seen, result);
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/mindmap_plus/validator.md#source
impl Default for MindmapValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_mindmap(json: serde_json::Value) -> MindmapDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_mindmap() {
        let mindmap = parse_mindmap(json!({
            "id": "test",
            "root": {
                "label": "Root",
                "children": [
                    { "label": "Child 1" },
                    { "label": "Child 2" }
                ]
            }
        }));

        let result = MindmapValidator::new().validate(&mindmap);
        assert!(result.valid);
    }

    #[test]
    fn test_empty_root_label() {
        let mindmap = parse_mindmap(json!({
            "id": "test",
            "root": { "label": "" }
        }));

        let result = MindmapValidator::new().validate(&mindmap);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "EMPTY_ROOT_LABEL"));
    }

    #[test]
    fn test_duplicate_label_warning() {
        let mindmap = parse_mindmap(json!({
            "id": "test",
            "root": {
                "label": "Root",
                "children": [
                    { "label": "Same" },
                    { "label": "Same" }
                ]
            }
        }));

        let result = MindmapValidator::new().validate(&mindmap);
        assert!(result.valid); // Warnings don't invalidate
        assert!(result.warnings.iter().any(|w| w.code == "DUPLICATE_LABEL"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/mindmap_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Mindmap+ validator module.
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
