---
name: aw:grill-me-to-change
description: Interview the human through AskUserQuestion until a change work item's Goal/How/Acceptance/Never sections are answerable by a command, then author it through `change.py create` or `change.py update`. Use when a change must be opened for a worker to implement, or when an existing change's body is thin, stale, or unvalidatable.
version: 0.1.0
user-invocable: true
---

# /aw-grill-me-to-change

Grill the human, then let the script write. This skill owns exactly one thing:
turning an underspecified intent into a change body that `change.py validate`
accepts **and** that a worker cannot satisfy by accident. It never writes
product source, never invents an answer the human did not give, and never edits
the tracker by hand.

Invoke the surface as

```
python3 ".claude/aw/scripts/change.py" <verb> [args]
```

If that path does not exist, the plugin is not loaded; the same script is in
the checkout at `.claude/aw/scripts/change.py`. The script finds the repository
from your **working directory**, walking up to the outermost `aw.toml` — never
from its own location, which may or may not sit inside a checkout. Run it from
inside the checkout you mean to write against. For readability this skill
writes the short form `change.py <verb>` from here on.

## The schema is older than this plugin

The epic schema is this plugin's own invention. This one is not. The GHAN rules
were written elsewhere, a now-retired CLI enforced them, and every live change
work item was judged by them; `change.py` began as a transliteration of that
implementation and is now the only copy of it.

Two consequences bind this interview:

- The rules judge a population that already exists. When `change.py validate`
  refuses a body, the answer is to fix the body — not to reshape it until the
  validator stops complaining, and not to decide the rule is wrong because it
  is inconvenient here.
- Never edit the rules to fit an answer. If a rule is genuinely wrong, that is
  a change to `change.py` with a gate behind it, and
  `.agents/rules/authoring/agent-instruction-ghan.md` is where its intent is
  written down.

## Identity branch

Resolve this before asking anything. It decides the write verb; nothing else
about the interview changes.

| Input | Path |
|---|---|
| an iid is given, the change exists | `change.py fetch <iid>`, edit, then `change.py update <iid>` |
| no iid is given, the change does not exist yet | author a new body, then `change.py create` |

On the update path, run `change.py fetch <iid>` **before writing anything**. It
overwrites the local staged copy unconditionally, so a fetch that happens after
you have authored will eat what you wrote. The overwrite is the point: a local
body carries no authority, and editing a copy left behind by an earlier session
is how a body written elsewhere gets silently reverted.

If the target's type is not change, stop: the script refuses it by name, and
the closed work-item enum converges by spawn-and-link, never by changing a type
in place.

## Interview scope

Run `change.py skeleton`. Its output is the authoritative section set — do not
carry a second copy of it in your head or in this file, because a copy drifts
the moment the schema moves. Each slot carries an HTML comment stating
what its answer must contain; grill one section per round until every comment is
discharged.

What the validator checks is *shape*: that a premise carries a `file:line`
coordinate, that the acceptance table has its columns, that a negative control
names a mutation and a restore digest. Shape is the cheap half. Everything
below is what the validator **cannot** see, which is exactly why a human is in
this loop.

### The four things only the interview can establish

1. **The coordinates are real.** A premise cites `file:line`. Open it and read
   that line before accepting it. A body whose coordinates are all well-formed
   and all point at the wrong lines passes validation cleanly.

2. **The `current` column was measured, not remembered.** Run each gate command
   now, against this checkout, and write down what it actually printed. Copying
   the current value from an issue description, a previous round, or the
   human's memory is the single most productive source of false work: the
   coordinates come out right, the baseline comes out stale, and the worker
   burns a round discovering that the gate was green before they started.
   If the baseline is not green, name every tolerated pre-existing failure
   verbatim rather than stating a count.

3. **The gate can fail.** `current` and `target` must differ, and the difference
   must be one the named command can actually observe. A gate whose command
   selects tests by name is the classic trap here — a filter that matches
   nothing exits 0 and reports success, so "0 passed" and "all passed" are the
   same observation. Prefer a command with no selector.

4. **The negative control is real.** This is the section workers most often
   satisfy on paper. It must name the mutation, require verbatim failure
   output, and require a byte-for-byte restore verified by sha256. Ask the
   human what specifically would be mutated and what red looks like. A control
   nobody can describe concretely is a control nobody will run.

## How to grill

1. Read what already exists first — the user's prompt, and for an update the
   current body from `change.py show <iid> --json`. Never ask for something
   already answered.
2. Ask with **AskUserQuestion**, in rounds of at most four questions. Give each
   question 2-4 concrete options drawn from the repository, not generic
   placeholders; the human can always answer "Other".
3. Ground every proposed gate in the repository's own contract: `CLAUDE.md` and
   the owning project's `CONTRIBUTING.md` decide which command is authoritative
   for a given claim. Never offer the human a gate the repository does not
   already run.
4. Run the candidate gate commands yourself during the interview. You are
   allowed to read the repository freely; the point of the interview is to
   arrive at values you observed, not values you were told.
5. Stop asking once every slot the skeleton emits is answered. Do not extend the
   interview into implementation — writing the code is the worker's job, and a
   change body that dictates the diff has stopped being a contract.

## Write

Assemble the complete body in English and stage it under whatever this prints:

```
change.py bodydir        # -> <repo>/.aw/workitems/changes, created if missing
```

Ask the script rather than rebuilding the path: `.aw/` is gitignored, so a
staged body never shows up as untracked residue, but `--body-file` resolves
against the *current* directory, so a hand-written relative path silently means
something different from a subdirectory.

The filename follows from which path you are on: on update, `fetch` already put
the body at `<iid>.md` and you edit that file in place; on create the number
does not exist yet, so stage a `<slug>.md` and `create` renames it to `<id>.md`
once the tracker hands the number back.

Check it before spending a write: `change.py validate --body-file <path>` reads
a file instead of a live issue, so a malformed body is caught before it reaches
the tracker.

Then call exactly one of:

```
change.py create --title "<title>" --epic <iid> --project <project> --priority <p0|p1|p2|p3> --body-file <path>
change.py update <iid> --body-file <path>
```

`change.py create` fixes the type from the axis — there is no `--type` flag.
`--epic` is optional and carries ownership: it attaches the label that makes
this change a child of that epic, which is the same link
`/aw-grill-epic-to-changes` reads when it audits an epic's child set. Omit it only
for a change that genuinely belongs to no epic. Both verbs accept `--dry-run`,
which prints the exact tracker command without running it.

Finally run `change.py validate <iid>` against the live issue and read the
result. If it does not pass, the emitted errors name the offending section: run
one more AskUserQuestion round on exactly those, and update again. Report the
change only once validate passes.

## Never

This addresses the agent running the interview, not the human answering it.

- Never fabricate a premise, a coordinate, a baseline value, or a negative
  control the human did not supply or confirm — an unanswered slot is an
  unfinished interview, not a place for a plausible default.
- Never write a `current` value you did not observe in this checkout during
  this interview.
- Never write the tracker body or any `src/**` path by hand, and never reach
  past the script to the tracker's own CLI; `change.py` is the only writer here.
- Never implement the change here, and never open its sibling work items —
  scope carving belongs to `/aw-grill-epic-to-changes`.
- Never report the change as authored on a create or update exit code alone —
  `change.py validate` passing is the signal. The two exit codes answer
  different questions: one says the write landed, the other says the body is
  admissible.
