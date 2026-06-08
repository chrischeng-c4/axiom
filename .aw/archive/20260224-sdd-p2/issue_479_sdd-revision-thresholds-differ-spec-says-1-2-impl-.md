---
number: 479
title: "SDD: Revision thresholds differ — spec says 1/2, impl uses 2/4 for clarifications"
state: open
labels: [bug, P2, crate:sdd]
---

# #479 — SDD: Revision thresholds differ — spec says 1/2, impl uses 2/4 for clarifications

## Summary

The revision thresholds for clarification phases differ between spec and implementation.

## Spec (revise-context-clarifications.md:31-33, revise-spec-clarifications.md:31-33)

```yaml
thresholds:
  reviewed: 1
  rejected: 2
```

## Implementation

- `clarify.rs:274` — `ClarificationsReviewed`: auto-approve at `revision_count >= 2`
- `clarify.rs:291` — `ClarificationsRejected`: `mainthread_must_fix` at `revision_count >= 4`
- `clarify.rs:384,401` — Same 2/4 thresholds for PostClarifications

## Main README Spec (README.md:166-169)

Uses `< 2` / `>= 2` for reviewed and `< 4` / `>= 4` for rejected — **matches implementation**, contradicts workflow-level spec files.

## Root Cause

The per-workflow spec files (`revise-*.md`) have lower thresholds (1/2) while the main README has higher thresholds (2/4) that match the implementation. The per-workflow specs were not updated when the README was written.

## Action

Update `revise-context-clarifications.md` and `revise-spec-clarifications.md` thresholds to `reviewed: 2, rejected: 4` to match README and implementation.
