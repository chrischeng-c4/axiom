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
