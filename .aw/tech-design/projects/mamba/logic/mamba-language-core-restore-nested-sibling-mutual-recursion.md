---
id: mamba-language-core-restore-nested-sibling-mutual-recursion
summary: Restore CPython-equivalent mutual recursion for nested sibling functions without changing closure-cell ownership semantics.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-nested-sibling-recursion-contract
entry: nested function definition
nodes:
  prescan: { kind: process, label: collect all cell backed binding events }
  closure: { kind: process, label: closure captures peer cell before textual bind }
  cell: { kind: process, label: pre vivify empty peer cell }
  bind: { kind: process, label: later sibling definition mutates same cell }
  call: { kind: terminal, label: both sibling calls resolve callable peer }
  boundary: { kind: terminal, label: unrelated free variable reads are unchanged }
edges:
  - { from: prescan, to: closure }
  - { from: closure, to: cell }
  - { from: cell, to: bind }
  - { from: bind, to: call }
  - { from: prescan, to: boundary }
---
flowchart TD
    prescan[collect all cell backed binding events] --> closure[closure captures peer cell before textual bind]
    closure --> cell[pre vivify empty peer cell]
    cell --> bind[later sibling definition mutates same cell]
    bind --> call([both sibling calls resolve callable peer])
    prescan --> boundary([unrelated free variable reads are unchanged])
```

Replace the Let-only pre-vivification inventory in `lower::hir_to_mir` with an inventory of cell-backed binding events. It includes a `FuncDefPlaceholder` bind symbol when that symbol is in `cell_override`, but continues to exclude a nested function body. Before the earlier sibling closure is created, the existing pre-vivification loop creates an empty cell for its later sibling. `bind_runtime_value` then observes an initialized cell and calls `mb_capture_cell_set_id`, preserving the captured handle rather than replacing it. Existing Let, parameter, and nonlocal behavior remains unchanged.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/lower/hir_to_mir.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-nested-sibling-recursion
    tracker: "#1477"
    reason: Existing lowering only pre-vivifies captured Let targets and therefore disconnects a closure from a later sibling function binding.
  - path: projects/mamba/tests/cpython/_regression/core/closure_capture/nested_sibling_mutual_recursion.py
    action: create
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-nested-sibling-recursion-tests
    tracker: "#1477"
    reason: A one-case CPython oracle fixture must prove both nested siblings resolve the same callable cell after all definitions execute.
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-nested-sibling-recursion-verification
requirements:
  closure_boundaries:
    id: R3
    text: "Forward capture and per-call closure identity remain correct while sibling bindings are fixed."
    kind: regression
    risk: medium
    verify: conformance::_regression/core/closure_capture/closure_late_binding.py
  first_call_order:
    id: R2
    text: "Either sibling may execute first and recursively call its peer."
    kind: regression
    risk: high
    verify: conformance::_regression/core/closure_capture/nested_sibling_mutual_recursion.py
  sibling_binding:
    id: R1
    text: "Nested sibling functions resolve each other through the enclosing invocation scope instead of a None placeholder."
    kind: functional
    risk: high
    verify: conformance::_regression/core/closure_capture/nested_sibling_mutual_recursion.py
---
flowchart TD
    r1[R1 sibling binding] --> conformance_regression_core_closure_capture_nested_sibling_mutual_recursion_py[conformance::_regression/core/closure_capture/nested_sibling_mutual_recursion.py]
    r2[R2 first call order] --> conformance_regression_core_closure_capture_nested_sibling_mutual_recursion_py
    r3[R3 closure boundaries] --> conformance_regression_core_closure_capture_closure_late_binding_py[conformance::_regression/core/closure_capture/closure_late_binding.py]
```
