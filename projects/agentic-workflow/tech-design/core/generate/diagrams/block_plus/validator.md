---
id: sdd-block-plus-validator-types
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Block Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BlockSeverity` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | enum | pub | 11 |  |
| `BlockValidationError` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | struct | pub | 18 |  |
| `BlockValidationResult` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | struct | pub | 27 |  |
| `BlockValidator` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | struct | pub | 55 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 61 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 34 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 65 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 70 | validate(&self, diagram: &BlockDef) -> BlockValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 42 | with_error(mut self, error: BlockValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs | function | pub | 48 | with_warning(mut self, warning: BlockValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  BlockSeverity:
    type: string
    enum: [Error, Warning]
    x-rust-enum:
      derive: [Debug, Clone, "serde::Serialize", PartialEq]
      serde_rename_all: lowercase

  BlockValidationError:
    type: object
    required: [code, message, path, severity]
    properties:
      code: { type: string }
      message: { type: string }
      path: { type: string }
      severity: { $ref: "#/definitions/BlockSeverity" }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize"]

  BlockValidationResult:
    type: object
    required: [valid, errors, warnings]
    properties:
      valid: { type: boolean }
      errors:
        type: array
        items: { $ref: "#/definitions/BlockValidationError" }
      warnings:
        type: array
        items: { $ref: "#/definitions/BlockValidationError" }
    x-rust-struct:
      derive: [Debug, Clone, "serde::Serialize", Default]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs -->
```rust
//! Block+ semantic validator

