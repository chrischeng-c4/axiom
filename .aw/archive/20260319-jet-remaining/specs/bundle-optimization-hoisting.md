---
id: bundle-optimization-hoisting
type: spec
title: "Bundle Optimization: Scope Hoisting & Module Flattening"
version: 1
spec_type: utility
main_spec_ref: cclab-jet/scope-hoisting.md
merge_strategy: extend
created_at: 2026-03-19T07:11:59.171928+00:00
updated_at: 2026-03-19T07:11:59.171928+00:00
requirements:
  total: 7
  ids:
    - R1
    - R2
    - R3
    - R4
    - R5
    - R6
    - R7
design_elements:
  has_mermaid: false
  has_json_schema: false
  has_pseudo_code: false
  has_api_spec: false
  has_semantic_diagrams: false
history:
  - timestamp: 2026-03-19T07:11:59.171928+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

<spec>

# Bundle Optimization: Scope Hoisting & Module Flattening

## Overview

This specification defines the implementation of advanced scope hoisting and 'true' module flattening to optimize bundle size for the Jet bundler. The primary goal is to reduce the bundle size of 'react-bench' from 215KB to ≤ 196KB, achieving parity with industry standards like Webpack and Vite. This is achieved by moving beyond simple module wrappers to a unified scope where variables are renamed with module prefixes, allowing for aggressive cross-module optimizations.

## Requirements

### R1 - Module Concatenation

```yaml
id: R1
priority: medium
status: draft
```

Implement module concatenation for non-circular, single-import modules to remove __jet__.define/require wrappers.

### R2 - True Module Flattening

```yaml
id: R2
priority: medium
status: draft
```

Implement true module flattening by merging module bodies into a single function scope, skipping only unsafe modules (eval, with, arguments).

### R3 - Prefix-based Renaming

```yaml
id: R3
priority: medium
status: draft
```

Implement a prefix-based renaming strategy (e.g., _mN_name) for all top-level variables within the flattened scope to prevent collisions.

### R4 - Cross-module Constant Inlining

```yaml
id: R4
priority: medium
status: draft
```

Enable cross-module constant inlining by identifying immutable bindings and propagating their values across module boundaries.

### R5 - Unified Cross-module DCE

```yaml
id: R5
priority: medium
status: draft
```

Implement unified cross-module dead code elimination (DCE) to remove unused exports and unreferenced functions/classes within the merged scope.

### R6 - Mangler Visibility Update

```yaml
id: R6
priority: medium
status: draft
```

Update the variable mangler to support visibility of module-prefixed variables for optimal compression.

### R7 - Bundle Size Target

```yaml
id: R7
priority: medium
status: draft
```

Target a final bundle size of ≤ 196KB for the react-bench project.

## Acceptance Criteria

### Scenario: Bundle Size Parity

- **WHEN** building 'react-bench'
- **THEN** the resulting JS bundle size must be ≤ 196KB.

### Scenario: Variable Renaming

- **WHEN** two modules both define a top-level 'App' variable
- **THEN** they must be renamed to unique identifiers (e.g., _m1_App, _m2_App) in the flattened output.

### Scenario: Circular Dependency Handling

- **WHEN** a module contains a circular dependency
- **THEN** it must preserve its module boundary and continue using the fallback require switch.

### Scenario: Cross-module DCE

- **WHEN** a constant is exported from one module and used as a condition in another
- **THEN** the branch must be constant-folded and dead paths removed during unified DCE.

</spec>
