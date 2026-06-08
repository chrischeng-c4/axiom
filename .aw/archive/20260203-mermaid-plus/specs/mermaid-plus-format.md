---
id: mermaid-plus-format
type: spec
title: "Mermaid+ Format and Tooling Specification"
version: 1
spec_type: algorithm
created_at: 2026-01-29T15:03:17.731932+00:00
updated_at: 2026-01-29T15:03:17.731932+00:00
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
    - type: flowchart
      title: "Mermaid+ Tool Processing Flow"
history:
  - timestamp: 2026-01-29T15:03:17.731932+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Mermaid+ Format and Tooling Specification

## Overview

This specification defines the Mermaid+ format, which combines a structured state machine definition in YAML frontmatter with a Mermaid stateDiagram-v2. It covers the data model, validation, and the generation logic integrated into the cclab-aurora crate, while maintaining the IR-dependent parser in cclab-prism to avoid circular dependencies.

## Requirements

### R1 - Data Model and Schema Definition

```yaml
id: R1
priority: high
status: draft
```

Define the structured state machine data model and update schemas/spec.schema.json to support Mermaid+ composite structures.

### R2 - Mermaid+ Generation Logic in Aurora

```yaml
id: R2
priority: high
status: draft
```

Implement the state machine to Mermaid stateDiagram-v2 conversion algorithm in crates/cclab-aurora/src/diagrams/mermaid_plus.rs. This implementation must be independent of cclab-prism IR.

### R3 - Prism Refactoring

```yaml
id: R3
priority: medium
status: draft
```

Refactor cclab-prism to use cclab-aurora for its state machine generation and validation. The Mermaid parser must remain in prism to map diagrams to Prism IR.

### R4 - Semantic Validation in Genesis

```yaml
id: R4
priority: medium
status: draft
```

Add Mermaid+ semantic validation in crates/cclab-genesis/src/validator/semantic.rs using the aurora-based validator to ensure consistency between structured definitions and visual diagrams.

### R5 - Orchestrator Prompt Optimization

```yaml
id: R5
priority: low
status: draft
```

Optimize Orchestrator prompts in crates/cclab-genesis/src/orchestrator/prompts.rs to guide LLMs toward using Mermaid+ for algorithms and workflows.

## Acceptance Criteria

### Scenario: Generate Mermaid+ from valid definition

- **GIVEN** A valid state machine definition with 'idle' and 'loading' states.
- **WHEN** The MermaidPlusGenerator is called.
- **THEN** The output should contain YAML frontmatter and a corresponding Mermaid stateDiagram-v2.

### Scenario: Validation error for invalid definition

- **GIVEN** An invalid state machine definition missing the 'initial' state.
- **WHEN** The StateMachineValidator is called.
- **THEN** The validator should return a structured error indicating the missing 'initial' state.

### Scenario: Nested state rendering verification

- **GIVEN** A state machine definition with hierarchical (nested) states.
- **WHEN** The MermaidPlusGenerator is called.
- **THEN** The Mermaid output should correctly use the 'state "..." as id { ... }' syntax.

### Scenario: Semantic validation failure for inconsistent specs

- **GIVEN** A specification file with an XState definition that contradicts the Mermaid diagram.
- **WHEN** genesis_validate_proposal is executed.
- **THEN** The semantic validator should report a HIGH severity inconsistency.

## Diagrams

### Mermaid+ Tool Processing Flow

```mermaid
flowchart TB
    Input[User Input (JSON)]
    Validate[Validate Schema]
    Errors[Return Errors]
    Convert[Convert to Mermaid AST]
    Generate[Generate Mermaid Code]
    Combine[Combine YAML + Mermaid]
    Output[Final Mermaid+ Output]
    Input --> Validate
    Validate -->|Valid| Convert
    Validate -->|Invalid| Errors
    Convert --> Generate
    Generate --> Combine
    Combine --> Output
```

</spec>
