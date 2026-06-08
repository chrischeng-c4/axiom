---
id: 191
type: proposal
version: 2
created_at: 2026-02-12T10:21:47.924046+00:00
updated_at: 2026-02-12T10:21:47.924046+00:00
iteration: 1
scope: minor
spec_plan:
  - id: block-plus-spec
    title: "Mermaid+ Block Diagram Specification"
    depends: []
    context_refs:
      codebase: ["crates/cclab-aurora/src/diagrams/flowchart_plus/schema.rs"]
      spec: ["Supported Diagram Types"]
      knowledge: ["Mermaid+ Output Format"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 0 }
    affected_code: ["crates/cclab-aurora/src/diagrams/block_plus/", "crates/cclab-aurora/src/diagrams/mod.rs"]
  - id: requirement-plus-enhancement
    title: "Enhanced Requirement+ Specification (SysML v1.6)"
    depends: []
    context_refs:
      codebase: ["crates/cclab-aurora/src/diagrams/requirement_plus/schema.rs"]
      spec: ["Supported Diagram Types"]
      knowledge: ["aurora_generate_*_plus"]
    gap_repairs:
      - { source: gap_codebase_spec, gap_index: 1 }
    affected_code: ["crates/cclab-aurora/src/diagrams/requirement_plus/", "crates/cclab-aurora/src/mcp/tools.rs"]
history:
  - timestamp: 2026-02-12T10:21:47.924046+00:00
    agent: "mcp"
    tool: "create_proposal"
    action: "created"
---

<proposal>

# Spec Navigation Map: 191

## Scope Overview (Mindmap)

```mermaid
mindmap
  root((191))  
    Aurora Engine
      block_plus generator implementation
      requirement_plus SysML v1.6 enhancement
      Mermaid+ format adherence
    MCP Layer
      aurora_generate_block_plus tool schema
      Enhanced aurora_generate_requirement_plus tool schema
    Validation
      block_plus frontmatter validation
      requirement_plus SysML v1.6 validation
```

## Spec Dependency Graph (Block Diagram)

```mermaid
block-beta
  columns 3

  block_plus_spec["block-plus-spec\n codebase: crates/cclab-aurora/src/diagrams/flowchart_plus/schema.rs\n gaps: codebase_spec#0"]
  requirement_plus_enhancement["requirement-plus-enhancement\n codebase: crates/cclab-aurora/src/diagrams/requirement_plus/schema.rs\n gaps: codebase_spec#1"]

```

## Spec Execution Order

1. **block-plus-spec** — Mermaid+ Block Diagram Specification
   - code: crates/cclab-aurora/src/diagrams/block_plus/, crates/cclab-aurora/src/diagrams/mod.rs
2. **requirement-plus-enhancement** — Enhanced Requirement+ Specification (SysML v1.6)
   - code: crates/cclab-aurora/src/diagrams/requirement_plus/, crates/cclab-aurora/src/mcp/tools.rs

</proposal>
