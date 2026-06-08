---
id: string-reverse-slice-fix
type: spec
title: "String Operations"
version: 1
spec_type: utility
files:
  - runtime/string_ops.rs
main_spec_ref: "crates/mamba/runtime/string-ops"
merge_strategy: extend
fill_sections: [overview, requirements, scenarios, changes]
filled_sections: [overview, requirements, scenarios, changes]
create_complete: true
---

# String Operations

## Overview

`mb_str_slice_full(s, start, stop, step)` handles Python `str[start:stop:step]` slicing. When `step < 0` and `start`/`stop` are absent (Python `None`), the original code passed the sentinel value through `clamp_rev_str`, which normalized `-1` to `len - 1` — collapsing `s_idx` and `e_idx` to the same value and making the loop condition `while i > e_idx` immediately false, producing an empty string.

Fix (commit bc5921e9): when `start`/`stop` are `None` and `step < 0`, bypass `clamp_rev_str` and use literal defaults — `len - 1` for absent start, `-1` for absent stop. The remaining work is converting the xfail conformance fixture (`string_edge_cases_xfail.py`) to a passing test and updating the main spec with an R6 requirement.

Issue: #1111
## Source Files

| File | LOC | Responsibility |
|------|-----|----------------|
| `runtime/string_ops.rs` | 989 | All `mb_string_*` function implementations |

## Requirements

| ID | Title | Priority | Acceptance Criteria |
|----|-------|----------|---------------------|
| R6 | Absent stop with negative step uses literal -1 | P0 | When `stop.as_int()` returns `None` and `step < 0`, `e_idx` is set to `-1` directly — not passed through `clamp_rev_str`. `'abcdef'[::-1]` produces `'fedcba'` |
| R7 | Absent start with negative step uses len-1 directly | P0 | When `start.as_int()` returns `None` and `step < 0`, `s_idx` is set to `len - 1` directly — not passed through `clamp_rev_str`. Consistent with R6 |
| R8 | Explicit negative indices still normalize correctly | P0 | `'abcdef'[-2::-1]` produces `'edcba'` (start=-2 → index 4 via `clamp_rev_str`, stop absent → `e_idx=-1`). Explicit negative indices continue through `clamp_rev_str` |
| R9 | Explicit stop with negative step still clamps | P0 | `'abcdef'[4:1:-1]` produces `'edc'` (start=4 and stop=1 both go through `clamp_rev_str`, loop 4→3→2) |
| R10 | Positive step slicing unchanged | P0 | `'abcdef'[1:4]` still produces `'bcd'`. The positive-step branch is not modified |
| R11 | Step=-1 full reverse matches CPython 3.12 | P0 | `s[::-1]` for any string `s` produces the same result as CPython 3.12. Unicode codepoint-based iteration |
| R12 | Xfail fixture converted to passing test | P1 | `string_edge_cases_xfail.py` is renamed/merged into `string_edge_cases.py` (or xfail directive removed) and passes conformance |

### Constraints

- Fix is localized to `mb_str_slice_full` in `string_ops.rs` — only the negative-step branch for absent start/stop
- `clamp_rev_str` function itself is not modified — it remains correct for explicit index values
- `mb_str_slice` (2-arg slice without step) is unaffected
## Acceptance Criteria

### Scenario: String split

- **WHEN** `'a,b,c'.split(',')`
- **THEN** Returns `['a','b','c']`

### Scenario: String join

- **WHEN** `','.join(['x','y','z'])`
- **THEN** Returns `'x,y,z'`

### Scenario: String strip

- **WHEN** `'  hello  '.strip()`
- **THEN** Returns `'hello'`

### Scenario: String predicates

- **WHEN** `'123'.isdigit()`
- **THEN** Returns True

### Scenario: String center

- **WHEN** `'hi'.center(6, '*')`
- **THEN** Returns `'**hi**'`

### Scenario: F-string formatting

- **WHEN** `f"Hello {name}"` with name = `"World"`
- **THEN** Returns `"Hello World"`


## Scenarios

### S1: Full reverse slice s[::-1] produces reversed string (R6, R7, R11)

