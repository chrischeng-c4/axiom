# Task: Implement Tests for Spec 'mamba-config-unify-spec' (Change 'mamba-config-unify')

## Instructions

Production code for spec 'mamba-config-unify-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-config-unify-spec**: `.score/changes/mamba-config-unify/groups/unify-mamba-config/specs/mamba-config-unify-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-config-unify` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-config-unify/groups/unify-mamba-config/specs/mamba-config-unify-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-config-unify
```