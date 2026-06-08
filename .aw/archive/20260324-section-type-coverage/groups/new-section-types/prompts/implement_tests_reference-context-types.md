# Task: Implement Tests for Spec 'reference-context-types' (Change 'section-type-coverage')

## Instructions

Production code for spec 'reference-context-types' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **reference-context-types**: `cclab/changes/section-type-coverage/groups/new-section-types/specs/reference-context-types.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation section-type-coverage` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/section-type-coverage/groups/new-section-types/specs/reference-context-types.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation section-type-coverage
```