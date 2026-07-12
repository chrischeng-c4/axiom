---
id: mamba-strict-type-expose-jit-owner-or-none-runtime-helper
summary: Runtime-owned typed Int owner-or-None projection for JIT consumers.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: mamba-strict-type-runtime-owner-or-none
entry: typed_int_result
nodes:
  typed_int_result: { kind: start, label: "typed Int result data" }
  runtime_projection: { kind: process, label: "runtime classifies owner-or-None" }
  bigint: { kind: decision, label: "live BigInt result" }
  owner: { kind: process, label: "return borrowed BigInt owner" }
  none: { kind: process, label: "return None sidecar" }
  jit: { kind: terminal, label: "JIT consumes declared sidecar" }
edges:
  - { from: typed_int_result, to: runtime_projection }
  - { from: runtime_projection, to: bigint }
  - { from: bigint, to: owner, label: "yes" }
  - { from: bigint, to: none, label: "no: raw or inline Int" }
  - { from: owner, to: jit }
  - { from: none, to: jit }
---
flowchart TD
    result([typed Int result]) --> classify[runtime owner-or-None projection]
    classify --> bigint{live BigInt?}
    bigint -- yes --> owner[borrowed BigInt owner]
    bigint -- no --> none[None sidecar]
    owner --> jit([JIT consumes explicit sidecar])
    none --> jit
```

The runtime, not JIT lowering, classifies the produced value. It returns the original `MbValue` only for a live BigInt and `MbValue::none()` for raw integers, inline boxed integers, and any pointer-shaped raw payload. The helper grants no new reference; a JIT producer chooses its declared fresh or borrowed transaction action after receiving this sidecar.

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: mamba-strict-type-runtime-owner-or-none-verification
requirements:
  jit_registration:
    id: R3
    text: "The helper is registered as a callable runtime symbol for JIT lowering without exposing payload inference to consumers."
    kind: regression
    risk: high
    verify: runtime::symbols::tests::runtime_typed_int_owner_projection_is_explicit
  raw_collision:
    id: R2
    text: "Raw and inline integer results, including pointer-shaped raw payloads, produce an explicit None sidecar."
    kind: regression
    risk: high
    verify: runtime::symbols::tests::runtime_typed_int_owner_projection_is_explicit
  runtime_projection:
    id: R1
    text: "The registered runtime owner projection returns a borrowed BigInt owner only for a live BigInt result."
    kind: functional
    risk: high
    verify: runtime::symbols::tests::runtime_typed_int_owner_projection_is_explicit
---
flowchart TD
    r1[R1 runtime projection] --> runtime_symbols_tests_runtime_typed_int_owner_projection_is_explicit[runtime::symbols::tests::runtime_typed_int_owner_projection_is_explicit]
    r2[R2 raw collision] --> runtime_symbols_tests_runtime_typed_int_owner_projection_is_explicit
    r3[R3 jit registration] --> runtime_symbols_tests_runtime_typed_int_owner_projection_is_explicit
```
