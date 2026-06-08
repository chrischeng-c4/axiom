---
id: mamba-tuple-cmp-spec
main_spec_ref: "cclab-mamba/runtime/tuple-ops.md"
---

# Mamba Tuple Cmp Spec

## Overview

Add lexicographic comparison operators (`<`, `<=`, `>`, `>=`) for tuples in the mamba runtime, matching Python 3.12 semantics. Element-by-element comparison with length tiebreaker. Existing `==`/`!=` remain unchanged. The implementation adds dedicated `mb_tuple_lt/le/gt/ge` functions in `tuple_ops.rs` and ensures they integrate with the centralized comparison dispatch in `builtins.rs` (`mb_values_lt` → `seq_lt` pattern).
## Requirements

### R5 - Tuple Ordering Operators

```yaml
id: R5
priority: high
```

| Function | Python Equivalent | Description |
|----------|-------------------|-------------|
| `mb_tuple_lt(a, b) -> MbValue` | `a < b` | Lexicographic less-than |
| `mb_tuple_le(a, b) -> MbValue` | `a <= b` | Lexicographic less-or-equal |
| `mb_tuple_gt(a, b) -> MbValue` | `a > b` | Lexicographic greater-than |
| `mb_tuple_ge(a, b) -> MbValue` | `a >= b` | Lexicographic greater-or-equal |

**Algorithm**: Compare element-by-element. First differing pair determines the result. If all compared elements are equal, shorter tuple is less.

**Heterogeneous types**: Elements are compared using `mb_values_lt`/`mb_values_eq` (same dispatch as standalone comparison). If elements are not orderable, return `false` (consistent with current builtins behavior).

**Integration**: These functions must be callable from both:
1. Direct `mb_tuple_*` calls (from tuple_ops module)
2. `mb_values_lt/le/gt/ge` dispatch in builtins.rs (already has `seq_lt` for lt; add `seq_le`, `seq_gt`, `seq_ge` or derive from lt+eq)
## Scenarios

### Scenario: Basic less-than

- **GIVEN** `t1 = (1, 2)`, `t2 = (1, 3)`
- **WHEN** `t1 < t2`
- **THEN** Returns `True`

### Scenario: Equal tuples not less-than

- **GIVEN** `t1 = (1, 2)`, `t2 = (1, 2)`
- **WHEN** `t1 < t2`
- **THEN** Returns `False`

### Scenario: Prefix ordering (shorter is less)

- **GIVEN** `t1 = (1, 2)`, `t2 = (1, 2, 0)`
- **WHEN** `t1 < t2`
- **THEN** Returns `True`

### Scenario: Greater-than

- **GIVEN** `t1 = (1, 3)`, `t2 = (1, 2)`
- **WHEN** `t1 > t2`
- **THEN** Returns `True`

### Scenario: Less-or-equal with equal

- **GIVEN** `t1 = (1, 2)`, `t2 = (1, 2)`
- **WHEN** `t1 <= t2`
- **THEN** Returns `True`

### Scenario: Greater-or-equal with greater

- **GIVEN** `t1 = (2,)`, `t2 = (1,)`
- **WHEN** `t1 >= t2`
- **THEN** Returns `True`

### Scenario: Empty tuple comparison

- **GIVEN** `t1 = ()`, `t2 = (1,)`
- **WHEN** `t1 < t2`
- **THEN** Returns `True`

### Scenario: Empty tuple equal

- **GIVEN** `t1 = ()`, `t2 = ()`
- **WHEN** `t1 <= t2`
- **THEN** Returns `True`
## Diagrams

## API Spec

## Changes

| File | Action | Description |
|------|--------|-------------|
| `crates/cclab-mamba/src/runtime/tuple_ops.rs` | Modify | Add `mb_tuple_lt`, `mb_tuple_le`, `mb_tuple_gt`, `mb_tuple_ge` functions |
| `crates/cclab-mamba/src/runtime/builtins.rs` | Modify | Add `seq_le`, `seq_gt`, `seq_ge` helpers; wire Tuple arms in `mb_values_le/gt/ge` |
| `crates/cclab-mamba/tests/fixtures/conformance/data_structures/tuple_ops.py` | Modify | Uncomment/add comparison test cases |
| `cclab/specs/cclab-mamba/runtime/tuple-ops.md` | Modify | Add R5 (ordering operators) requirement |