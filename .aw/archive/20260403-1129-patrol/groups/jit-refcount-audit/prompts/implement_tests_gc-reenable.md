# Task: Implement Tests for Spec 'gc-reenable' (Change '1129-patrol')

## Instructions

Production code for spec 'gc-reenable' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **gc-reenable**: `cclab/changes/1129-patrol/groups/jit-refcount-audit/specs/gc-reenable.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation 1129-patrol` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/1129-patrol/groups/jit-refcount-audit/specs/gc-reenable.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation 1129-patrol
```