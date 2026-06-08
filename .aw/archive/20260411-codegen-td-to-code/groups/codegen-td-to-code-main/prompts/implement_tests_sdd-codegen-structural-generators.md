# Task: Implement Tests for Spec 'sdd-codegen-structural-generators' (Change 'codegen-td-to-code')

## Instructions

Production code for spec 'sdd-codegen-structural-generators' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **sdd-codegen-structural-generators**: `.score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation codegen-td-to-code` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/codegen-td-to-code/groups/codegen-td-to-code-main/specs/sdd-codegen-structural-generators.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation codegen-td-to-code
```