# Task: Implement Tests for Spec 'mamba-py312-conformance-spec' (Change 'mamba-py312-conformance')

## Instructions

Production code for spec 'mamba-py312-conformance-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **mamba-py312-conformance-spec**: `cclab/changes/mamba-py312-conformance/groups/mamba-py312-conformance/specs/mamba-py312-conformance-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-py312-conformance` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-py312-conformance/groups/mamba-py312-conformance/specs/mamba-py312-conformance-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-py312-conformance
```