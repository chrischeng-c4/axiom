---
change_id: 191
type: knowledge_context
created_at: 2026-02-12T10:16:22.709341+00:00
updated_at: 2026-02-12T10:16:22.709341+00:00
iteration: 2
complexity: high
stage: knowledge
scanned_categories:
  - 40-mcp
  - changelogs
---

# Knowledge Context

## Relevant Documents

- **cclab/knowledge/40-mcp/dynamic-config.md**
  - summary: Mentions 8 Mermaid diagram tools and their classification as part of the 'Plan' phase.
  - relevant sections: Overview

## Patterns

- **aurora_generate_*_plus** (source: cclab/specs/cclab-aurora/mermaid-plus-format.md)
  - Naming convention for Mermaid+ generators. Example: aurora_generate_state_plus, aurora_generate_flowchart_plus.
- **Mermaid+ Output Format** (source: cclab/specs/cclab-aurora/mermaid-plus-conversion.md)
  - Combined output containing code fence, frontmatter, diagram syntax, and validation warnings. Example: ```mermaid\n---\nid: x\n---\nstateDiagram-v2\n...```\n<!-- Validation Warnings: ... -->

## Pitfalls

- Mermaid+ format requires YAML frontmatter to be placed INSIDE the mermaid code block, not outside. External Mermaid documentation might suggest otherwise, but cclab-aurora rigorously follows the internal specification.
