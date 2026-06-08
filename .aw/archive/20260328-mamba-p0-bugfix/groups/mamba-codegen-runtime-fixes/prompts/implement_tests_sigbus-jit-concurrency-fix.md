# Task: Implement Tests for Spec 'sigbus-jit-concurrency-fix' (Change 'mamba-p0-bugfix')

## Instructions

Production code for spec 'sigbus-jit-concurrency-fix' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **sigbus-jit-concurrency-fix**: `cclab/changes/mamba-p0-bugfix/groups/mamba-codegen-runtime-fixes/specs/sigbus-jit-concurrency-fix.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-p0-bugfix` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-p0-bugfix/groups/mamba-codegen-runtime-fixes/specs/sigbus-jit-concurrency-fix.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-p0-bugfix
```