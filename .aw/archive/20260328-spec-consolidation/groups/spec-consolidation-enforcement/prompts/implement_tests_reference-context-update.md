# Task: Implement Tests for Spec 'reference-context-update' (Change 'spec-consolidation')

## Instructions

Production code for spec 'reference-context-update' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **reference-context-update**: `cclab/changes/spec-consolidation/groups/spec-consolidation-enforcement/specs/reference-context-update.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation spec-consolidation` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/spec-consolidation/groups/spec-consolidation-enforcement/specs/reference-context-update.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation spec-consolidation
```