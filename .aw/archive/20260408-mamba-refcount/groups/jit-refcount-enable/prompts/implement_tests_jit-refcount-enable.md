# Task: Implement Tests for Spec 'jit-refcount-enable' (Change 'mamba-refcount')

## Instructions

Production code for spec 'jit-refcount-enable' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **jit-refcount-enable**: `.score/changes/mamba-refcount/groups/jit-refcount-enable/specs/jit-refcount-enable.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-refcount` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-refcount/groups/jit-refcount-enable/specs/jit-refcount-enable.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-refcount
```