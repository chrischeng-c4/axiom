---
id: '1717'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: td-fill-active-scope
entry: resolve
nodes:
  resolve: { kind: start, label: Resolve active spec and its Changes paths }
  scoped: { kind: process, label: Enumerate only markers under those paths }
  foreign: { kind: terminal, label: Ignore markers outside the active scope }
  local: { kind: terminal, label: Dispatch next local marker or code-check }
edges:
  - { from: resolve, to: scoped }
  - { from: scoped, to: foreign, label: foreign marker exists }
  - { from: scoped, to: local, label: local queue }
---
flowchart TD
    resolve([active TD]) --> scoped[Changes-path marker queue]
    scoped --> foreign([foreign markers ignored])
    scoped --> local([local marker or code-check])
```

Both brief and apply modes resolve the same active TD Changes paths. They enumerate and reconcile only HANDWRITE markers inside that scope. An unrelated marker can neither become the next fill target nor prevent the scoped queue from reaching code-check.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/agentic-workflow/src/cli/cb_fill.rs
    action: modify
    section: logic
    impl_mode: codegen
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
id: td-fill-active-scope-verification
requirements:
  local_marker_queue:
    id: R1
    text: "Applying a marker continues only through the active TD Changes paths and reaches code-check when that scoped queue is exhausted."
    kind: regression
    risk: high
    verify: cargo test -p agentic-workflow --lib cb_fill_apply_scopes_remaining_markers_to_active_changes -- --nocapture
---
flowchart TD
    r1[R1 local marker queue] --> cargo_test_p_agentic_workflow_lib_cb_fill_apply_scopes_remaining_markers_to_active_changes_nocapture[cargo test -p agentic-workflow --lib cb_fill_apply_scopes_remaining_markers_to_active_changes -- --nocapture]
```
