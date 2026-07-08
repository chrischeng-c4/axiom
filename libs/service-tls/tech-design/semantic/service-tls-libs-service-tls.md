---
id: semantic-service-tls-libs-service-tls
summary: Semantic coverage for the service-tls library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-peer-mtls-material-loading
    role: primary
    claim: shared-peer-mtls-material-loading-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Tls library contract."
fill_sections: [schema, changes]
---

# Semantic TD: service-tls

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "service-tls"
  source_group: "libs/service-tls"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/service-tls/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/service-tls"
- path: "libs/service-tls/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/service-tls"
- path: "libs/service-tls/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/service-tls"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/service-tls/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-service-tls-cargo-toml>"
- path: "libs/service-tls/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-service-tls-src-lib-rs>"
- path: "libs/service-tls/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
