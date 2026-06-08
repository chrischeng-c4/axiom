# Task: Implement Tests for Spec 'py312-behavioral-conformance' (Change 'mamba-conformance-p0')

## Instructions

Production code for spec 'py312-behavioral-conformance' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **py312-behavioral-conformance**: `cclab/changes/mamba-conformance-p0/groups/mamba-py312-conformance/specs/py312-behavioral-conformance.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation mamba-conformance-p0` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/mamba-conformance-p0/groups/mamba-py312-conformance/specs/py312-behavioral-conformance.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation mamba-conformance-p0
```