---
id: semantic-service-metrics-libs-service-metrics
summary: Semantic coverage for the service-metrics library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-prometheus-metric-primitives
    role: primary
    claim: shared-prometheus-metric-primitives-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Metrics library contract."
fill_sections: [schema, changes]
---

# Semantic TD: service-metrics

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "service-metrics"
  source_group: "libs/service-metrics"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/service-metrics/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/service-metrics"
- path: "libs/service-metrics/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/service-metrics"
- path: "libs/service-metrics/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/service-metrics"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/service-metrics/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-service-metrics-cargo-toml>"
- path: "libs/service-metrics/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-service-metrics-src-lib-rs>"
- path: "libs/service-metrics/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
