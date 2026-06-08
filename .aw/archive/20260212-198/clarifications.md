---
change: 198
date: 2026-02-12
---

# Clarifications

## Q1: Change Type
- **Question**: Is this spec-only or also code?
- **Answer**: Spec-only. Update the action enum in run-change/README.md OpenRPC definition.
- **Rationale**: Issue only mentions spec file synchronization, no code changes needed.

## Q2: Git Workflow
- **Question**: Which git workflow?
- **Answer**: in_place — same branch as other genesis consistency fixes.
- **Rationale**: All 12 issues use in_place workflow.

## Q3: Scope
- **Question**: Which specific changes to the action enum?
- **Answer**: Add: review_task, revise_task, task_terminal_failure, all_tasks_done, merge_complete. Remove: complete. Also verify the Prompt Sources table references are consistent.
- **Rationale**: Matches issue description exactly.

