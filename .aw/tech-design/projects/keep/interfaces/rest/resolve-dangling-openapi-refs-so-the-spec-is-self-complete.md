---
id: keep-openapi-ref-integrity
summary: >
  Make Keep's OpenAPI document self-complete: every $ref in the served spec
  resolves to a registered components.schemas key. Reference ApiError and
  ClusterState by their short ToSchema names inside #[utoipa::path]
  annotations (utoipa 4 derives ref names from the literal type-path text, so
  fully-qualified crate::... paths emit dangling dotted refs), and tighten the
  spec_cli self-completeness test to walk every $ref with no allowlist.
fill_sections: [logic, unit-test, changes]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: keep-openapi-ref-integrity-contract
entry: annotate
nodes:
  annotate: { kind: start, label: "handlers.rs #[utoipa::path] responses name body types" }
  short_refs: { kind: process, label: "reference ApiError and ClusterState by imported short name, not crate::-qualified path" }
  register: { kind: process, label: "openapi.rs components(schemas(...)) registers the same short ToSchema names" }
  emit: { kind: process, label: "ApiDoc::openapi() emits $refs that match components.schemas keys" }
  serve: { kind: process, label: "keep spec and /openapi.json serve one self-complete document" }
  codegen: { kind: process, label: "spec gen ts/py/rust compose on resolvable schema names" }
  done: { kind: terminal, label: "no dangling crate.*-dotted refs remain" }
edges:
  - { from: annotate, to: short_refs }
  - { from: short_refs, to: register }
  - { from: register, to: emit }
  - { from: emit, to: serve }
  - { from: serve, to: codegen }
  - { from: codegen, to: done }
---
flowchart TD
    annotate([handlers.rs utoipa path responses name body types]) --> short_refs[reference ApiError and ClusterState by imported short name not crate-qualified path]
    short_refs --> register[openapi.rs components schemas registers the same short ToSchema names]
    register --> emit[ApiDoc openapi emits refs that match components.schemas keys]
    emit --> serve[keep spec and /openapi.json serve one self-complete document]
    serve --> codegen[spec gen ts py rust compose on resolvable schema names]
    codegen --> done([no dangling crate-dotted refs remain])
```

## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: keep-openapi-ref-integrity-tests
requirements:
  refs_resolve:
    id: R1
    text: "Every $ref in the emitted OpenAPI document resolves to a key present in components.schemas; no crate.*-dotted dangling refs."
    kind: behavior
    risk: medium
    verify: test
  strict_walk:
    id: R2
    text: "The spec_cli self-completeness test walks all $refs recursively across the whole document and asserts each resolves, with no allowlist."
    kind: behavior
    risk: medium
    verify: test
  wire_unchanged:
    id: R3
    text: "The wire contract is otherwise unchanged: same paths, operations, and response shapes; only dotted schema ref names become short registered names."
    kind: behavior
    risk: medium
    verify: test
elements:
  keep_spec_cli_tests:
    kind: test
    path: apps/keep/tests/spec_cli.rs
relations:
  - { from: keep_spec_cli_tests, verifies: refs_resolve }
  - { from: keep_spec_cli_tests, verifies: strict_walk }
  - { from: keep_spec_cli_tests, verifies: wire_unchanged }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "all refs resolve"
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "strict recursive ref walk"
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: "wire contract unchanged"
      risk: medium
      verifymethod: test
    }
    element keep_spec_cli_tests {
      type: "rs/#[test]"
    }
    keep_spec_cli_tests - verifies -> R1
    keep_spec_cli_tests - verifies -> R2
    keep_spec_cli_tests - verifies -> R3
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: apps/keep/src/http/handlers.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Import ApiError and ClusterState and reference them by short name in every #[utoipa::path] responses(body = ...) annotation (and the cluster handler signature) so utoipa 4 emits refs matching the registered component keys instead of crate.*-dotted names."
  - path: apps/keep/src/http/openapi.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Register ClusterState in components(schemas(...)) via an imported short path for one consistent short-name registration style across the ApiDoc."
  - path: apps/keep/tests/spec_cli.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Add the strict self-completeness test: recursively collect every $ref in the keep spec output and assert each resolves to a components.schemas key — no allowlist."
```
