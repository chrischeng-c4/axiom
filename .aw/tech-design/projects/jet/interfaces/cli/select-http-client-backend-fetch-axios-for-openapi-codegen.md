---
id: jet-codegen-openapi-http-client
summary: Select the generated client's HTTP runtime (fetch or axios) for jet codegen openapi via CLI flag, jet.toml, or package.json stack detection; only runtime.ts changes between backends.
capability_refs:
  - id: rust-native-frontend-toolchain
    role: primary
    gap: production-replacement-readiness
    claim: full-toolchain-dogfood-flow
    coverage: partial
    rationale: "The OpenAPI codegen HTTP backend selector is part of the Jet frontend toolchain surface for generated client integration."
fill_sections: [logic, unit-test]
---

# TD: jet/codegen-openapi-http-client

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-http-client-logic
entry: start
nodes:
  start: { kind: start, label: "jet codegen openapi --http <fetch|axios>" }
  parse: { kind: process, label: "Resolve http_client from --http, jet.toml [codegen.openapi].http, package.json axios, else fetch" }
  emit_types: { kind: process, label: "Emit types.ts (backend independent)" }
  http_dec: { kind: decision, label: "http_client == axios?" }
  fetch_rt: { kind: process, label: "Emit fetch runtime.ts (native fetch)" }
  axios_rt: { kind: process, label: "Emit axios runtime.ts (import axios, AxiosInstance)" }
  emit_rest: { kind: process, label: "Emit client.ts and hooks.ts (identical for both backends)" }
  done: { kind: terminal, label: "Ok (exit 0)" }
edges:
  - { from: start, to: parse }
  - { from: parse, to: emit_types }
  - { from: emit_types, to: http_dec }
  - { from: http_dec, to: axios_rt, label: "yes" }
  - { from: http_dec, to: fetch_rt, label: "no" }
  - { from: fetch_rt, to: emit_rest }
  - { from: axios_rt, to: emit_rest }
  - { from: emit_rest, to: done }
---
flowchart TD
    start([jet codegen openapi http flag]) --> parse[Resolve http client from CLI jet toml package json or fetch fallback]
    parse --> emit_types[Emit types ts backend independent]
    emit_types --> http_dec{http client is axios?}
    http_dec -->|yes| axios_rt[Emit axios runtime ts import axios AxiosInstance]
    http_dec -->|no| fetch_rt[Emit fetch runtime ts native fetch]
    fetch_rt --> emit_rest[Emit client ts and hooks ts identical]
    axios_rt --> emit_rest
    emit_rest --> done([Ok exit 0])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: jet-codegen-openapi-http-client-unit-test
requirements:
  R1:
    text: "--http axios emits a runtime that imports axios and dispatches via an AxiosInstance."
    risk: high
    verify: unit
  R2:
    text: "fetch from CLI, jet.toml, or fallback emits the existing fetch runtime unchanged."
    risk: high
    verify: unit
  R3:
    text: "client.ts, hooks.ts, types.ts, and index.ts are byte-identical across backends."
    risk: high
    verify: unit
  R4:
    text: "axios is a generated-output peer dependency only; jet adds no new Rust crate."
    risk: medium
    verify: command
  R5:
    text: "package.json axios dependency selects the axios runtime when --http and jet.toml do not override it."
    risk: medium
    verify: unit
---
requirementDiagram
requirement R1 {
  id: R1
  text: "Axios runtime emission"
  risk: High
  verifymethod: Test
}
requirement R2 {
  id: R2
  text: "Fetch runtime unchanged"
  risk: High
  verifymethod: Test
}
requirement R3 {
  id: R3
  text: "Backend invariant surface"
  risk: High
  verifymethod: Test
}
requirement R4 {
  id: R4
  text: "No new Rust crate"
  risk: Medium
  verifymethod: Test
}
requirement R5 {
  id: R5
  text: "package.json axios auto-detection"
  risk: Medium
  verifymethod: Test
}
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "projects/jet/src/cli.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Add and parse `jet codegen openapi --http fetch|axios`; when omitted,
      resolve HTTP backend from [codegen.openapi].http, package.json axios,
      then fetch fallback before passing the backend into GenOptions.
  - path: "projects/jet/src/codegen/mod.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Carry the HttpClient backend choice through stack-aware generation while
      keeping non-runtime generated files backend-invariant.
  - path: "projects/jet/src/codegen/client_emit.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    description: |
      Emit either the fetch runtime or axios runtime from the selected
      HttpClient without changing client.ts or hooks.ts.
  - path: "projects/jet/tests/codegen/openapi_golden.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: |
      Assert axios runtime output matches its golden snapshot and that
      types/client/hooks/index are byte-identical across fetch and axios.
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Contract complete: the flowchart captures the `--http` branch — types.ts is emitted backend-independently, the decision selects the fetch or axios runtime emitter, and client.ts/hooks.ts are shared, matching the backend-invariance requirement.
- [unit-test] Contract complete: R1-R4 map onto the acceptance criteria (axios runtime emission, fetch runtime unchanged, byte-identical client/hooks/types/index across backends, no new Rust crate).
