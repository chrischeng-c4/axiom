# Task: Implement Tests for Spec 'conformance-xfail-reduction-spec' (Change 'mamba-conformance-xfail')

## Instructions

Production code for spec 'conformance-xfail-reduction-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **conformance-xfail-reduction-spec**: `cclab/changes/mamba-conformance-xfail/groups/conformance-xfail-reduction/specs/conformance-xfail-reduction-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-conformance-xfail` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-conformance-xfail/groups/conformance-xfail-reduction/specs/conformance-xfail-reduction-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-conformance-xfail
```