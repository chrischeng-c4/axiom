---
project:
  name: axiom
  owner: chrischeng-c4
  url: https://github.com/chrischeng-c4/axiom
  ssh: git@github.com:chrischeng-c4/axiom.git
  default_branch: main
---

# CLAUDE.md - Claude Code bootstrap

This file and `.claude/rules/**/*.md` are what loads at launch, and they are the
whole of it. `AGENTS.md` is deliberately not imported here.

Both halves of that were measured. `codex exec` loads `AGENTS.md` from its
workdir into its instructions with no tool call; Claude Code loads no
`AGENTS.md` at all unless a `CLAUDE.md` imports one. So the two files have one
reader each, and **editing `AGENTS.md` changes what the reviewer is told, not
what you are told**. It is a production input to `/aw:codex-e2e-review` and
`/aw:codex-code-review`. Do not treat it as documentation and do not move
repo facts into it for tidiness.

`.claude/rules/**/*.md` are hand-maintained — nothing generates them and nothing
detects drift — so a rule that has stopped being true stays in every session's
context until a human deletes it.

Read `README.md` for repository inventory. Project promises live in each
project's own `README.md`, under `## Capabilities`; local workflow and
verification live in its `CONTRIBUTING.md`. There is no third META-doc —
`<project>/CAPABILITIES.md` was deleted on 2026-08-17 and its content merged
into the README.

## `aw` is `plugins/aw`

`aw` is the phase scripts under `plugins/aw/scripts/` and the skills under
`plugins/aw/skills/`. There is no CLI behind them — the protocol is their stdout
and their exit codes.

Launch every one of them through `uv run --python 3.13 --no-project`. They read
TOML, `tomllib` is 3.11+, and a bare `python3` is 3.9 on at least one machine
here — where the failure is a `ModuleNotFoundError` traceback that reads like a
broken script rather than a wrong interpreter.

There is no `aw` binary. The Rust application at `apps/agentic-workflow` that
carried the name is deleted, and `cargo uninstall agentic-workflow` removed the
copy on `PATH` — so a stray `aw wi …` now fails with "command not found"
instead of running, mutating the tracker, and printing something plausible.
That failure is the whole of the enforcement; nothing rewrites an `aw` verb
into the script that replaced it.

## Skills

Eight entry points. Each is invoked by a human, and each hands off to a script
that can refuse it.

| Skill | Reach for it when | It does |
|---|---|---|
| `/aw:wi-change-grill` | a change must be opened, or its body is thin, stale, or unvalidatable | interviews you, then `change.py create\|update` writes the body |
| `/aw:wi-epic-grill` | the same, for an epic | interviews you, then `epic.py create\|update` |
| `/aw:wi-epic-reconcile` | before driving an epic, or whenever its child set is suspect | opens what the epic promises but never opened, and resolves duplicates and misfiled children |
| `/aw:wi-tdd` | the work item is ready to implement | drives one change — or an epic's children in dependency order — through the ladder below |
| `/aw:codex-e2e-review` | the `e2e` phase printed it as the next command | routes the case to the other model and binds its verdict to the reviewed bytes |
| `/aw:codex-code-review` | the `logic` phase printed it as the next command | the same, for the implementation and its colocated tests |
| `/aw:prepare-goal` | a work item, or a bare intent, has to become a `/goal` the session's evaluator can actually decide | interviews you or reads the tracker through `epic.py`/`change.py`, then prints conditions for you to paste |
| `/aw:meta-check` | before trusting a `CLAUDE.md`, `README.md` or `CONTRIBUTING.md`, and after editing one | `meta.py check` reports every doc fact whose owner is gone |

The usual sequence is grill → reconcile (epics only) → `wi-tdd`; the two review
skills are reached from the phase that prints them, not chosen, and
`prepare-goal` and `meta-check` stand outside the lifecycle entirely.

- A grill never writes product source and never invents an answer you did not
  give. It offers only gates the repository already runs, and it stops asking
  once the body's own sections are answered.
- On reconcile, the label is what makes a child a child. An unlabelled issue is
  not one, whatever its body claims.
- `wi-tdd` on an epic reads the order out of the epic's own `Depends On`
  column. A line beginning `!` means there is no order to follow: report it and
  stop. An epic body you edited to make the graph parse is an epic whose
  dependencies you decided.
- `prepare-goal` prints text and sets nothing. `/goal` is a Claude Code
  built-in that nothing here implements, and a goal exists only once the human
  pastes one of the printed lines — which starts a turn on the spot, and
  supersedes whatever goal was already running.
- `meta-check` reads and never writes, and its baseline is **not** zero: 103
  findings over 182 documents today, nearly all of them markers left behind by
  the deleted CLI. So it is not yet wired into `run_all.py` as a ratchet — it
  is a report you run over what you touched, and a rising count in a file you
  edited is the signal.

## Authority Order

- `<project>/README.md` owns product promises, work roots, and gates.
- `<project>/CONTRIBUTING.md` owns project-local edit and verification rules.
- `.claude/rules/**/*.md` owns reusable agent instructions, one concern per
  semantic path.
