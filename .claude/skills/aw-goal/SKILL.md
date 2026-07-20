---
name: aw:goal
description: aw's single loop-verb dispatcher — drive a work item, a capability/project root, a project's whole open backlog, or an ad-hoc verifiable condition to a machine-checked terminal state. Route every "do X until done" request through the closed four-leaf decision tree below.
user-invocable: true
---

# /aw:goal

`aw goal` is the one loop verb in aw. Every invocation names a root and a
verifier. There are exactly **four** root kinds — a closed enum, not a
style choice.

## Closed leaf enum

| kind | CLI form | verifier |
|---|---|---|
| wi | `aw goal wi <id>` | lifecycle chain of that root (EC / terminal / rollup) |
| capability | `aw goal capability [<capability-id>] --project <p>` | capability work-root closure / project promise rollup |
| backlog | `aw goal backlog --project <p>` | zero open unparked WIs for the project |
| adhoc | `aw goal set --gate "<cmd>" <intent>` → `aw goal check <id>` | every recorded gate command exits 0 |

## Decision tree

Evaluate in order. Stop at the first match; execute that leaf's CLI form.

| # | Match condition | Leaf |
|---|---|---|
| 1 | Names a specific issue/WI id (numeric id, `#NNNN`, or a known local slug) | **wi** |
| 2 | Names a capability id, or asks for a product promise / production readiness / "is `<capability>` done" for a project | **capability** |
| 3 | Asks to finish, clear, or drain ALL open issues/backlog of a project (any phrasing, any language — e.g. "跑完 X 的所有 issue", "clear the backlog") | **backlog** |
| 4 | States a condition with a derivable, machine-runnable check (a single bounded test/build/lint/grep command proves it), and does not match 1-3 | **adhoc** |
| 5 | None of 1-4 match unambiguously | **ask ONE clarifying question** offering the four kinds (wi / capability / backlog / adhoc); never guess |

- Rule 2's `[<capability-id>]` is optional: omit it to run the whole
  project end to end.
- Rule 4: derive the *narrowest* proving command, then
  `aw goal set --gate "<narrowest proving command>" <prose intent>`
  (repeat `--gate` for multiple conditions; `--budget-checks`/
  `--budget-minutes` bound the loop, plus a hard 24h expiry). Do the work,
  then re-run the emitted `next.command` (`aw goal check <id>`) and read:
  `status: "done"` → report completion; `status: "blocked"` → read
  `gates[].output_tail`, fix, retry; `status: "gave_up"` → report the
  recorded intent and current blocker, do not claim success.

## Rules

- The leaf set is closed: `wi`, `capability`, `backlog`, `adhoc` — never a
  fifth. New phrasing patterns (new languages, new synonyms for "finish
  everything") extend the decision-tree *match rules* above; they never
  extend the *leaf set*. A genuinely new behavior is a CLI product change
  (a new `aw goal` subcommand), not a skill edit.
- Never declare a goal complete without its verifier's terminal signal:
  `completion.workflow_complete = true` for wi/capability/backlog
  envelopes, `status: "done"` (`completion.workflow_complete = true`) for
  adhoc.
- For wi/capability/backlog: follow the envelope's `invoke.command` /
  `next.command` / `agent_prompt` exactly, same as any other `aw` loop
  output. If a HITL envelope carries `hitl_question`, invoke the host's
  native user-question tool immediately (Claude Code: `AskUserQuestion`);
  never fabricate approval or treat the envelope as terminal.
- For adhoc: prose alone is never a gate — every recorded condition must be
  a single bounded, machine-runnable command.

## CLI drift & defect reporting

- Before treating this skill's documented command shapes as ground truth,
  verify the live CLI surface for the verb in use, e.g. `aw goal --help`
  (or the specific leaf, e.g. `aw goal wi --help`).
- If actual CLI behavior diverges from what this skill documents, capture a
  minimal reproduction: the exact command run and its actual vs. expected
  output.
- File the reproduction as a confirmed AW-owned defect via `aw issue create`
  instead of silently working around the drift in-session.
