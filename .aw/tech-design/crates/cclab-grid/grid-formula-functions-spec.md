---
id: grid-formula-functions-spec
type: spec
title: "Grid Formula Functions Specification"
version: 1
spec_type: algorithm
created_at: 2026-01-28T07:53:19.635760+00:00
updated_at: 2026-01-28T07:53:19.635760+00:00
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
  has_semantic_diagrams: true
  diagrams:
    - type: flowchart
      title: "Wildcard Matching Logic"
history:
  - timestamp: 2026-01-28T07:53:19.635760+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Grid Formula Functions Specification

## Overview

This specification defines the implementation of missing standard formula functions and enhancements to existing ones, specifically focusing on INDEX and wildcard support in VLOOKUP and MATCH.

## Requirements

### R1 - INDEX Function

```yaml
id: R1
priority: medium
status: draft
```

Implement the INDEX function: INDEX(array, row_num, [column_num]). Support for both 1D and 2D arrays.

### R2 - Wildcard Support

```yaml
id: R2
priority: medium
status: draft
```

Add support for wildcards (* and ?) in MATCH and VLOOKUP when match_type is 0 (exact match).

### R3 - Additional Functions

```yaml
id: R3
priority: medium
status: draft
```

Implement additional mathematical and statistical functions (e.g., MEDIAN, STDEV.P, STDEV.S).

## Acceptance Criteria

### Scenario: INDEX 2D

- **GIVEN** A 2D range A1:C3 with values
- **WHEN** Evaluating =INDEX(A1:C3, 2, 3)
- **THEN** The value from row 2, column 3 should be returned.

### Scenario: MATCH with Wildcard

- **GIVEN** A list of names ["Apple", "Banana", "Cherry"]
- **WHEN** Evaluating =MATCH("B*", A1:A3, 0)
- **THEN** The index 2 (Banana) should be returned.

### Scenario: VLOOKUP with Wildcard

- **GIVEN** A table with names and scores
- **WHEN** Evaluating =VLOOKUP("App?", A1:B3, 2, FALSE)
- **THEN** The score for the matching name should be returned.

## Diagrams

### Wildcard Matching Logic

```mermaid
flowchart TB
    start[Start Match]
    check_wildcard{Contains * or ?} 
    regex_convert[Convert to Regex]
    exact_match[String Equality]
    regex_match[Regex Match]
    end[End Match]
    start --> check_wildcard
    check_wildcard -->|Yes| regex_convert
    check_wildcard -->|No| exact_match
    regex_convert --> regex_match
    regex_match --> end
    exact_match --> end
```

<semantic-data>

```json
{
  "edges": [],
  "metadata": null,
  "nodes": [
    {
      "id": "start",
      "semantic": {
        "type": "start"
      }
    },
    {
      "id": "check_wildcard",
      "semantic": {
        "type": "condition"
      }
    },
    {
      "id": "regex_convert",
      "semantic": {
        "type": "transform"
      }
    },
    {
      "id": "exact_match",
      "semantic": {
        "type": "condition"
      }
    },
    {
      "id": "regex_match",
      "semantic": {
        "type": "condition"
      }
    },
    {
      "id": "end",
      "semantic": {
        "type": "end"
      }
    }
  ]
}
```

</semantic-data>

</spec>
