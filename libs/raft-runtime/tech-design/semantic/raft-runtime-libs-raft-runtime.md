---
id: semantic-raft-runtime-libs-raft-runtime
summary: Semantic coverage for the raft-runtime library source, manifest, tests, and project-root context artifact.
capability_refs:
  - id: shared-raft-runtime-driver
    role: primary
    claim: shared-raft-runtime-driver-contract
    coverage: full
    rationale: "The source, tests, and manifest implement the Raft Host library contract."
fill_sections: [schema, changes]
---

# Semantic TD: raft-runtime

## Schema
<!-- type: schema lang: yaml -->

```yaml
semantic_domain:
  key: "raft-runtime"
  source_group: "libs/raft-runtime"
  coverage_kind: semantic
  evidence:
    source_units:
- path: "libs/raft-runtime/Cargo.toml"
  language: "toml"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "toml"
    role: "manifest"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/cluster.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/config.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/host.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/lib.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/llm.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/outcome_window.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/read_consistency.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/state_machine.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/store.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/src/view.rs"
  language: "rust"
  ownership_state: "handwrite"
  generator_primitives: ["source_unit"]
  source_evidence_node:
    layer: "source"
    ecosystem: "rust"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
- path: "libs/raft-runtime/llms.txt"
  language: "llms"
  ownership_state: "codegen"
  generator_primitives: ["project_root_llms"]
  source_evidence_node:
    layer: "project-root"
    ecosystem: "llms"
    role: "source"
    section_type: "schema"
    domain: "libs/raft-runtime"
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
coverage_kind: semantic
changes:
- path: "libs/raft-runtime/Cargo.toml"
  action: modify
  section: schema
  description: |
    Existing manifest behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-cargo-toml>"
- path: "libs/raft-runtime/src/cluster.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-cluster-rs>"
- path: "libs/raft-runtime/src/config.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-config-rs>"
- path: "libs/raft-runtime/src/host.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-host-rs>"
- path: "libs/raft-runtime/src/lib.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-lib-rs>"
- path: "libs/raft-runtime/src/llm.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-llm-rs>"
- path: "libs/raft-runtime/src/outcome_window.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-outcome-window-rs>"
- path: "libs/raft-runtime/src/read_consistency.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-read-consistency-rs>"
- path: "libs/raft-runtime/src/state_machine.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-state-machine-rs>"
- path: "libs/raft-runtime/src/store.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-store-rs>"
- path: "libs/raft-runtime/src/view.rs"
  action: modify
  section: schema
  description: |
    Existing source behavior is covered by this library semantic TD.
  impl_mode: hand-written
  replaces:
    - "<handwrite-tracker:libs-raft-runtime-src-view-rs>"
- path: "libs/raft-runtime/llms.txt"
  action: modify
  section: schema
  description: |
    Generated TD-first agent context map from project config, README capability map,
    TD root, and workspace test command.
  impl_mode: codegen
```
