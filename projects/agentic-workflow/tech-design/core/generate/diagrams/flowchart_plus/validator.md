---
id: sdd-generate-flowchart-plus-validator
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Flowchart Plus Validator Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FlowchartSeverity` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | enum | pub | 18 |  |
| `FlowchartValidationError` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | struct | pub | 38 |  |
| `FlowchartValidationResult` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | struct | pub | 26 |  |
| `FlowchartValidator` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | struct | pub | 51 |  |
| `new` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 79 | new() -> Self |
| `ok` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 57 | ok() -> Self |
| `strict` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 83 | strict(mut self) -> Self |
| `validate` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 89 | validate(&self, flowchart: &FlowchartDef) -> FlowchartValidationResult |
| `with_error` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 65 | with_error(mut self, error: FlowchartValidationError) -> Self |
| `with_warning` | projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs | function | pub | 71 | with_warning(mut self, warning: FlowchartValidationError) -> Self |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FlowchartSeverity:
    type: string
    enum: [Error, Warning]
    description: Error severity.
    x-rust-enum:
      derive: [Debug, Clone, Serialize, PartialEq]
      serde_rename_all: lowercase

  FlowchartValidationResult:
    type: object
    required: [valid, errors, warnings]
    description: Validation result.
    properties:
      valid:
        type: boolean
        description: "Whether validation passed (no errors)."
      errors:
        type: array
        items: { $ref: "#/definitions/FlowchartValidationError" }
        x-rust-type: "Vec<FlowchartValidationError>"
        description: "Validation errors."
      warnings:
        type: array
        items: { $ref: "#/definitions/FlowchartValidationError" }
        x-rust-type: "Vec<FlowchartValidationError>"
        description: "Validation warnings."
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Default]

  FlowchartValidationError:
    type: object
    required: [code, message, path, severity]
    description: Validation error/warning.
    properties:
      code:
        type: string
        description: "Error code."
      message:
        type: string
        description: "Human-readable message."
      path:
        type: string
        description: "JSON pointer path."
      severity:
        type: string
        x-rust-type: "FlowchartSeverity"
        description: "Severity level."
    x-rust-struct:
      derive: [Debug, Clone, Serialize]

  FlowchartValidator:
    type: object
    required: [strict]
    description: Flowchart diagram validator.
    properties:
      strict:
        type: boolean
        x-rust-visibility: private
        description: "Strict mode flag."
    x-rust-struct:
      derive: []
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs -->
```rust
//! Flowchart+ semantic validator
//!
//! Validates flowchart definitions for:
//! - Structural correctness (node IDs unique, edge endpoints exist)
//! - Semantic correctness (semantic type consistency, subgraph validity)

use super::schema::{FlowchartDef, SemanticType};
use std::collections::HashSet;

use serde::Serialize;

/// Error severity.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FlowchartSeverity {
    Error,
    Warning,
}

/// Validation result.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#schema
#[derive(Debug, Clone, Serialize, Default)]
pub struct FlowchartValidationResult {
    /// Whether validation passed (no errors).
    pub valid: bool,
    /// Validation errors.
    pub errors: Vec<FlowchartValidationError>,
    /// Validation warnings.
    pub warnings: Vec<FlowchartValidationError>,
}

/// Validation error/warning.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#schema
#[derive(Debug, Clone, Serialize)]
pub struct FlowchartValidationError {
    /// Error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// JSON pointer path.
    pub path: String,
    /// Severity level.
    pub severity: FlowchartSeverity,
}

/// Flowchart diagram validator.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#schema
pub struct FlowchartValidator {
    /// Strict mode flag.
    strict: bool,
}
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#source
impl FlowchartValidationResult {
    pub fn ok() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    pub fn with_error(mut self, error: FlowchartValidationError) -> Self {
        self.valid = false;
        self.errors.push(error);
        self
    }

