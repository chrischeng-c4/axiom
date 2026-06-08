---
number: 471
title: "SDD: DAG context loop ignores complexity — always routes to explore_codebase"
state: open
labels: [bug, P1, crate:sdd]
---

# #471 — SDD: DAG context loop ignores complexity — always routes to explore_codebase

## Summary

When processing multiple issues via DAG, the context loop always returns `action: "explore_codebase"` for subsequent issues, ignoring the complexity-based routing that the spec requires.

## Spec (README.md:227)

```
context_index < main_issues.len() - 1 → explore_spec (or first context per complexity)
```

- **high/critical**: should route to `explore_spec` first
- **medium**: should route to `explore_knowledge` first
- **low**: `explore_codebase`

## Implementation (dag_loop.rs:104)

```rust
"action": "explore_codebase",  // hardcoded, ignores complexity
```

Always returns `explore_codebase` regardless of complexity level.

## Additional Issue

`dag_loop.rs:103` also hardcodes `current_phase: "clarified"` when the actual phase is `codebase_context_approved`.

## Impact

For high/critical complexity changes with multiple issues, subsequent issues skip spec context and knowledge context exploration entirely, leading to incomplete analysis.
