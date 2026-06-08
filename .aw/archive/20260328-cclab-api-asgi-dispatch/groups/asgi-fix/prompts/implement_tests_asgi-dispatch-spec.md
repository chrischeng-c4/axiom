# Task: Implement Tests for Spec 'asgi-dispatch-spec' (Change 'cclab-api-asgi-dispatch')

## Instructions

Production code for spec 'asgi-dispatch-spec' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **asgi-dispatch-spec**: `cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/specs/asgi-dispatch-spec.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `cclab sdd workflow create-change-implementation cclab-api-asgi-dispatch` to advance

## CLI Commands

```
# Read spec
Read file: cclab/changes/cclab-api-asgi-dispatch/groups/asgi-fix/specs/asgi-dispatch-spec.md

# Run tests
cargo test

# Advance implementation workflow
cclab sdd workflow create-change-implementation cclab-api-asgi-dispatch
```