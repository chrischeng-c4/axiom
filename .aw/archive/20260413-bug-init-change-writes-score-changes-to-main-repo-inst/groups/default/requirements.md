---
change: bug-init-change-writes-score-changes-to-main-repo-inst
group: default
date: 2026-04-13
source: structured-issue
---

# Requirements

## Problem

TD spec pseudocode in `issue-centric-workflow.md` (line 441-447) is ambiguous about where `.score/changes/<id>/` lives. The pseudocode implies `change_dir` might be relative to worktree, but the correct design is:

- **Main repo (control plane)**: `.score/issues/`, `.score/changes/<id>/` (STATE/specs/prompts/payloads)
- **Worktree (data plane)**: code changes, `.score/tech_design/` changes

The current implementation is actually correct — `init_change` writes change artifacts to main. The TD spec needs clarification, not the code.

## Requirements

- **R1**: Update `issue-centric-workflow.md` Changes pseudocode to explicitly state `change_dir` is relative to `project_root` (main repo), not `worktree_path`
- **R2**: Add a "Storage Model" section to the spec documenting control plane (main) vs data plane (worktree) split
- **R3**: Close the original investigation finding as "by design" — the execution order (change_dir before worktree) is correct because change_dir belongs on main
