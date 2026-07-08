---
id: semantic-openapi-codegen-libs-openapi-codegen
summary: Semantic coverage for the openapi-codegen library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: multi-language-openapi-client-generation
    role: primary
    claim: multi-language-openapi-client-generation-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Openapi Codegen library contract."
fill_sections: [schema, changes]
---

# Semantic TD: openapi-codegen

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "openapi-codegen"
  source_group: "libs/openapi-codegen"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/openapi-codegen/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/py/client_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/py/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/py/models_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/py/pymap.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/py/runtime_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/rust/client_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/rust/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/rust/models_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/rust/rsmap.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/client_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/hooks_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/plan.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/tsmap.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/emit/ts/types_emit.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/ir/mod.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/ir/names.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/ir/openapi.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/ir/operations.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/ir/typemap.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/src/llm.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
- path: "libs/openapi-codegen/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/openapi-codegen"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/openapi-codegen/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-cargo-toml>"
- path: "libs/openapi-codegen/src/emit/mod.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-mod-rs>"
- path: "libs/openapi-codegen/src/emit/py/client_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-py-client-emit-rs>"
- path: "libs/openapi-codegen/src/emit/py/mod.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-py-mod-rs>"
- path: "libs/openapi-codegen/src/emit/py/models_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-py-models-emit-rs>"
- path: "libs/openapi-codegen/src/emit/py/pymap.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-py-pymap-rs>"
- path: "libs/openapi-codegen/src/emit/py/runtime_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-py-runtime-emit-rs>"
- path: "libs/openapi-codegen/src/emit/rust/client_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-rust-client-emit-rs>"
- path: "libs/openapi-codegen/src/emit/rust/mod.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-rust-mod-rs>"
- path: "libs/openapi-codegen/src/emit/rust/models_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-rust-models-emit-rs>"
- path: "libs/openapi-codegen/src/emit/rust/rsmap.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-rust-rsmap-rs>"
- path: "libs/openapi-codegen/src/emit/ts/client_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-client-emit-rs>"
- path: "libs/openapi-codegen/src/emit/ts/hooks_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-hooks-emit-rs>"
- path: "libs/openapi-codegen/src/emit/ts/mod.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-mod-rs>"
- path: "libs/openapi-codegen/src/emit/ts/plan.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-plan-rs>"
- path: "libs/openapi-codegen/src/emit/ts/tsmap.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-tsmap-rs>"
- path: "libs/openapi-codegen/src/emit/ts/types_emit.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-emit-ts-types-emit-rs>"
- path: "libs/openapi-codegen/src/ir/mod.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-ir-mod-rs>"
- path: "libs/openapi-codegen/src/ir/names.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-ir-names-rs>"
- path: "libs/openapi-codegen/src/ir/openapi.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-ir-openapi-rs>"
- path: "libs/openapi-codegen/src/ir/operations.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-ir-operations-rs>"
- path: "libs/openapi-codegen/src/ir/typemap.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-ir-typemap-rs>"
- path: "libs/openapi-codegen/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-lib-rs>"
- path: "libs/openapi-codegen/src/llm.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-openapi-codegen-src-llm-rs>"
- path: "libs/openapi-codegen/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
