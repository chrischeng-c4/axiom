---
name: codex-e2e-review
description: Route one work item's end-to-end contract to `codex exec` for the semantic judgement no script can make — does this case pin what the work item asked for, and would it refuse a wrong implementation — and bind the resulting verdict to the exact bytes reviewed. Reached from the E2E phase's own printed next command, and usable without a work item to read a project's whole case surface.
version: 0.1.0
user-invocable: true
---

# /aw:codex-e2e-review

A case can be red for the right work item and still be worthless. It can pin
something the work item never asked for, pin the right thing so loosely that a
hard-coded return would satisfy it, or fail on an import error that looks
exactly like the product being absent. `e2e.py test <iid>` proves the case
fails; it cannot read what the failure is about. This skill exists for that gap
and nothing else: it hands one work item's contract to a second model and
records what came back.

You are a pipe here. You do not review the case, you do not form an opinion
about it, and you do not decide the outcome.

## Run it

Three commands, in this order, none of them optional.

```
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <name> review-prompt <iid> > <prompt-path>
codex exec - < <prompt-path> > <transcript-path> 2>&1
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" --project <name> verdict <iid> --transcript <transcript-path>
```

Write both files under a scratch directory, not into the checkout.
`--project <name>` is required and must come before the verb: it sits on the
top-level parser, so after the verb it exits 2. There is no default project.

The subcommand is `exec`, not `review`, and that was measured rather than
chosen. `codex review` emits a **fixed report schema** — priority-tagged
`[P1]`/`[P2]` comments against a diff — and ignores output instructions in the
prompt entirely: run against this rubric it produced two genuinely correct
findings and zero `VERDICT:` lines, so the verdict verb refused the transcript.
`codex exec` answers the prompt it is given, honours the output contract, and
on the same case returned four findings. Do not "fix" this back to `review`.

The interpreter pin is load-bearing — the phase scripts read TOML and `tomllib`
is 3.11+, while a bare `python3` is 3.9 on at least one machine this runs on.
The script is in the checkout at `.claude/aw/scripts/e2e.py`, and that path is
relative to the repository root — run it from there. For readability this skill
writes the short form `e2e.py <verb>` from here on.

## The whole-surface form

Omitting the iid reviews every case in the project instead of one change:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/e2e.py" review-prompt > <prompt-path>
codex exec - < <prompt-path> > <transcript-path> 2>&1
```

There is no third command, and the omission is deliberate. A verdict is a
record bound to a change; with no change there is nothing for it to bind to,
and a file shaped like the one a commit gate reads, holding an approval of
nothing, is worse than no file. This form is advisory — relay the findings and
open work items for the ones worth acting on. The verdict verb refuses a
missing iid rather than writing an unbound record.

## Why the prompt is built, not written

`e2e.py review-prompt` emits the whole thing: the rubric, the work item's body,
every path the change touches, each case's source with its inventory `promise`
and `oracle`, the digest, and the exception each case currently dies on. Scope
is guaranteed by construction rather than by asking the reviewer to stay in
bounds.

This is why `codex review --uncommitted` is the wrong tool for this job even
though it exists. It would hand over whatever happens to be dirty in the tree
— and, worse, it would hand it over with no standard to judge it against and
without the work item the case is supposed to pin. A diff does not tell a
reviewer what a case is for, what this change promised, or what would make it
worthless. The rubric does, and the rubric is inside the script so that its
digest travels with the code that reads it rather than sitting in a loose file
anyone could weaken without touching either.

Pass the prompt through unaltered. Do not append context, do not summarise the
case for the reviewer, and do not tell it what you think the answer is.

## Why the transcript is parsed, not reported

`e2e.py verdict` reads the raw transcript itself. It requires the `VERDICT:`
lines to agree, requires one of them to be the final non-empty line, and
refuses a `rejected` verdict that names no finding. Then it computes `sha256`
over the work item's body and every byte of the change — itself, from the
files — and writes the verdict bound to that digest. The work item is inside
the digest because the question was a comparison: editing what was asked for
can flip the answer just as editing a case can.

So the record cannot say the reviewer accepted bytes the reviewer never saw,
and it cannot say anything you would have preferred it to say. Keep the
transcript intact: `tee` it, do not retype it, and do not edit it before
passing it in. What the verdict verb stores is a copy of the file it parsed,
which is what makes the record auditable later by someone who was not here.

Be clear about what this does and does not establish. It cannot show that the
reviewer read carefully. It guarantees two things only: the approval dies the
moment the reviewed bytes change, and the raw text it was derived from is
kept.

## Report

| verdict | what it means | next |
|---|---|---|
| `accepted` | the reviewer found nothing disqualifying | `e2e.py commit <iid>` |
| `rejected` | at least one `FINDING:` line names a defect | quote every finding to the user verbatim, then go back to authoring the cases |

On `rejected`, the findings are the whole output. Relay them as they were
written. Do not rank them, do not decide which ones matter, and do not offer
your own view on whether the reviewer was right — a second opinion delivered
by the agent that wanted the case to pass is not a second opinion.

The commit gate reads the record, not this report. It refuses a commit with no
verdict, and refuses one whose digest no longer matches the tree, so nothing
you write here can substitute for having run the third command.

## Never

This addresses the agent running the review, not the reviewer.

- Never write the `VERDICT:` line yourself, never construct a transcript, and
  never run the verdict verb against a file you authored rather than captured.
- Never re-run the reviewer because the first verdict was `rejected`. A second
  sample is not a correction; if a finding is factually wrong, say so to the
  user with the evidence and let them decide.
- Never edit the case in this skill. Any edit here invalidates the digest the
  review is bound to, which means the review you just ran covers bytes that no
  longer exist.
- Never drop, trim, or reflow the prompt to make it shorter, and never
  substitute `codex review --uncommitted` or `--base` for the built prompt.
- Never run this over the implementation. Reading the tests and the code
  together is a different review with a different rubric, and it is
  `/aw:codex-code-review`. A reviewer shown both at this phase would judge the
  case by whether the code passes it, which is the one comparison the E2E
  phase exists to prevent.
- Never report the review as complete without the verdict verb having exited.
  `codex` producing output and a verdict existing are different events.
