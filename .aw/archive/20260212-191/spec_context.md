---
change_id: 191
type: spec_context
created_at: 2026-02-12T10:14:08.884692+00:00
updated_at: 2026-02-12T10:14:08.884692+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-aurora
---

# Spec Context

## Relevant Specs

- **mermaid-plus-format** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the Mermaid+ format and listed supported types.
  - key sections: Supported Diagram Types, Format
- **mermaid-plus-conversion** (group: cclab-aurora)
  - relevance: high
  - reason: Defines the algorithm for converting structured input to Mermaid+ output.
  - key sections: Requirements, Implementation Details
- **architecture** (group: cclab-aurora)
  - relevance: medium
  - reason: Lists supported diagram types in the Aurora engine.
  - key sections: Mermaid Diagram Types
- **spec-validator** (group: cclab-aurora)
  - relevance: medium
  - reason: Provides the pattern for semantic validation of specifications.
  - key sections: Requirements

## Dependencies

- cclab-aurora/mermaid-plus-format
- cclab-aurora/mermaid-plus-conversion

## Gaps

- block_plus is missing from both codebase and specifications.
- requirement_plus spec in mermaid-plus-format is minimal and lacks detail on SysML v1.6 specific enhancements.
