---
name: wi-ec-commit
description: The only thing that may commit a change work item's external-contract change. Re-runs the full mechanical list itself, requires a semantic verdict bound to the exact bytes being committed, and commits the diff it just measured. Use after /aw:wi-ec-review; a failing check means the change goes back, not that the commit gets a second try.
version: 0.2.0
user-invocable: true
---

# /aw:wi-ec-commit

This skill decides nothing. It runs one verb, reads its exit code, and does
what the exit code says. Everything it would otherwise have to be trusted to
do in the right order is inside the verb.

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" commit <iid> --baseline <path>
```

On exit 0 that verb prints an `EC-Commit:` line and a `next.command` naming
the second verb, which records the leg on the work item. Run it verbatim as
printed — the sha and the digest are its arguments, and retyping either is how
the tracker ends up pointing at a commit that is not the one that landed.

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/change.py" lifecycle <iid> --leg ec --commit <sha> --digest <digest>
```

The interpreter pin is load-bearing — `ec.py` reads TOML and `tomllib` is
3.11+, while a bare `python3` is 3.9 on at least one machine this runs on. If
`${CLAUDE_PLUGIN_ROOT}` does not resolve, the plugin is not loaded; the script
is in the checkout at `plugins/aw/scripts/ec.py`.

## Who decides whether it passes

Two different questions, answered by two different things, and the whole
design of this leg is that they are never merged.

**Is the change admissible?** Mechanical, closed, and entirely re-run inside
`ec.py commit` — the same ladder `/aw:wi-ec-verify` prints. You have no
discretion over any of it; every row is a command's exit code, not a
judgement.

**Does the change satisfy work item `<iid>`?** Not mechanical, and not yours
either. That question routed through `/aw:wi-ec-review` to whoever
`apps/<project>/aw.toml` names, and its answer arrives here as the `C7
verdict` row. `ec.py` reads that key itself; you do not parse the TOML and you
do not pick the reviewer.

The verdict is bound to `sha256` over the work item's body **and** every byte
of the change. Editing either after the review invalidates the approval, and
this verb says so rather than reusing it. An approval that survives edits to
the thing approved is not an approval; it is a sticky boolean.

## The loop

Run the verb. Read the rows. There are exactly three outcomes.

| outcome | what it means | what you do |
|---|---|---|
| exit 0 | committed | run the printed `lifecycle` command, then report the hash |
| a `FAIL` row naming `C7` | no valid verdict for these bytes | `/aw:wi-ec-review <iid>`, then re-run this verb |
| any other `FAIL` row | the change is not admissible | `/aw:wi-ec-start`, fix it, then `/aw:wi-ec-verify` |

A failed commit touches git not at all. That is deliberate and it is also your
check on yourself: if you were to report this leg as finished while a row is
red, `git status` would still show the whole change sitting there uncommitted,
and the next `/aw:wi-ec-start` would refuse to open a second leg over the top
of it. Do not put the user in the position of discovering that.

Re-running after a fix is a full re-run, not a resume. Nothing is cached, and
a check that passed on the previous attempt is measured again against the
current bytes.

## What lands

**The diff, exactly.** The allowlist is the dirty set — the same paths `C0
scope` confirmed are all inside `apps/<project>/external-contracts/`, and the
same paths the digest covered. Hand-writing that list would let a path be
reviewed and not committed, or committed without ever having been reviewed,
and neither would show up as a red row.

This is why `/aw:wi-ec-start` refused a dirty tree. From a clean start the
diff *is* the change; from a dirty one, committing the dirty set would sweep
up whatever else was lying around.

The commit message is `ec(wi-<iid>): pin the contract before the
implementation`, carrying a `Refs #<iid>` trailer, the reviewer, the change
digest, and one `EC-Pending:` line for every check that is named but not yet
wired — so the gaps are in permanent history rather than in nobody's memory.

**A back-link on the work item.** The commit carries `Refs #<iid>`, but that
link only runs one way: from a checkout you can find the work item, and from
the work item you can find nothing. `change.py lifecycle` closes it by writing
the sha and the digest into a fenced block at the end of the body.

Two properties of that block are what make writing to the body safe, and both
are the verb's, not yours. It is fenced by HTML comments and keyed by leg, so a
re-run or a later leg rewrites its row instead of stacking a second record of
the same event. And the verb validates the result *before* pushing it — the
same schema check the next leg's precondition runs — so a body it would refuse
can never reach the tracker. The block sits outside the four authored sections
because that H2 set is closed: a `## Lifecycle` section would make the TD leg
fail on a body this leg wrote.

Both verbs take `--dry-run`: `commit` prints the exact paths and message,
`lifecycle` prints the exact `gh` invocation, and neither writes anything.

## Never

This addresses the agent running the commit, not the human who asked for it.

- Never run `git add`, `git commit`, `git stash`, or any other git write
  yourself. `ec.py commit` is the only writer, and reaching past it discards
  every check it was about to run.
- Never edit a case, the inventory, or any authored section of the work-item
  body **in this skill**. Authoring is `/aw:wi-ec-start`, and an edit made
  here is an edit the reviewer never saw — which the digest binding will
  refuse anyway, one step later and with less of your time spent. The
  lifecycle block is the one exception and it is not yours either: it is
  written by `change.py lifecycle` from values the commit produced, never by
  you and never by hand.
- Never reach past `change.py` to the tracker's own CLI to record the leg, and
  never hand-edit the block on the issue. Both skip the schema check that is
  the entire reason a body write is permitted here at all.
- Never produce, transcribe, summarise, or paraphrase the semantic verdict.
  It comes from the reviewer named in `aw.toml`, is parsed out of the raw
  transcript by `ec.py verdict`, and is not something you may be persuaded to
  restate.
- Never re-run a check outside the verb to get a second, more favourable
  reading, and never present a check as passing because it passed on an
  earlier attempt.
- Never report this leg as done without a commit hash **and** the work item
  showing it. Exit 0 and a commit are the same event; anything else is the
  change still sitting in the working tree. And a commit whose leg was never
  recorded leaves the tracker saying nothing happened, which is the state this
  skill is supposed to end.
