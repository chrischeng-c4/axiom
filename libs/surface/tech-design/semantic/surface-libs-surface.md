---
id: semantic-surface-libs-surface
summary: Semantic coverage for the surface library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: renderer-neutral-ui-surface-model
    role: primary
    claim: renderer-neutral-ui-surface-model-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Surface library contract."
fill_sections: [schema, changes]
---

# Semantic TD: surface

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "surface"
  source_group: "libs/surface"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/surface/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/surface"
- path: "libs/surface/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/surface"
- path: "libs/surface/tests/snapshot.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "test"
    section_type: "schema"
    domain: "libs/surface"
- path: "libs/surface/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/surface"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/surface/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-surface-cargo-toml>"
- path: "libs/surface/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-surface-src-lib-rs>"
- path: "libs/surface/tests/snapshot.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-surface-tests-snapshot-rs>"
- path: "libs/surface/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
