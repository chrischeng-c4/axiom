---
id: sdd-codegen-marker-system
main_spec_ref: crates/sdd/logic/codegen-markers.md
merge_strategy: new
fill_sections: [overview, requirements, changes]
filled_sections: [overview, requirements, changes]
create_complete: true
---

# Sdd Codegen Marker System

## Overview

<!-- type: overview lang: markdown -->

The CODEGEN marker system enables selective regeneration of generated code blocks within otherwise hand-crafted files. Target files contain `// CODEGEN-BEGIN` / `// CODEGEN-END` block markers; `score gen apply` updates only the content between them, preserving surrounding code.

**Marker format** (Rust example):
```rust
// SPEC-MANAGED: <spec-path>#<section-id>
// CODEGEN-BEGIN
<generated content>
// CODEGEN-END
```

**SPEC-REF markers** (inside CODEGEN blocks):
```rust
// SPEC-REF: <spec-path>#<section-id>
// TODO: <task description>
```

**Three operations**:
1. **Parse**: extract all CODEGEN-BEGIN/END blocks from a file, return (prefix, block_map, suffix)
2. **Replace**: given new generated content, replace block content while preserving prefix/suffix
3. **Init**: scaffold new CODEGEN-BEGIN/END markers in an existing file at a given location

Multiple CODEGEN blocks per file are supported (one per spec section). Each block is identified by its SPEC-MANAGED comment. SPEC-REF markers within blocks are allowed — they are part of generated content. Markers outside blocks are hand-written and always preserved.

All emitted markers are tracked in `.score/codegen_markers.yaml` for CI visibility. Reducing marker count over time indicates improving TD quality.
## Requirements

<!-- type: requirements lang: mermaid -->

```mermaid
---
id: sdd-codegen-marker-requirements
title: CODEGEN Marker System Requirements
requirements:
  R1:
    text: Parser extracts CODEGEN-BEGIN/END blocks from target files
    type: functional
    priority: high
    risk: medium
    verification: test
    notes: |
      Parse all CODEGEN-BEGIN...CODEGEN-END regions from a file.
      Extract SPEC-MANAGED comment above each block (spec-path + section-id).
      Return: Vec<CodegenBlock { spec_ref, start_line, end_line, content }>
  R2:
    text: Replacer updates CODEGEN block content preserving wrapper code
    type: functional
    priority: high
    risk: medium
    verification: test
    notes: |
      Given new generated content string and file content,
      replace the interior of CODEGEN-BEGIN/END block identified by SPEC-MANAGED ref.
      Preserve all code outside CODEGEN blocks unchanged.
  R3:
    text: Multiple CODEGEN blocks per file supported
    type: functional
    priority: medium
    risk: low
    verification: test
    notes: |
      A single file may have 1..N CODEGEN blocks, one per spec section.
      Each block is identified by its SPEC-MANAGED comment (spec-path#section-id).
      Blocks do not overlap; order preserved.
  R4:
    text: SPEC-REF markers inside CODEGEN blocks are valid generated content
    type: functional
    priority: high
    risk: low
    verification: test
    notes: |
      SPEC-REF markers inside CODEGEN blocks are written by generators for non-deterministic parts.
      They are regenerated on each apply (not hand-written, not preserved separately).
      Markers outside CODEGEN blocks are hand-written and always preserved.
  R5:
    text: All SPEC-REF markers tracked in .score/codegen_markers.yaml
    type: functional
    priority: medium
    risk: low
    verification: test
    notes: |
      After each gen apply, update .score/codegen_markers.yaml with all emitted markers.
      Format: spec-path -> [{ section, file, line, task }]
      Enables CI visibility of remaining TODOs.
  R6:
    text: score gen init-markers scaffolds new CODEGEN blocks in existing files
    type: functional
    priority: medium
    risk: low
    verification: test
    notes: |
      Given a file path and spec reference, insert empty CODEGEN-BEGIN/END block.
      Does not overwrite existing content. Finds insertion point near specified symbol.
---
requirementDiagram
    requirement R1 {
      id: R1
      text: Block parser
      risk: medium
      verifymethod: test
    }
    requirement R2 {
      id: R2
      text: Block replacer
      risk: medium
      verifymethod: test
    }
    requirement R3 {
      id: R3
      text: Multiple blocks per file
      risk: low
      verifymethod: test
    }
    requirement R4 {
      id: R4
      text: SPEC-REF inside blocks
      risk: low
      verifymethod: test
    }
    requirement R5 {
      id: R5
      text: Marker tracking file
      risk: low
      verifymethod: test
    }
    requirement R6 {
      id: R6
      text: Init markers helper
      risk: low
      verifymethod: test
    }
```
## Scenarios
<!-- type: scenarios lang: yaml -->

