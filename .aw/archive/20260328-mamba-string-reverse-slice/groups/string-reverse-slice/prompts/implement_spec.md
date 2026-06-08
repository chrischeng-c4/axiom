# Task: Implement Spec 'string-reverse-slice-fix.base' for Change 'mamba-string-reverse-slice'

## Instructions

1. Read spec **string-reverse-slice-fix.base**: `cclab/changes/mamba-string-reverse-slice/groups/string-reverse-slice/specs/string-reverse-slice-fix.base.md`
2. Implement **production code only** (no `#[test]` functions) according to spec requirements
3. When done, run `cclab sdd workflow create-change-implementation mamba-string-reverse-slice` to advance

## Change Targets

### crates/mamba/tests/fixtures/conformance/data_structures/string_edge_cases.py
- **function `(module-level script)`**: append s[::-1] reverse slicing test case after existing forward-slicing tests

### cclab/specs/crates/mamba/runtime/string-ops.md
- **function `R6 (new requirement)`**: add R6 requirement section documenting absent start/stop defaults with negative step
- **DO NOT MODIFY**: R1, R2, R3, R4, R5

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-string-reverse-slice/groups/string-reverse-slice/specs/string-reverse-slice-fix.base.md

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-string-reverse-slice
```