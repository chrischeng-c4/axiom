---
id: '1717'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: td-fill-active-scope-contract
entry: load
nodes:
  load: { kind: start, label: Load active issue and TD spec }
  scope: { kind: process, label: Parse TD Changes into marker scope }
  apply: { kind: process, label: Replace requested local marker body }
  remaining: { kind: decision, label: Scoped queue has another marker }
  next: { kind: terminal, label: Lock and dispatch the next local marker }
  check: { kind: terminal, label: Mark filled and dispatch code-check }
edges:
  - { from: load, to: scope }
  - { from: scope, to: apply }
  - { from: apply, to: remaining }
  - { from: remaining, to: next, label: yes }
  - { from: remaining, to: check, label: no }
---
flowchart TD
    load([load issue + TD]) --> scope[parse Changes scope]
    scope --> apply[apply local payload]
    apply --> remaining{scoped markers remain?}
    remaining -->|yes| next([dispatch next local marker])
    remaining -->|no| check([dispatch code-check])
```

`run_apply` derives the active TD spec exactly as brief mode does, parses its Changes paths, and uses `markers_for_td_changes` both to locate the requested marker and to compute `remaining`. If the local queue is empty after the replacement, it advances to `cb_filled` and dispatches code-check even when unrelated unfilled markers exist elsewhere.
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/cb_fill.rs
    action: modify
    section: logic
    impl_mode: hand-written
  - path: apps/agentic-workflow/tech-design/surface/interfaces/src/cb_fill.md
    action: modify
    section: logic
    impl_mode: codegen
  - path: apps/agentic-workflow/CAPABILITIES.md
    action: modify
    section: logic
    impl_mode: hand-written
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: scope-td-fill-re-enumeration-to-the-active-work-item-verification
requirements:
  active_td_marker_scope:
    id: R1
    text: "Applying a marker only queues unresolved HANDWRITE markers whose source paths are declared by the active TD Changes section; a marker in an unrelated app cannot prevent code-check for this work item."
    kind: regression
    risk: high
    verify: cb_fill_apply_scopes_remaining_markers_to_active_changes
---
flowchart TD
    r1[R1 active td marker scope] --> cb_fill_apply_scopes_remaining_markers_to_active_changes[cb_fill_apply_scopes_remaining_markers_to_active_changes]
```
