---
id: block-plus-spec
type: spec
title: "Mermaid+ Block Diagram Specification"
version: 1
spec_type: algorithm
tags: [logic]
created_at: 2026-02-12T10:23:05.753276+00:00
updated_at: 2026-02-12T10:23:05.753276+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Block+ Processing Flow"
changes:
  - file: crates/cclab-sdd/src/diagrams/block_plus/mod.rs
    action: CREATE
    description: "Add block_plus diagram module."
  - file: crates/cclab-sdd/src/diagrams/block_plus/schema.rs
    action: CREATE
    description: "Add block_plus schema."
  - file: crates/cclab-sdd/src/diagrams/block_plus/generator.rs
    action: CREATE
    description: "Add block_plus generator."
  - file: crates/cclab-sdd/src/diagrams/block_plus/validator.rs
    action: CREATE
    description: "Add block_plus validator."
  - file: crates/cclab-sdd/src/diagrams/mod.rs
    action: MODIFY
    description: "Register block_plus module."
history:
  - timestamp: 2026-02-12T10:23:05.753276+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
capability_refs:
  - id: td-cb-lifecycle-automation
    role: primary
    gap: cb-lifecycle-dispatch
    claim: cb-lifecycle-dispatch
    coverage: full
    rationale: "Codegen TDs support CB lifecycle generation and regenerable artifact production."
---

<spec>

# Mermaid+ Block Diagram Specification

## Overview
<!-- type: overview lang: markdown -->

Specification for the Mermaid+ Block Diagram generator. This diagram type supports columns, nested blocks, edges, and shapes with YAML frontmatter validation.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: block-plus-requirements
---
requirementDiagram
    requirement R1 {
        id: R1
        text: Support Mermaid block-beta syntax including columns, blocks, edges, and shapes.
        risk: medium
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: Validate YAML frontmatter for block definitions, nested blocks, and connections.
        risk: medium
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Emit Mermaid+ output with frontmatter inside the diagram code block.
        risk: medium
        verifymethod: test
    }
```

## Acceptance Criteria
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - name: valid-block-generation
    given: A valid block definition with columns and nested blocks.
    when: Calling sdd_generate_block_plus.
    then: Returns Mermaid+ output with correct block-beta syntax and frontmatter.
  - name: missing-id-validation
    given: A block definition missing required fields such as id.
    when: Calling sdd_generate_block_plus.
    then: Returns a validation error.
```

## Diagrams
<!-- type: diagram lang: mermaid -->

### Block+ Processing Flow

```mermaid
flowchart TB
    Input[Structured Input]
    Validate{Schema Validation} 
    Convert[Convert to Mermaid Syntax]
    GenerateMermaid[Generate Mermaid Code]
    CombineOutput[Combine: code fence + frontmatter + diagram]
    Result[Mermaid+ Output]
    ReturnErrors[Return Validation Errors]
    Input --> Validate
    Validate -->|Valid| Convert
    Validate -->|Invalid| ReturnErrors
    Convert --> GenerateMermaid
    GenerateMermaid --> CombineOutput
    CombineOutput --> Result
```

</spec>

## Changes
<!-- type: changes lang: yaml -->

```yaml
changes:
  - action: annotate
    section: requirements
    impl_mode: hand-written
    description: "Traceability metadata edge for the requirements section."

  - action: annotate
    section: scenarios
    impl_mode: hand-written
    description: "Traceability metadata edge for the scenarios section."

```