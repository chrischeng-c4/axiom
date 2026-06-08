# Task: Implement Tests for Spec 'fix-conformance-xfails-spec' (Change 'fix-conformance-xfails')

## Instructions

Production code for spec 'fix-conformance-xfails-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **fix-conformance-xfails-spec**: `cclab/changes/fix-conformance-xfails/groups/mamba-conformance-xfails/specs/fix-conformance-xfails-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation fix-conformance-xfails` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/fix-conformance-xfails/groups/mamba-conformance-xfails/specs/fix-conformance-xfails-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation fix-conformance-xfails
```