# Task: Implement Tests for Spec 'jet-console-error-relay' (Change 'jet-browser-console-errors')

## Instructions

Production code for spec 'jet-console-error-relay' has been implemented and verified to compile.
Now implement **test functions only** (`#[test]` functions / `#[cfg(test)]` blocks).

1. Read spec **jet-console-error-relay**: `.score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md`
2. Read the `## Test Plan` section to understand required test cases
3. Implement `#[test]` functions (in `#[cfg(test)]` blocks) that cover the test plan
4. Run `cargo test` to verify tests pass
5. When done, run `score workflow create-change-implementation jet-browser-console-errors` to advance

## CLI Commands

```
# Read spec
Read file: .score/changes/jet-browser-console-errors/groups/browser-console-error-relay/specs/jet-console-error-relay.md

# Run tests
cargo test

# Advance implementation workflow
score workflow create-change-implementation jet-browser-console-errors
```