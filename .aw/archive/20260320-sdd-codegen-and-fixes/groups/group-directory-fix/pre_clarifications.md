---
change: sdd-codegen-and-fixes
group: group-directory-fix
date: 2026-03-20
status: answered
---

# Pre-Clarifications

### Q1: General
- **Question**: Merge phase reader strategy
- **Answer**: Iterate all groups and collect specs from each groups/{group-id}/specs/. Best group traceability.

### Q2: General
- **Question**: Backward compatibility with existing changes
- **Answer**: Only apply new layout to newly-created changes. Existing changes keep root-level layout.

### Q3: General
- **Question**: Payload write routing in mainthread
- **Answer**: STATE.yaml tracks the active group during spec/impl phases via active_group field — use that for routing payloads to groups/{group-id}/payloads/.

