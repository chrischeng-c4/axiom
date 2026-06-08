# Task: Implement Tests for Spec 'e2e-test-infrastructure' (Change 'e2e-test-reorg')

## Instructions

Production code for spec 'e2e-test-infrastructure' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **e2e-test-infrastructure**: `cclab/changes/e2e-test-reorg/groups/e2e-test-reorg/specs/e2e-test-infrastructure.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation e2e-test-reorg` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/e2e-test-reorg/groups/e2e-test-reorg/specs/e2e-test-infrastructure.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation e2e-test-reorg
```