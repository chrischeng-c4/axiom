---
number: 468
title: "SDD: Revise actions use action=\"create\" instead of action=\"revise\" in write_artifact calls"
state: open
labels: [bug, P1, crate:sdd]
---

# #468 — SDD: Revise actions use action="create" instead of action="revise" in write_artifact calls

## Summary

Multiple revise prompts instruct the agent to call `sdd_write_artifact` with `action="create"` instead of `action="revise"`. If `sdd_write_artifact` handles these differently (e.g., overwrite vs. append, different phase transitions), this is a functional bug.

## Affected Files

| Spec says | Implementation does | File:Line |
|-----------|-------------------|-----------|
| `action="revise"` | `action="create"` | `clarify.rs:360` (revise_context_clarifications) |
| `action="revise"` | `action="create"` | `proposal.rs:171` (revise_change_proposal) |

## Expected Behavior

Revise prompts should instruct agents to call `sdd_write_artifact(..., action="revise", ...)` as specified.
