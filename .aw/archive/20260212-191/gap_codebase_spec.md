---
change_id: 191
type: gap_codebase_spec
created_at: 2026-02-12T10:18:41.044137+00:00
updated_at: 2026-02-12T10:18:41.044137+00:00
---

# Gap Analysis: Codebase vs Spec

## Gaps: Code without Spec

- **crates/cclab-aurora/src/diagrams/requirement_plus/schema.rs**
  - severity: medium
  - description: Contains implementation for SysML v1.6 types (FunctionalRequirement, etc.), but the system specification `mermaid-plus-format.md` only mentions the diagram type without detailing these specific types.

## Gaps: Spec without Code

- **cclab-aurora/mermaid-plus-format**
  - severity: high
  - description: The change request for 'block_plus' has no corresponding specification or implementation in the codebase.
  - severity: medium
  - description: The 'requirement_plus' section in the spec is minimal and does not cover the SysML v1.6 enhancements mentioned in the change description.
