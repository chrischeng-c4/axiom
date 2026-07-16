---
id: '1766'
summary: (fill)
capability_refs:
  - id: cli-interface
    role: primary
    claim: lumen-spec-schema-openapi-json-yaml-json-schema-offline
    coverage: full
    rationale: "Executable Python generated clients are part of the public lumen spec gen contract."
fill_sections: [logic, changes, unit-test]
---

## Logic
<!-- type: logic lang: mermaid -->

```mermaid
---
id: python-reference-alias-order
entry: generate
nodes:
  generate: { kind: start, label: "lumen spec gen --lang py" }
  canonical: { kind: process, label: "read Lumen canonical OpenAPI components" }
  partition: { kind: process, label: "shared Python emitter partitions concrete models and reference aliases" }
  models: { kind: process, label: "emit concrete Pydantic models, including SearchRequest" }
  aliases: { kind: process, label: "emit dependency-ordered aliases, including BatchSearchItem = SearchRequest" }
  execute: { kind: process, label: "Python compiles and executes generated models.py" }
  done: { kind: terminal, label: "generated package is import-safe" }
edges:
  - { from: generate, to: canonical }
  - { from: canonical, to: partition }
  - { from: partition, to: models }
  - { from: models, to: aliases }
  - { from: aliases, to: execute }
  - { from: execute, to: done }
---
flowchart TD
    generate([lumen spec gen --lang py]) --> canonical[read canonical OpenAPI components]
    canonical --> partition[partition concrete models and reference aliases]
    partition --> models[emit concrete Pydantic models including SearchRequest]
    models --> aliases[emit dependency-ordered aliases including BatchSearchItem = SearchRequest]
    aliases --> execute[compile and execute generated models.py]
    execute --> done([generated package is import-safe])
```
## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: libs/openapi-codegen/src/emit/py/models_emit.rs
    action: modify
    section: logic
    impl_mode: hand-written
    description: "Partition concrete Pydantic models from reference aliases and dependency-order aliases after their referenced declarations."
  - path: apps/lumen/tests/spec_gen_e2e.rs
    action: modify
    section: unit-test
    impl_mode: hand-written
    description: "Extend the real lumen spec gen --lang py e2e test to assert SearchRequest is declared before BatchSearchItem = SearchRequest and execute the emitted models.py with python3, locking import-time safety for reference aliases."
```
## Unit Test
<!-- type: unit-test lang: mermaid -->

```mermaid
---
id: python-reference-alias-order-verification
requirements:
  generated_models_execute:
    id: R2
    text: "The unmodified models.py emitted by lumen spec gen --lang py compiles and executes successfully under python3 with Pydantic installed."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test spec_gen_e2e gen_py_writes_pydantic_h2c_client
  reference_alias_follows_concrete_model:
    id: R1
    text: "For Lumen's canonical OpenAPI, generated Python declares SearchRequest before the top-level BatchSearchItem = SearchRequest alias so module evaluation never references an undefined name."
    kind: regression
    risk: high
    verify: cargo test -p lumen --test spec_gen_e2e gen_py_writes_pydantic_h2c_client
---
flowchart TD
    r1[R1 reference alias follows concrete model] --> cargo_test_p_lumen_test_spec_gen_e2e_gen_py_writes_pydantic_h2c_client[cargo test -p lumen --test spec_gen_e2e gen_py_writes_pydantic_h2c_client]
    r2[R2 generated models execute] --> cargo_test_p_lumen_test_spec_gen_e2e_gen_py_writes_pydantic_h2c_client
```
