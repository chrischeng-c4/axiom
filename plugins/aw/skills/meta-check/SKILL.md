---
name: meta-check
description: Refuse a META-doc whose facts have rotted. Runs the validator over every tracked CLAUDE.md, README.md and CONTRIBUTING.md and reports orphaned generator markers, commands naming a CLI that was deleted, links whose targets are gone, and project READMEs missing a required section. Reads only; it never edits a document and never regenerates one.
version: 0.1.0
user-invocable: true
---

# /aw:meta-check

Three files carry everything this repository says about itself in prose:
`CLAUDE.md` is what an agent loads at launch, `README.md` is what a thing
promises, and `CONTRIBUTING.md` is how to change it. Nothing read them until
this. The verdict is the script's exit code, not your reading of its output.

## What it refuses

| Rule | The defect | Why it is one |
|---|---|---|
| `M1` | a generator marker whose producer does not exist | the block claims something regenerates it, so a reader edits around it or trusts it stale |
| `M2` | a command naming a CLI that was deleted | copied and run, it fails with "command not found" and reads as a broken checkout |
| `M3` | a relative link whose target is not in the tree | the document points at a file the reader then cannot find |
| `M4` | a project README with no `## Brief` or `## Capabilities` | the two sections a reader goes there for |

`M1` is the largest by far, and it is the reason this exists. The verb that
wrote those blocks was deleted with the crate that carried it, leaving the
markers behind — a marker with no producer is worse than plain prose, because
plain prose does not claim to be maintained.

## Run it

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/meta.py" check
```

`meta.py check` is the whole surface — one verb, and the singleton is the
point: a second verb here would be a verb that writes. The population is
`git ls-files`, so it measures the checkout you are standing in and never a
scratch file in an ignored build directory. Three exit codes, and the third is
the one to read carefully:

| Exit | Meaning |
|---|---|
| `0` | every scanned document is clean; the summary still names how many it scanned |
| `1` | findings, listed by file with a line number and the rule that fired |
| `2` | the invocation was wrong — an unknown rule, a `--path` matching nothing, or not a git checkout |

Exit `2` is never a finding count. A `--path` that matches nothing exits `2`
rather than reporting clean, because a mistyped path that certifies the
repository is worse than no run at all.

## Narrowing it

Both flags repeat, and both are for reading a large report rather than for
shrinking it into a green one:

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/meta.py" check --rule M3
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/meta.py" check --path apps/keep
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/meta.py" check --format json
```

`--format json` carries the population alongside the findings — how many
documents were scanned out of how many are tracked, and how many of them are
project READMEs. Report those numbers with any count you relay. A finding count
with no population behind it cannot be told apart from a run that scanned
nothing.

## What it cannot see

It decides nothing that needs judgement, which is why no model is in this loop.
Every rule resolves against the filesystem: a marker, a command, a path, a
heading.

So it cannot tell you whether a promise under `## Capabilities` is *true*, and
it does not try to run the gate command written beneath one. Most of those
commands are `cargo` invocations, and resolving a crate and target name needs
`cargo metadata` — a different and much larger job. A capability whose gate
command nobody runs is caught here by a reader, not by this script.

## Fixing what it reports

The script writes nothing, so every finding is repaired by hand in the document
it names.

- `M1` — delete the marker pair and keep the content. The content is now
  authored prose owned by whoever edits that file.
- `M2` — the phase scripts that replaced the deleted CLI sit beside
  `plugins/aw/scripts/meta.py`. Repoint the line at one of them, or, if the
  sentence exists to record that the command is gone, add it to
  `DEAD_COMMAND_EXEMPT` quoting the line verbatim. An exemption that stops
  matching its line is itself reported, so a stale one cannot go quiet.
- `M3` — repoint the link, or drop it. Prose *about* a path that no longer
  exists is not a link and is not reported.
- `M4` — add the missing section. If nothing is claimed yet, say so under the
  heading rather than leaving the heading out.

Re-run the verb after each file. The count falling is the evidence; a file you
believe you fixed is not.

## Never

You are the agent running this skill.

- Never write, generate, or splice content into a META-doc as part of running
  it. This verb replaced one that wrote, and it was deleted for writing: a
  checker that also repairs is a checker whose green means it agreed with
  itself.
- Never add a `DEAD_COMMAND_EXEMPT` entry to silence a live command. The
  exemption is for prose recording a deletion, and adding one for a command a
  reader would still copy hides exactly the defect the rule exists to catch.
- Never declare a producer in `PRODUCERS` that does not exist. That table is
  what makes every marker orphaned by derivation; an entry naming nothing turns
  the whole rule off for that marker name.
- Never report a rule as clean because you narrowed the run to skip it. `--rule`
  and `--path` restrict what was measured, and what was not measured is not
  green.
- Never relay a finding count without the population the run printed beside it.
