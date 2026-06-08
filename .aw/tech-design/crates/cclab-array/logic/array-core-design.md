---
id: pulsar-array-core-design
type: spec
title: "Pulsar Array Core Design"
version: 1
spec_type: utility
target_crate: cclab-pulsar-array-core
created_at: 2026-01-30T03:34:56.857618+00:00
updated_at: 2026-01-30T03:34:56.857618+00:00
main_spec_ref: "crates/cclab-array/src"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
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
history:
  - timestamp: 2026-01-30T03:34:56.857618+00:00
    agent: "mcp"
    tool: "create_spec"
    action: "created"
---

# Pulsar Array Core Design

## Overview
<!-- type: overview lang: markdown -->

`cclab-pulsar-array-core` provides a high-performance pure-Rust
N-dimensional array type with NumPy-like behavior. The core design centers on
contiguous storage, shape and stride metadata, dtype support, broadcasting,
slicing, indexing, and element-wise arithmetic without external dependencies.

The old file lived at
`.aw/tech-design/crates/cclab-array/pulsar-array-core-design.md`. The
canonical TD now lives under `logic/`.

## Requirements
<!-- type: requirements lang: mermaid -->

```mermaid
---
id: array-core-design-requirements
entry: R1
---
requirementDiagram
    requirement R1 {
        id: R1
        text: NdArray stores arbitrary rank data with shape and strides
        risk: high
        verifymethod: test
    }
    requirement R2 {
        id: R2
        text: DType supports core numeric and boolean types
        risk: medium
        verifymethod: test
    }
    requirement R3 {
        id: R3
        text: Operations follow NumPy-compatible broadcasting rules
        risk: high
        verifymethod: test
    }
    requirement R4 {
        id: R4
        text: Slicing and indexing avoid unnecessary copies
        risk: high
        verifymethod: test
    }
    requirement R5 {
        id: R5
        text: Arithmetic operations are element-wise Rust trait operations
        risk: medium
        verifymethod: test
    }
```

### R1: N-dimensional Storage

Arrays of arbitrary rank are stored in a contiguous memory block with shape and
stride metadata.

### R2: Flexible DType System

The dtype system supports `f32`, `f64`, `i32`, `i64`, and boolean values through
one unified representation.

### R3: Broadcasting Support

Operations between arrays of different but compatible dimensions automatically
expand shapes according to NumPy broadcasting rules.

### R4: Slicing and Indexing

Slicing and indexing provide views or targeted access to subsets of data without
unnecessary copying.

### R5: Basic Arithmetic Operations

Addition, subtraction, multiplication, and division are implemented as
element-wise operations through standard Rust traits.

## Scenarios
<!-- type: scenarios lang: yaml -->

```yaml
scenarios:
  - id: S1
    requirement: R1
    given: A 2x2 array is created with data [1, 2, 3, 4]
    when: The array is indexed at [1, 1]
    then: The value at [1, 1] is returned correctly
  - id: S2
    requirement: R3
    given: A 1D array [10] and a 2x2 array [[1, 2], [3, 4]]
    when: The arrays are added
    then: The result is [[11, 12], [13, 14]]
  - id: S3
    requirement: R4
    given: A 2x2 array [[1, 2], [3, 4]]
    when: The second column is sliced
    then: The result is a 1D array [2, 4]
```

## Changes
<!-- type: changes lang: yaml -->

```yaml
files:
  - path: .aw/tech-design/crates/cclab-array/logic/array-core-design.md
    action: MODIFY
    impl_mode: hand-written
    desc: Move array core design TD under logic and normalize sections.
  - path: crates/cclab-array/src
    action: MODIFY
    impl_mode: hand-written
    desc: Implement NdArray dtype broadcasting slicing indexing and arithmetic primitives.
```
