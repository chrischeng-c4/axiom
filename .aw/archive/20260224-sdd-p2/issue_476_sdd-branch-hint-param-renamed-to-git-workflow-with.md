---
number: 476
title: "SDD: branch_hint param renamed to git_workflow with different enum values"
state: open
labels: [bug, P2, crate:sdd]
---

# #476 — SDD: branch_hint param renamed to git_workflow with different enum values

## Summary

The spec defines a `branch_hint` parameter; the implementation uses `git_workflow` with different enum values.

## Spec (tools/run-change/README.md:38-44)

```json
{"name": "branch_hint", "enum": ["main", "feature"]}
```

## Implementation (mod.rs:81-86)

```rust
"git_workflow": { "type": "string", "enum": ["new_branch", "in_place"] }
```

## Action

Update spec to match implementation (the `git_workflow` / `new_branch|in_place` naming is more precise).
