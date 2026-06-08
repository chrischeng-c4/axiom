---
number: 475
title: "SDD: Tasks phase uses sdd_write_artifact instead of spec's sdd_generate_tasks"
state: open
labels: [bug, P2, crate:sdd]
---

# #475 — SDD: Tasks phase uses sdd_write_artifact instead of spec's sdd_generate_tasks

## Summary

The spec defines `sdd_generate_tasks` as a dedicated tool with a deterministic algorithm for task generation. The implementation delegates everything to `sdd_write_artifact(artifact="tasks", action="generate")` instead.

## Spec (change-tasks.md:14-15, 65-120)

- Dedicated tool: `sdd_generate_tasks(change_id)`
- Algorithm: read specs → parse spec_type → extract file paths → build dependency graph → topological ordering → language detection
- Called directly (no agent delegation)

## Implementation (tasks.rs:43-51)

```
mcp__cclab-mcp__sdd_write_artifact(... artifact="tasks", action="generate", payload={})
```

No dedicated tool call. The deterministic algorithm is not embedded in the prompt.

## Additional Issues

- `tasks.rs:17` uses file-existence check (`has_tasks`) instead of phase-based routing
- `Finish` action sets `next_phase: "planned"` but doesn't self-advance (spec expects automatic transition)
- The `tasks.rs:53-66` relies on caller to advance phase via `advance_to`

## Decision

Is `sdd_write_artifact(action="generate")` internally equivalent to `sdd_generate_tasks`? If so, update spec. If not, implement the dedicated tool.
