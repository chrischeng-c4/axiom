---
id: lumen-openapi-self-complete
summary: >
  Make the emitted OpenAPI self-complete and correct: register the six
  ToSchema types that are $ref'd but undefined (SortSpec, SortOrder, SortMissing,
  IdsQuery, HasChildQuery, HammingQuery) in utoipa's components(schemas) list,
  and set the servers block to the real serving port 7373 (not 8080).
capability_refs:
  - id: "competitor-feature-parity"
    role: primary
    claim: "schema-and-metadata-breadth"
    coverage: partial
    rationale: >
      The published OpenAPI must be self-complete and point at the real port so a
      generated client works turnkey.
fill_sections: [logic, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: openapi-self-complete
entry: start
nodes:
  start:    { kind: start,    label: "emit OpenAPI from utoipa derive" }
  register: { kind: process,  label: "register 6 $ref'd ToSchema types in components(schemas)" }
  port:     { kind: process,  label: "set servers urls to :7373" }
  check:    { kind: decision, label: "every $ref resolves to a defined schema?" }
  ok:       { kind: terminal, label: "self-complete OpenAPI" }
  gap:      { kind: terminal, label: "dangling $ref (regression)" }
edges:
  - { from: start,    to: register }
  - { from: register, to: port }
  - { from: port,     to: check }
  - { from: check,    to: ok,  label: "yes" }
  - { from: check,    to: gap, label: "no" }
---
flowchart TD
    start([emit OpenAPI]) --> register[register 6 ToSchema types]
    register --> port[servers urls -> :7373]
    port --> check{every $ref resolves?}
    check -->|yes| ok([self-complete OpenAPI])
    check -->|no| gap([dangling $ref])
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: openapi-self-complete-verification
requirements:
  no_dangling_refs:
    id: R1
    text: "every $ref in the emitted OpenAPI resolves to a defined component schema"
    kind: interface
    risk: high
    verify: test
  servers_port_correct:
    id: R2
    text: "the servers block uses port 7373 for both the in-cluster and local urls"
    kind: interface
    risk: medium
    verify: test
elements:
  test_openapi_has_no_dangling_refs:
    kind: test
    type: "rs/#[test]"
  test_servers_use_7373:
    kind: test
    type: "rs/#[test]"
relations:
  - { from: test_openapi_has_no_dangling_refs, verifies: no_dangling_refs }
  - { from: test_servers_use_7373,             verifies: servers_port_correct }
---
requirementDiagram
    requirement R1 {
      id: R1
      text: "no dangling $ref in OpenAPI"
      risk: high
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: "servers use port 7373"
      risk: medium
      verifymethod: test
    }
    element test_openapi_has_no_dangling_refs {
      type: "rs/#[test]"
    }
    test_openapi_has_no_dangling_refs - verifies -> R1
```

# Reviews

### Review 1
**Verdict:** approved

- [logic] Applicable: control-flow contract for the change.
- [unit-test] Applicable: behavior verified by unit tests.
