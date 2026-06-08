---
number: 473
title: "SDD: StatePhase enum missing ClarificationsCreated — uses legacy Clarified instead"
state: open
labels: [bug, P2, crate:sdd]
---

# #473 — SDD: StatePhase enum missing ClarificationsCreated — uses legacy Clarified instead

## Summary

`Clarified` is a legacy v1 phase being used as a catch-all for two semantically distinct states:

1. **After `init_change`** — change directory just created, no clarifications collected yet
2. **After `create_context_clarifications`** — clarifications actually gathered from user

This violates the naming convention used by every other phase group and makes routing fragile.

## Current Behavior (wrong)

```
init_change              → Clarified   ← only created directory, nothing clarified
create_clarifications    → Clarified   ← actually collected clarifications
```

Both land on the same phase, distinguished only by filesystem state (does `context_clarifications.md` exist?).

## Expected Behavior

```
init_change              → Inited                          ← new phase
create_clarifications    → ContextClarificationsCreated    ← spec already defines this
```

This matches the naming convention of all other groups:
- `spec_context_created`, `knowledge_context_created`, `codebase_context_created`
- `gap_codebase_spec_created`, `proposal_created`, `spec_created`

## Spec References

- `README.md:566-567` — variant table lists `clarifications_{created,reviewed,revised,approved,rejected}`
- `README.md:583` — marks `clarified` as deprecated (legacy v1)
- `init-change.md:25` — `result_phase: clarified` (should become `inited`)

## Implementation (frontmatter.rs:829-835)

```rust
Clarified,               // legacy catch-all, should be split
ClarificationsReviewed,
ClarificationsRevised,
ClarificationsApproved,
ClarificationsRejected,
// Missing: Inited, ContextClarificationsCreated (or ClarificationsCreated)
```

## Changes Required (8-location checklist)

| # | File | Change |
|---|------|--------|
| 1 | `models/frontmatter.rs` | Add `Inited` and `ClarificationsCreated` variants to StatePhase enum; update `is_decide_phase()`, Serialize, Deserialize |
| 2 | `models/change.rs` | Add to `WorkflowArtifact` enum + config fields |
| 3 | `mcp/tools/workflow_common.rs` | Add to `phase_to_string()` |
| 4 | `mcp/tools/run_change/mod.rs` | `phase_order()` + `validate_transition()` — insert `Inited` before `ClarificationsCreated` |
| 5 | `mcp/tools/run_change/mod.rs` | `route()`: add `StatePhase::Inited` arm → `clarify::handle()`, change `Clarified` arm to `ClarificationsCreated` → `handle_clarifications_review()` |
| 6 | `cli/status.rs` | `phase_icon`, `phase_color`, test data |
| 7 | `mcp/tools/context.rs` | State transition match |
| 8 | `services/file_service.rs` | `read_file()` match |

## Additional Changes

| File | Change |
|------|--------|
| `mcp/tools/init_change.rs:73` | `"phase": "clarified"` → `"phase": "inited"` |
| `mcp/tools/phase_transition.rs` | Add valid transitions: `Inited → ClarificationsCreated`, `Inited → Clarified` (backward compat) |
| `init-change.md:25` | `result_phase: clarified` → `result_phase: inited` |
| `README.md:583` | Remove `clarified` from standalone variants or mark as fully deprecated |

## Migration

`Clarified` should remain in the enum (deserializable) for backward compatibility with existing STATE.yaml files, but should no longer be produced by any code path. Add a comment `// @deprecated — use Inited or ClarificationsCreated`.
