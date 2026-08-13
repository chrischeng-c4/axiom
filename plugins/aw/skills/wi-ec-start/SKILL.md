---
name: wi-ec-start
description: Open the external-contract leg of a change work item — refuse a dirty tree, print the work item's Goal and Acceptance as the authoring brief, then author the EC cases that fail against the current tree. Use after a change work item is validated and before any implementation exists. Hands off to /aw:wi-ec-verify.
version: 0.2.0
user-invocable: true
---

# /aw:wi-ec-start

Write the verifier before the thing it verifies. This skill owns the first of
four legs — `start`, `verify`, `review`, `commit` — and produces exactly one
thing: the external-contract cases that pin what work item `<iid>` promises,
proven **red for the reason they claim**. It never writes `tech-design/`,
never writes `src/`, and never commits.

Invoke the surface as

```
uv run --python 3.13 --no-project "${CLAUDE_PLUGIN_ROOT}/scripts/ec.py" <verb> [args]
```

The interpreter is pinned on purpose. `ec.py` reads TOML, `tomllib` arrived in
3.11, and a bare `python3` is 3.9 on at least one machine this runs on — where
the failure is a `ModuleNotFoundError` traceback that reads like a broken
script rather than a wrong interpreter. Do not drop the pin. If
`${CLAUDE_PLUGIN_ROOT}` does not resolve, the plugin is not loaded; the same
script is in the checkout at `plugins/aw/scripts/ec.py`. It finds the
repository from your **working directory**, walking up to the outermost
`aw.toml`. For readability this skill writes the short form `ec.py <verb>`
from here on.

## Fetch the work item first

```
change.py fetch <iid>
```

`ec.py` never reads the tracker. It reads the body `change.py` stages at
`.aw/workitems/changes/<iid>.md`, and `fetch` overwrites that file
unconditionally — which is the point. A local body carries no authority, and
grilling against a copy an earlier session left behind is how a change written
elsewhere gets silently reverted.

## Open the leg

```
ec.py start <iid>
```

This refuses a dirty working tree, and the refusal is load-bearing rather than
tidiness. From a clean start, everything `git status` reports afterwards was
written by this leg — so the EC change is **derived from git** rather than
remembered in a case list, a side table, or a constant inside each case. A
side table can point at a case that was deleted and nothing would notice;
`git status` cannot. Every later verb reads the change that way, so if this
one is skipped there is no change for them to read.

If it refuses, commit or stash the unrelated work. There is no
`--allow-dirty`, and adding one would delete the definition the other three
verbs depend on.

On success it prints the work item's `## Goal` and `## Acceptance`. Those are
the brief. `Goal` names a trigger, an observation point, a current value, and
a target value — those four are the cases' subject, and a case pinning
something else is a case for a different change.

## Why this layer is first

`CLAUDE.md` fixes the order: `external-contracts` → `tech-design` → `src`.
Reaching implementation before the contract exists means nothing can refuse
the implementation afterwards — whatever gets written becomes the definition
of correct, and the contract degenerates into a description of it.

The same file states the part that surprises people: the cases must **fail
against the current tree, and you must run them and watch them fail**. A
contract that was green before the change was written proves nothing about the
change. Red is not a problem to fix here. Red is the deliverable.

## What red has to mean

A case that fails proves nothing on its own. Every one of these is red:

| shape | why it is worthless |
|---|---|
| the import path is wrong | fails before reaching any assertion |
| a fixture is missing | fails in setup; the contract never ran |
| the case points at a path that moved | fails forever, on nothing |
| `assert False, "<the sentence>"` | fails on demand, observing nothing |

So each case must declare, in a module-level `ASSERTIONS` tuple, the sentences
it can fail on — and the failure you observe has to be one of them, raised as
an `AssertionError`. `ec.py verify` refuses every other shape by name. This is
not a style rule: it is the only thing that separates "the behaviour does not
exist yet" from "I typed the path wrong", and those two are indistinguishable
from an exit code.

## Author

1. **Write each case** at
   `apps/<project>/external-contracts/src/cases/<case-id>.py`, carrying
   module-level `CASE_ID`, `DIMENSION`, `TARGET_COMMAND`, and `ASSERTIONS`.
   `CASE_ID` must equal the filename stem.

2. **Register each** as a `[[tool.aw.python-ec.cases]]` entry in that project's
   `external-contracts/pyproject.toml`, with `id`, `dimension`, `command`,
   `promise`, and `oracle`. Write `promise` and `oracle` as what the case
   actually asserts — a reviewer reads those two beside the source, and prose
   that oversells the code is caught there.

3. **Assert on externally observable behaviour**: exit codes, stdout, files on
   disk, tracker state. Reaching into internals produces a case that breaks on
   refactors and passes on wrong behaviour, which is backwards.

4. **Derive the expectation independently.** If the expected value is computed
   by calling the same code path under test, the case is an identity transform
   and passes for every implementation, including no implementation.

5. **Cover the work item, not a corner of it.** One work item may need several
   cases; the review asks whether the change satisfies `<iid>`, so a single
   case pinning one of three promised observables is an incomplete change, not
   a small one.

Nothing outside `apps/<project>/external-contracts/` may change. `ec.py
verify` refuses a `src/**` or `tech-design/**` path by name — which is only
possible because `start` demanded a clean tree.

## Inspect one case while authoring

```
ec.py red   --case <case-id>          # what exception, what message
ec.py check --case <case-id> --baseline <path>
```

These two are debugging primitives, narrowed to a single case. They cannot see
the work item, so they cannot tell whether the case belongs to this change —
that is `verify`, and passing `check` is not passing the gate.

## Hand off

Do not commit.

```
/aw:wi-ec-verify <iid>
```

## Never

This addresses the agent authoring the cases, not the human who asked for it.

- Never write `tech-design/**` or `src/**` in this skill. The change's
  implementation does not exist yet, and creating it here destroys the only
  moment at which the contract could have been written blind to it.
- Never run `git commit`, `git add`, `git stash`, or any other git write.
  Committing is `/aw:wi-ec-commit`, and the separation is the point.
- Never work around a dirty-tree refusal by editing anyway. The refusal is not
  a formality that has been satisfied by intent; it is the only thing that
  makes the diff mean this change.
- Never make a case green, and never soften an assertion to reduce the
  failure. Red at a declared assertion is the finished state of this leg.
- Never write an assertion that cannot fail: `assert True`, a comparison
  between two literals, a loop over a collection that is empty at runtime, a
  search whose pattern matches nothing, or a `try`/`except` that swallows the
  failure it was supposed to surface.
- Never satisfy `ASSERTIONS` by raising the declared sentence directly. The
  message has to come from an assertion that looked at the product.
- Never report this leg as done on `ec.py red` alone — a non-zero exit is the
  weakest of the checks, and it is the one every broken case also passes.
