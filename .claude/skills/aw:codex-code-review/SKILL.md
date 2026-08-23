---
name: codex-code-review
description: Route one work item's implementation and its colocated unit tests to `codex exec` for the semantic judgement no script can make — does this code satisfy the work item, or only its own tests — and bind the resulting verdict to the exact bytes reviewed. Reached from the LOGIC phase's own printed next command, and usable without a work item to read a project's whole test-and-source surface.
version: 0.1.0
user-invocable: true
---

# /aw:codex-code-review

By the time this runs, both oracles are green: the unit tests pass and the
end-to-end cases accept the product. That is exactly the state in which the
remaining defect is invisible to every gate in the ladder — an implementation
written to the shape of its test rather than to the requirement, and a test
that admits it. Neither oracle can see it, because each of them is half of the
pair. This skill exists for that gap and nothing else: it hands the tests and
the code to a second model *together* and records what came back.

You are a pipe here. You do not review the change, you do not form an opinion
about it, and you do not decide the outcome.

## Run it

Three commands, in this order, none of them optional.

```
uv run --python 3.13 --no-project ".claude/aw/scripts/logic.py" --project <name> review-prompt <iid> > <prompt-path>
codex exec - < <prompt-path> > <transcript-path> 2>&1
uv run --python 3.13 --no-project ".claude/aw/scripts/logic.py" --project <name> verdict <iid> --transcript <transcript-path>
```

Write both files under a scratch directory, not into the checkout.
`--project <name>` is required and must come before the verb: it sits on the
top-level parser, so after the verb it exits 2. There is no default project.

The subcommand is `exec`, not `review`, and that was measured rather than
chosen. `codex review` emits a **fixed report schema** — priority-tagged
`[P1]`/`[P2]` comments against a diff — and ignores output instructions in the
prompt entirely: run against a rubric of this shape it produced two genuinely
correct findings and zero `VERDICT:` lines, so the verdict verb refused the
transcript. `codex exec` answers the prompt it is given and honours the output
contract. Do not "fix" this back to `review`.

The interpreter pin is load-bearing — the phase scripts read TOML and `tomllib`
is 3.11+, while a bare `python3` is 3.9 on at least one machine this runs on.
The script is in the checkout at `.claude/aw/scripts/logic.py`, and that path is
relative to the repository root — run it from there. For readability this skill
writes the short form `logic.py <verb>` from here on.

## The whole-surface form

Omitting the iid reviews every colocated test file in the project, each beside
the source it sits on, instead of one change:

```
uv run --python 3.13 --no-project ".claude/aw/scripts/logic.py" review-prompt > <prompt-path>
codex exec - < <prompt-path> > <transcript-path> 2>&1
```

There is no third command, and the omission is deliberate. A verdict is a
record bound to a change; with no change there is nothing for it to bind to,
and a file shaped like the one a commit gate reads, holding an approval of
nothing, is worse than no file. The prompt also drops the two questions that
need a work item to be answerable, and says so. This form is advisory — relay
the findings and open work items for the ones worth acting on. The verdict verb
refuses a missing iid rather than writing an unbound record.

## Why the prompt is built, not written

`logic.py review-prompt` emits the whole thing: the rubric, the work item's
body, every path the change touches, the test files as the `unit` phase
committed them, and the source each one sits beside. Scope is guaranteed by
construction rather than by asking the reviewer to stay in bounds.

Two details are load-bearing. The tests come out of the `unit` commit rather
than off disk, because that commit is the same evidence the phase gate accepted
as proof the tests landed red — so the reviewer reads the artifact this phase
is measured against, not whatever a test file happens to say now. And the work
item is in the prompt because the question is a comparison: a prompt carrying
only the code can be answered "this is well-tested" by a reviewer who never
learned what was asked for, and that answer is indistinguishable from the one
worth having.

This is why `codex review --uncommitted` is the wrong tool for this job even
though it exists. It would hand over whatever happens to be dirty in the tree,
with no standard to judge it against. The rubric supplies the standard, and it
is inside the script so that its digest travels with the code that reads it
rather than sitting in a loose file anyone could weaken without touching
either.

Pass the prompt through unaltered. Do not append context, do not summarise the
change for the reviewer, and do not tell it what you think the answer is.

## Why the transcript is parsed, not reported

`logic.py verdict` reads the raw transcript itself. It requires the `VERDICT:`
lines to agree, requires one of them to be the final non-empty line, and
refuses a `rejected` verdict that names no finding. Then it computes `sha256`
over the work item's body and every byte of the change — itself, from the
files — and writes the verdict bound to that digest. The work item is inside
the digest because editing what was asked for can flip the answer just as
editing the implementation can.

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
| `accepted` | the reviewer found nothing disqualifying | `logic.py commit <iid>` |
| `rejected` | at least one `FINDING:` line names a defect | quote every finding to the user verbatim, then go back to the implementation |

On `rejected`, the findings are the whole output. Relay them as they were
written. Do not rank them, do not decide which ones matter, and do not offer
your own view on whether the reviewer was right — a second opinion delivered
by the agent that wrote the code is not a second opinion.

A finding that names a hole in a test is not answered by tightening the test
alone. The tests landed red in an earlier phase and were committed there, so a
test edit here is a change to the standard the implementation was measured
against; say that to the user rather than quietly widening it.

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
- Never edit the code or the tests in this skill. Any edit here invalidates the
  digest the review is bound to, which means the review you just ran covers
  bytes that no longer exist.
- Never drop, trim, or reflow the prompt to make it shorter, and never
  substitute `codex review --uncommitted` or `--base` for the built prompt.
- Never re-run the tests or the cases to answer a finding. Both were green
  before this started; green is this review's precondition, not its subject,
  and a passing run is not evidence against a finding about what the run fails
  to observe.
- Never run this over an end-to-end contract. Judging a case is a different
  review with a different rubric, and it is `/aw:codex-e2e-review`.
- Never report the review as complete without the verdict verb having exited.
  `codex` producing output and a verdict existing are different events.
