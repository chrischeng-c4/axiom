---
id: semantic-ui-runtime-libs-ui-runtime
summary: Semantic coverage for the ui-runtime library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: renderer-neutral-component-runtime
    role: primary
    claim: renderer-neutral-component-runtime-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Ui Runtime library contract."
fill_sections: [schema, changes]
---

# Semantic TD: ui-runtime

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "ui-runtime"
  source_group: "libs/ui-runtime"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/ui-runtime/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/ui-runtime"
- path: "libs/ui-runtime/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/ui-runtime"
- path: "libs/ui-runtime/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/ui-runtime"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/ui-runtime/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-ui-runtime-cargo-toml>"
- path: "libs/ui-runtime/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-ui-runtime-src-lib-rs>"
- path: "libs/ui-runtime/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
