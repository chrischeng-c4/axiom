---
id: string-reverse-slice-fix
main_spec_ref: "crates/mamba/runtime/string-ops"
merge_strategy: new
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# String Reverse Slice Fix

## Overview

`mb_str_slice_full(s, start, stop, step)` handles Python `str[start:stop:step]` slicing. When `step < 0` and `stop` is absent (Python `None`), the code passes the sentinel value `-1` to `clamp_rev_str(-1, len)`. `clamp_rev_str` normalizes negative indices by adding `len`, so `-1` becomes `len - 1` (the last character index). For `'abcdef'[::-1]` (len=6): `s_idx = clamp_rev_str(5, 6) = 5`, `e_idx = clamp_rev_str(-1, 6) = 5`. The loop condition `while i > e_idx` becomes `while 5 > 5` — immediately false, producing an empty string.

Fix: when `stop` is `None` (i.e., `stop.as_int()` returns `None`) and `step < 0`, bypass `clamp_rev_str` and use `-1` directly as `e_idx`. The loop `while i > -1` then iterates from index `len-1` down to `0` inclusive, producing the correct reversed string. Same treatment for `start` when absent: use `len - 1` directly without clamping.

Issue: #1111
## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R1 | Absent stop with negative step uses literal -1 | P0 | When `stop.as_int()` returns `None` and `step < 0`, `e_idx` is set to `-1` directly — not passed through `clamp_rev_str`. `'abcdef'[::-1]` produces `'fedcba'` |
| R2 | Absent start with negative step uses len-1 directly | P0 | When `start.as_int()` returns `None` and `step < 0`, `s_idx` is set to `len - 1` directly — not passed through `clamp_rev_str`. Consistent with R1 |
| R3 | Explicit negative indices still normalize correctly | P0 | `'abcdef'[-2::-1]` produces `'edcba'` (start at index -2 → 4, iterate down to 0). Explicit negative indices continue to go through `clamp_rev_str` |
| R4 | Explicit stop with negative step still clamps | P0 | `'abcdef'[4:1:-1]` produces `'edc'` (start at 4, stop at 1, step -1). Explicit stop values continue through `clamp_rev_str` |
| R5 | Positive step slicing unchanged | P0 | `'abcdef'[1:4]` still produces `'bcd'`. Positive step path is not modified |
| R6 | Step=-1 full reverse matches CPython 3.12 | P0 | `s[::-1]` for any string `s` produces the same result as CPython 3.12's `s[::-1]` |

### Constraints

- Fix is localized to `mb_str_slice_full` in `string_ops.rs` — only the negative-step branch for absent start/stop
- `clamp_rev_str` function itself is not modified — it remains correct for explicit index values
- `mb_str_slice` (2-arg slice without step) is unaffected
## Scenarios

### S1: Full reverse slice s[::-1] produces reversed string (R1, R2, R6)

**GIVEN** `s = 'abcdef'; print(s[::-1])`
**WHEN** executed through Mamba JIT
**THEN** output is `fedcba` — `s_idx=5`, `e_idx=-1`, loop iterates 5→4→3→2→1→0

### S2: Full reverse of single-char string (R1, R2, R6)

**GIVEN** `s = 'x'; print(s[::-1])`
**WHEN** executed
**THEN** output is `x` — `s_idx=0`, `e_idx=-1`, loop iterates once at index 0

### S3: Full reverse of empty string (R1, R2, R6)

**GIVEN** `s = ''; print(s[::-1])`
**WHEN** executed
**THEN** output is empty string — `len=0`, `s_idx=-1` (or 0-1), loop condition `while -1 > -1` is false

### S4: Partial reverse with explicit start (R3)

**GIVEN** `s = 'abcdef'; print(s[-2::-1])`
**WHEN** executed
**THEN** output is `edcba` — start=-2 normalizes to 4 via `clamp_rev_str`, stop absent → `e_idx=-1`

### S5: Partial reverse with explicit start and stop (R4)

**GIVEN** `s = 'abcdef'; print(s[4:1:-1])`
**WHEN** executed
**THEN** output is `edc` — both start=4 and stop=1 go through `clamp_rev_str`, loop 4→3→2

### S6: Positive step slicing unaffected (R5)

**GIVEN** `s = 'abcdef'; print(s[1:4])`
**WHEN** executed
**THEN** output is `bcd` — positive step path uses `normalize_index`, unchanged

### S7: Step=-2 skipping reverse (R1, R2)

**GIVEN** `s = 'abcdef'; print(s[::-2])`
**WHEN** executed
**THEN** output is `fdb` — start at index 5, step -2: indices 5, 3, 1

### S8: Unicode string reverse (R6)

**GIVEN** `s = '你好世界'; print(s[::-1])`
**WHEN** executed
**THEN** output is `界世好你` — char-level iteration handles multi-byte correctly
## Diagrams

### Interaction
<!-- type: interaction lang: mermaid -->
<!-- TODO -->

### Logic
<!-- type: logic lang: mermaid -->
<!-- TODO -->

### Dependencies
<!-- type: dependency lang: mermaid -->
<!-- TODO -->

### State Machine
<!-- type: state-machine lang: mermaid -->
<!-- TODO -->

### Data Model
<!-- type: db-model lang: mermaid -->
<!-- TODO -->

## API Spec

### REST API
<!-- type: rest-api lang: yaml -->
<!-- TODO -->

### RPC API
<!-- type: rpc-api lang: json -->
<!-- TODO -->

### Async API
<!-- type: async-api lang: yaml -->
<!-- TODO -->

### CLI
<!-- type: cli lang: yaml -->
<!-- TODO -->

### Schema
<!-- type: schema lang: json -->
<!-- TODO -->

### Config
<!-- type: config lang: json -->
<!-- TODO -->

## Test Plan
<!-- type: test-plan lang: markdown -->

<!-- TODO -->

## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/string_ops.rs
    action: MODIFY
    desc: "In `mb_str_slice_full`, replace the negative-step branch that unconditionally calls `clamp_rev_str` on both start and stop with None-aware logic: (1) `s_idx = match start.as_int() { Some(v) => clamp_rev_str(v, len), None => len - 1 }` — absent start defaults to last char index without clamping; (2) `e_idx = match stop.as_int() { Some(v) => clamp_rev_str(v, len), None => -1 }` — absent stop uses literal -1 so loop condition `while i > -1` iterates down to index 0 inclusive. The positive-step branch and `clamp_rev_str` function are unchanged."
```
## Wireframe
<!-- type: wireframe lang: yaml -->

<!-- TODO -->

## Component
<!-- type: component lang: json -->

<!-- TODO -->

## Design Token
<!-- type: design-token lang: json -->

<!-- TODO -->

## Doc
<!-- type: doc lang: markdown -->

<!-- TODO -->

# Reviews
