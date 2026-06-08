// SPEC-MANAGED: projects/agentic-workflow/tech-design/core/generate/diagrams/content/logic.md#source
// CODEGEN-BEGIN
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

// CODEGEN-END