<!-- TODO: Use YAML GWT structured format. Example:
```yaml
- id: S1
  given: Initial state description
  when: Action or event that triggers the scenario
  then: Expected outcome

- id: S2
  given: Another initial state
  when: Another action
  then: Another expected outcome
  diagram_ref: interaction-S2
```
-->

## Diagrams

### Mindmap
<!-- type: mindmap lang: mermaid -->
<!-- TODO: Use Mermaid Plus mindmap (YAML frontmatter inside mermaid block).
```mermaid
---
id: mindmap
---
mindmap
  root((System))
    Component A
    Component B
```
-->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO: Use Mermaid Plus stateDiagram-v2 (YAML frontmatter inside mermaid block).
```mermaid
---
id: state-machine
initial: idle
---
stateDiagram-v2
    [*] --> idle
```
-->

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO: Use Mermaid Plus sequenceDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: interaction
---
sequenceDiagram
    actor User
    User->>System: action
```
-->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO: Use Mermaid Plus flowchart (YAML frontmatter inside mermaid block).
```mermaid
---
id: logic
---
flowchart TD
    A([Start]) --> B{Decision}
```
-->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO: Use Mermaid Plus classDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: dependency
---
classDiagram
    class ComponentA
    class ComponentB
    ComponentA --> ComponentB
```
-->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO: Use Mermaid Plus erDiagram (YAML frontmatter inside mermaid block).
```mermaid
---
id: db-model
---
erDiagram
    ENTITY {
        string id PK
    }
```
-->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: yaml -->
<!-- TODO: OpenRPC 1.3 as YAML. Example:
```yaml
openrpc: "1.3.2"
info:
  title: Service Name
  version: "1.0.0"
methods: []
```
-->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: yaml -->
<!-- TODO: JSON Schema as YAML. Example:
```yaml
"$schema": "https://json-schema.org/draft/2020-12/schema"
type: object
properties:
  id:
    type: string
required: [id]
```
-->

### Config
<!-- type: config lang: yaml -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: mermaid -->

<!-- TODO: Use Mermaid Plus requirementDiagram with element nodes and verifies relationships.
```mermaid
---
id: test-plan
---
requirementDiagram

element T1 {
  type: "Test"
}

element T2 {
  type: "Test"
}

T1 - verifies -> R1
T2 - verifies -> R2
```
-->

## Changes

<!-- type: changes lang: yaml -->

```yaml
changes:
  - path: crates/sdd/src/generate/marker.rs
    action: create
    description: |
      CODEGEN marker parser, replacer, and SPEC-REF emitter.
      pub struct CodegenBlock { pub spec_ref: String, pub content: String }
      pub fn parse_codegen_blocks(file: &str) -> Vec<CodegenBlock>
      pub fn replace_codegen_block(file: &str, spec_ref: &str, new_content: &str) -> String
      pub fn emit_spec_ref(spec_path: &str, section: &str, task: &str, lang: Lang) -> String
  - path: crates/sdd/src/generate/markers_tracker.rs
    action: create
    description: |
      Reads/writes .score/codegen_markers.yaml.
      pub fn update_markers(markers_path: &Path, spec_ref: &str, markers: Vec<MarkerEntry>) -> Result<()>
      pub fn read_markers(markers_path: &Path) -> Result<HashMap<String, Vec<MarkerEntry>>>
  - path: crates/sdd/src/generate/mod.rs
    action: modify
    description: Add pub mod marker; pub mod markers_tracker;
  - path: projects/score/cli/src/commands.rs
    action: modify
    description: |
      Add to GenCommands enum:
      InitMarkers { file: String, spec: String, symbol: Option<String> }
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: yaml -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: yaml -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
