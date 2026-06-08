---
id: grid-formula-array-spec
type: spec
title: "Grid Array Formula Specification"
version: 1
spec_type: integration
created_at: 2026-01-28T07:53:35.246439+00:00
updated_at: 2026-01-28T07:53:35.246439+00:00
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
history:
  - timestamp: 2026-01-28T07:53:35.246439+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Grid Array Formula Specification

## Overview

This specification defines the support for array formulas and dynamic arrays in cclab-grid. It covers the evaluation process where a single formula can return multiple values that 'spill' into adjacent cells.

## Requirements

### R1 - Range Value Support

```yaml
id: R1
priority: medium
status: draft
```

Allow the formula evaluator to return a 2D array of CellValue (RangeValue).

### R2 - Dynamic Spilling

```yaml
id: R2
priority: medium
status: draft
```

Automatically populate adjacent cells with the results of an array-returning formula.

### R3 - Spill Collision Detection

```yaml
id: R3
priority: medium
status: draft
```

Detect and handle cases where the spill range is blocked by existing data (#SPILL! error).

### R4 - Spill Range Integrity

```yaml
id: R4
priority: medium
status: draft
```

Ensure that editing a cell in a spill range (other than the origin) is prevented or handled correctly.

## Acceptance Criteria

### Scenario: Successful Spill

- **GIVEN** A formula that returns a 3x1 array in A1
- **WHEN** Evaluating the formula.
- **THEN** A1, A2, and A3 should show the results, and the selection of A1 should show the spill border.

### Scenario: Spill Blocked

- **GIVEN** A formula in A1 that needs to spill to A2, but A2 has data
- **WHEN** Evaluating the formula.
- **THEN** A1 should display a #SPILL! error.

### Scenario: Clear Spill Source

- **GIVEN** An active spill range A1:A3
- **WHEN** Deleting the formula in A1.
- **THEN** A1, A2, and A3 should all become empty.

## Flow Diagram

```mermaid
sequenceDiagram
    participant Evaluator as Formula Evaluator
    participant Function as Array Function (e.g. FILTER)
    participant SheetManager as Sheet Manager
    participant GridStorage as Grid Storage (Chunks)

    Evaluator->>Function: evaluate(Expr)
    Function-->>Evaluator: return RangeValue(2x2)
    Evaluator-->>SheetManager: return RangeValue(2x2)
    SheetManager->>SheetManager: calculate_spill_range(A1, 2x2) -> A1:B2
    SheetManager->>GridStorage: is_blocked(A1:B2)?
    GridStorage-->>SheetManager: No
    SheetManager->>GridStorage: write_spill(A1:B2)
```

</spec>
