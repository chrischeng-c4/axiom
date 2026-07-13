---
id: mamba-language-core-restore-class-comprehension-cells
summary: (fill)
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-generator-expression-closure-context
entry: generator expression after a list-comprehension closure
nodes:
  list_closure: { kind: start, label: prior list-comprehension closure remains usable }
  construct: { kind: process, label: create generator wrapper with capture context }
  drain: { kind: process, label: list drains generator through iterator protocol }
  bind: { kind: process, label: bind j for each range item }
  closure: { kind: process, label: create lambda over shared j cell }
  yield: { kind: process, label: yield through pushed caller context }
  complete: { kind: process, label: complete through the same caller context }
  result: { kind: terminal, label: three closures observe final j equals 2 }
  boundary: { kind: terminal, label: list comprehension, default args, and class cells remain unchanged }
edges:
  - { from: list_closure, to: construct }
  - { from: construct, to: drain }
  - { from: drain, to: bind }
  - { from: bind, to: closure }
  - { from: closure, to: yield }
  - { from: yield, to: drain }
  - { from: drain, to: complete }
  - { from: complete, to: result }
  - { from: construct, to: boundary }
---
flowchart TD
    list_closure([prior list-comprehension closure remains usable]) --> construct[create generator wrapper with capture context]
    construct --> drain[list drains generator through iterator protocol]
    drain --> bind[bind j for each range item]
    bind --> closure[create lambda over shared j cell]
    closure --> yield[yield through pushed caller context]
    yield --> drain
    drain --> complete[complete through the same caller context]
    complete --> result([three closures observe final j equals 2])
    construct --> boundary([list comprehension, default args, and class cells remain unchanged])
```

The generator runtime must preserve one pushed caller context from the wrapper's first resume until each yield or terminal completion switches back. It must not reset or pop that context while a previously materialized list-comprehension closure exists. The generator-expression wrapper owns the loop-variable capture cell; every yielded lambda references that same cell, so consuming the expression through `list()` produces three closures whose later calls return the final value `2`. This change is limited to generator-expression lowering and generator resume/yield context management; it must preserve existing list-comprehension late binding, default-argument early binding, class `__class__` cells, imports, and built-in-library behavior.
