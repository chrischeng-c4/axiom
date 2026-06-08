---
id: syntax-and-codegen
type: spec
title: "Mamba Syntax and Codegen Enhancements"
version: 1
spec_type: utility
created_at: 2026-02-20T17:35:45.659791+00:00
updated_at: 2026-02-20T17:35:45.659791+00:00
requirements:
  total: 5
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-02-20T17:35:45.659791+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Mamba Syntax and Codegen Enhancements

## Overview

Implementation of advanced Python syntax features and their corresponding code generation.

## Requirements

### R1 - Loop Else Clause

```yaml
id: R1
priority: medium
status: draft
```

Support else blocks for for and while loops

### R2 - F-String Formatting

```yaml
id: R2
priority: medium
status: draft
```

Implement complex f-string format specifiers

### R3 - Starred Unpacking

```yaml
id: R3
priority: medium
status: draft
```

Support extended iterable unpacking in assignments

### R4 - Assert Statement

```yaml
id: R4
priority: medium
status: draft
```

Implement assert statement codegen

### R5 - Del Statement

```yaml
id: R5
priority: medium
status: draft
```

Implement del statement

## Acceptance Criteria

### Scenario: Loop Else Execution

- **WHEN** a for loop completes normally
- **THEN** the else block is executed

### Scenario: F-String Formatting

- **WHEN** evaluating f\"{x:.2f}\"
- **THEN** the result is formatted correctly

### Scenario: Starred Unpacking

- **WHEN** executing a, *b, c = [1, 2, 3, 4, 5]
- **THEN** variables are assigned correctly

</spec>
