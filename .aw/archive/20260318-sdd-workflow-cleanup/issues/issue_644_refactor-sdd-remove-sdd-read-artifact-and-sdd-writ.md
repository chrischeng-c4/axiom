---
number: 644
title: "refactor(sdd): remove sdd_read_artifact and sdd_write_artifact generic tools"
state: open
labels: [enhancement, crate:sdd]
group: "tools-cleanup"
---

# #644 — refactor(sdd): remove sdd_read_artifact and sdd_write_artifact generic tools

## Summary

Remove the generic `sdd_read_artifact` and `sdd_write_artifact` MCP tools. Each artifact now has its own dedicated tool (`sdd_artifact_create_change_spec`, `sdd_artifact_create_reference_context`, etc.), making the generic tools redundant.

**Keep `sdd_run_change`** — it's the workflow bridge used by CLI and skill template.

## What to remove

- `sdd_read_artifact` — use file Read tool directly
- `sdd_write_artifact` — each phase has dedicated `sdd_artifact_*` tools

## Files

- `crates/cclab-sdd/src/tools/artifact_read.rs` — delete
- `crates/cclab-sdd/src/tools/artifact_write.rs` — delete
- `crates/cclab-sdd/src/tools/mod.rs` — remove registrations
- Agent prompts — update any references from `sdd_read_artifact` to "Read file"
