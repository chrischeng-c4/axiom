# Task: Implement Tests for Spec 'jet-aot-build' (Change 'jet-test-gaps')

## Instructions

Production code for spec 'jet-aot-build' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **jet-aot-build**: `cclab/changes/jet-test-gaps/groups/jet-test-gaps/specs/jet-aot-build.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation jet-test-gaps` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/jet-test-gaps/groups/jet-test-gaps/specs/jet-aot-build.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation jet-test-gaps
```