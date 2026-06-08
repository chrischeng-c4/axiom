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
  - file: crates/cclab-aurora/src/diagrams/block_plus/mod.rs
    action: CREATE
    description: "Add block_plus diagram module."
  - file: crates/cclab-aurora/src/diagrams/block_plus/schema.rs
    action: CREATE
    description: "Add block_plus schema."
  - file: crates/cclab-aurora/src/diagrams/block_plus/generator.rs
    action: CREATE
    description: "Add block_plus generator."
  - file: crates/cclab-aurora/src/diagrams/block_plus/validator.rs
    action: CREATE
    description: "Add block_plus validator."
  - file: crates/cclab-aurora/src/diagrams/mod.rs
    action: MODIFY
    description: "Register block_plus module."
history:
  - timestamp: 2026-02-12T10:23:05.753276+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Mermaid+ Block Diagram Specification

## Overview

Specification for the Mermaid+ Block Diagram generator. This diagram type supports columns, nested blocks, edges, and shapes with YAML frontmatter validation.

## Requirements

### R1 - Mermaid Block Syntax Support

```yaml
id: R1
priority: medium
status: draft
```

Support Mermaid block-beta diagram syntax including 'columns' and 'block' definitions.

### R2 - Frontmatter Validation

```yaml
id: R2
priority: medium
status: draft
```

Support YAML frontmatter validation for block definitions, including nested blocks and connections.

### R3 - Mermaid+ Format Compliance

```yaml
id: R3
priority: medium
status: draft
```

Adhere to Mermaid+ format with frontmatter inside the code block.

## Acceptance Criteria

### Scenario: Valid Block Generation

- **GIVEN** A valid block definition with columns and nested blocks.
- **WHEN** Calling aurora_generate_block_plus.
- **THEN** Returns Mermaid+ output with correct block-beta syntax and frontmatter.

### Scenario: Missing ID Validation

- **GIVEN** A block definition missing required fields (e.g. id).
- **WHEN** Calling aurora_generate_block_plus.
- **THEN** Returns a validation error.

## Diagrams

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
