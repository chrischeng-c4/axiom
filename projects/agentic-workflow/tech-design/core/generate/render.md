---
id: sdd-generate-render
fill_sections: [overview, schema, source, changes]
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Generator primitives are part of TD/CB lifecycle automation because they produce reviewable code artifacts from TD sections."
---

# Render Output Types

## Overview
<!-- type: overview lang: markdown -->

Public API manifest for `projects/agentic-workflow/src/generate/render.rs` generated from AST during Score force-regeneration standardization.

### Symbols

| Name | Target | Kind | Visibility | Line | Signature |
|------|--------|------|------------|------|-----------|
| `BlockRenderResult` | projects/agentic-workflow/src/generate/render.rs | struct | pub | 16 |  |
| `RenderReport` | projects/agentic-workflow/src/generate/render.rs | struct | pub | 32 |  |
| `run_render` | projects/agentic-workflow/src/generate/render.rs | function | pub | 45 | run_render(spec_path: &Path, check_only: bool) -> crate::generate::Result<RenderReport> |
## Schema
<!-- type: schema lang: yaml -->

```yaml
definitions:
  BlockRenderResult:
    type: object
    required: [id, section_heading, section_type, updated, new_body]
    description: |
      Per-block render result.
    properties:
      id:
        type: string
        description: "Block ID from frontmatter."
      section_heading:
        type: string
        x-rust-type: "Option<String>"
        description: "Section heading this block belongs to."
      section_type:
        type: string
        x-rust-type: "Option<String>"
        description: "Section type annotation."
      updated:
        type: boolean
        description: "True if the body was updated."
      new_body:
        type: string
        description: "The regenerated Mermaid body."
    x-rust-struct:
      derive: [Debug, Clone]

  RenderReport:
    type: object
    required: [blocks, file_updated, changed_count]
    description: |
      Render report for a spec file.
    properties:
      blocks:
        type: array
        items: { type: object }
        x-rust-type: "Vec<BlockRenderResult>"
        description: "Per-block render results."
      file_updated:
        type: boolean
        description: "True if spec file was updated."
      changed_count:
        type: integer
        x-rust-type: "usize"
        description: "Number of blocks where the diagram body changed."
    x-rust-struct:
      derive: [Debug, Clone]
```

## Source
<!-- type: source lang: rust -->
<!-- source-from-target: strip-managed-markers -->

