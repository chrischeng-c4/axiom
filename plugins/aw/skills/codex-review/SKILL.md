---
name: codex-review
description: Route one work item's external-contract change to `codex exec` for the semantic judgement no script can make — does this change satisfy the work item, and would its cases refuse a wrong implementation — and bind the resulting verdict to the exact bytes reviewed. Invoked by /aw:wi-ec-review when apps/<project>/aw.toml names `[review] ec = "skill:codex-review"`.
version: 0.1.0
user-invocable: true
---

# /aw:codex-review

An external-contract change can pass every mechanical check and still pin
something its work item never asked for, or pin the right thing in a way no
wrong implementation could fail. This skill exists for that gap and nothing
else: it hands one work item's change to a second model and records what came
back.

You are a pipe here. You do not review the change, you do not form an opinion
about it, and you do not decide the outcome.

## Run it

Three commands, in this order, none of them optional.

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" review-prompt <iid> --baseline <path> > <prompt-path>
codex exec - < <prompt-path> > <transcript-path> 2>&1
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" verdict <iid> --baseline <path> --transcript <transcript-path>
```

Write both files under a scratch directory, not into the checkout.

The subcommand is `exec`, not `review`, and that was measured rather than
chosen. `codex review` emits a **fixed report schema** — priority-tagged
`[P1]`/`[P2]` comments against a diff — and ignores output instructions in the
prompt entirely: run against this rubric it produced two genuinely correct
findings and zero `VERDICT:` lines, so `ec.py verdict` refused the transcript.
`codex exec` answers the prompt it is given, honours the output contract, and
on the same case returned four findings. Do not "fix" this back to `review`.

The interpreter pin is load-bearing — `ec.py` reads TOML and `tomllib` is
3.11+, while a bare `python3` is 3.9 on at least one machine this runs on. If
`${CLAUDE_PLUGIN_ROOT}` does not resolve, the plugin is not loaded; the script
is in the checkout at `plugins/aw/scripts/ec.py`. For readability this skill
writes the short form `ec.py <verb>` from here on.

## Why the prompt is built, not written

`ec.py review-prompt` emits the whole thing: the rubric, the work item's body,
every path the change touches, each case's source with its inventory `promise`
and `oracle`, the digest, and the exception each case currently dies on. Scope
is guaranteed by construction rather than by asking the reviewer to stay in
bounds.

This is why `codex review --uncommitted` is the wrong tool for this job even
though it exists. It would hand over whatever happens to be dirty in the tree
— and, worse, it would hand it over with no standard to judge it against and
without the work item the change is supposed to satisfy. A diff does not tell
a reviewer what an EC case is for, what this change promised, or what would
make it worthless. The rubric does, and the rubric is inside
`ec.py` so that its digest travels with the script rather than sitting in a
loose file anyone could weaken without touching the thing that reads it.

Pass the prompt through unaltered. Do not append context, do not summarise the
case for the reviewer, and do not tell it what you think the answer is.

## Why the transcript is parsed, not reported

`ec.py verdict` reads the raw transcript itself. It requires the `VERDICT:`
lines to agree, requires one of them to be the final non-empty line, and
refuses a `rejected` verdict that names no finding. Then it computes `sha256`
over the work item's body and every byte of the change — itself, from the
files — and writes the verdict bound to that digest. The work item is inside
the digest because the question was a comparison: editing what was asked for
can flip the answer just as editing a case can.

So the record cannot say the reviewer accepted bytes the reviewer never saw,
and it cannot say anything you would have preferred it to say. Keep the
transcript intact: `tee` it, do not retype it, and do not edit it before
passing it in. What `verdict` stores is a copy of the file it parsed, which is
what makes the record auditable later by someone who was not here.

Be clear about what this does and does not establish. It cannot show that the
reviewer read carefully. It guarantees two things only: the approval dies the
moment the reviewed bytes change, and the raw text it was derived from is
kept.

## Report

| verdict | what it means | next |
|---|---|---|
| `accepted` | the reviewer found nothing disqualifying | `/aw:wi-ec-commit <iid>` |
| `rejected` | at least one `FINDING:` line names a defect | quote every finding to the user verbatim, then `/aw:wi-ec-start` |

On `rejected`, the findings are the whole output. Relay them as they were
written. Do not rank them, do not decide which ones matter, and do not offer
your own view on whether the reviewer was right — a second opinion delivered
by the agent that wanted the case to pass is not a second opinion.

## Never

This addresses the agent running the review, not the reviewer.

- Never write the `VERDICT:` line yourself, never construct a transcript, and
  never run `ec.py verdict` against a file you authored rather than captured.
- Never re-run the reviewer because the first verdict was `rejected`. A second
  sample is not a correction; if a finding is factually wrong, say so to the
  user with the evidence and let them decide.
- Never edit the case in this skill. Any edit here invalidates the digest the
  review is bound to, which means the review you just ran covers bytes that no
  longer exist.
- Never drop, trim, or reflow the prompt to make it shorter, and never
  substitute `codex review --uncommitted` or `--base` for the built prompt.
- Never report the review as complete without `ec.py verdict` having exited.
  `codex` producing output and a verdict existing are different events.
