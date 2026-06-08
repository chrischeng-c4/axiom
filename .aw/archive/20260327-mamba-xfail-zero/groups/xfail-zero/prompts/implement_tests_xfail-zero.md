# Task: Implement Tests for Spec 'xfail-zero' (Change 'mamba-xfail-zero')

## Instructions

Production code for spec 'xfail-zero' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **xfail-zero**: `cclab/changes/mamba-xfail-zero/groups/xfail-zero/specs/xfail-zero.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-xfail-zero` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-xfail-zero/groups/xfail-zero/specs/xfail-zero.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-xfail-zero
```