    pub fn with_warning(mut self, warning: FlowchartValidationError) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#source
impl FlowchartValidator {
    pub fn new() -> Self {
        Self { strict: false }
    }

    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Validate a flowchart definition
    pub fn validate(&self, flowchart: &FlowchartDef) -> FlowchartValidationResult {
        let mut result = FlowchartValidationResult::ok();

        // Collect all node IDs
        let node_ids: HashSet<String> = flowchart.nodes.keys().cloned().collect();

        // 1. Check for empty flowchart
        if flowchart.nodes.is_empty() {
            result = result.with_error(FlowchartValidationError {
                code: "EMPTY_FLOWCHART".to_string(),
                message: "Flowchart must have at least one node".to_string(),
                path: "nodes".to_string(),
                severity: FlowchartSeverity::Error,
            });
        }

        // 2. Validate edge endpoints
        for (idx, edge) in flowchart.edges.iter().enumerate() {
            if !node_ids.contains(&edge.from) {
                result = result.with_error(FlowchartValidationError {
                    code: "INVALID_EDGE_FROM".to_string(),
                    message: format!("Edge source '{}' not found in nodes", edge.from),
                    path: format!("edges[{}].from", idx),
                    severity: FlowchartSeverity::Error,
                });
            }
            if !node_ids.contains(&edge.to) {
                result = result.with_error(FlowchartValidationError {
                    code: "INVALID_EDGE_TO".to_string(),
                    message: format!("Edge target '{}' not found in nodes", edge.to),
                    path: format!("edges[{}].to", idx),
                    severity: FlowchartSeverity::Error,
                });
            }
        }

        // 3. Validate subgraph node references
        for (sg_idx, subgraph) in flowchart.subgraphs.iter().enumerate() {
            for (node_idx, node_id) in subgraph.nodes.iter().enumerate() {
                if !node_ids.contains(node_id) {
                    result = result.with_error(FlowchartValidationError {
                        code: "INVALID_SUBGRAPH_NODE".to_string(),
                        message: format!(
                            "Subgraph '{}' references non-existent node '{}'",
                            subgraph.id, node_id
                        ),
                        path: format!("subgraphs[{}].nodes[{}]", sg_idx, node_idx),
                        severity: FlowchartSeverity::Error,
                    });
                }
            }
        }

        // 4. Check for circular subgraph containment (nodes in multiple subgraphs)
        let mut node_subgraph_map: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for subgraph in &flowchart.subgraphs {
            for node_id in &subgraph.nodes {
                node_subgraph_map
                    .entry(node_id.clone())
                    .or_default()
                    .push(subgraph.id.clone());
            }
        }
        for (node_id, subgraphs) in &node_subgraph_map {
            if subgraphs.len() > 1 {
                result = result.with_warning(FlowchartValidationError {
                    code: "NODE_IN_MULTIPLE_SUBGRAPHS".to_string(),
                    message: format!(
                        "Node '{}' is in multiple subgraphs: {}",
                        node_id,
                        subgraphs.join(", ")
                    ),
                    path: format!("nodes.{}", node_id),
                    severity: FlowchartSeverity::Warning,
                });
            }
        }

        // 5. Validate semantic consistency
        self.validate_semantics(flowchart, &mut result);

        // 6. Check for unreachable nodes (warning)
        self.check_unreachable_nodes(flowchart, &node_ids, &mut result);

        // 7. In strict mode, promote certain warnings to errors
        if self.strict {
            let strict_codes = ["NODE_IN_MULTIPLE_SUBGRAPHS"];
            let (promoted, remaining): (Vec<_>, Vec<_>) = result
                .warnings
                .into_iter()
                .partition(|w| strict_codes.contains(&w.code.as_str()));

            result.warnings = remaining;
            for mut warning in promoted {
                warning.severity = FlowchartSeverity::Error;
                result.errors.push(warning);
            }
            if !result.errors.is_empty() {
                result.valid = false;
            }
        }

        result
    }

    /// Validate semantic type consistency
    fn validate_semantics(&self, flowchart: &FlowchartDef, result: &mut FlowchartValidationResult) {
        let mut has_start = false;
        let mut has_end = false;

        for (node_id, node) in &flowchart.nodes {
            if let Some(ref semantic) = node.semantic {
                match semantic {
                    SemanticType::Start => {
                        has_start = true;
                    }
                    SemanticType::End { .. } => {
                        has_end = true;
                    }
                    SemanticType::Condition { expression } => {
                        // Check that condition node has outgoing edges
                        let outgoing: Vec<_> = flowchart
                            .edges
                            .iter()
                            .filter(|e| e.from == *node_id)
                            .collect();

                        if outgoing.len() < 2 {
                            *result =
                                std::mem::take(result).with_warning(FlowchartValidationError {
                                    code: "CONDITION_FEW_BRANCHES".to_string(),
                                    message: format!(
                                        "Condition node '{}' should have at least 2 outgoing edges",
                                        node_id
                                    ),
                                    path: format!("nodes.{}", node_id),
                                    severity: FlowchartSeverity::Warning,
                                });
                        }

                        if expression.is_empty() {
                            *result =
                                std::mem::take(result).with_warning(FlowchartValidationError {
                                    code: "EMPTY_CONDITION".to_string(),
                                    message: format!(
                                        "Condition node '{}' has empty expression",
                                        node_id
                                    ),
                                    path: format!("nodes.{}.semantic.expression", node_id),
                                    severity: FlowchartSeverity::Warning,
                                });
                        }
                    }
                    SemanticType::DbQuery { table, .. }
                    | SemanticType::DbMutation { table, .. } => {
                        if table.is_empty() {
                            *result = std::mem::take(result).with_error(FlowchartValidationError {
                                code: "EMPTY_TABLE".to_string(),
                                message: format!("Node '{}' has empty table name", node_id),
                                path: format!("nodes.{}.semantic.table", node_id),
                                severity: FlowchartSeverity::Error,
                            });
                        }
                    }
                    SemanticType::ApiCall { url, .. } => {
                        if url.is_empty() {
                            *result = std::mem::take(result).with_error(FlowchartValidationError {
                                code: "EMPTY_URL".to_string(),
                                message: format!("Node '{}' has empty URL", node_id),
                                path: format!("nodes.{}.semantic.url", node_id),
                                severity: FlowchartSeverity::Error,
                            });
                        }
                    }
                    SemanticType::Validation { rules, .. } => {
                        if rules.is_empty() {
                            *result =
                                std::mem::take(result).with_warning(FlowchartValidationError {
                                    code: "EMPTY_VALIDATION_RULES".to_string(),
                                    message: format!("Validation node '{}' has no rules", node_id),
                                    path: format!("nodes.{}.semantic.rules", node_id),
                                    severity: FlowchartSeverity::Warning,
                                });
                        }
                    }
                    _ => {}
                }
            }
        }

        // Warn if no start or end node with semantic
        if !flowchart.nodes.is_empty() {
            if !has_start {
                *result = std::mem::take(result).with_warning(FlowchartValidationError {
                    code: "NO_START_SEMANTIC".to_string(),
                    message: "No node with 'start' semantic type found".to_string(),
                    path: "nodes".to_string(),
                    severity: FlowchartSeverity::Warning,
                });
            }
            if !has_end {
                *result = std::mem::take(result).with_warning(FlowchartValidationError {
                    code: "NO_END_SEMANTIC".to_string(),
                    message: "No node with 'end' semantic type found".to_string(),
                    path: "nodes".to_string(),
                    severity: FlowchartSeverity::Warning,
                });
            }
        }
    }

