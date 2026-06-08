---
id: pulsar-stats
type: spec
title: "Pulsar Stats"
version: 1
spec_type: utility
created_at: 2026-01-31T09:34:21.019120+00:00
updated_at: 2026-01-31T09:34:21.019120+00:00
requirements:
  total: 3
  ids:
    - R1
    - R2
    - R3
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-01-31T09:34:21.019120+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Pulsar Stats

## Overview

Implementation of a comprehensive statistical library for Pulsar, mirroring core SciPy.stats functionality including a wide range of distributions and hypothesis tests.

## Requirements

### R1 - Comprehensive Distributions

```yaml
id: R1
priority: high
status: draft
```

Implement a broad set of probability distributions: Normal, Binomial, Poisson, Uniform, Exponential, Gamma, Beta, and Student-t.

### R2 - Statistical Hypothesis Testing

```yaml
id: R2
priority: high
status: draft
```

Provide core hypothesis tests: Independent/Related T-tests, One-way ANOVA, Chi-square tests, and Pearson correlation test.

### R3 - Advanced Descriptive Stats

```yaml
id: R3
priority: medium
status: draft
```

Implement descriptive statistics expansion: skew, kurtosis, z-score, and moment calculations.

## Acceptance Criteria

### Scenario: Beta Distribution PDF happy path

- **GIVEN** Parameters alpha=2, beta=5.
- **WHEN** Beta::new(2.0, 5.0).pdf(0.5) is called.
- **THEN** The PDF value at x=0.5 should be calculated correctly.

### Scenario: One-way ANOVA happy path

- **GIVEN** Multiple groups of samples.
- **WHEN** anova_oneway(&[group1, group2, group3]) is called.
- **THEN** The F-statistic and p-value are returned.

### Scenario: Z-score calculation happy path

- **GIVEN** An array of samples.
- **WHEN** zscore(&arr) is called.
- **THEN** An array of z-scores is returned where each element is (x - mean) / std.

</spec>
