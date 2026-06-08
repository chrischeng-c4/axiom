# Task: Implement Tests for Spec 'mamba-refcount-jit-spec' (Change 'mamba-refcount-jit')

## Instructions

Production code for spec 'mamba-refcount-jit-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-refcount-jit-spec**: `cclab/changes/mamba-refcount-jit/groups/refcount-jit/specs/mamba-refcount-jit-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-refcount-jit` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-refcount-jit/groups/refcount-jit/specs/mamba-refcount-jit-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-refcount-jit
```