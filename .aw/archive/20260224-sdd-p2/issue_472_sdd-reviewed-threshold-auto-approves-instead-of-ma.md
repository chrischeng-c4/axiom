---
number: 472
title: "SDD: REVIEWED threshold auto-approves instead of mainthread evaluation"
state: open
labels: [bug, P2, crate:sdd]
---

# #472 — SDD: REVIEWED threshold auto-approves instead of mainthread evaluation

## Summary

When `revision_count >= 2` at REVIEWED verdict, the spec says mainthread should evaluate whether to approve or keep fixing. The implementation auto-approves without giving mainthread a choice.

## Spec (README.md:166-167, 214-215, 256-257)

> `*_reviewed | revision_count >= 2` → `revise_*` — Mainthread evaluates: revise or `sdd_run_change(advance_to="*_approved")`

## Implementation

All affected handlers return an immediate auto-approve:
```rust
if sm.revision_count("...") >= 2 {
    return Ok(json!({ "advance_to": "..._approved" }));
}
```

## Affected Files

- `clarify.rs:273-289` (ClarificationsReviewed)
- `clarify.rs:384-397` (PostClarificationsReviewed)
- `gap_codebase_spec.rs:20-35`
- `gap_codebase_knowledge.rs:20-35`
- `gap_spec_knowledge.rs:20-35`
- `proposal.rs:38-42` (silently proceeds to spec phase)
- `spec.rs:75-88` (silently falls through)

## Decision Needed

- **Option A**: Update spec to document auto-approve behavior (simpler, current behavior may be intentional)
- **Option B**: Change implementation to return a prompt for mainthread evaluation before approving
