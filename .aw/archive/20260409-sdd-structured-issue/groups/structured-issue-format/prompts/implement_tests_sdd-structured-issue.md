# Task: Implement Tests for Spec 'sdd-structured-issue' (Change 'sdd-structured-issue')

## Instructions

Production code for spec 'sdd-structured-issue' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **sdd-structured-issue**: `.score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation sdd-structured-issue` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-structured-issue/groups/structured-issue-format/specs/sdd-structured-issue.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation sdd-structured-issue
```