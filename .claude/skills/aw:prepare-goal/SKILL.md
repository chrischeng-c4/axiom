---
name: prepare-goal
description: Turn a work item — or an intent interviewed out of the human — into a list of copy-pasteable `/goal` conditions, each written so the session's goal evaluator can actually decide it. Given an iid it reads the tracker through `epic.py`/`change.py`; given nothing it interviews through AskUserQuestion. It prepares text and sets no goal itself.
version: 0.1.0
user-invocable: true
---

# /aw:prepare-goal

`/goal` is a Claude Code built-in. It is not a skill, not a script in this
plugin, and nothing in this repository implements it: the human types one
condition, Claude Code registers that condition as a session-scoped `Stop`
hook, and the session is refused permission to stop until a separate evaluator
decides the condition holds.

This skill writes those conditions and nothing else. Its last section is a
block the human copies into their own prompt. It types none of them, and a goal
this skill claims to have set is a goal nobody set.

## The mechanism you are writing for

Measured by reading the shipped binary of Claude Code `2.1.234`. Each row is a
property the condition you write has to survive.

| Measured | What it costs a condition that ignores it |
|---|---|
| the condition string becomes the `Stop` hook's prompt **verbatim** | nothing expands it, rewrites it, or supplies what you left out |
| a separate evaluator decides it, and the model-facing goal tool states that evaluator's limit in its own words — it "verifies the condition from the conversation alone — it cannot run commands or read files" | a condition about the state of the tree is undecidable; a condition about what this conversation shows is decidable |
| the verdict is met, not met, or **impossible**, and a not-met verdict returns its reason to the agent as the reason it may not stop | a condition with no failure branch turns a dead end into a loop |
| evaluation is **deferred** while background agents or shells are still running | a condition met by work that has not reported yet is not judged until it does |
| one goal is active at a time, and setting a second **supersedes** the first | a list of conditions is a queue the human pastes one at a time, never a batch |
| the condition is capped at 4000 characters | length is not the constraint; decidability is |
| `/goal` with no argument prints the active condition, the number of turns it has been evaluated, and the last check's reason | the human can always see which one is running |
| `/goal clear` clears it — as do `stop`, `off`, `reset`, `none`, and `cancel` | there is an exit that is not "satisfy it" |
| it is refused in an untrusted workspace, and refused while `disableAllHooks` or `allowManagedHooksOnly` is set | on a machine where hooks are restricted, none of this output runs |

The second row is the one that decides everything below. The evaluator sees the
transcript, so **"the tests pass" is not a condition** — it is a claim about a
tree the evaluator cannot open. "`cargo test -p raft-runtime` exits 0 and its
output is in this conversation" is a condition, because the evidence for it and
the thing the evaluator can read are the same object.

## The shape of one condition

Four parts. Write them as one sentence or two, in this order, and drop none of
them — each is refused by a different failure.

| Part | Written as | What its absence causes |
|---|---|---|
| the end state | one observable difference, in the present tense, that is either true or false | "work on X" has no true, so the session cannot stop |
| the check | the verbatim command, plus the requirement that its output appear in this conversation | the evaluator cannot run anything, so an unwitnessed check is an unjudgeable one |
| the constraint | what must not change on the way there | the cheapest route to a green command is to edit the command's input |
| the stop clause | what to report instead, when the end state turns out to be unreachable | the evaluator has no ground to return *impossible*, and the session grinds |

Worked, on the same intent:

| | |
|---|---|
| refused | `make the raft tests pass` |
| accepted | `cargo test -p raft-runtime exits 0 with its full output pasted in this conversation, reached without editing any tests.rs under libs/raft-runtime/src; if the failure is in a crate this session does not own, say which and stop instead of widening.` |

The accepted form is longer for one reason: every clause in it is something the
evaluator can find in the transcript.

## Route A — you were given an iid

Do not guess the kind. Ask for its order:

```
uv run --python 3.13 --no-project "plugins/aw/scripts/epic.py" order <iid> --open-only
```

If that path does not exist the plugin is not loaded; the same script is in the
checkout at `plugins/aw/scripts/epic.py`. On a change work item this refuses
and names the type it actually found — that message is the answer, not an error
to route around.

**An epic.** The printed sequence is the queue: one condition per child, in the
order printed, each condition naming that child's iid. Emit them in that order
and say plainly that they are pasted one at a time, because the second one
cancels the first. When the output carries a `!` or `?` line instead of an
order, quote it and stop — an epic whose graph does not yield a sequence has no
queue to prepare, and picking one yourself makes the order yours.

**A change.** Read the body:

```
uv run --python 3.13 --no-project "plugins/aw/scripts/change.py" show <iid>
```

Its `## Acceptance` table already holds the verbatim command, the current
observation, and the target observation. That is the check and the end state,
authored and validated before you arrived — take them from there rather than
composing your own, and take the constraint from `## Never`. A change body is
the one input to this skill that has already been refused by a validator, so
inventing around it is discarding the only evidence in the room.

## Route B — you were given no iid

Interview. Use `AskUserQuestion`, one question per part of the shape above, and
offer only options you can point at in the checkout — a gate this project's
`README.md` or `CONTRIBUTING.md` already declares, a path that exists, a
command someone here already runs. An option you invented is an answer you gave
on the human's behalf.

| Ask | Until you have |
|---|---|
| what is different when this is done | one observable difference, not a description of activity |
| which command shows it | something runnable, whose output the human is willing to see pasted |
| what must not change to get there | the near miss — the file, the fixture, or the gate that would make the check green dishonestly |
| where it stops if it cannot be done | the report the human wants instead of an open-ended retry |

Stop asking when those four are answered. If the human declines to name a
check, say that the condition cannot be written and stop; a goal whose check
you supplied is a goal the human never set.

## The output

One section, at the end, and nothing after it. One fenced block per condition,
each a single line, so it survives a copy:

```
/goal <condition>
```

Above each block, one line naming which work item or which intent it belongs
to. Below the last one, these two, verbatim, because the human will need them
before the queue is done:

```
/goal
/goal clear
```

The first prints what is currently active and the last check's reason; the
second ends it early. Say once — not once per block — that the list is a queue
and that pasting a second condition cancels the first.

## Never

You are the agent running this skill.

- Never set a goal. This skill emits text for the human to paste. The
  model-facing route to a goal is a separate built-in tool gated behind a flag
  that is off by default, it is not reachable from here, and it is not what
  this skill is.
- Never write a condition whose check the evaluator cannot see. "The build is
  clean", "the bug is fixed", and "the migration is done" are all claims about a
  tree, and the evaluator has no tree.
- Never invent the proving command. On Route A it comes from the change body's
  `## Acceptance`; on Route B it comes from the human or from a gate the project
  already declares. A command you composed is a gate nobody agreed to.
- Never emit the queue as something to paste at once, and never present two
  conditions as simultaneously active. A second goal supersedes the first, so a
  batch silently discards every condition but the last.
- Never edit a work item, an epic body, or a project's gates to make a condition
  easier to write. This skill reads the tracker and writes to it never; a body
  you reshaped is a condition you authored on both sides.
- Never let the condition describe effort. "Keep trying until it works" has no
  state the evaluator can call met, and the hook it becomes is one that refuses
  every stop.
- Never report a goal as met, unmet, or impossible. That verdict is the
  evaluator's, it is printed in the human's session, and relaying one you did
  not see is a fabricated verdict about work nobody checked.
