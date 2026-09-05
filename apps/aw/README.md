# aw

The AW workflow engine and its Typer CLI. Managed with `uv`; requires
Python 3.13. The canonical invocation, from the repository root, is
`uv run --project apps/aw aw <group> ...` — that exact prefix is what the
engine prints in its `next.command:` lines and what the seven `aw-*` skills
run.

## Capabilities

### CLI entry point

Outcome: `uv run --project apps/aw aw` with no arguments prints usage and
exits non-zero; `uv run --project apps/aw aw version` prints the project
version and exits zero.

Gate: `uv run --project apps/aw --directory apps/aw pytest e2e/` (run from
the repository root).

### Workflow command groups

Outcome: nine groups delegate to the engine modules under
`src/aw/scripts/`, one module per group, with argparse staying the single
source of argument validation — `change`, `milestone`, `e2e`, `impl`,
`maint`, `wis`, `meta`, `metadoc`, and `release-plan`. Every group, verb, and
option that the Typer surface accepts rebuilds an argv that the engine
module's own parser accepts.

Gate: `uv run --project apps/aw --directory apps/aw pytest e2e/` (run from
the repository root). `e2e/test_cli.py` measures the delegation with
`_delegate` stubbed; the other half of the printed protocol — that every
`next.command:` line the engine prints parses in the engine's argparse — is
`.claude/aw/verification/check_next_command.py`'s claim.

## Layout

- `src/aw/main.py` — the typer surface; `aw.main:app` is the `aw` console
  script. Each subcommand rebuilds the argv its engine module already
  parses and hands it to that module's `main(argv)`.
- `src/aw/scripts/` — the engine: the argparse scripts the `aw-*` skills
  drive (`change.py`, `milestone.py`, `e2e.py`, `impl.py`, `maint.py`,
  `wis.py`, `meta.py`, `metadoc.py`, `release_plan.py`, and their shared
  modules). Moved here from `.claude/aw/scripts/` on 2026-09-02; the
  verification suite stayed at `.claude/aw/verification/` and resolves this
  path through its `_paths.SCRIPTS`.
- `e2e/` — black-box CLI cases, run with pytest via the gate above.

### Release plans

`aw release-plan validate --plan <path|->` reads and canonicalizes one closed
`release-plan-v1` JSON document. It accepts an unsealed draft, adds
`plan_sha256`, and prints one sealed canonical plan. The digest covers the
canonical plan with `plan_sha256` omitted. Validation does not write files or
contact the tracker. Its output can be saved directly as a later `apply` file.

`apply` needs a file, one `apps/name` or `libs/name` project, and that exact
approved digest. Each project carries exact document bytes plus a complete
tracker baseline summary and digest. A release plan uses
`{{milestone_number}}` in an indexed promise heading and in that promise's
exact repository Milestone Tracking link. It uses `{{development_order}}` in
the Milestone description. The facade replaces both tokens after GitHub
assigns the real numbers. Every planned issue also carries its approved `p0`
to `p5` priority.

The project list is an execution chain. A later project can apply only after
all earlier project receipts are complete. Put the first project to apply at
the start of the list. For a new Milestone, the planning skill uses
`milestone next-version` and its normal minor bump. Only an initial release or
an explicit human exception selects another version.

Before any mutation, `apply` checks the repository, project order, Git commit,
clean working tree, document hashes, tracker summary, Milestone identity,
issue type, owner label, and order. It renders the approved documents in a
disposable clone and runs both META checks there. A new Milestone uses two
different preview numbers, so a hard-coded number cannot pass by matching one
probe. The preview also requires each bound promise to carry the exact
Milestone Tracking link in its owner field. It then creates the durable
receipt at `.aw/release-plans/<digest>/<project>.json`.

The receipt records the META commit, Milestone number, each issue number, and
the final reconciliation evidence. `resume --receipt <path>` recovers only an
exact accepted write. Zero or multiple matches stop. A complete receipt is
read again and fails if Git or tracker state drifted. Resume also refuses any
working-tree change outside exact planned META bytes. The final gap evidence
contains all G1 through G7 rows. G1 through G5 must be measured and clear.
G6 and G7 may remain as recorded delivery work for the planned issues.

## Development

```
uv run --project apps/aw aw release-plan --help
uv run --project apps/aw --directory apps/aw pytest e2e/
```
