# Task: Implement Tests for Spec 'mamba-all-support-spec' (Change 'mamba-all-support')

## Instructions

Production code for spec 'mamba-all-support-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-all-support-spec**: `.score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-all-support` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-all-support/groups/all-support/specs/mamba-all-support-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-all-support
```