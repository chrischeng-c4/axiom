---
id: '2537'
summary: (fill)
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: openapi-codegen-git-version-boundary
entry: inventory
nodes:
  inventory: { kind: start, label: "inventory active legacy identities" }
  package: { kind: process, label: "set package openapi-codegen 0.5.0 publish false" }
  crate: { kind: process, label: "rename Rust crate to openapi_codegen" }
  consumers: { kind: process, label: "rewrite local consumers to path plus version 0.5" }
  manifest: { kind: process, label: "rename sidecar and generator identity" }
  projections: { kind: process, label: "regenerate TD EC AW and Cargo projections" }
  residue: { kind: decision, label: "active legacy identity remains" }
  verify: { kind: process, label: "run target matrix and reverse consumer checks" }
  fail: { kind: terminal, label: "reject migration" }
  ready: { kind: terminal, label: "ready for openapi-codegen at 0.5.0 tag" }
edges:
  - { from: inventory, to: package }
  - { from: package, to: crate }
  - { from: crate, to: consumers }
  - { from: consumers, to: manifest }
  - { from: manifest, to: projections }
  - { from: projections, to: residue }
  - { from: residue, to: fail, label: "yes" }
  - { from: residue, to: verify, label: "no" }
  - { from: verify, to: fail, label: "failure" }
  - { from: verify, to: ready, label: "pass" }
---
flowchart TD
  inventory([inventory active legacy identities]) --> package[set package openapi-codegen 0.5.0 publish false]
  package --> crate[rename Rust crate to openapi_codegen]
  crate --> consumers[rewrite local consumers to path plus version 0.5]
  consumers --> manifest[rename sidecar and generator identity]
  manifest --> projections[regenerate TD EC AW and Cargo projections]
  projections --> residue{active legacy identity remains}
  residue -->|yes| fail([reject migration])
  residue -->|no| verify[run target matrix and reverse consumer checks]
  verify -->|failure| fail
  verify -->|pass| ready([ready for openapi-codegen at 0.5.0 tag])
```
