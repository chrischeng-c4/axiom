---
id: mamba-language-core-restore-nested-sibling-mutual-recursion
summary: Restore CPython-equivalent mutual recursion for nested sibling functions without changing closure-cell ownership semantics.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-nested-sibling-recursion-applicability
entry: nested sibling function call
nodes:
  outer: { kind: start, label: outer invocation }
  bind: { kind: process, label: allocate sibling bindings in one closure scope }
  first: { kind: process, label: is_even resolves is_odd }
  second: { kind: process, label: is_odd resolves is_even }
  result: { kind: terminal, label: mutual recursion returns CPython equivalent result }
  reject: { kind: terminal, label: unbound peer must not become None }
edges:
  - { from: outer, to: bind }
  - { from: bind, to: first }
  - { from: bind, to: second }
  - { from: first, to: result }
  - { from: second, to: result }
  - { from: first, to: reject, label: missing peer binding }
---
flowchart TD
    outer([outer invocation]) --> bind[allocate sibling bindings in one closure scope]
    bind --> first[is_even resolves is_odd]
    bind --> second[is_odd resolves is_even]
    first --> result([mutual recursion returns CPython equivalent result])
    second --> result
    first -- missing peer binding --> reject([unbound peer must not become None])
```

Applicability is confined to nested function definitions that share an enclosing invocation. The existing closure runtime already supports per-call cells; this slice must prove the lowering creates and wires both sibling placeholders into that same invocation scope before either closure body can resolve its peer. Module-level recursion, class bodies, and general dynamic dispatch are not touched.

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
