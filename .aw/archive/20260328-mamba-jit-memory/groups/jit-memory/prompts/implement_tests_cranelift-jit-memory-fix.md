# Task: Implement Tests for Spec 'cranelift-jit-memory-fix' (Change 'mamba-jit-memory')

## Instructions

Production code for spec 'cranelift-jit-memory-fix' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **cranelift-jit-memory-fix**: `cclab/changes/mamba-jit-memory/groups/jit-memory/specs/cranelift-jit-memory-fix.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-jit-memory` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-jit-memory/groups/jit-memory/specs/cranelift-jit-memory-fix.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-jit-memory
```