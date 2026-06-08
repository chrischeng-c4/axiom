---
id: pulsar-frame-ext
type: spec
title: "Pulsar Frame Extensions"
version: 1
spec_type: algorithm
created_at: 2026-01-31T09:34:15.102445+00:00
updated_at: 2026-01-31T09:34:15.102445+00:00
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
      title: "Missing Value Filling Flow"
history:
  - timestamp: 2026-01-31T09:34:15.102445+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Frame Extensions

## Overview

Expansion of pulsar-frame with high-priority missing value handling, advanced GroupBy transformations, and improved Join operations.

## Requirements

### R1 - Missing Value Handling

```yaml
id: R1
priority: high
status: draft
```

Implement missing value detection and handling: isna, fillna, dropna, and basic interpolation.

### R2 - GroupBy Transformations

```yaml
id: R2
priority: high
status: draft
```

Extend GroupBy with transform() and filter() methods for more complex data processing.

### R3 - Advanced Relational Joins

```yaml
id: R3
priority: medium
status: draft
```

Enhance Join/Merge with support for left_on, right_on, and customizable suffixes.

### R4 - JSON I/O Support

```yaml
id: R4
priority: low
status: draft
```

Add support for JSON reading and writing to DataFrames.

## Acceptance Criteria

### Scenario: Fillna with constant value happy path

- **GIVEN** A DataFrame with some Null values.
- **WHEN** df.fillna(Value::Int(0)) is called.
- **THEN** A new DataFrame is returned where all Nulls are replaced by 0.

### Scenario: Dropna happy path

- **GIVEN** A DataFrame with one Null column.
- **WHEN** df.dropna() is called.
- **THEN** A new DataFrame is returned without the rows containing Nulls.

### Scenario: Merge with different keys happy path

- **GIVEN** Two DataFrames with keys 'k1' and 'k2'.
- **WHEN** df1.merge(&df2, left_on="k1", right_on="k2") is called.
- **THEN** A merged DataFrame is returned using 'k1' and 'k2' as join keys.

## Diagrams

### Missing Value Filling Flow

```mermaid
flowchart TB
    input_df[Input DataFrame]
    detect_nulls[Detect Null/NaN entries]
    select_method{Select Fill Method} 
    apply_val[Apply Constant Value]
    apply_ffill[Apply Forward Fill (last valid)]
    return_df[Return Modified DataFrame]
    end((End))
    input_df --> detect_nulls
    detect_nulls --> select_method
    select_method -->|fillna(val)| apply_val
    select_method -->|fillna(forward)| apply_ffill
    apply_val --> return_df
    apply_ffill --> return_df
    return_df --> end
```

</spec>
