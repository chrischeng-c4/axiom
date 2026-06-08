---
change_id: sdd-p2
type: gap_codebase_knowledge
created_at: 2026-02-23T16:34:04.741843+00:00
updated_at: 2026-02-23T16:34:04.741843+00:00
---

# Gap Analysis: Codebase vs Knowledge for Change 'sdd-p2'

## Identified Gaps

| Gap # | Type | Severity | Action Needed | Repair Action |
|---|---|---|---|---|
| 1 | Convention Violation | HIGH | Yes | Update StatePhase enum to include `ClarificationsCreated` (or rename `Clarified`) and ensure it matches spec `result_phase`. |
| 2 | Pattern Mismatch | MEDIUM | Yes | Align revision thresholds in `clarify.rs` with `revise-context-clarifications.md` (1 for reviewed, 2 for rejected). |
| 3 | Convention Violation | MEDIUM | Yes | Standardize scope prefix across `run_change/mod.rs` and `sdd_read_artifact` (use `context_clarifications`). |
| 4 | Undocumented Pattern | LOW | Yes | Add `implement_task_with_codegen` to the Action enum in `implement-change.md`. |
| 5 | Convention Violation | MEDIUM | Yes | Search and replace legacy `branch_hint` with `git_workflow` in all workflow specs. |
| 6 | Convention Violation | MEDIUM | Yes | Update `tasks.rs` to use `sdd_generate_tasks` if that is the intended tool in the spec, or update spec to match `sdd_write_artifact`. |
| 7 | Undocumented Pattern | LOW | No | Implement Requirement+ test generation in Prism (long-term roadmap). |
| 8 | Undocumented Pattern | MEDIUM | Yes | Update merge workflow prompts to explicitly document `codebase_paths` and `knowledge_refs` enrichment. |
| 9 | Optimization Gap | LOW | No | Implement stage-specific tool filtering in `run_change` prompts as per `40-mcp/dynamic-config.md`. |

## Summary

This analysis focuses on the discrepancies between the SDD codebase (Rust/MCP) and the documentation in `cclab/knowledge` and `cclab/specs`. The most critical issues involve state machine phase naming and revision thresholds, which directly affect the reliability of the automated workflow.
