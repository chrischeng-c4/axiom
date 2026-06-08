# Task: Implement Tests for Spec 'mamba-conformance-p0-spec' (Change 'mamba-conformance-p0')

## Instructions

Production code for spec 'mamba-conformance-p0-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-conformance-p0-spec**: `cclab/changes/mamba-conformance-p0/groups/mamba-p0-fixes/specs/mamba-conformance-p0-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-conformance-p0` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-conformance-p0/groups/mamba-p0-fixes/specs/mamba-conformance-p0-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-conformance-p0
```