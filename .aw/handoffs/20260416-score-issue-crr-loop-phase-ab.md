---
topic: score-issue-crr-loop-phase-ab
date: '20260416'
project: main
branch: main
---

## Status

Phase A (lifecycle commit schema) + Phase B (reviewer step) committed as `f9da8fb5` on `main`. CLI + hooks verified end-to-end via real subagent + scratch repo. Autonomous SubagentStop hook firing **NOT yet verified** under a fresh Claude Code session — work is not blocked, just unverified.

Phases C (reviser subagent), D (merge CLI), E (TD spec + docs) are deferred and not started.

## Findings

- `claude -c` does NOT reload `.claude/settings.json` — same session ID continues with original hook bindings. Only fresh `claude` (no `-c`) re-registers SubagentStop matchers. Symptom during this session: 0 SubagentStop events in `~/.claude/projects/.../<session>.jsonl`, only stale PreToolUse errors from a long-removed path.
- Per-agent frontmatter `hooks:` block supports **only** PreToolUse / PostToolUse / SessionStart / UserPromptSubmit — `SubagentStart` and `SubagentStop` are mainthread-level lifecycle events and **must** live in `.claude/settings.json` (confirmed via claude-code-guide). Architecture is at best-case split:
  - per-agent frontmatter: PreToolUse Bash readonly + PreToolUse Write whitelist
  - global settings.json: PreToolUse artifact catchall + SubagentStart brief + SubagentStop apply
- Envelope schema accommodates mainthread-stage dispatches by making `agent` field optional (`#[serde(skip_serializing_if = "Option::is_none")]`). Approved review verdict emits dispatch-to-merge with no `agent` → mainthread executes (Phase D).
- Reviewer scope intentionally limited to evaluating already-listed specs — does NOT hunt for missing-but-should-be-linked specs (user explicit decision; documented in `score-issue-reviewer.md`).
- Multiple review iterations accumulate as bullets under a single `# Reviews` H1 (not multiple `## Review N` H2 sections). H1 is created on first apply; subsequent applies append bullets only.
- Hook scripts call bare `score` from PATH — works under shell-spawned envs. If Claude Code spawns hooks with a stripped PATH this would silently fail; not yet observed.

## Done

