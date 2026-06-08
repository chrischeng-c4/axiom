---
id: sdd-content-logic
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# LogicContent

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/diagrams/content/logic.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `FlowEdge` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | struct | pub | 74 |  |
| `FlowNode` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | struct | pub | 33 |  |
| `FlowNodeKind` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | enum | pub | 18 |  |
| `LogicContent` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | struct | pub | 84 |  |
| `decision_ids` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | function | pub | 125 | decision_ids(&self) -> Vec<&str> |
| `edges_from` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | function | pub | 120 | edges_from(&self, node_id: &str) -> Vec<&FlowEdge> |
| `terminal_ids` | projects/agentic-workflow/src/generate/diagrams/content/logic.rs | function | pub | 134 | terminal_ids(&self) -> Vec<&str> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  FlowNodeKind:
    type: string
    enum: [Start, Process, Decision, Terminal]
    description: Kind of a node in a flowchart/logic diagram.
    x-rust-enum:
      derive: [Debug, Clone, PartialEq, Serialize, Deserialize, Default]
      serde_rename_all: lowercase
      variants:
        - { name: Start,                       doc: "Start node." }
        - { name: Process, is_default: true,   doc: "Process node (default)." }
        - { name: Decision,                    doc: "Decision branch." }
        - { name: Terminal,                    doc: "Terminal end node." }

  FlowNode:
    type: object
    required: [kind, params, inputs]
    description: A node in a logic flowchart.
    properties:
      kind:
        $ref: "#/definitions/FlowNodeKind"
        x-serde-default: true
      label:
        type: string
        description: "Optional display label."
      fn_name:
        type: string
        description: "Explicit Rust function name to call for this node."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      params:
        type: array
        items: { type: string }
        description: "Optional call-argument names."
        x-serde-default: true
        x-serde-skip-if: "Vec::is_empty"
      is_async:
        type: boolean
        x-rust-type: "Option<bool>"
        description: "When Some(true), append .await to the call."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      primitive:
        type: string
        x-rust-type: "Option<String>"
        description: "Snake_case primitive name (matches `PrimitiveEntry::name`)."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      inputs:
        type: object
        x-rust-type: "HashMap<String, String>"
        description: "Template variable to expression bindings (for example, path -> CHANNEL_PATH)."
        x-serde-default: true
        x-serde-skip-if: "HashMap::is_empty"
      output:
        type: string
        x-rust-type: "Option<String>"
        description: "Output binding (variable name). Substituted as `{out}` in the template."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      type_param:
        type: string
        x-rust-type: "Option<String>"
        description: "Generic type parameter. Substituted as `{T}` in the template."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      loop_kind:
        type: string
        x-rust-type: "Option<String>"
        description: 'Loop form for cycle entry nodes. Currently recognized: "for_each".'
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      iter:
        type: string
        x-rust-type: "Option<String>"
        description: "Rust expression iterated by the for_each form (for example, reader.lines())."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
      loop_var:
        type: string
        x-rust-type: "Option<String>"
        description: "Rust identifier bound to each iteration (for example, line)."
        x-serde-default: true
        x-serde-skip-if: "Option::is_none"
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize, Default]

  FlowEdge:
    type: object
    required: [from, to]
    description: An edge in a logic flowchart.
    properties:
      from:
        type: string
      to:
        type: string
      label:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]

  LogicContent:
    type: object
    required: [id, entry, nodes, edges]
    description: Content type for `logic` section (flowchart). Parsed from Mermaid Plus YAML frontmatter.
    properties:
      id:
        type: string
      entry:
        type: string
      nodes:
        type: object
        x-rust-type: "HashMap<String, FlowNode>"
        description: "Nodes keyed by id."
        x-serde-default: true
      edges:
        type: array
        items:
          $ref: "#/definitions/FlowEdge"
        x-serde-default: true
      title:
        type: string
    x-rust-struct:
      derive: [Debug, Clone, Serialize, Deserialize]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/diagrams/content/logic.rs -->
````rust
//! LogicContent — per-diagram Content type for logic (flowchart).
//!
//! Replaces the XState-based schema in `flowchart_plus/schema.rs` (design decision D8).
//! Content is parsed from Mermaid Plus YAML frontmatter in spec files.

// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#source

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Kind of a node in a flowchart/logic diagram.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#schema
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FlowNodeKind {
    /// Start node.
    Start,
    /// Process node (default).
    #[default]
    Process,
    /// Decision branch.
    Decision,
    /// Terminal end node.
    Terminal,
}

/// A node in a logic flowchart.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#schema
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FlowNode {
    #[serde(default)]
    pub kind: FlowNodeKind,
    /// Optional display label.
    #[serde(default)]
    pub label: Option<String>,
    /// Explicit Rust function name to call for this node.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fn_name: Option<String>,
    /// Optional call-argument names.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<String>,
    /// When Some(true), append .await to the call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_async: Option<bool>,
    /// Snake_case primitive name (matches `PrimitiveEntry::name`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primitive: Option<String>,
    /// Template variable to expression bindings (for example, path -> CHANNEL_PATH).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub inputs: HashMap<String, String>,
    /// Output binding (variable name). Substituted as `{out}` in the template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    /// Generic type parameter. Substituted as `{T}` in the template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_param: Option<String>,
    /// Loop form for cycle entry nodes. Currently recognized: "for_each".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_kind: Option<String>,
    /// Rust expression iterated by the for_each form (for example, reader.lines()).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iter: Option<String>,
    /// Rust identifier bound to each iteration (for example, line).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_var: Option<String>,
}

/// An edge in a logic flowchart.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// Content type for `logic` section (flowchart). Parsed from Mermaid Plus YAML frontmatter.
/// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicContent {
    pub id: String,
    pub entry: String,
    /// Nodes keyed by id.
    #[serde(default)]
    pub nodes: HashMap<String, FlowNode>,
    #[serde(default)]
    pub edges: Vec<FlowEdge>,
    #[serde(default)]
    pub title: Option<String>,
}

/// Content type for `logic` section (flowchart).
///
/// Parsed from Mermaid Plus YAML frontmatter:
/// ```yaml
/// id: my-logic
/// entry: start
/// nodes:
///   start: { kind: start, label: "Begin" }
///   validate: { kind: decision, label: "Valid?" }
///   error: { kind: terminal, label: "Return error" }
///   ok: { kind: terminal, label: "Return ok" }
/// edges:
///   - from: start
///     to: validate
///   - from: validate
///     to: error
///     label: "no"
///   - from: validate
///     to: ok
///     label: "yes"
/// ```
// @spec projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#source
impl LogicContent {
    /// Return outgoing edges from a node.
    pub fn edges_from(&self, node_id: &str) -> Vec<&FlowEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }

    /// Return decision node IDs (nodes with multiple outgoing edges).
    pub fn decision_ids(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.kind == FlowNodeKind::Decision)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    /// Return terminal node IDs.
    pub fn terminal_ids(&self) -> Vec<&str> {
        self.nodes
            .iter()
            .filter(|(_, n)| n.kind == FlowNodeKind::Terminal)
            .map(|(id, _)| id.as_str())
            .collect()
    }
}
````

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/diagrams/content/logic.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete logic content module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] 4-type scope clean. is_default on Process replaces manual impl Default.
- [schema] HashMap via x-rust-type; Option<bool> override; Vec/Option skip-if patterns.
- [changes] codegen + hand-written split correct.
