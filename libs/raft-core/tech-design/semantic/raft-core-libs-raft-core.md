---
id: semantic-raft-core-libs-raft-core
summary: Semantic coverage for the raft-core library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: step-driven-raft-consensus-core
    role: primary
    claim: step-driven-raft-consensus-core-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Core library contract."
fill_sections: [schema, changes]
---

# Semantic TD: raft-core

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "raft-core"
  source_group: "libs/raft-core"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/raft-core/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/raft-core"
- path: "libs/raft-core/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-core"
- path: "libs/raft-core/tests/consensus.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "test"
    section_type: "schema"
    domain: "libs/raft-core"
- path: "libs/raft-core/tests/snapshot.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "test"
    section_type: "schema"
    domain: "libs/raft-core"
- path: "libs/raft-core/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-core"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/raft-core/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-core-cargo-toml>"
- path: "libs/raft-core/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-core-src-lib-rs>"
- path: "libs/raft-core/tests/consensus.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-core-tests-consensus-rs>"
- path: "libs/raft-core/tests/snapshot.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-core-tests-snapshot-rs>"
- path: "libs/raft-core/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
