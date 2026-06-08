# Task: Implement Tests for Spec 'issues-cli-crud-spec' (Change 'issues-cli-crud')

## Instructions

Production code for spec 'issues-cli-crud-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **issues-cli-crud-spec**: `.score/changes/issues-cli-crud/groups/issues-cli-crud/specs/issues-cli-crud-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation issues-cli-crud` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/issues-cli-crud/groups/issues-cli-crud/specs/issues-cli-crud-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation issues-cli-crud
```