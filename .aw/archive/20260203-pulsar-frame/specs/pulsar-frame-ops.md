---
id: pulsar-frame-ops
type: spec
title: "Pulsar Frame Ops"
version: 1
spec_type: utility
created_at: 2026-01-30T06:32:37.085794+00:00
updated_at: 2026-01-30T06:32:37.085794+00:00
requirements:
  total: 2
  ids:
    - R1
    - R2
design_elements:
  has_mermaid: true
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
  diagrams:
    - type: flowchart
      title: "Join Logic"
history:
  - timestamp: 2026-01-30T06:32:37.085794+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Frame Ops

## Overview

Defines the GroupBy and Join operations for DataFrames. This includes aggregation functions (sum, mean, count) on grouped data and database-style joins (inner, left, outer) between two DataFrames.

## Requirements

### R1 - GroupBy

```yaml
id: R1
priority: medium
status: draft
```

Implement GroupBy aggregations.

### R2 - Join

```yaml
id: R2
priority: medium
status: draft
```

Implement Join operations.

## Acceptance Criteria

### Scenario: Sum Aggregation

- **GIVEN** Grouped DataFrame
- **WHEN** Call sum
- **THEN** Sum returned

### Scenario: Inner Join

- **GIVEN** Two DataFrames
- **WHEN** Call join
- **THEN** Joined DF returned

### Scenario: Left Join

- **GIVEN** Two DataFrames
- **WHEN** Call left_join
- **THEN** Left Joined DF returned

## Diagrams

### Join Logic

```mermaid
flowchart TB
    Start((Start Join))
    CheckKeys{Check Keys} 
    Merge[Merge Rows]
    Error[Return Error]
    Start -->|call| CheckKeys
    CheckKeys -->|keys match| Merge
    CheckKeys -.->|no match| Error
```

</spec>
