# Task: Implement Tests for Spec 'mamba-compile-builtin-value-rc' (Change 'mamba-compile-builtin')

## Instructions

Production code for spec 'mamba-compile-builtin-value-rc' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-compile-builtin-value-rc**: `.score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-value-rc.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-compile-builtin` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-compile-builtin/groups/default/specs/mamba-compile-builtin-value-rc.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-compile-builtin
```