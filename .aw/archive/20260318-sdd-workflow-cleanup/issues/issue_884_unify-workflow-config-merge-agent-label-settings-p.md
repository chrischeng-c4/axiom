---
number: 884
title: "Unify workflow config: merge agent + label settings per workflow step"
state: open
labels: [enhancement, crate:sdd]
group: "config-unification"
---

# #884 — Unify workflow config: merge agent + label settings per workflow step

## Summary

Restructure `config.toml` so each workflow step has its own section containing both agent chain and label config, instead of separating them into `[workflow.agents]` and `[sdd.issue_platform.labels]`.

## Current

```toml
[sdd.issue_platform.labels]
proposal = "cclab:sdd:proposal"
spec = "cclab:sdd:spec"

[sdd.issue_platform.labels.status]
draft = "status:draft"
review = "status:review"
...

[workflow.agents]
restructure_input = ["gemini:gemini-3-flash-preview", "claude:claude-sonnet-4-6"]
create_change_spec = ["gemini:gemini-3.1-pro-preview", "claude:claude-opus-4-6"]
...
```

## Proposed

```toml
[workflow]
label_prefix = "cclab:"

[workflow.restructure_input]
agents = ["gemini:gemini-3-flash-preview", "claude:claude-sonnet-4-6"]

[workflow.create_change_spec]
agents = ["gemini:gemini-3.1-pro-preview", "claude:claude-opus-4-6"]
label = "cclab:sdd:spec"

[workflow.create_change_implementation]
agents = ["claude:claude-sonnet-4-6"]
label = "status:implementing"
```

## Benefits

- Each step is self-contained (agents + labels + future per-step config)
- `label_prefix` avoids repetition
- Easier to extend per-step settings (e.g. timeout, max_retries)
- Removes the separate `[sdd.issue_platform.labels]` section

## Migration

- Need to update `SddConfig` model, `determine_agents()` in init, and all agent resolution code
- Backward-compat migration in `migrate_config()`
