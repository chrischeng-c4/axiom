# aw

The AW workflow engine and its Typer CLI. Managed with `uv`; requires
Python 3.13. The canonical invocation, from the repository root, is
`uv run --project apps/aw aw <group> ...` — that exact prefix is what the
engine prints in its `next.command:` lines and what the ten `aw-*` skills
run.

## Capabilities

### CLI entry point

Outcome: `uv run --project apps/aw aw` with no arguments prints usage and
exits non-zero; `uv run --project apps/aw aw version` prints the project
version and exits zero.

Gate: `uv run --project apps/aw --directory apps/aw pytest e2e/` (run from
the repository root).

### Workflow command groups

Outcome: eight groups delegate to the engine modules under
`src/aw/scripts/`, one module per group, with argparse staying the single
source of argument validation — `change`, `milestone`, `e2e`, `impl`,
`maint`, `wis`, `meta`, and `metadoc`. Every group/verb/option the typer
surface accepts rebuilds an argv the engine module's own parser accepts.

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
  `wis.py`, `meta.py`, `metadoc.py`, and their shared modules). Moved here
  from `.claude/aw/scripts/` on 2026-09-02; the verification suite stayed
  at `.claude/aw/verification/` and resolves this path through its
  `_paths.SCRIPTS`.
- `e2e/` — black-box CLI cases, run with pytest via the gate above.

## Development

```
uv run --project apps/aw aw --help
uv run --project apps/aw --directory apps/aw pytest e2e/
```
