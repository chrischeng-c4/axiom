---
number: 929
title: "bug(sdd): delegated agent exceeds prompt scope — calls artifact commands beyond assigned action"
state: open
labels: [bug, P0, crate:sdd]
group: "artifact-guard"
---

# #929 — bug(sdd): delegated agent exceeds prompt scope — calls artifact commands beyond assigned action

## Summary

Delegated agents (especially Gemini) call artifact CLI commands beyond the scope defined in their prompt. The prompt clearly specifies which artifact to produce, but the agent autonomously calls additional artifact commands after completing its assigned task.

## Reproduction

1. Run `cclab sdd workflow restructure-input lens-p2`
2. Gemini agent dispatched with prompt that only instructs: read issues → group → write `cclab sdd artifact restructure-input`
3. Agent completes restructure-input correctly
4. Agent then **autonomously** calls `cclab sdd artifact create-pre-clarifications` x4, self-answering all pre-clarification questions
5. STATE.yaml phase advances to `pre_clarifications_created` without mainthread involvement

## Root Cause

- Prompt only says what TO do, not what NOT to do
- Gemini CLI has no `--disallowedTools` flag (unlike Claude's `--disallowedTools`)
- After completing the assigned artifact, the agent sees pending files on disk and decides to "help"
- `cclab sdd artifact` commands have no delegation guard check — they accept calls from any context

## Impact

- **Bypasses user confirmation** — pre-clarification answers are design decisions that should come from the user
- **Breaks workflow contract** — only mainthread should advance phases via workflow commands
- **Silent** — mainthread sees the state already advanced and assumes it was done correctly

## Fix: Artifact-level delegation guard check

`delegation_guard` already exists in STATE.yaml with an `action` field. The fix is to have artifact CLI commands check the guard before executing.

### Current behavior

```
delegation_guard:
  action: "restructure_input"
  expected_phases: [...]
  started_at: "2026-03-18T..."

cclab sdd artifact restructure-input ...       → executes (correct)
cclab sdd artifact create-pre-clarifications → executes (BUG)
```

### Fixed behavior

```
cclab sdd artifact {action} {change_id} ...
  → read STATE.yaml
  → if delegation_guard exists:
      if action != guard.action → REJECT
      else → continue
  → execute artifact
```

### Files to change

- `crates/cclab-sdd-cli/src/commands.rs` or `crates/cclab-sdd/src/tools/mod.rs` — add guard check before artifact dispatch
- Action name mapping: CLI kebab-case (`create-pre-clarifications`) → guard snake_case (`create_pre_clarifications`)

### Why this works

- Pure code defense — does not rely on LLM instruction following
- Works for all agents (Gemini, Codex, Claude) regardless of CLI capabilities
- `delegation_guard` is already set/cleared by `agent.rs` during dispatch
- Minimal change: one check at the artifact entry point
