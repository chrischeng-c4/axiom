---
id: unified-refactoring-engine
type: spec
title: "Unified Refactoring Engine"
version: 1
spec_type: algorithm
created_at: 2026-01-31T10:33:01.498327+00:00
updated_at: 2026-01-31T10:33:01.498327+00:00
requirements:
  total: 4
  ids:
    - R1
    - R2
    - R3
    - R4
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Unified Refactoring Flow"
history:
  - timestamp: 2026-01-31T10:33:01.498327+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Unified Refactoring Engine

## Overview

A language-agnostic refactoring engine that supports complex operations like Rename, Extract Function, and Extract Variable across Python, TypeScript, and Rust. It uses Prism's deep semantic understanding to ensure refactorings are safe and maintain code correctness.

## Requirements

### R1 - Unified Mutable AST

```yaml
id: R1
priority: high
status: draft
```

Implement a unified 'MutableAst' model that can represent and manipulate code across different languages.

### R2 - Cross-File Rename

```yaml
id: R2
priority: high
status: draft
```

Implement cross-file symbol renaming that correctly updates all references in all supported languages.

### R3 - Core Refactorings (TS/Rust)

```yaml
id: R3
priority: high
status: draft
```

Provide 'Extract Function' and 'Extract Variable' refactorings for Rust and TypeScript, matching the existing Python capabilities.

### R4 - Refactoring Validation

```yaml
id: R4
priority: medium
status: draft
```

Include a validation step that checks for potential name collisions or type errors introduced by the refactoring.

## Acceptance Criteria

### Scenario: Rust Cross-File Rename

- **GIVEN** A Rust project with a struct used in multiple modules.
- **WHEN** Renaming the struct.
- **THEN** All occurrences of the struct name and its usages should be updated correctly.

### Scenario: TS Extract Function

- **GIVEN** A TypeScript function with complex logic.
- **WHEN** Extracting a block of code into a new function.
- **THEN** The selected code should be moved to a new function with correct parameters and return type inferred by Prism.

### Scenario: Refactoring Collision Detection

- **GIVEN** A variable name that already exists in the scope.
- **WHEN** Attempting to rename a symbol to a name that would cause a conflict.
- **THEN** Prism should issue a warning or error about the name collision.

## Diagrams

### Unified Refactoring Flow

```mermaid
flowchart TB
    Request[Refactoring Request]
    Analyze[Semantic & Data Flow Analysis]
    SearchRefs[Find All References (Cross-File)]
    GenEdits[Generate AST-based Edits (MutableAst)]
    Validate[Validate Edits (Syntax/Type Check)]
    Apply[Apply Text Edits to Files]
    Request --> Analyze
    Analyze --> SearchRefs
    SearchRefs --> GenEdits
    GenEdits --> Validate
    Validate --> Apply
```

</spec>
