---
id: semantic-storage-durable-libs-storage-durable
summary: Semantic coverage for the storage-durable library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-storage-durable-contract
    role: primary
    claim: shared-storage-durable-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Durability library contract."
fill_sections: [schema, changes]
---

# Semantic TD: storage-durable

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "storage-durable"
  source_group: "libs/storage-durable"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "libs/storage-durable/Cargo.toml"
        language: "toml"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "toml"
          role: "manifest"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/src/atomic.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/src/framed_log.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/src/fsync.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/src/lib.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/src/snapshot_store.rs"
        language: "rust"
        ownership_state: "codegen"
        generator_primitives: ["rust_source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
      - path: "libs/storage-durable/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "libs/storage-durable"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/storage-durable/Cargo.toml"
    action: modify
    section: schema
    description: |
      Existing manifest behavior is covered by this library semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:libs-storage-durable-cargo-toml>"
  - path: "libs/storage-durable/src/atomic.rs"
    action: modify
    section: schema
    description: |
      Atomic replacement behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/storage-durable/src/framed_log.rs"
    action: modify
    section: schema
    description: |
      CRC-framed append log behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/storage-durable/src/fsync.rs"
    action: modify
    section: schema
    description: |
      Fsync policy behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/storage-durable/src/lib.rs"
    action: modify
    section: schema
    description: |
      Public library exports are generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/storage-durable/src/snapshot_store.rs"
    action: modify
    section: schema
    description: |
      Sequence snapshot store behavior is generated from the per-file rust source-unit TD.
    impl_mode: codegen
  - path: "libs/storage-durable/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, and workspace test command.
    impl_mode: codegen
```