**GIVEN** `s = 'abcdef'; print(s[::-1])`
**WHEN** executed through Mamba JIT
**THEN** output is `fedcba` — `s_idx=5`, `e_idx=-1`, loop iterates 5→4→3→2→1→0

### S2: Full reverse of single-char string (R6, R7, R11)

**GIVEN** `s = 'x'; print(s[::-1])`
**WHEN** executed
**THEN** output is `x` — `s_idx=0`, `e_idx=-1`, loop iterates once at index 0

### S3: Full reverse of empty string (R6, R7, R11)

**GIVEN** `s = ''; print(s[::-1])`
**WHEN** executed
**THEN** output is empty string — `len=0`, `s_idx=-1`, loop condition `while -1 > -1` is false

### S4: Partial reverse with explicit negative start (R8)

**GIVEN** `s = 'abcdef'; print(s[-2::-1])`
**WHEN** executed
**THEN** output is `edcba` — start=-2 normalizes to 4 via `clamp_rev_str`, stop absent → `e_idx=-1`

### S5: Partial reverse with explicit start and stop (R9)

**GIVEN** `s = 'abcdef'; print(s[4:1:-1])`
**WHEN** executed
**THEN** output is `edc` — both start=4 and stop=1 go through `clamp_rev_str`, loop 4→3→2

### S6: Positive step slicing unaffected (R10)

**GIVEN** `s = 'abcdef'; print(s[1:4])`
**WHEN** executed
**THEN** output is `bcd` — positive step path uses `normalize_index`, unchanged

### S7: Step=-2 skipping reverse (R6, R7)

**GIVEN** `s = 'abcdef'; print(s[::-2])`
**WHEN** executed
**THEN** output is `fdb` — start at index 5, step -2: indices 5, 3, 1

### S8: Unicode string reverse (R11)

**GIVEN** `s = '你好世界'; print(s[::-1])`
**WHEN** executed
**THEN** output is `界世好你` — char-level codepoint iteration handles multi-byte correctly

### S9: Xfail fixture passes conformance (R12)

**GIVEN** `string_edge_cases_xfail.py` fixture content `s = 'abcdef'; print(s[::-1])`
**WHEN** `cargo test -p mamba --test conformance_tests` executed
**THEN** fixture produces `fedcba` matching `.expected` file, xfail directive removed or fixture merged
## Changes

```yaml
files:
  - path: crates/mamba/src/runtime/string_ops.rs
    action: VERIFY
    desc: "Verify fix from bc5921e9 in `mb_str_slice_full` negative-step branch: (1) absent start → `len - 1` directly (no `clamp_rev_str`); (2) absent stop → `-1` directly (no `clamp_rev_str`); (3) explicit values still go through `clamp_rev_str`. Fix is already applied — this change validates and updates specs only."
    targets:
      - type: function
        name: mb_str_slice_full
        change: verify negative-step branch uses literal defaults for absent start (len-1) and absent stop (-1) instead of clamp_rev_str
    do_not_touch: [clamp_rev_str, mb_str_slice, mb_string_split, mb_string_join, mb_string_strip, mb_string_replace, mb_string_find, mb_string_format, mb_string_encode, mb_string_upper, mb_string_lower]

  - path: crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py
    action: MODIFY
    desc: "Merge reverse slicing tests from string_edge_cases_xfail.py into this passing fixture."
    targets:
      - type: function
        name: "(module-level script)"
        change: "append s[::-1] reverse slicing test case after existing forward-slicing tests"

  - path: crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.expected
    action: MODIFY
    desc: "Append expected output 'fedcba' for the merged reverse slicing test."

  - path: crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.py
    action: DELETE
    desc: "Remove xfail fixture — content merged into string_edge_cases.py."

  - path: crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases_xfail.expected
    action: DELETE
    desc: "Remove xfail expected file — content merged into string_edge_cases.expected."

  - path: cclab/specs/crates/mamba/runtime/string-ops.md
    action: MODIFY
    desc: "Add R6 requirement for negative-step slice with absent start/stop defaults and acceptance scenarios for reverse slicing."
    targets:
      - type: function
        name: "R6 (new requirement)"
        change: "add R6 requirement section documenting absent start/stop defaults with negative step"
    do_not_touch: [R1, R2, R3, R4, R5]
```
# Reviews
