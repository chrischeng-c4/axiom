# Task: Implement Tests for Spec 'mamba-type-3arg-spec' (Change 'mamba-type-3arg')

## Instructions

Production code for spec 'mamba-type-3arg-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-type-3arg-spec**: `.score/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-type-3arg` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-type-3arg/groups/mamba-type-3arg-core/specs/mamba-type-3arg-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-type-3arg
```