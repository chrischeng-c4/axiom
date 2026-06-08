---
change_id: sdd-p1
type: spec_context
created_at: 2026-02-23T14:10:51.716918+00:00
updated_at: 2026-02-23T14:10:51.716918+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-sdd
---

# Spec Context

## Relevant Specs

- **run-change-skill**
  - relevance: high
- **run-change-dag-loop**
  - relevance: high
- **run-change-diagrams**
  - relevance: high
- **config**
  - relevance: medium
- **spec-ir-evaluation**
  - relevance: medium
- **spec-ir-yaml-schema**
  - relevance: low

## Dependencies

- run-change-skill
- run-change-dag-loop
- run-change-diagrams
- config
- spec-ir-evaluation
- spec-ir-yaml-schema

## Gaps

- behavior_mismatch: Code auto-approves when revision threshold met vs mainthread evaluation in spec
- naming_inconsistency: mixed naming (explore_spec, gap_codebase_spec) vs spec create_{artifact}
- missing_implementation: Post-clarification phases are described in specs but skipped in code
- content_mismatch: Review checklists in code missing or mismatched items vs specs
