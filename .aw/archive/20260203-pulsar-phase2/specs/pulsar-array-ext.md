---
id: pulsar-array-ext
type: spec
title: "Pulsar Array Extensions"
version: 1
spec_type: algorithm
created_at: 2026-01-31T09:34:07.964668+00:00
updated_at: 2026-01-31T09:34:07.964668+00:00
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
      title: "Skewness/Kurtosis Calculation Flow"
history:
  - timestamp: 2026-01-31T09:34:07.964668+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Array Extensions

## Overview

Expansion of pulsar-array with a focus on comprehensive statistical functions, complex number support, and advanced linear algebra.

## Requirements

### R1 - Advanced Statistical Functions

```yaml
id: R1
priority: high
status: draft
```

Implement missing high-priority statistical functions: median, mode, skewness, and kurtosis with axis support.

### R2 - Complex Number Support

```yaml
id: R2
priority: high
status: draft
```

Add support for complex numbers (Complex32, Complex64) to the DType system and array operations.

### R3 - Matrix Decompositions

```yaml
id: R3
priority: medium
status: draft
```

Implement QR and SVD decompositions for advanced linear algebra workflows.

### R4 - Random Module Expansion

```yaml
id: R4
priority: medium
status: draft
```

Enhance the Random module with more distributions (e.g., Gamma, Beta, Exponential).

## Acceptance Criteria

### Scenario: Median Calculation happy path

- **GIVEN** An NdArray of floats.
- **WHEN** arr.median() is called.
- **THEN** The median value is returned correctly, handling even/odd lengths.

### Scenario: Skewness Calculation with NaN happy path

- **GIVEN** An NdArray with missing values (NaN).
- **WHEN** arr.skew() is called.
- **THEN** The skewness is computed ignoring NaNs if specified, or returns NaN.

### Scenario: Complex Matrix Multiplication happy path

- **GIVEN** A complex NdArray.
- **WHEN** a.matmul(&b) is called.
- **THEN** The result is a correctly computed complex matrix.

## Diagrams

### Skewness/Kurtosis Calculation Flow

```mermaid
flowchart TB
    start((Start Skew/Kurtosis))
    check_nan{Has NaNs?} 
    filter_nan[Filter NaNs]
    calc_mean[Calculate Mean]
    calc_diffs[Calculate (x - mean)^n]
    calc_moments[Calculate n-th Moments]
    finalize[Apply normalization]
    end((End))
    start --> check_nan
    check_nan -->|Yes| filter_nan
    check_nan -->|No| calc_mean
    filter_nan --> calc_mean
    calc_mean --> calc_diffs
    calc_diffs --> calc_moments
    calc_moments --> finalize
    finalize --> end
```

</spec>
