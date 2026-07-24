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

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: "libs/openapi-codegen/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Rename the package, assign independent version 0.5.0, rename the Rust library crate, and disable registry publication."
  - path: "libs/openapi-codegen/src/lib.rs"
    action: modify
    section: logic
    impl_mode: codegen
    description: "Rename the manifest filename and serialized generator identity."
  - path: "libs/openapi-codegen/tests/target_profile_matrix.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "opts for `fn opts`"
    description: "Consume the renamed Rust crate and assert the renamed materialized sidecar contract."
  - path: "libs/openapi-codegen/README.md"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Document the independent Git version boundary and renamed commands/artifacts."
  - path: "libs/openapi-codegen/aw.toml"
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Project all EC and test commands through the renamed Cargo package."
  - path: "libs/openapi-codegen/external-contracts/behavior/multi-language-openapi-client-generation-contract.md"
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Bind behavior contracts to the renamed package, crate, and sidecar."
  - path: "apps/defer/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "apps/defer/src/bin/defer.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "spec for `fn spec`"
    description: "Import the renamed Rust library crate."
  - path: "apps/keep/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "apps/keep/src/bin/keep.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "spec_gen for `fn spec_gen`"
    description: "Import the renamed Rust library crate."
  - path: "apps/keep/tests/spec_cli.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "spec_gen_composes_openapi_codegen_for_every_language for `fn spec_gen_composes_openapi_codegen_for_every_language`"
    description: "Verify Keep composes the renamed crate."
  - path: "apps/lumen/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "apps/lumen/src/bin/lumen.rs"
    action: modify
    section: logic
    impl_mode: codegen
    description: "Import the renamed Rust library crate."
  - path: "apps/lumen/src/spec.rs"
    action: modify
    section: logic
    impl_mode: codegen
    description: "Compose the renamed library LLM topic."
  - path: "apps/lumen/tests/generated_clients_crud_e2e.rs"
    action: modify
    section: unit-test
    impl_mode: codegen
    description: "Run generated-client journeys through the renamed crate."
  - path: "apps/lumen/tests/spec_gen_e2e.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "gen_target_override_writes_the_requested_contract for `fn gen_target_override_writes_the_requested_contract`"
    description: "Assert the renamed sidecar contract."
  - path: "apps/relay/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "apps/relay/src/bin/relay.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "spec_gen for `fn spec_gen`"
    description: "Import the renamed Rust library crate."
  - path: "apps/relay/tests/spec_cli.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "spec_gen_target_override_writes_the_requested_contract for `fn spec_gen_target_override_writes_the_requested_contract`"
    description: "Assert the renamed sidecar contract."
  - path: "apps/tape/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "apps/tape/src/bin/tape.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "spec_gen for `fn spec_gen`"
    description: "Import the renamed Rust library crate."
  - path: "projects/sift/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "projects/sift/src/bin/sift.rs"
    action: modify
    section: logic
    impl_mode: hand-written
    anchor: "spec_gen for `fn spec_gen`"
    description: "Import the renamed Rust library crate."
  - path: "examples/client-transport-policy/Cargo.toml"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the renamed local package with version 0.5 compatibility validation."
  - path: "examples/client-transport-policy/src/lib.rs"
    action: modify
    section: unit-test
    impl_mode: hand-written
    anchor: "opts for `fn opts`"
    description: "Compile the example through the renamed Rust library crate."
  - path: "CONTRIBUTING.md"
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Use the independent openapi-codegen project identity in repository guidance."
  - path: "aw.toml"
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Refresh the generated project registry and renamed package test command."
```
