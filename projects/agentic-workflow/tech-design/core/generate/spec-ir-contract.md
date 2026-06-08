---
id: spec-ir-contract
type: spec
title: "SpecIR Contract Definition"
version: 1
spec_type: utility
spec_group: sdd
merge_strategy: new
created_at: 2026-02-14T11:29:35.793425+00:00
updated_at: 2026-02-14T11:29:35.793425+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: class
      title: "SpecIR Type Hierarchy"
changes:
  - file: crates/cclab-sdd/src/spec_ir/mod.rs
    action: CREATE
    description: "New module defining SpecIR enum, SpecMetadata, SpecBundle, and From impls"
  - file: crates/cclab-sdd/src/spec_ir/types.rs
    action: CREATE
    description: "SpecIR variant types and SpecMetadata struct"
  - file: crates/cclab-sdd/src/lib.rs
    action: MODIFY
    description: "Add pub mod spec_ir and re-export SpecIR, SpecBundle"
status: implemented
history:
  - timestamp: 2026-02-14T11:29:35.793425+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
  - timestamp: 2026-02-14
    agent: "mainthread"
    action: "implemented and merged"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

<spec>

# SpecIR Contract Definition

## Overview
<!-- type: doc lang: markdown -->

Define the SpecIR (Specification Intermediate Representation) contract in cclab-sdd that serves as the universal input format for code generators in cclab-lens. SpecIR unifies structured specs (OpenAPI/JSON Schema) and diagram specs (Mermaid Plus YAML frontmatter) into a single typed representation that generators consume. This addresses GitHub issue #325 and is the foundational type for the entire spec-to-code pipeline.

## Requirements
<!-- type: doc lang: markdown -->

### R1 - SpecIR enum type

```yaml
id: R1
priority: high
status: implemented
```

Define a SpecIR enum in cclab-sdd/src/spec_ir/ that represents all 6 spec types from the knowledge base: ApiSpec (OpenAPI/JSON Schema), SequencePlus, FlowchartPlus (with SemanticType), ClassPlus (with Stereotype), ErdPlus (with FK/PK), RequirementPlus (with N:M mapping). Each variant wraps the existing Generate schema types (e.g., FlowchartDef, ClassDef, ErdDef, SequenceDef).

### R2 - SpecIR metadata

```yaml
id: R2
priority: high
status: implemented
```

Each SpecIR variant carries common metadata: source file path, spec group, spec ID, and a list of tags. This metadata enables generators to make routing decisions (can_generate) without parsing the full spec.

### R3 - SpecIR construction from Generate types

```yaml
id: R3
priority: high
status: implemented
```

Provide From<T> implementations to construct SpecIR from existing Generate types: From<JsonSchema>, From<FlowchartDef>, From<ClassDiagramDef>, From<ERDDef>, From<SequenceDef>, From<RequirementDiagramDef>. Parsing is Generate's responsibility; SpecIR is the output contract.

### R4 - Public API export

```yaml
id: R4
priority: medium
status: implemented
```

Export SpecIR and all related types from cclab-sdd's lib.rs so that cclab-lens (which already depends on cclab-sdd) can import them directly. The types must be Serialize + Deserialize for JSON transport.

### R5 - SpecBundle for multi-spec input

```yaml
id: R5
priority: medium
status: implemented
```

Define a SpecBundle struct that holds Vec<SpecIR> plus a dependency graph (which specs reference which). This allows generators to receive the complete context for a change, not just individual specs.

## Diagrams
<!-- type: doc lang: markdown -->

### SpecIR Type Hierarchy

```mermaid
classDiagram
    class SpecIR {
    }
    class SpecMetadata {
    }
    class SpecBundle {
    }
```

</spec>
