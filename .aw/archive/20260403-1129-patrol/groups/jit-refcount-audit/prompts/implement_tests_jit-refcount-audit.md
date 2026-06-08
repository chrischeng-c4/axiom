# Task: Implement Tests for Spec 'jit-refcount-audit' (Change '1129-patrol')

## Instructions

Production code for spec 'jit-refcount-audit' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **jit-refcount-audit**: `cclab/changes/1129-patrol/groups/jit-refcount-audit/specs/jit-refcount-audit.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation 1129-patrol` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/1129-patrol/groups/jit-refcount-audit/specs/jit-refcount-audit.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation 1129-patrol
```