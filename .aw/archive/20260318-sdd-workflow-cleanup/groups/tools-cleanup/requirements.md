---
change: sdd-workflow-cleanup
group: tools-cleanup
date: 2026-03-17
---

# Requirements

Remove redundant generic tools `sdd_read_artifact` and `sdd_write_artifact`. Each artifact now has dedicated tools. Update registrations in `crates/cclab-sdd/src/tools/mod.rs` and update agent prompts in `orchestrator/prompts.rs` (and other tool files) to use standard file read tools instead. Keep `sdd_run_change` as the workflow bridge. Update all logic specs in `cclab/specs/cclab-sdd/logic/` to use direct file reads.
