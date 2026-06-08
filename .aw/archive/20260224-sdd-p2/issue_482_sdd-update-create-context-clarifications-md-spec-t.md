---
number: 482
title: "SDD: Update create-context-clarifications.md spec to match implementation"
state: open
labels: [enhancement, P2, crate:sdd]
---

# #482 — SDD: Update create-context-clarifications.md spec to match implementation

## Summary

`create-context-clarifications.md` spec is outdated — the implementation has several useful features not documented in the spec. The spec should be updated to match.

## Spec vs Implementation Diff

### Prompt Template: Spec has 6 steps, implementation has 8

| Step | Spec (`create-context-clarifications.md:58-63`) | Implementation (`clarify.rs:117-146`) |
|------|------|------|
| 1 | Analyze description for ambiguities | Same |
| **1b** | **—** | **If description references issues (#NNN) or label patterns ("all P1"): fetch via `sdd_write_artifact(artifact="issues_context", action="fetch")`** |
| 2 | Use AskUserQuestion | Same |
| 3 | Evaluate answers | Same |
| 4 | Follow-up if needed | Same |
| **5** | **—** | **MANDATORY: Ask about affected modules/scope (crates, paths, unknown, whole project)** |
| 6→6 | Call `sdd_write_artifact` | Same |
| 6→7 | Call `sdd_run_change` | Same |

### MCP Tools: Spec lists 2, implementation lists 4

| Tool | In Spec | In Impl |
|------|---------|---------|
| `sdd_write_artifact(..., artifact="context_clarifications", action="create")` | Yes | Yes |
| `sdd_run_change(...)` | Yes | Yes |
| `sdd_write_artifact(..., artifact="issues_context", action="fetch", payload={issue_refs: [...]})` | **No** | Yes |
| `sdd_write_artifact(..., artifact="issues_context", action="fetch", payload={labels: [...]})` | **No** | Yes |

### Response Fields: Spec missing 3 fields

| Field | In Spec | In Impl | Description |
|-------|---------|---------|-------------|
| `description` | No | `clarify.rs:148` | User's original description (from param or `user_input.md`) |
| `suggested_topics` | No | `clarify.rs:149` | Auto-generated from description keywords |
| `scope_hints` | No | `clarify.rs:152` | Extracted from issue labels (e.g. `crate:sdd` → `cclab-sdd`) |

### State Machine: `result_phase` wrong

| Field | Spec | Implementation |
|-------|------|---------------|
| `result_phase` | `context_clarifications_created` | `clarified` |

This is tracked by #473 — once `Inited` and `ClarificationsCreated` are added, the spec's `result_phase` is correct and the implementation needs to change.

## Proposed Spec Update

Update `create-context-clarifications.md`:

1. **Prompt Template** — Add step 1b (issue fetch) and step 5 (mandatory scope question) to match implementation
2. **MCP Tools** — Add the two `issues_context` fetch tool calls
3. **State Machine** — Keep `result_phase: context_clarifications_created` (correct per #473)
4. **Payload Schema** — Add note about `scope_hints` and `suggested_topics` in response
5. **Requirements** — Add R3 for scope collection, R4 for issue fetch from description

## Related Issues

- #473 — `Clarified` → `Inited` + `ClarificationsCreated` phase split
- #474 — Scope name prefix inconsistency
- #480 — Undocumented patterns (includes issue fetch, scope hints)
