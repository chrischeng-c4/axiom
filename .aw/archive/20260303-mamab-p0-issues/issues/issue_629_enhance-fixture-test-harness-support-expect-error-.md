---
number: 629
title: "Enhance fixture test harness: support EXPECT-ERROR for negative parse tests"
state: open
labels: [enhancement, P0, crate:mamba]
---

# #629 — Enhance fixture test harness: support EXPECT-ERROR for negative parse tests

## Context
The current fixture harness supports `# RUN: parse` (expect success) but the `# EXPECT-ERROR:` directive for parse mode is not implemented. We need this for negative test cases.

## Current harness code (fixture_tests.rs)
```rust
fn run_parse(src: &str, _directives: &Directives, path: &std::path::Path) {
    parser::parse(src, FileId(0))
        .unwrap_or_else(|e| panic!("{}: parse failed: {e}", path.display()));
}
```

## Required changes
When `# RUN: parse` + `# EXPECT-ERROR: <substring>`:
- Call `parser::parse()` and expect it to return `Err`
- Verify the error message contains the expected substring
- If parse succeeds when error expected, fail the test

## Acceptance
- `# RUN: parse` without `# EXPECT-ERROR:` = expect parse success (current behavior)
- `# RUN: parse` with `# EXPECT-ERROR: invalid syntax` = expect parse failure containing "invalid syntax"
- Harness reports clear messages for both pass and fail
