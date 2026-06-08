---
topic: sdd-arch-migration
date: '20260413'
project: cclab
branch: main
---

## Status

SDD architecture migration planned + audited, 0% executed. 30 tasks created (20 TD specs, 4 skills, 1 agent, CLAUDE.md, memory, verification). Issue rename pending. No code changes this round — TD + skills + CLAUDE.md only.

## Findings

- **89 files total** reference old SDD flow (STATE.yaml + groups): ~55 Rust, ~25 TD specs, ~6 skills/agents, ~3 CLAUDE.md/memory
- **Rust code deferred** to separate impl phase (StateManager 133 call sites + group_id 15 files)
- **TD specs are the primary target** this round: 22 spec files need updating to reflect new architecture
- **17 section types** from AUTHORING.md must be followed in all spec rewrites
- **New architecture decisions made this session:**
  - Control plane (main: `.score/issues/`, `.score/changes/`) vs data plane (worktree: code + TD)
  - `score artifact` = pure data write, no phase side effects
  - SubagentStop hook = validate + advance phase (4 hooks: SubagentStart, PreToolUse, PostToolUse, SubagentStop)
  - `score-reference-context` agent deleted → `score-issue-author` replaces it
  - All 4 agents set to `background: true`
  - 3 CRR loops: Issue, TD (change-spec), Codebase (impl) — CRR ≠ validation (validation = format check, CRR = content review by score-review agent)
  - init_change writes change_dir to main FIRST, then creates worktree — by design

## Done

- ✅ Takeoff from previous session (all criteria passed after score rebuild)
- ✅ Pushed 49 commits from previous session
- ✅ `bug-score-init-missing-5-skill-templates-handoff-takeo` — merged via SDD (`1ff33c21`)
  - 5 missing skills wired into `score init` (handoff, takeoff, build-debug, release-patch, mamba-test-coverage)
  - 3 script-based skills get scripts/ with chmod +x
- ✅ Section-loop reference context committed (`6876e966`)
- ✅ Mamba epics consolidated: 8 → 1 master epic (`23a90e0c`)
- ✅ Issue triage: 262 → 188 open issues, 74 closed (`0ef2a6e2`)
- ✅ SDD architecture epics consolidated: 5 → 1 unified epic (`75030192`)
- ✅ Cross-label fix: 15 dual-labeled issues → single owner (`b19b5f97`)
- ✅ `/score:epic` skill + CLI issue created
- ✅ `SubagentStop validation hook` issue created (`6c2239aa`)
- ✅ `issue-lifecycle-crr` updated with R13-R16 (hook-based validation design)
- ✅ `clarify(sdd): storage model` — merged via SDD (`4ed29157`)
  - Storage Model section added to issue-centric-workflow.md
  - Pseudocode clarified: change_dir = project_root, not worktree
- ✅ All 4 score agents set to `background: true` (`2d8a8d5b`)
- ✅ `score-reference-context` agent deleted
- ✅ Full audit: 89 files referencing old SDD flow (STATE.yaml + groups)
- ✅ 30 tasks created for migration execution
- ⬚ `init_change` bug reclassified as TD clarification (was wrong, current behavior correct)

## Next

```bash
# 1. Rename issue
score issues update enhancement-issue-lifecycle-draft-open-crr-change-phase-tracki \
  --title "SDD architecture migration: eliminate STATE.yaml + groups, background agents + SubagentStop hook"

# 2. TD specs batch 1: core workflow (tasks #1-#5)
#    Files: issue-centric-workflow.md, state-machine.md, change-spec-logic.md,
#           pre-clarifications.md, post-clarifications.md
#    All in .score/tech_design/crates/sdd/logic/

# 3. TD specs batch 2: tools + interfaces (tasks #6-#11)
#    Files: implement-task.md, reference-context.md, write-artifact.md,
#           delegate-agent.md, artifact-tools.md, workflow-tools.md

# 4. TD specs batch 3: skills + remaining (tasks #12-#22)
#    Files: run-change.md, revise-artifact.md, merge.md, dispatch-model.md,
#           README.md, docs-phase.md, tdd-gate.md, restructure-input.md,
#           fetch-issues.md, commands.md, change-spec.md

# 5. Skills + agents (tasks #23-#27)
#    .claude/skills/score-{run-change,merge,revise-artifact,issue-patrol}/SKILL.md
#    .claude/agents/score-change-spec.md

# 6. CLAUDE.md + memory (tasks #28-#29)

# 7. Verification (task #30)
grep -rn "STATE\.yaml\|StateManager\|group_id\|groups/default\|groups_progress\|in_place\|score-reference-context" \
  .score/tech_design/ .claude/skills/ .claude/agents/ CLAUDE.md \
  --include="*.md" | grep -v archive | grep -v closed

# 8. Commit + push
```

## Criteria

- [ ] `grep -rn "STATE\.yaml" .score/tech_design/ .claude/skills/ .claude/agents/ CLAUDE.md --include="*.md" | grep -v archive | grep -v closed | wc -l` returns 0
- [ ] `grep -rn "group_id\|groups/default\|groups_progress" .score/tech_design/ .claude/skills/ .claude/agents/ CLAUDE.md --include="*.md" | grep -v archive | grep -v closed | wc -l` returns 0
- [ ] `grep -rn "score-reference-context" .score/tech_design/ .claude/skills/ .claude/agents/ --include="*.md" | grep -v archive | grep -v closed | wc -l` returns 0
- [ ] `cargo test -p sdd --lib` — 1561+ tests, 0 failures (no code changes, should be unchanged)
- [ ] `cargo test -p score-cli --lib` — 76+ tests, 0 failures
- [ ] All 22 TD specs have `<!-- type: X lang: Y -->` annotations on every section (manual check)
