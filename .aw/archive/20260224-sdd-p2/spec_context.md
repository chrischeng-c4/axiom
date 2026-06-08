---
change_id: sdd-p2
type: spec_context
created_at: 2026-02-23T16:06:17.650933+00:00
updated_at: 2026-02-23T16:06:17.650933+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-sdd
---

# Spec Context

## Relevant Specs

- **cclab-sdd/config**
  - relevance: high
- **cclab-sdd/spec-ir-evaluation**
  - relevance: high
- **cclab-sdd/spec-ir-yaml-schema**
  - relevance: medium

## Dependencies

- main_spec:cclab-sdd/config
- main_spec:cclab-sdd/spec-ir-evaluation
- main_spec:cclab-sdd/spec-ir-yaml-schema

## Gaps

- SpecIR files lack context cascade logic (injecting prior summaries) present in implementation.
- The 'executor' field defined in specs is not emitted by the implementation in normal-path responses.
- Post-clarification phases are missing from the StatePhase enum but present in specs.
- Task generation uses sdd_write_artifact instead of the dedicated sdd_generate_tasks tool defined in specs.