use super::schema::{BlockDef, BlockNodeDef};
use std::collections::HashSet;

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlockSeverity {
    Error,
    Warning,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockValidationError {
    pub code: String,
    pub message: String,
    pub path: String,
    pub severity: BlockSeverity,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#schema
#[derive(Debug, Clone, serde::Serialize, Default)]
pub struct BlockValidationResult {
    pub valid: bool,
    pub errors: Vec<BlockValidationError>,
    pub warnings: Vec<BlockValidationError>,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#source
impl BlockValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: BlockValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: BlockValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#source
pub struct BlockValidator {
    strict: bool,
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#source
impl BlockValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    pub fn validate(&self, diagram: &BlockDef) -> BlockValidationResult {
        let mut result = BlockValidationResult::ok();

        // 1. Check for empty diagram
        if diagram.blocks.is_empty() {
            result = result.with_error(BlockValidationError {
                code: "EMPTY_DIAGRAM".to_string(),
                message: "Block diagram must have at least one block".to_string(),
                path: "blocks".to_string(),
                severity: BlockSeverity::Error,
            });
        }

        // 2. Collect all block IDs (including nested)
        let mut all_ids = HashSet::new();
        self.collect_block_ids(&diagram.blocks, &mut all_ids, &mut result, "blocks");

        // 3. Validate edges
        for (idx, edge) in diagram.edges.iter().enumerate() {
            if !all_ids.contains(&edge.from) {
                result = result.with_error(BlockValidationError {
                    code: "INVALID_EDGE_FROM".to_string(),
                    message: format!("Edge source '{}' not found in blocks", edge.from),
                    path: format!("edges[{}].from", idx),
                    severity: BlockSeverity::Error,
                });
            }
            if !all_ids.contains(&edge.to) {
                result = result.with_error(BlockValidationError {
                    code: "INVALID_EDGE_TO".to_string(),
                    message: format!("Edge target '{}' not found in blocks", edge.to),
                    path: format!("edges[{}].to", idx),
                    severity: BlockSeverity::Error,
                });
            }
        }

        // 4. Validate column spans (recursive)
        self.validate_column_spans(&diagram.blocks, diagram.columns, &mut result, "blocks");

        // 5. Orphan blocks (not connected by any edge)
        if !diagram.edges.is_empty() {
            let connected: HashSet<String> = diagram
                .edges
                .iter()
                .flat_map(|e| vec![e.from.clone(), e.to.clone()])
                .collect();
            for block in &diagram.blocks {
                self.check_orphans(&block.id, &connected, &mut result);
            }
        }

        // 6. Columns must be > 0
        if diagram.columns == 0 {
            result = result.with_error(BlockValidationError {
                code: "ZERO_COLUMNS".to_string(),
                message: "Column count must be at least 1".to_string(),
                path: "columns".to_string(),
                severity: BlockSeverity::Error,
            });
        }

        // Strict mode: promote warnings to errors
        if self.strict {
            let strict_codes = ["ORPHAN_BLOCK"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));
            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = BlockSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }

    fn validate_column_spans(
        &self,
        blocks: &[BlockNodeDef],
        parent_columns: u32,
        result: &mut BlockValidationResult,
        path: &str,
    ) {
        for (idx, block) in blocks.iter().enumerate() {
            if block.width > parent_columns && parent_columns > 0 {
                *result = std::mem::take(result).with_error(BlockValidationError {
                    code: "SPAN_EXCEEDS_COLUMNS".to_string(),
                    message: format!(
                        "Block '{}' width {} exceeds parent columns {}",
                        block.id, block.width, parent_columns
                    ),
                    path: format!("{}[{}].width", path, idx),
                    severity: BlockSeverity::Error,
                });
            }
            if !block.children.is_empty() {
                let child_cols = block.child_columns.unwrap_or(block.children.len() as u32);
                let child_path = format!("{}[{}].children", path, idx);
                self.validate_column_spans(&block.children, child_cols, result, &child_path);
            }
        }
    }

    fn collect_block_ids(
        &self,
        blocks: &[BlockNodeDef],
        ids: &mut HashSet<String>,
        result: &mut BlockValidationResult,
        path: &str,
    ) {
        for (idx, block) in blocks.iter().enumerate() {
            if !ids.insert(block.id.clone()) {
                *result = std::mem::take(result).with_error(BlockValidationError {
                    code: "DUPLICATE_BLOCK_ID".to_string(),
                    message: format!("Duplicate block ID '{}'", block.id),
                    path: format!("{}[{}].id", path, idx),
                    severity: BlockSeverity::Error,
                });
            }
            if block.label.trim().is_empty() {
                *result = std::mem::take(result).with_error(BlockValidationError {
                    code: "EMPTY_BLOCK_LABEL".to_string(),
                    message: format!("Block '{}' has empty label", block.id),
                    path: format!("{}[{}].label", path, idx),
                    severity: BlockSeverity::Error,
                });
            }
            if !block.children.is_empty() {
                let child_path = format!("{}[{}].children", path, idx);
                self.collect_block_ids(&block.children, ids, result, &child_path);
            }
        }
    }

    fn check_orphans(
        &self,
        id: &str,
        connected: &HashSet<String>,
        result: &mut BlockValidationResult,
    ) {
        if !connected.contains(id) {
            *result = std::mem::take(result).with_warning(BlockValidationError {
                code: "ORPHAN_BLOCK".to_string(),
                message: format!("Block '{}' has no edge connections", id),
                path: format!("blocks.{}", id),
                severity: BlockSeverity::Warning,
            });
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/block_plus/validator.md#source
impl Default for BlockValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_diagram(json: serde_json::Value) -> BlockDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 2,
            "blocks": [
                { "id": "a", "label": "Block A" },
                { "id": "b", "label": "Block B" }
            ],
            "edges": [
                { "from": "a", "to": "b" }
            ]
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(result.valid);
    }

    #[test]
    fn test_empty_diagram() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "blocks": []
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(!result.valid);
        assert_eq!(result.errors[0].code, "EMPTY_DIAGRAM");
    }

    #[test]
    fn test_invalid_edge() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 2,
            "blocks": [{ "id": "a", "label": "A" }],
            "edges": [{ "from": "a", "to": "nonexistent" }]
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(!result.valid);
        assert_eq!(result.errors[0].code, "INVALID_EDGE_TO");
    }

    #[test]
    fn test_span_exceeds_columns() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 2,
            "blocks": [{ "id": "a", "label": "A", "width": 3 }]
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(!result.valid);
        assert_eq!(result.errors[0].code, "SPAN_EXCEEDS_COLUMNS");
    }

    #[test]
    fn test_duplicate_ids() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 2,
            "blocks": [
                { "id": "a", "label": "A" },
                { "id": "a", "label": "Also A" }
            ]
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(!result.valid);
        assert_eq!(result.errors[0].code, "DUPLICATE_BLOCK_ID");
    }

    #[test]
    fn test_orphan_warning() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 3,
            "blocks": [
                { "id": "a", "label": "A" },
                { "id": "b", "label": "B" },
                { "id": "c", "label": "C" }
            ],
            "edges": [{ "from": "a", "to": "b" }]
        }));

        let result = BlockValidator::new().validate(&diagram);
        assert!(result.valid);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.warnings[0].code, "ORPHAN_BLOCK");
    }

    #[test]
    fn test_strict_mode() {
        let diagram = parse_diagram(json!({
            "id": "test",
            "columns": 3,
            "blocks": [
                { "id": "a", "label": "A" },
                { "id": "b", "label": "B" },
                { "id": "c", "label": "C" }
            ],
            "edges": [{ "from": "a", "to": "b" }]
        }));

        let result = BlockValidator::new().strict().validate(&diagram);
        assert!(!result.valid);
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/block_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Block+ validator module.
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
