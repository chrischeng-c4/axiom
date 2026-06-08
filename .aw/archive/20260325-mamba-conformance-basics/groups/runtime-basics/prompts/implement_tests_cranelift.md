# Task: Implement Tests for Spec 'cranelift' (Change 'mamba-conformance-basics')

## Instructions

Production code for spec 'cranelift' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **cranelift**: `cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/cranelift.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-conformance-basics` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-conformance-basics/groups/runtime-basics/specs/cranelift.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-conformance-basics
```