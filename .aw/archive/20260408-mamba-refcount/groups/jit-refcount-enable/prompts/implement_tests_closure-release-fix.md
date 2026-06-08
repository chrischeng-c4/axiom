# Task: Implement Tests for Spec 'closure-release-fix' (Change 'mamba-refcount')

## Instructions

Production code for spec 'closure-release-fix' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **closure-release-fix**: `.score/changes/mamba-refcount/groups/jit-refcount-enable/specs/closure-release-fix.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-refcount` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-refcount/groups/jit-refcount-enable/specs/closure-release-fix.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-refcount
```