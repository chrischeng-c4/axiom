---
id: jet-codegen-openapi-named-types
summary: Emit named per-operation XxxData (grouped path/query/headers/body) and XxxResponse type aliases for jet codegen openapi; client and hooks take a grouped data argument. No runtime classes.
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "jet codegen openapi is part of the Rust-native frontend toolchain surface for generating typed frontend API clients."
fill_sections: [logic, unit-test]
---

# TD: jet/codegen-openapi-named-types

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-named-types-logic
entry: start
nodes:
  start: { kind: start, label: "jet codegen openapi <spec> --out <dir>" }
  parse: { kind: process, label: "Parse spec and build per-operation plans" }
  emit_components: { kind: process, label: "Emit component schema interfaces" }
  op_loop: { kind: process, label: "For each operation, group inputs into path/query/headers/body" }
  has_inputs: { kind: decision, label: "Operation has any inputs?" }
  emit_data: { kind: process, label: "Emit XxxData type (only present keys); emit XxxResponse type" }
  emit_resp_only: { kind: process, label: "Emit XxxResponse type only (no XxxData)" }
  emit_client: { kind: process, label: "Emit client fn: fnName(data: XxxData) returns Promise<XxxResponse>" }
  emit_hooks: { kind: process, label: "Emit hooks using XxxData and XxxResponse" }
  done: { kind: terminal, label: "Ok (exit 0)" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: emit_components }
  - { from: emit_components, to: op_loop }
  - { from: op_loop, to: has_inputs }
  - { from: has_inputs, to: emit_data, label: "yes" }
  - { from: has_inputs, to: emit_resp_only, label: "no" }
  - { from: emit_data, to: emit_client }
  - { from: emit_resp_only, to: emit_client }
  - { from: emit_client, to: emit_hooks }
  - { from: emit_hooks, to: done }
---
flowchart TD
    start([jet codegen openapi spec out dir]) --> parse[Parse spec and build per operation plans]
    parse --> emit_components[Emit component schema interfaces]
    emit_components --> op_loop[Group each operation inputs into path query headers body]
    op_loop --> has_inputs{Operation has any inputs?}
    has_inputs -->|yes| emit_data[Emit XxxData with present keys and XxxResponse]
    has_inputs -->|no| emit_resp_only[Emit XxxResponse only]
    emit_data --> emit_client[Emit client fn data XxxData returns Promise XxxResponse]
    emit_resp_only --> emit_client
    emit_client --> emit_hooks[Emit hooks using XxxData and XxxResponse]
    emit_hooks --> done([Ok exit 0])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-named-types-unit-test
requirements:
  R1:
    text: "Each operation with inputs emits an XxxData grouping present path/query/headers/body keys."
    risk: high
    verify: unit
  R2:
    text: "Each operation emits an XxxResponse aliasing the 2xx/default response type or void."
    risk: high
    verify: unit
  R3:
    text: "Client signature is fnName(data: XxxData) returning Promise<XxxResponse>; no inputs means no arg."
    risk: high
    verify: unit
  R4:
    text: "Hooks use XxxData and XxxResponse; types stay interface/type with no runtime class."
    risk: medium
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Grouped XxxData"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Named XxxResponse"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Client takes data arg"
  risk: High
  verifymethod: Test
}
requirement R4 {
  id: R4
  text: "Hooks and no classes"
  risk: Medium
  verifymethod: Test
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/codegen/plan.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Build one OperationPlan per OpenAPI operation with grouped
      path/query/header/body inputs, collision-safe XxxData names, and
      XxxResponse aliases shared by type/client/hook emitters.
  - path: "projects/jet/src/codegen/types_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Emit per-operation XxxData and XxxResponse TypeScript types alongside
      component schema types.
  - path: "projects/jet/src/codegen/client_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Emit generated client functions that accept the grouped data argument
      when an operation has inputs and return Promise<XxxResponse>.
  - path: "projects/jet/src/codegen/hooks_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Emit React Query hook types using XxxData and XxxResponse without adding
      runtime classes.
  - path: "projects/jet/tests/codegen/openapi_golden.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: |
      Golden snapshot and TypeScript smoke coverage for named data/response
      codegen output.
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract complete: the flowchart captures component emission, per-operation input grouping into XxxData (present keys only), XxxResponse aliasing, and the client/hooks emission consuming the named types.
- [unit-test] Contract complete: R1-R4 map onto the acceptance criteria (grouped XxxData, named XxxResponse incl. void, the client data-argument signature, hooks using the named types with no runtime class).
