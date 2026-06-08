---
number: 480
title: "SDD: Undocumented actions and missing spec for executor field, sdd_run_change in review prompts"
state: open
labels: [enhancement, P2, crate:sdd]
---

# #480 — SDD: Undocumented actions and missing spec for executor field, sdd_run_change in review prompts

## Summary

Several systematic patterns in the implementation are not documented in the spec.

## 1. Undocumented Action Labels

These actions are returned by the implementation but not in the spec's action enum (README.md:74-91):

| Action | Source | Description |
|--------|--------|-------------|
| `init_change` | `mod.rs:207` | Returned when change dir doesn't exist |
| `clarify` | `clarify.rs:100`, `dag_loop.rs:47` | Spec lists `create_clarifications`, not `clarify` |
| `confirm_understanding` | `clarify.rs:156,279` | Auto-approve confirmation |
| `transition_decided` | `clarify.rs:498` | Transition to decided phase |
| `mainthread_fix` | `helpers.rs:148` | Terminal fix required |
| `complete` | `mod.rs:415` | Terminal state |

## 2. `executor` Field Not Emitted

All 9+ spec state machines define `executor` chains (e.g., `[codex:balanced, mainthread]`). Normal-path responses in the implementation never emit the `executor` field — only auto-approve bypass paths do.

## 3. DAG Counter Increment Ownership

| Counter | Spec says incremented by | Implementation increments via |
|---------|--------------------------|------------------------------|
| `clarify_index` | `sdd_write_artifact(artifact="clarifications")` | `sdd_run_change(last_action="dag_clarify_next")` |
| `context_index` | `run_change` automatically at `codebase_context_approved` | Caller must pass `last_action="dag_context_next"` |

## 4. `validate_transition()` Catch-All

Implementation has an undocumented 3-step skip rule: any transition where target is ≤ 3 positions ahead in `phase_order()` is allowed, even without an explicit match arm.

## 5. `sdd_update_state` Lifecycle

Spec says deprecated/moved to `phase_transition.rs`. The file `state_update.rs` (635 lines) still fully functions and is registered.

## 6. Explore Create Specs — Impl Has Undocumented Features

### 6a. `create-spec-context.md` — missing `codebase_paths`/`knowledge_refs` passthrough

Implementation (`explore_spec.rs:76-78`) adds a step:
> "If a spec has `codebase_paths` or `knowledge_refs` in its YAML frontmatter, include them prominently in the spec_context output"

Also adds a 6th review checklist item (`explore_spec.rs:97-104`):
> "codebase_paths and knowledge_refs from specs surfaced (if present)"

Spec has 5 checklist items, impl has 6.

### 6b. `create-knowledge-context.md` — missing cascade injection

Implementation (`explore_knowledge.rs:63-65`) reads `spec_context.md` summary and injects it as "Prior Context" into the prompt. Spec has no mention of this.

### 6c. `create-codebase-context.md` — missing double cascade + conditional prism

Implementation (`explore_codebase.rs:68-73`) injects both `spec_context.md` and `knowledge_context.md` summaries. Also:
- Conditional prism tool usage with "If data-model/algorithm" and "If bug-fix/refactoring" guards (`explore_codebase.rs:93-99`)
- `CRITICAL` block not in spec (`explore_codebase.rs:99-100`)
- `prism_pdg` and `prism_diagnostics` absent from impl's MCP Tools block (spec lists them)

### 6d. All 3 explore create specs — emit `review_checklist` at create time

Implementation emits `review_checklist` in the JSON response during the CREATE phase as guidance for the creating agent. Spec only defines review checklists in the review workflow files, not in create.

- `explore_spec.rs:97-104`
- `explore_knowledge.rs:71-77`
- `explore_codebase.rs:79-85`

## Action

Update specs to document all of the above, or decide to remove undocumented features from implementation.
