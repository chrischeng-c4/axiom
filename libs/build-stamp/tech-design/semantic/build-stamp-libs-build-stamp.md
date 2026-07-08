---
id: semantic-build-stamp-libs-build-stamp
summary: Semantic coverage for the build-stamp library source, manifest, and project-root context artifact.
capability_refs:
  - id: build-script-version-stamp
    role: primary
    claim: build-script-version-stamp-contract
    coverage: full
    rationale: "The source and manifest implement the shared build-script metadata stamping contract."
fill_sections: [schema, changes]
---

# Semantic TD: build-stamp

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "build-stamp"
  source_group: "libs/build-stamp"
  coverage_kind: semantic
  evidence:
    source_units:
      - path: "libs/build-stamp/src/lib.rs"
        language: "rust"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "rust"
          role: "source"
          section_type: "schema"
          domain: "libs/build-stamp"
      - path: "libs/build-stamp/Cargo.toml"
        language: "toml"
        ownership_state: "handwrite"
        generator_primitives: ["source_unit"]
        source_evidence_node:
          layer: "source"
          ecosystem: "toml"
          role: "manifest"
          section_type: "schema"
          domain: "libs/build-stamp"
      - path: "libs/build-stamp/llms.txt"
        language: "llms"
        ownership_state: "codegen"
        generator_primitives: ["project_root_llms"]
        source_evidence_node:
          layer: "project-root"
          ecosystem: "llms"
          role: "source"
          section_type: "schema"
          domain: "libs/build-stamp"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
  - path: "libs/build-stamp/src/lib.rs"
    action: modify
    section: schema
    description: |
      Existing source behavior is covered by this library semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:libs-build-stamp-src-lib-rs>"
  - path: "libs/build-stamp/Cargo.toml"
    action: modify
    section: schema
    description: |
      Existing manifest behavior is covered by this library semantic TD.
    impl_mode: hand-written
    replaces:
      - "<handwrite-tracker:libs-build-stamp-cargo-toml>"
  - path: "libs/build-stamp/llms.txt"
    action: modify
    section: schema
    description: |
      Generated TD-first agent context map from project config, README capability map,
      TD root, and workspace test command.
    impl_mode: codegen
```
