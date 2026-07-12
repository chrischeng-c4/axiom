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
