---
change_id: 199
type: spec_context
created_at: 2026-02-12T08:23:16.044843+00:00
updated_at: 2026-02-12T08:23:16.044843+00:00
iteration: 1
complexity: high
stage: spec
scanned_groups:
  - cclab-genesis
---

# Spec Context

## Relevant Specs

- **delegate-agent** (group: cclab-genesis)
  - relevance: high
  - reason: Target file
  - key sections: action enum, Verification Table
- **implement-change** (group: cclab-genesis)
  - relevance: medium
  - reason: Source of per-task actions
- **merge-change** (group: cclab-genesis)
  - relevance: medium
  - reason: Source of merge actions
- **create-spec** (group: cclab-genesis)
  - relevance: medium
  - reason: Correct artifact names for spec

## Gaps

- delegate-agent action enum missing gap-create, merge, per-task impl actions
- Verification table has wrong artifact names for spec
