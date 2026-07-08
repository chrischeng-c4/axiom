---
id: semantic-claimtoken-libs-claimtoken
summary: Semantic coverage for the claimtoken library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: scoped-claim-tokens
    role: primary
    claim: scoped-claim-tokens-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Claimtoken library contract."
fill_sections: [schema, changes]
---

# Semantic TD: claimtoken

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "claimtoken"
  source_group: "libs/claimtoken"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/claimtoken/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/claimtoken"
- path: "libs/claimtoken/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/claimtoken"
- path: "libs/claimtoken/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/claimtoken"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/claimtoken/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-claimtoken-cargo-toml>"
- path: "libs/claimtoken/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-claimtoken-src-lib-rs>"
- path: "libs/claimtoken/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
