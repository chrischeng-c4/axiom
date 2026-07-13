---
id: mamba-language-core-materialize-generator-expression-closures
summary: Preserve distinct MIR entry symbols for generator bodies and nested lambda closures.
fill_sections: [logic]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-generator-expression-closure-symbol-isolation
entry: generator expression whose body yields a lambda
nodes:
  generator: { kind: start, label: synthesize generator body entry symbol }
  lambda: { kind: process, label: synthesize nested lambda entry symbol }
  unique: { kind: decision, label: symbols are distinct }
  emit: { kind: process, label: emit both MIR bodies without overwrite }
  consume: { kind: process, label: list consumes yielded closures }
  result: { kind: terminal, label: three closures observe final j equals 2 }
  boundary: { kind: terminal, label: async bodies, list comprehensions, and ordinary lambdas remain unchanged }
edges:
  - { from: generator, to: lambda }
  - { from: lambda, to: unique }
  - { from: unique, to: emit, label: yes }
  - { from: emit, to: consume }
  - { from: consume, to: result }
  - { from: unique, to: boundary, label: no collision with sibling namespaces }
---
flowchart TD
    generator([synthesize generator body entry symbol]) --> lambda[synthesize nested lambda entry symbol]
    lambda --> unique{symbols are distinct}
    unique -- yes --> emit[emit both MIR bodies without overwrite]
    emit --> consume[list consumes yielded closures]
    consume --> result([three closures observe final j equals 2])
    unique -- no collision with sibling namespaces --> boundary([async bodies, list comprehensions, and ordinary lambdas remain unchanged])
```

`lower_generator_function` currently derives a generator-body `SymbolId` in the same 4,000,000 range used by lambda lowering. A generator expression that yields a lambda can therefore emit two MIR bodies with the same name; the later body overwrites the lambda entry in codegen, so invoking the yielded closure re-enters a generator body without a caller context. Allocate generator body symbols from a disjoint namespace. The yielded lambdas then retain their own entry body and their shared generator loop cell, so consuming the expression through `list()` produces three closures whose later calls return `2`. This is limited to MIR synthetic-symbol allocation; generator runtime switching, list-comprehension late binding, default arguments, class cells, imports, and built-in libraries remain unchanged.

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: projects/mamba/src/lower/hir_to_mir.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-generator-expression-symbol-isolation
    tracker: "#1490"
    reason: Generator-body and lambda MIR entries currently share a synthetic SymbolId namespace and can overwrite one another during code generation.
  - path: projects/mamba/src/driver/tests/generator_conformance.rs
    action: modify
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-generator-expression-symbol-isolation-tests
    tracker: "#1490"
    reason: The JIT generator conformance suite needs a focused regression for a generator expression consumed after a list-comprehension closure.
  - path: projects/mamba/tests/cpython/_regression/core/comprehension_scope/generator_expression_closure_context.py
    action: create
    section: logic
    impl_mode: hand-written
    gap: missing-generator:mamba-generator-expression-closure-oracle
    tracker: "#1490"
    reason: A one-case CPython oracle fixture must pin closure late binding through generator expression iteration after a prior list-comprehension closure.
```
