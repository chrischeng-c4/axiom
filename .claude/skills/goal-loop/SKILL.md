---
name: goal-loop
description: >
  Agent-invocable substitute for Claude Code's native /goal. Runs a forked
  subagent that keeps working across its own turns until a tool-verified
  evaluator confirms a completion condition holds, then returns control.
  Unlike native /goal (chat-input only, evaluator judges from transcript
  alone), this skill can be invoked by Claude itself mid-task, and its
  evaluator actually runs commands to check the condition instead of
  trusting the conversation. Use for substantial, self-contained work with
  a verifiable end state: draining an issue backlog, migrating every call
  site to a new API, hitting a test-pass gate, splitting a file until every
  part is under a size budget.
argument-hint: [condition]
context: fork
hooks:
  Stop:
    - hooks:
        - type: agent
          prompt: |
            Read ${CLAUDE_PROJECT_DIR}/.claude/goal-loop/active.md.

            If it does not exist, respond {"ok": true} immediately — there
            is no condition to check.

            If it exists, its content is the completion condition for the
            current goal-loop. Verify — for real — whether it currently
            holds. Run whatever commands are actually needed (tests, build,
            lint, git status, grep for remaining call sites, gh issue list,
            etc.). Do not trust the conversation transcript alone; confirm
            against the real state of the repo/system.

            If the condition text includes a turn or time budget clause
            (e.g. "or stop after 20 turns"), weigh it against what the
            transcript shows: if the budget is clearly exceeded, treat the
            condition as holding even if the primary goal isn't fully met,
            and say so in the reason.

            - Holds (or budget exceeded): delete the file, then respond
              {"ok": true}.
            - Doesn't hold: leave the file in place, then respond
              {"ok": false, "reason": "<specific, actionable next step —
              what to do this turn, not just what's wrong>"}.
          timeout: 180
---

# /goal-loop

Substitute for Claude Code's native `/goal`: set a completion condition,
keep working across turns without being re-prompted, stop once a
tool-verified evaluator confirms it holds. Built as a skill instead of a
native command so Claude itself can invoke this mid-task — native `/goal`
only accepts human-typed chat input, an agent has no tool to trigger it.

This skill runs as a forked subagent (`context: fork`): it does **not**
see the conversation that invoked it, and its `Stop` hook auto-converts to
`SubagentStop`. Write the condition as a self-contained task, the same way
you'd brief a dispatched subagent — name the specific check and any
constraints. "Fix it" won't work; "run `cargo test -p cclab-jet`, fix every
failure, until it exits 0" will.

## Usage

```
/goal-loop <condition>
```

## What this turn does

1. Create `.claude/goal-loop/` if it doesn't exist.
2. Write `$ARGUMENTS` verbatim to `.claude/goal-loop/active.md`, overwriting
   any prior content — one condition active at a time, same as native
   `/goal`.
3. Start working toward the condition immediately.
4. Before producing your final response, verify the condition yourself —
   for real, by running the actual check the condition names (test suite,
   build, `git status`, issue query, grep for remaining call sites, file
   existence, whatever it says). Do not infer satisfaction from what you
   remember doing; run the check.
   - Doesn't hold, and any stated turn/time budget isn't exceeded: keep
     working. Do not produce a final response yet — take more tool-use
     turns and re-verify.
   - Holds, or the stated budget is exceeded: delete
     `.claude/goal-loop/active.md`, then produce your final response.
5. Report back what was done and the final verified state (including "gave
   up because the budget was exceeded, condition still doesn't hold" if
   that's what happened — don't report success you didn't verify).

Step 4 is the actual loop — it is enforced by your own instructions, not by
an external gate, so treat it as a hard requirement, not a suggestion. The
skill also declares a `SubagentStop` hook (below) that independently
verifies the same condition and feeds back a `reason` if you try to stop
early. Treat it as defense-in-depth: `type: "agent"` hooks are an
experimental Claude Code feature and empirically did not fire in testing
during this skill's development (see Known gaps), so step 4's
self-verification is what the loop actually relies on.

## Write conditions like native /goal

- One measurable end state: a test result, a build exit code, an empty
  issue queue, a file count.
- A stated check: "`cargo test -p cclab-jet` exits 0", not "tests pass."
- Constraints that must hold throughout: "no other test file is modified."
- A bound: "or stop after 20 turns" — step 4's self-verification honors
  this, so a condition that can never be verified true doesn't run
  forever. Claude Code's built-in 8-consecutive-block Stop-hook cap is a
  backstop on top of this regardless.

## Known gaps vs native `/goal`

- No `◎ /goal active` status indicator or turn/token counters.
- No access to the calling conversation — it's a fresh subagent, not the
  session that invoked it. Use this for a bounded, describable task, not
  "keep refining what we were just discussing."
- Every self-verification costs real tool calls and tokens (it actually
  runs commands, unlike native /goal's transcript-only read). Scope
  conditions to the narrowest check that proves the claim — `cargo test -p
  X`, not the whole workspace — to keep each verification cheap.
- The frontmatter `SubagentStop` hook is a documented feature
  (`type: "agent"`, experimental) but did not fire during this skill's own
  development testing — the subagent's own transcript showed no hook
  invocation at all around its Stop event, on Claude Code 2.1.209. The
  likely cause is that a project skill's `hooks:` capability needs a
  separate trust acceptance beyond ordinary skill invocation, which a
  skill folder created mid-session never gets prompted for. The loop is
  therefore enforced by step 4's self-verification instruction, not by
  this hook. Leave the hook declared (harmless, and may start working in a
  future version or after a fresh session), but don't depend on it.