- `projects/score/cli/src/issues.rs` — `IssueEnvelope` (Dispatch with optional `agent`, Done, Error), `LifecycleStage` enum (5 variants: Create, Fill, Review, Merge, Reject), `LifecycleMetrics` struct (7 fields), `commit_lifecycle()` helper, `run_create` retrofit, `FillSectionArgs` + `ReviewArgs` with metric flags, `run_review_brief`, `run_review_apply`, `parse_review_bullet`, `append_review_bullet`. **Tested**: 86/86 unit tests + manual e2e via scratch repo.
- `projects/score/cli/src/init.rs` — install reviewer agent + 3 reviewer hooks + 3 author hooks (whole `issue-author/` dir was untracked); test assertions for reviewer matchers in settings.json template + merged install. **Tested**: cargo test passes.
- `projects/score/cli/templates/mainthread/agents/score-issue-author.md` — STRICT TURN BUDGET (max 7 turns), envelope-aware Inputs section, Write-only output to `.score/payloads/<slug>/body.md`. **Tested**: real subagent ran successfully under model:sonnet, ~13 tool_uses, ~88s.
- `projects/score/cli/templates/mainthread/agents/score-issue-reviewer.md` (NEW) — model:sonnet, maxTurns:12, STRICT TURN BUDGET, explicit out-of-scope rule (don't hunt missing specs), output format with verdict calibration. **Tested**: real subagent ran successfully, ~12 tool_uses, ~75s.
- `projects/score/cli/templates/mainthread/hooks/agents/issue-author/{pretooluse-write-guard,subagentstart-brief,subagentstop-apply}.sh` (3 NEW). **Tested**: subagentstop-apply.sh manually invoked against real transcript → emits dispatch-to-reviewer envelope correctly.
- `projects/score/cli/templates/mainthread/hooks/agents/issue-reviewer/{pretooluse-write-guard,subagentstart-brief,subagentstop-apply}.sh` (3 NEW). **Tested**: subagentstop-apply.sh manually invoked against real transcript → emits dispatch-to-merge envelope correctly.
- `projects/score/cli/templates/mainthread/hooks/global/subagentstart-setup.sh` + `subagentstop-validate.sh` — early-skip for `score-issue-author|score-issue-reviewer`. **Tested**: cargo test passes.
- `projects/score/cli/templates/mainthread/settings.json` — reviewer SubagentStart + SubagentStop matchers added before `score-*` catchall. **Tested**: cargo test passes (3 settings tests including upgrade-existing).
- `projects/score/cli/templates/mainthread/CLAUDE.md` — envelope protocol section with dispatch/done/error semantics + mainthread loop pattern. **Untested** (doc only).
- `projects/score/cli/templates/mainthread/skills/score-issue/SKILL.md` — slim down: delegates envelope handling to CLAUDE.md instead of hardcoding loop. **Untested** (doc only).
- `.claude/{agents,hooks,settings.json,skills}` — installed copies of all the above (will be re-materialized on next `score init --force`). **Tested**: same as templates.

## Next

1. Verify autonomous SubagentStop chain in a **fresh Claude Code session**:
   ```bash
   # Exit current session, then:
   cd /Users/chris.cheng/cclab/main && claude
   ```
   In the new session, run:
   ```
   /score:issue create "smoke verify autonomous chain" --type enhancement --label "crate:sdd"
   ```
   Expected: mainthread receives dispatch envelope, dispatches `score-issue-author`, SubagentStop hook auto-fires `fill-section --apply`, mainthread receives next dispatch envelope via `additionalContext`, dispatches `score-issue-reviewer`, SubagentStop hook auto-fires `review --apply`, mainthread receives dispatch-to-merge envelope.
   - If hooks fire: chain works end-to-end, ready for Phase C/D.
   - If hooks don't fire: check `~/.claude/projects/.../<session-id>.jsonl` for `hookEvent` entries; investigate why SubagentStop matchers aren't registering (possible Claude Code version-specific bug).

2. Phase C — reviser subagent (when reviewer emits `verdict: needs-revision`, currently returns error envelope as placeholder):
   - Add `score issues revise` verb (brief + apply) — same round-trip pattern as fill-section, payload at `.score/payloads/<slug>/body.md` again.
   - New `score-issue-reviser` agent definition + 3 hooks (start brief, write-guard, stop apply).
   - In `run_review_apply` needs-revision branch, replace error envelope with `Dispatch{agent: Some("score-issue-reviser"), invoke: "score issues revise"}`.
   - Loop terminates when reviewer emits `approved` (currently dispatches to merge).

3. Phase D — `score issues merge` verb:
   - Mainthread-executed (no agent in dispatch).
   - Validates issue is in `# Reviews → approved` final state, frontmatter `state: open`.
   - Merges issue worktree branch back into main (or moves issue file from worktree to main repo).
   - Final envelope `action: done`.

4. Phase E — TD spec + docs:
   - New TD spec `.score/tech_design/projects/score/specs/issue-cli-envelope.md` with R1–R8 envelope contract.
   - CLAUDE.md cross-link from envelope section to the TD spec.

## Criteria

- [ ] `cargo build -p score-cli --bin score` passes
- [ ] `cargo test -p score-cli` passes
- [ ] `git log -1 --format='%s'` returns `feat(score): issue CRR loop — review step + lifecycle commit schema`
- [ ] `git diff HEAD~1 -- projects/score/cli/templates/mainthread/agents/score-issue-reviewer.md | head -1` shows new file (reviewer agent committed)
- [ ] `ls .claude/hooks/agents/issue-reviewer/ | wc -l | tr -d ' '` returns `3`
- [ ] autonomous SubagentStop chain verified in fresh session (manual)
