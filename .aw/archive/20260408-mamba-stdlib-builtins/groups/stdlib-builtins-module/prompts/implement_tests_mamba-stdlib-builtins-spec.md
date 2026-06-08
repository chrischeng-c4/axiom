# Task: Implement Tests for Spec 'mamba-stdlib-builtins-spec' (Change 'mamba-stdlib-builtins')

## Instructions

Production code for spec 'mamba-stdlib-builtins-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-stdlib-builtins-spec**: `.score/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-stdlib-builtins` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-stdlib-builtins/groups/stdlib-builtins-module/specs/mamba-stdlib-builtins-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-stdlib-builtins
```