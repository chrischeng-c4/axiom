---
change: sdd-phase-advance-timeout
group: phase-advance-and-timeout
date: 2026-03-26
---

# Requirements

1. Fix reference_context phase advance bug (#1124):
   - execute_artifact in create_reference_context.rs must call set_phase(ReferenceContextCreated) after writing artifact
   - execute_artifact in review_reference_context.rs must call set_phase(ReferenceContextReviewed) after writing review
   - Guard with matches!() to only advance from expected phases
2. Add agent execution timeout (#1126):
   - Wrap child.wait() with tokio::time::timeout() in script_runner.rs
   - Add per-action default timeouts in agent.rs (explore 5m, spec 10m, impl 20m, default 15m)
   - On timeout: kill child process, return error
   - Thread timeout through workflow_common.rs to agent.rs to script_runner.rs
