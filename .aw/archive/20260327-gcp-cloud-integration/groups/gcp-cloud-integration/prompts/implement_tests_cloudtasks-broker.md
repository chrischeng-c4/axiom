# Task: Implement Tests for Spec 'cloudtasks-broker' (Change 'gcp-cloud-integration')

## Instructions

Production code for spec 'cloudtasks-broker' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **cloudtasks-broker**: `cclab/changes/gcp-cloud-integration/groups/gcp-cloud-integration/specs/cloudtasks-broker.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation gcp-cloud-integration` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/gcp-cloud-integration/groups/gcp-cloud-integration/specs/cloudtasks-broker.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation gcp-cloud-integration
```