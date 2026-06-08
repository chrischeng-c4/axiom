# Task: Implement Tests for Spec 'generator-thread-pool-design' (Change 'gen-thread-pool')

## Instructions

Production code for spec 'generator-thread-pool-design' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **generator-thread-pool-design**: `cclab/changes/gen-thread-pool/groups/gen-thread-pool/specs/generator-thread-pool-design.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation gen-thread-pool` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/gen-thread-pool/groups/gen-thread-pool/specs/generator-thread-pool-design.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation gen-thread-pool
```