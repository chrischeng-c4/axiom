---
id: semantic-metrics-prometheus-libs-metrics-prometheus
summary: Semantic coverage for the metrics-prometheus library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-prometheus-metric-primitives
    role: primary
    claim: shared-prometheus-metric-primitives-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Service Metrics library contract."
fill_sections: [schema, changes]
---

# Semantic TD: metrics-prometheus

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "metrics-prometheus"
  source_group: "libs/metrics-prometheus"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/metrics-prometheus/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/metrics-prometheus"
- path: "libs/metrics-prometheus/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/metrics-prometheus"
- path: "libs/metrics-prometheus/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/metrics-prometheus"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/metrics-prometheus/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-metrics-prometheus-cargo-toml>"
- path: "libs/metrics-prometheus/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-metrics-prometheus-src-lib-rs>"
- path: "libs/metrics-prometheus/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