- Skills are thin human-invoked entry points. Hard enforcement is the exit code
  of the phase scripts, not a guard binary.

## External implementation workers

Repository operating policy. It does not override a host's own approval gate
for sending data outside the machine.

When a bounded task can use an external worker, prefer one fresh
`dispatch-operator` subagent. Its model is Sonnet at low reasoning. Create it
only after the user authorizes the exact headless AGY payload. Make it directly
inherit that user turn. Do not reuse an older operator. Do not forward
authorization through a controller message.

The controller freezes the profile, task key, action, snapshot mode, and all
input digests before dispatch. The snapshot mode is `create`, `reuse`, or
`refresh`. The operator only checks those inputs and runs the exact matching
`doctor` / `snapshot` / `dispatch` / `resume` / status sequence. It must return
`HANDOFF_INCOMPLETE` if the authorization or frozen handoff is incomplete.

The operator never authors the contract, oracle, injection, or prompt. It does
not verify or accept the result. It does not create or change a permission or
worktree. It does not run Git, tracker, publication, or cleanup actions.

The controller owns the profile, task contract, oracle, injection, prompt,
worktree creation, independent verification, semantic acceptance, Git, tracker
changes, publication, and cleanup. Follow `.claude/skills/agy-dispatch/` as the
source of truth for the AGY model, Project, permission, snapshot, command, and
write rules. Run every adapter verb from the repository root as
`python3 scripts/agy_dispatch.py ...`. Do not use an installed, skill-local, or
legacy dispatcher copy. Send only task-required repository material. Never send
secrets. Do not run workers with overlapping write ownership in parallel.

The `$copilot-dispatch` this paragraph named until 2026-08-17 has never existed
in this checkout, so the policy covers `agy-dispatch` and whatever is added
beside it, not a second adapter someone might go looking for.

## Work-item lifecycle

The lifecycle is linear: `wi → e2e → unit → logic`. Each phase runs `start`,
`verify`, `test`, and `commit`, and each prints the command that follows it.
`e2e` and `logic` additionally carry `review-prompt` and `verdict`.

A phase's green must be attributable to a named red measured immediately before
it in the same tree. That is what the commit trailers `E2E-Red:`, `Unit-Red:`,
and `Logic-Contract:` carry, and what the next phase's predecessor check reads.

A review verdict is the other model's answer, never yours. Run the reviewer,
relay verbatim what it returned, and treat a rejection as blocking; a verdict
you wrote yourself is a fabricated approval and the commit gate that reads it is
measuring nothing.

### Work-item terminal states

The closed work-item enum lives at `plugins/aw/scripts/workitem.py`:

| Type | Terminal state |
|---|---|
| `epic` | all owned children are terminal |
| `change` | the phase-1 case is green and the lifecycle closes the change |
| `spike` | an ADR-style decision records spawned WI refs or explicit no-action; expiry converges to `gave_up` |
| `report` | typed `triage` accepts and links a spawned change/epic, or closes as `duplicate`, `invalid`, or `by-design` |

Only `change` enters executable backlog work. A `spike` never lands
investigation code in product source. A `report` remains in the intake queue
until triage, and both converge by spawn-and-link instead of changing type in
place.

## Artifact write order

Every change to a project under `apps/<name>/` is authored one phase at a time,
red first. A phase does not start until its predecessor has landed its commit.

| Phase | Writes | What lands there |
|---|---|---|
| 1 `e2e` | `apps/<name>/e2e/` | The black-box case, written to fail against the current tree |
| 2 `unit` | `apps/<name>/src/` | Colocated tests plus the skeleton they fail against; at least one test file is required |
| 3 `logic` | `apps/<name>/src/` | The implementation, which may not touch a test file |

- This table is not advice. `C0` refuses any dirty path outside the phase's
  write root against `leg.LEG_ROOTS`, and tells `unit` from `logic` inside their
  shared root against `leg.LEG_TEST_FILES` — by filename, never by reading a
  `#[cfg(test)]` span.
- Never open `src/**` first. Reaching implementation before the contract exists
  means nothing can refuse the implementation afterwards.
- Write the phase-1 case so it **fails against the current tree**, and run it to
  observe that failure before moving on. A contract that was green before the
  change was written proves nothing about the change.
- `libs/<name>/` has no ladder. `leg.leg_root` resolves under `apps/` only.
- `external-contracts/` and `tech-design/` are not write roots and not authoring
  surfaces, and a `// SPEC-MANAGED: <path>#<anchor>` header names a producer that
  no longer exists. The `.rs` file is the authoring surface; editing the markdown
  a header names propagates nowhere.

## Phase-1 boundaries

- Externally observable product behavior belongs in the project-local Python
  phase-1 project at `apps/<name>/e2e/`, where `pyproject.toml` is the inventory
  and `src/cases/*.py` holds one black-box verifier per case.
- Rules observable only inside the Rust implementation are colocated tests under
  their semantic `src/**` owner, in a `tests.rs` wired in with
  `#[cfg(test)] mod tests;`. They are authored by the `unit` phase, before the
  implementation exists.
- Never wrap a Python phase-1 case in an app-level Rust tree and never delegate
  one to `cargo test`.
