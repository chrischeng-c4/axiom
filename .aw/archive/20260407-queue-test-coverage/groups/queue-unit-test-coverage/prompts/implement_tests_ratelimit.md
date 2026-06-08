# Task: Implement Tests for Spec 'ratelimit' (Change 'queue-test-coverage')

## Instructions

Production code for spec 'ratelimit' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **ratelimit**: `cclab/changes/queue-test-coverage/groups/queue-unit-test-coverage/specs/ratelimit.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation queue-test-coverage` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/queue-test-coverage/groups/queue-unit-test-coverage/specs/ratelimit.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation queue-test-coverage
```