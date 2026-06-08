# Task: Implement Tests for Spec 'mamba-756-patrol-spec' (Change 'mamba-756-patrol')

## Instructions

Production code for spec 'mamba-756-patrol-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-756-patrol-spec**: `.score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-756-patrol` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-756-patrol/groups/default/specs/mamba-756-patrol-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-756-patrol
```