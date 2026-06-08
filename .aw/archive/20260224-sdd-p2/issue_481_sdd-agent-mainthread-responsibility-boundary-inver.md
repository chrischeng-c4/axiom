---
number: 481
title: "SDD: Agent/mainthread responsibility boundary inverted in implement and merge prompts"
state: open
labels: [enhancement, P2, crate:sdd]
---

# #481 — SDD: Agent/mainthread responsibility boundary inverted in implement and merge prompts

## Summary

The spec consistently says "return to mainthread — mainthread calls `sdd_run_change(advance_to=...)`" for phase advancement. The implementation tells the agent to call `sdd_run_change(advance_to=...)` directly.

## Spec Pattern

```
When done, return to mainthread — mainthread calls sdd_run_change(advance_to="...")
**Do NOT call sdd_update_state...**
```

## Implementation Pattern

```
5. Update STATE.yaml phase to '...'
## MCP Tools:
mcp__cclab-mcp__sdd_run_change(..., advance_to="...")
```

## Affected Prompts — Implement & Merge

| Action | Spec says | Code does | File:Line |
|--------|-----------|-----------|-----------|
| `implement_task` | mainthread advances | agent calls `advance_to="implemented"` | `implement.rs:364-370` |
| `revise_task` | mainthread advances | agent calls `advance_to="impl_revised"` | `implement.rs:474-481` |
| `begin_implementation` | — | agent calls `advance_to="implementing"` (spec omits this step) | `implement.rs:313-324` |
| `fix_merge` | mainthread advances | agent calls `advance_to="merge_revised"` | `merge.rs:191-196` |

## Affected Prompts — All 9 Review Workflow Specs

All review specs only list `sdd_read_artifact` + `sdd_write_artifact` in MCP Tools. The implementation adds a final step calling `sdd_run_change(advance_to=<phase>)`. Every review spec needs a step added.

| Review Spec | Implementation | Impl Line |
|-------------|---------------|-----------|
| `review-spec-context.md` | `explore_spec.rs` | L119,124 |
| `review-knowledge-context.md` | `explore_knowledge.rs` | L125,130 |
| `review-codebase-context.md` | `explore_codebase.rs` | L141,146 |
| `review-gap-codebase-spec.md` | `gap_codebase_spec.rs` | review block |
| `review-gap-codebase-knowledge.md` | `gap_codebase_knowledge.rs` | review block |
| `review-gap-spec-knowledge.md` | `gap_spec_knowledge.rs` | review block |
| `review-context-clarifications.md` | `clarify.rs` | L333-334 |
| `review-spec-clarifications.md` | `clarify.rs` | L441-444 |
| `review-change-proposal.md` | `proposal.rs` | review block |

### What to add in each review spec

Add a final step to the Prompt Template and MCP Tools section:

```markdown
N. Call `sdd_run_change` with `advance_to=<verdict_phase>` to advance state

## MCP Tools
...
mcp__cclab-mcp__sdd_run_change(project_path="{{project_path}}", change_id="{{change_id}}", advance_to="<phase>")
```

## Decision

The implementation pattern is arguably better (agent self-advances, fewer round-trips). Update all 9 review specs + 4 implement/merge specs to match.
