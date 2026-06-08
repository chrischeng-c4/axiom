---
change: score-handoff-takeoff
group: default
date: 2026-04-13
status: skipped
source: structured-issue
---

# Post-Clarifications

## Scope Summary

### Problem
-> See requirements.md

### Success Criteria
-> See requirements.md

### Boundary
- **In scope**: In-scope:
- `projects/score/cli/src/handoff.rs` — new module implementing create/list/show/takeoff subcommands
- `projects/score/cli/src/lib.rs` — add `pub mod handoff`
- `projects/score/cli/src/commands.rs` — add `Handoff` and `Takeoff` top-level subcommands
- `.claude/skills/score-handoff/SKILL.md` — new skill definition (model fills content, CLI handles structure)
- `.claude/skills/score-takeoff/SKILL.md` — new skill definition
- Renaming `.claude/skills/handoff/` to `.claude/skills/score-handoff/`
- Storage: `~/.score/handoffs/<YYYYMMDD>-<topic>.md` (global, not per-project)

Out-of-scope:
- Per-project handoff storage (global `~/.score/handoffs/` only)
- Sync to GitHub/GitLab
- Handoff review/revise CRR cycle
- `score init` installation of these skills (separate enhancement)

