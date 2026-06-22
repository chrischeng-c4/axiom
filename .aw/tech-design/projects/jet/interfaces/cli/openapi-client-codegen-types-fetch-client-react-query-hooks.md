---
id: jet-codegen-openapi
summary: Standalone jet codegen openapi command reading an OpenAPI 3.0/3.1 spec and emitting TypeScript types, a typed fetch client, and TanStack Query hooks.
fill_sections: [logic, unit-test]
---

# TD: jet/codegen-openapi

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-logic
entry: start
nodes:
  start: { kind: start, label: "jet codegen openapi <spec> --out <dir>" }
  read: { kind: process, label: "Read spec file from disk" }
  parse: { kind: process, label: "Parse JSON into OpenAPI Spec model (serde, tolerate unknown keys)" }
  parse_ok: { kind: decision, label: "Parse succeeded?" }
  err_parse: { kind: terminal, label: "Err: invalid spec (exit 2)" }
  normalize: { kind: process, label: "Normalize 3.0 nullable and 3.1 type-array nullability" }
  resolve: { kind: process, label: "Resolve #/components/schemas $refs with cycle guard" }
  emit_types: { kind: process, label: "Emit types.ts from component and inline schemas" }
  client_dec: { kind: decision, label: "Emit client? (not --types-only)" }
  emit_client: { kind: process, label: "Emit runtime.ts + client.ts (createClient, one fn per operation)" }
  hooks_dec: { kind: decision, label: "Emit hooks? (not --types-only and not --no-hooks)" }
  emit_hooks: { kind: process, label: "Emit hooks.ts (createHooks, useQuery/useMutation)" }
  write: { kind: process, label: "Write generated files to out dir + index.ts barrel" }
  done: { kind: terminal, label: "Ok (exit 0)" }
edges:
  - { from: start, to: read }
  - { from: read, to: parse }
  - { from: parse, to: parse_ok }
  - { from: parse_ok, to: err_parse, label: "no" }
  - { from: parse_ok, to: normalize, label: "yes" }
  - { from: normalize, to: resolve }
  - { from: resolve, to: emit_types }
  - { from: emit_types, to: client_dec }
  - { from: client_dec, to: hooks_dec, label: "no" }
  - { from: client_dec, to: emit_client, label: "yes" }
  - { from: emit_client, to: hooks_dec }
  - { from: hooks_dec, to: write, label: "no" }
  - { from: hooks_dec, to: emit_hooks, label: "yes" }
  - { from: emit_hooks, to: write }
  - { from: write, to: done }
---
flowchart TD
    start([jet codegen openapi spec --out dir]) --> read[Read spec file from disk]
    read --> parse[Parse JSON into OpenAPI Spec model]
    parse --> parse_ok{Parse succeeded?}
    parse_ok -->|no| err_parse([Err: invalid spec exit 2])
    parse_ok -->|yes| normalize[Normalize 3.0 nullable and 3.1 type-array nullability]
    normalize --> resolve[Resolve components.schemas refs with cycle guard]
    resolve --> emit_types[Emit types.ts from component and inline schemas]
    emit_types --> client_dec{Emit client? not --types-only}
    client_dec -->|no| hooks_dec{Emit hooks? not --no-hooks}
    client_dec -->|yes| emit_client[Emit runtime.ts and client.ts]
    emit_client --> hooks_dec
    hooks_dec -->|no| write[Write generated files to out dir]
    hooks_dec -->|yes| emit_hooks[Emit hooks.ts createHooks useQuery useMutation]
    emit_hooks --> write
    write --> done([Ok exit 0])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-unit-test
requirements:
  R1:
    text: "3.0 nullable and 3.1 type-array nullability both produce a nullable union."
    risk: high
    verify: unit
  R2:
    text: "Schema mapping covers object, array, enum, oneOf, anyOf, allOf, ref, additionalProperties, required vs optional."
    risk: high
    verify: unit
  R3:
    text: "Client functions name from operationId with method-path fallback and type path, query, body, and response."
    risk: high
    verify: unit
  R4:
    text: "Hooks emit useQuery for GET and useMutation for write methods."
    risk: medium
    verify: unit
  R5:
    text: "Generated output is deterministic and matches committed golden snapshots."
    risk: high
    verify: command
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Nullable reconciliation"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Schema to TS mapping"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Typed client functions"
  risk: High
  verifymethod: Test
}
requirement R4 {
  id: R4
  text: "React Query hooks"
  risk: Medium
  verifymethod: Test
}
requirement R5 {
  id: R5
  text: "Deterministic golden output"
  risk: High
  verifymethod: Test
}
```
