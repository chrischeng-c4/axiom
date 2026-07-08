---
id: semantic-service-durability-libs-service-durability
summary: Semantic coverage for the service-durability library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-service-durability-contract
    role: primary
    claim: shared-service-durability-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Durability library contract."
fill_sections: [schema, changes]
---

# Semantic TD: service-durability

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "service-durability"
  source_group: "libs/service-durability"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "libs/service-durability/Cargo.toml"
        language: "toml"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "toml"
          role: "manifest"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/src/atomic.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/src/framed_log.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/src/fsync.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/src/snapshot_store.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
      - path: "libs/service-durability/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "libs/service-durability"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/service-durability/Cargo.toml"
    action: modify
    section: schema
    description: |
      Existing manifest behavior is covered by this library semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:libs-service-durability-cargo-toml>"
  - path: "libs/service-durability/src/atomic.rs"
    action: modify
    section: schema
    description: |
      Atomic replacement behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/service-durability/src/framed_log.rs"
    action: modify
    section: schema
    description: |
      CRC-framed append log behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/service-durability/src/fsync.rs"
    action: modify
    section: schema
    description: |
      Fsync policy behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/service-durability/src/lib.rs"
    action: modify
    section: schema
    description: |
      Public library exports are generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/service-durability/src/snapshot_store.rs"
    action: modify
    section: schema
    description: |
      Sequence snapshot store behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/service-durability/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, and workspace test command.
    impl_mode: codegen
```