    /// Check for unreachable nodes
    fn check_unreachable_nodes(
        &self,
        flowchart: &FlowchartDef,
        node_ids: &HashSet<String>,
        result: &mut FlowchartValidationResult,
    ) {
        if flowchart.nodes.is_empty() || flowchart.edges.is_empty() {
            return;
        }

        // Find nodes with start semantic or no incoming edges as potential roots
        let mut roots: Vec<String> = Vec::new();
        for (node_id, node) in &flowchart.nodes {
            if let Some(SemanticType::Start) = &node.semantic {
                roots.push(node_id.clone());
            }
        }

        // If no start semantic, use nodes with no incoming edges
        if roots.is_empty() {
            let targets: HashSet<&String> = flowchart.edges.iter().map(|e| &e.to).collect();
            for node_id in node_ids {
                if !targets.contains(node_id) {
                    roots.push(node_id.clone());
                }
            }
        }

        if roots.is_empty() {
            return; // All nodes have incoming edges, could be a cycle
        }

        // BFS to find reachable nodes
        let mut reachable: HashSet<String> = HashSet::new();
        let mut queue: Vec<String> = roots;

        while let Some(current) = queue.pop() {
            if reachable.contains(&current) {
                continue;
            }
            reachable.insert(current.clone());

            for edge in &flowchart.edges {
                if edge.from == current && !reachable.contains(&edge.to) {
                    queue.push(edge.to.clone());
                }
            }
        }

        // Report unreachable nodes
        for node_id in node_ids {
            if !reachable.contains(node_id) {
                *result = std::mem::take(result).with_warning(FlowchartValidationError {
                    code: "UNREACHABLE_NODE".to_string(),
                    message: format!("Node '{}' may be unreachable", node_id),
                    path: format!("nodes.{}", node_id),
                    severity: FlowchartSeverity::Warning,
                });
            }
        }
    }
}

/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/flowchart_plus/validator.md#source
impl Default for FlowchartValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse_flowchart(json: serde_json::Value) -> FlowchartDef {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn test_valid_simple_flowchart() {
        let flowchart = parse_flowchart(json!({
            "id": "test",
            "nodes": {
                "start": { "label": "Start", "semantic": { "type": "start" } },
                "end": { "label": "End", "semantic": { "type": "end" } }
            },
            "edges": [
                { "from": "start", "to": "end" }
            ]
        }));

        let result = FlowchartValidator::new().validate(&flowchart);
        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_invalid_edge_target() {
        let flowchart = parse_flowchart(json!({
            "id": "test",
            "nodes": {
                "a": { "label": "A" }
            },
            "edges": [
                { "from": "a", "to": "nonexistent" }
            ]
        }));

        let result = FlowchartValidator::new().validate(&flowchart);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "INVALID_EDGE_TO"));
    }

    #[test]
    fn test_invalid_subgraph_node() {
        let flowchart = parse_flowchart(json!({
            "id": "test",
            "nodes": {
                "a": { "label": "A" }
            },
            "edges": [],
            "subgraphs": [
                { "id": "sg1", "label": "Group", "nodes": ["a", "nonexistent"] }
            ]
        }));

        let result = FlowchartValidator::new().validate(&flowchart);
        assert!(!result.valid);
        assert!(result
            .errors
            .iter()
            .any(|e| e.code == "INVALID_SUBGRAPH_NODE"));
    }

    #[test]
    fn test_empty_table_error() {
        let flowchart = parse_flowchart(json!({
            "id": "test",
            "nodes": {
                "query": {
                    "label": "Query",
                    "semantic": { "type": "db_query", "table": "", "output": "result" }
                }
            },
            "edges": []
        }));

        let result = FlowchartValidator::new().validate(&flowchart);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "EMPTY_TABLE"));
    }
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/flowchart_plus/validator.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete Flowchart+ validator module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Four types: result, error, severity enum, validator.
- [schema] All well-formed; validator has private bool field.
- [changes] Standard split with all four in `replaces`.