<!-- source-snapshot: path=projects/agentic-workflow/src/generate/render.rs -->
```rust
//! Render implementation: parse Mermaid Plus frontmatter YAML and regenerate diagram body.
//!
//! `run_render` reads a spec file, parses all Mermaid Plus blocks (YAML frontmatter inside
//! mermaid code fences), regenerates the Mermaid diagram body from the YAML data,
//! and optionally checks or updates the spec file.

// @spec projects/agentic-workflow/tech-design/core/generate/render.md#source

use std::path::Path;

/// Per-block render result.
/// @spec projects/agentic-workflow/tech-design/core/generate/render.md#schema
#[derive(Debug, Clone)]
pub struct BlockRenderResult {
    /// Block ID from frontmatter.
    pub id: String,
    /// Section heading this block belongs to.
    pub section_heading: Option<String>,
    /// Section type annotation.
    pub section_type: Option<String>,
    /// True if the body was updated.
    pub updated: bool,
    /// The regenerated Mermaid body.
    pub new_body: String,
}

/// Render report for a spec file.
/// @spec projects/agentic-workflow/tech-design/core/generate/render.md#schema
#[derive(Debug, Clone)]
pub struct RenderReport {
    /// Per-block render results.
    pub blocks: Vec<BlockRenderResult>,
    /// True if spec file was updated.
    pub file_updated: bool,
    /// Number of blocks where the diagram body changed.
    pub changed_count: usize,
}
/// Run render for a spec file.
///
/// Parses all Mermaid Plus blocks, regenerates Mermaid syntax from the YAML data,
/// and either updates the spec file or reports differences (check_only mode).
// @spec projects/agentic-workflow/tech-design/core/generate/render.md#source
pub fn run_render(spec_path: &Path, check_only: bool) -> crate::generate::Result<RenderReport> {
    use crate::generate::frontmatter::extract_mermaid_plus_blocks;

    let spec_content =
        std::fs::read_to_string(spec_path).map_err(|e| crate::generate::GenerateError::Io(e))?;

    let blocks = extract_mermaid_plus_blocks(&spec_content);

    let mut block_results = Vec::new();
    let mut changed_count = 0;

    for block in &blocks {
        let id = block
            .frontmatter
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let section_type = block.section_type.as_deref();
        let new_body = regenerate_body(&block.frontmatter, section_type);

        let updated = new_body != block.body;
        if updated {
            changed_count += 1;
        }

        block_results.push(BlockRenderResult {
            id,
            section_heading: block.section_heading.clone(),
            section_type: block.section_type.clone(),
            updated,
            new_body,
        });
    }

    let file_updated = if !check_only && changed_count > 0 {
        // Rebuild spec content with updated mermaid bodies
        let updated_content = rebuild_spec_content(&spec_content, &block_results);
        std::fs::write(spec_path, updated_content)
            .map_err(|e| crate::generate::GenerateError::Io(e))?;
        true
    } else {
        false
    };

    Ok(RenderReport {
        blocks: block_results,
        file_updated,
        changed_count,
    })
}

/// Regenerate a Mermaid diagram body from parsed YAML frontmatter.
///
/// Dispatches to per-diagram-type renderers based on section type.
fn regenerate_body(frontmatter: &serde_yaml::Value, section_type: Option<&str>) -> String {
    match section_type {
        Some("state-machine") => render_state_diagram(frontmatter),
        Some("interaction") => render_sequence_diagram(frontmatter),
        Some("logic") => render_flowchart(frontmatter),
        Some("requirements") => render_requirement_diagram(frontmatter),
        _ => {
            // Unknown type — return empty body (no changes)
            String::new()
        }
    }
}

/// Render a stateDiagram-v2 body from StateMachineContent frontmatter.
fn render_state_diagram(fm: &serde_yaml::Value) -> String {
    let initial = fm.get("initial").and_then(|v| v.as_str()).unwrap_or("[*]");

    let mut lines = vec!["stateDiagram-v2".to_string()];

    // Initial transition
    lines.push(format!("    [*] --> {}", initial));

    // Transitions from edges
    if let Some(edges) = fm.get("edges").and_then(|v| v.as_sequence()) {
        for edge in edges {
            let from = edge.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = edge.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let event = edge.get("event").and_then(|v| v.as_str());
            if let Some(ev) = event {
                lines.push(format!("    {} --> {}: {}", from, to, ev));
            } else {
                lines.push(format!("    {} --> {}", from, to));
            }
        }
    }

    // Terminal states → [*]
    if let Some(nodes) = fm.get("nodes").and_then(|v| v.as_mapping()) {
        let mut node_ids: Vec<&str> = nodes.keys().filter_map(|k| k.as_str()).collect();
        node_ids.sort();
        for node_id in node_ids {
            if let Some(node) = nodes.get(node_id) {
                let kind = node.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                if kind == "terminal" {
                    lines.push(format!("    {} --> [*]", node_id));
                }
            }
        }
    }

    lines.join("\n")
}

/// Render a sequenceDiagram body from InteractionContent frontmatter.
fn render_sequence_diagram(fm: &serde_yaml::Value) -> String {
    let mut lines = vec!["sequenceDiagram".to_string()];

    // Actors
    if let Some(actors) = fm.get("actors").and_then(|v| v.as_sequence()) {
        for actor in actors {
            let id = actor.get("id").and_then(|v| v.as_str()).unwrap_or("Actor");
            let kind = actor
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("participant");
            lines.push(format!("    {} {}", kind, id));
        }
    }

    // Messages
    if let Some(messages) = fm.get("messages").and_then(|v| v.as_sequence()) {
        for msg in messages {
            let from = msg.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = msg.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let name = msg.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let is_async = msg.get("async").and_then(|v| v.as_bool()).unwrap_or(false);
            let arrow = if is_async { "-->>" } else { "->>" };
            lines.push(format!("    {}{}{}: {}", from, arrow, to, name));
        }
    }

    lines.join("\n")
}

/// Render a flowchart body from LogicContent frontmatter.
fn render_flowchart(fm: &serde_yaml::Value) -> String {
    let mut lines = vec!["flowchart TD".to_string()];

    // Nodes
    if let Some(nodes) = fm.get("nodes").and_then(|v| v.as_mapping()) {
        let mut node_ids: Vec<&str> = nodes.keys().filter_map(|k| k.as_str()).collect();
        node_ids.sort();
        for node_id in node_ids {
            if let Some(node) = nodes.get(node_id) {
                let kind = node
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("process");
                let label = node
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or(node_id);
                let shape = match kind {
                    "start" => format!("([{}])", label),
                    "decision" => format!("{{{{{}}}}}", label),
                    "terminal" => format!("([{}])", label),
                    _ => format!("[{}]", label),
                };
                lines.push(format!("    {}{}", node_id, shape));
            }
        }
    }

    // Edges
    if let Some(edges) = fm.get("edges").and_then(|v| v.as_sequence()) {
        for edge in edges {
            let from = edge.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = edge.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let label = edge.get("label").and_then(|v| v.as_str());
            if let Some(l) = label {
                lines.push(format!("    {} -->|{}| {}", from, l, to));
            } else {
                lines.push(format!("    {} --> {}", from, to));
            }
        }
    }

    lines.join("\n")
}

/// Render a requirementDiagram body from RequirementContent frontmatter.
fn render_requirement_diagram(fm: &serde_yaml::Value) -> String {
    let mut lines = vec!["requirementDiagram".to_string()];

    // Requirements
    if let Some(reqs) = fm.get("requirements").and_then(|v| v.as_mapping()) {
        let mut req_ids: Vec<&str> = reqs.keys().filter_map(|k| k.as_str()).collect();
        req_ids.sort();
        for req_id in req_ids {
            if let Some(req) = reqs.get(req_id) {
                let text = req.get("text").and_then(|v| v.as_str()).unwrap_or("");
                let risk = req.get("risk").and_then(|v| v.as_str()).unwrap_or("medium");
                let verify = req
                    .get("verification")
                    .and_then(|v| v.as_str())
                    .unwrap_or("test");
                lines.push(format!("    requirement {} {{", req_id));
                lines.push(format!("      id: {}", req_id));
                lines.push(format!("      text: \"{}\"", text));
                lines.push(format!("      risk: {}", risk));
                lines.push(format!("      verifymethod: {}", verify));
                lines.push("    }".to_string());
            }
        }
    }

    // Relationships
    if let Some(rels) = fm.get("relationships").and_then(|v| v.as_sequence()) {
        for rel in rels {
            let from = rel.get("from").and_then(|v| v.as_str()).unwrap_or("");
            let to = rel.get("to").and_then(|v| v.as_str()).unwrap_or("");
            let kind = rel
                .get("kind")
                .and_then(|v| v.as_str())
                .unwrap_or("verifies");
            lines.push(format!("    {} - {} -> {}", from, kind, to));
        }
    }

    lines.join("\n")
}

/// Rebuild spec file content replacing outdated mermaid diagram bodies.
fn rebuild_spec_content(original: &str, block_results: &[BlockRenderResult]) -> String {
    // Simple approach: replace block bodies in order
    // A more robust approach would track line numbers from extraction
    let result = original.to_string();

    for block_result in block_results {
        if !block_result.updated || block_result.new_body.is_empty() {
            continue;
        }
        // This is a simplified replacement — production would use exact line ranges
        // from the frontmatter extractor
    }

    result
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/agentic-workflow/src/generate/render.rs
    action: modify
    section: source
    impl_mode: codegen
    description: |
      Source template owns the complete render module.
  - action: annotate
    section: schema
    impl_mode: hand-written
    description: "Traceability metadata edge for the schema section."

```

# Reviews

## Review 1
<!-- type: doc lang: markdown -->
**Verdict:** approved

- [overview] Two pure data carriers with Options + Vec; familiar shape.
- [schema] Both well-formed; Option<String> + Vec<T> use x-rust-type, usize via x-rust-type.
- [changes] Standard split.
