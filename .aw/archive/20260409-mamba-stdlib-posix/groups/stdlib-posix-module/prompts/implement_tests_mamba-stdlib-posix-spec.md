# Task: Implement Tests for Spec 'mamba-stdlib-posix-spec' (Change 'mamba-stdlib-posix')

## Instructions

Production code for spec 'mamba-stdlib-posix-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-stdlib-posix-spec**: `.score/changes/mamba-stdlib-posix/groups/stdlib-posix-module/specs/mamba-stdlib-posix-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation mamba-stdlib-posix` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/mamba-stdlib-posix/groups/stdlib-posix-module/specs/mamba-stdlib-posix-spec.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation mamba-stdlib-posix
```