---
name: wi-epic-grill
description: Interview the human through AskUserQuestion until every required Epic section is answered, then author the epic through `epic.py create` or `epic.py update`. Use when a new epic must be opened, or when an existing epic's body is thin, stale, or unvalidatable.
version: 0.1.0
user-invocable: true
---

# /aw:wi-epic-grill

Grill the human, then let the script write. This skill owns exactly one
thing: turning an underspecified intent into an epic body that
`epic.py validate` accepts. It never writes product source, never invents an
answer the human did not give, and never edits the tracker by hand.

The epic surface is this plugin's own bundled prototype. Invoke it as

```
python3 ".claude/aw/scripts/epic.py" <verb> [args]
```

If that path does not exist, the plugin is not loaded; the same script is in
the checkout at `.claude/aw/scripts/epic.py`. Do not improvise a third
location — there is one script, and running a copy of it is how the epic
schema silently forks.

Where the plugin root lands is not fixed: installed from a git marketplace it
sits under `~/.claude/plugins/`, outside every checkout; installed from this
directory it *is* the checkout. So the script never infers the repository from
its own location — it walks up from your **working directory** to the outermost
`aw.toml`. Run it from anywhere inside the checkout you mean to write against;
run it from outside one and it refuses by name rather than guessing. For
readability this skill writes the short form `epic.py <verb>` from here on.
Every read and every write in this skill goes through it.

## Identity branch

Resolve this before asking anything. It decides the write verb and nothing
else about the interview changes.

| Input | Path |
|---|---|
| an iid is given, the epic exists | `epic.py fetch <iid>`, edit, then `epic.py update <iid>` |
| no iid is given, the epic does not exist yet | author a new body, then `epic.py create` |

On the update path, run `epic.py fetch <iid>` **before writing anything**. It
overwrites the local staged copy unconditionally, so a fetch that happens after
you have authored will eat what you wrote. The overwrite is the point: a local
body carries no authority, and editing a copy left behind by an earlier session
is how a body written elsewhere gets silently reverted. What `fetch` puts on
disk is the epic's current state, and that is what you grill against.

If the target's type is not epic, stop: the script refuses it by name, and
the closed work-item enum converges by spawn-and-link, never by changing a
type in place.

## Interview scope

Run `epic.py skeleton`. Its output is the authoritative section set — do not
carry a second copy of it in your head or in this file, because a copy drifts
the moment the script's schema moves. Each section carries an HTML comment
stating what its answer must contain; grill one section per round until every
comment is discharged.

Two of those sections decide whether the epic is verifiable at all, and the
rest are only as good as they are:

- every `R<n>` in `## Requirements` must be observable — reject a requirement
  no command can ever disagree with;
- every `## Verification Inventory` row must name a **command that exists**
  and an oracle stating what its output looks like when the requirement
  holds.

`validate` refuses a requirement with no inventory row, naming the missing
`R<n>`, so a requirement and its gate have to be grilled together. One row may
discharge several requirements — `R1-R3` in the first column is accepted — but
only where one gate genuinely covers all of them. What the validator still
cannot judge is whether the command exists or the oracle is real, so those two
remain yours.

## How to grill

1. Read what already exists first — the user's prompt, and for an update the
   current body from `epic.py show <iid> --json`. Never ask for something
   already answered.
2. Ask with **AskUserQuestion**, in rounds of at most four questions. Give
   each question 2-4 concrete options drawn from the repository, not generic
   placeholders; the human can always answer "Other".
3. Ground every proposed gate in the repository's own contract: `CLAUDE.md`
   and the owning project's `CONTRIBUTING.md` decide which command is
   authoritative for a given claim. Never offer the human a gate the
   repository does not already run.
4. Stop asking once every section the skeleton emits is answered. Do not
   extend the interview to design, sequencing, or implementation detail —
   that is the child work items' job.

## Write

Assemble the complete body in English and stage it under whatever this prints:

```
epic.py bodydir          # -> <repo>/.aw/workitems/epics, created if missing
```

Ask the script rather than rebuilding the path: `.aw/` is gitignored, so a
staged body never shows up as untracked residue, but `--body-file` resolves
against the *current* directory, so a hand-written relative path silently
means something different from a subdirectory. The file is a transient input
to exactly one tracker write — once the body lands, the tracker owns it.

The filename follows from which path you are on, and you never have to compute
it: on update, `fetch` already put the body at `<iid>.md` and you edit that file
in place; on create the number does not exist yet, so stage a `<slug>.md` and
`create` renames it to `<id>.md` once the tracker hands the number back.

Check it before spending a write: `epic.py validate --body-file <path>` reads
a file instead of a live issue, so a malformed body is caught before it
reaches the tracker.

Then call exactly one of:

```
epic.py create --title "<title>" --project <project> --priority <p0|p1|p2|p3> --body-file <path>
epic.py update <iid> --body-file <path>
```

`epic.py create` fixes the type from the axis — there is no `--type` flag to
pass, and no owning-epic flag, because an epic has none. An epic takes 0 or 1
`--project`; a multi-project span belongs in the body. Both verbs accept
`--dry-run`, which prints the exact tracker command without running it.

Finally run `epic.py validate <iid>` against the live issue and read the
result. If it does not pass, the emitted errors name the missing or malformed
sections: run one more AskUserQuestion round on exactly those sections and
update again. Report the epic only once validate passes.

## Never

This addresses the agent running the interview, not the human answering it.

- Never fabricate an answer, a gate command, or an oracle the human did not
  supply or confirm — an unanswered section is an unfinished interview, not a
  place for a plausible default.
- Never write the tracker body or any `src/**` path by hand, and never reach
  past the script to the tracker's own CLI; `epic.py` is the only writer here.
- Never open child work items here. Scope carving is `/aw:wi-epic-reconcile`.
- Never report the epic as authored on a create or update exit code alone —
  `epic.py validate` passing is the signal. The two exit codes answer
  different questions: one says the write landed, the other says the body is
  admissible.
