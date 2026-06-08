# Task: Implement Tests for Spec 'sdd-spec-format-unification' (Change 'sdd-spec-format-unify')

## Instructions

Production code for spec 'sdd-spec-format-unification' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **sdd-spec-format-unification**: `.score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation sdd-spec-format-unify` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/sdd-spec-format-unify/groups/sdd-spec-format-unify/specs/sdd-spec-format-unification.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation sdd-spec-format-unify
```