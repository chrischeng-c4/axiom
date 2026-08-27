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
reader each, and **editing `AGENTS.md` changes what codex is told, not what
you are told**. Until 2026-08-26 it was a production input to two codex review
skills; those left the ladder, so it is now codex's bootstrap for a
human-driven session and nothing here reads it. Do not treat it as
documentation and do not move repo facts into it for tidiness.

`.claude/rules/**/*.md` are hand-maintained — nothing generates them and nothing
detects drift — so a rule that has stopped being true stays in every session's
context until a human deletes it.

Read `README.md` for repository inventory. Project promises live in each
project's own `README.md`, under `## Capabilities`; local workflow and
verification live in its `CONTRIBUTING.md`. There is no third META-doc —
`<project>/CAPABILITIES.md` was deleted on 2026-08-17 and its content merged
into the README.

## `aw` is `.claude/aw` plus `.claude/skills/aw-*`

`aw` is the phase scripts under `.claude/aw/scripts/` and the eleven skills
under `.claude/skills/aw-*/`. There is no CLI behind them — the protocol is their
stdout and their exit codes.

It was a Claude Code plugin at `plugins/aw/` until 2026-08-21. That tree is
deleted: the scripts moved to `.claude/aw/scripts/`, the verification suite to
`.claude/aw/verification/`, and `plugins/aw/skills/`, `plugins/aw/.claude-plugin/plugin.json`
and `.claude-plugin/marketplace.json` were removed outright, along with the
`enabledPlugins` entry in `.claude/settings.json`. The skills in
`.claude/skills/` — eight then, eleven since 2026-08-26, still eleven after
2026-08-27 swapped the two `grill-*-to-prd` for `grill-me-to-prd` and
`ask-user` — were already the copy
every session read, so nothing about what loads changed — what changed is that there is now one copy of each file
instead of two, and `${CLAUDE_PLUGIN_ROOT}` resolves nowhere.

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

Twelve entry points. Each is invoked by a human. Lifecycle skills hand off to
a phase script that can refuse them; the PRD grill writes prose under the
owning project's `docs/product/` before any work item exists, and hands it to
`prd.py`, which refuses a run that wrote anywhere else and is the only writer
of its commit; the two TD grills write under `docs/technical/` and hand off to
nothing at all; `ask-user` writes nothing; the project-document checker uses
its own read-only validators and a clean-context reader.

They load as project skills out of `.claude/skills/`, not as the `aw` plugin,
and that copy is what a session actually reads. Each directory is named
`aw-<skill>`, and the directory name is the command — `/aw-<skill>` — because
the loader keys off the path and ignores the frontmatter `name:`, measured with
a probe whose two names disagreed, and the directory won. The frontmatter
`name:` is `aw:<skill>` and is only the label the skill list shows; since the
rename on 2026-08-26 no `/aw:<skill>` resolves, and `check_plugin.py` refuses a
body that still writes one. These eleven are the only copy. A second set lived
under
`plugins/aw/skills/` with `${CLAUDE_PLUGIN_ROOT}` paths where these write
`.claude/aw/scripts/`, nothing detected drift between them, and that pair was
collapsed on 2026-08-21 by deleting the plugin.

| Skill | Reach for it when | It does |
|---|---|---|
| `/aw-grill-me-to-prd` | a project's product promise is not written down yet, or a section of it is stale, wrong, or no longer wanted | interviews you, then creates, modifies, or deletes one `## <title>` section — or a capability area — under the owning project's `docs/product/`; no work item exists yet and none is opened |
| `/aw-grill-me-to-epic` | a `docs/product/` section has no epic yet, or an epic's body is thin, stale, or unvalidatable | reads the unbound section, interviews you for the rest, then `epic.py create\|update` writes the body and the section heading gains ` (#<iid>)` |
| `/aw-grill-epic-to-changes` | before driving an epic, or whenever its child set is suspect | opens what the epic promises but never opened, and resolves duplicates and misfiled children |
| `/aw-grill-me-to-change` | a change must be opened, or its body is thin, stale, or unvalidatable | interviews you, then `change.py create\|update` writes the body |
| `/aw-grill-epic-to-td` | an epic has its PRD and its design — premises, change points, interfaces, the e2e case that will judge it — is not yet written down | interviews you, then writes one `## <title> (#<iid>)` section into `docs/technical/<subsystem>.md`, plus an ADR under `docs/technical/adr/` for each decision that outlives the item |
| `/aw-grill-change-to-td` | the same, for a change | the same |
| `/aw-go-tdd-for-epic` | an epic's children are ready to implement | asks `epic.py order` for the dependency order and runs `/aw-go-tdd-for-change` on each child; stops when the script prints no order |
| `/aw-go-tdd-for-change` | one change is ready to implement | drives it through the ladder below |
| `/aw-prepare-goal` | a work item, or a bare intent, has to become a `/goal` the session's evaluator can actually decide | interviews you or reads the tracker through `epic.py`/`change.py`, then prints conditions for you to paste |
| `/aw-check-meta` | before trusting a `CLAUDE.md`, `README.md` or `CONTRIBUTING.md`, and after editing one | `meta.py check` reports every doc fact whose owner is gone |
| `/aw-ask-user` | the session has been deciding for you — stated assumptions, a route picked alone, `Open:` lines it read past | walks the context for every pending question, asks each through AskUserQuestion, and prints a decision table; writes nothing |
| `/project-readme-check` | after creating or editing an app or library README, STATUS, ROADMAP, protocol, generated-client, indexing, querying, GKE, client-integration, or migration guide | validates the adopted product-document set and cross-file references, then asks a context-free reader to restate the current, future, and interface contract |

The usual sequence is `grill-me-to-prd` → `grill-me-to-epic` (which binds
the section to the epic it opens) → `grill-epic-to-changes` →
`grill-epic-to-td` → `grill-change-to-td` → `go-tdd-for-*`. The first step
is a hard stop rather than a convention: `grill-me-to-epic` refuses to open
an epic that no `docs/product/` section promises. The rest of the order is
convention — nothing in the ladder checks that a TD exists. `prepare-goal`,
`check-meta` and `ask-user` stand outside the lifecycle entirely.

- A grill never writes product source and never invents an answer you did not
  give. It offers only gates the repository already runs, and it stops asking
  once the body's own sections are answered.
- On `grill-epic-to-changes`, the label is what makes a child a child. An
  unlabelled issue is not one, whatever its body claims.
- `grill-me-to-prd` writes only under the owning project's `docs/product/`,
  one `## <title>` section per promise, into a file named for its capability
  area — never for an issue number, because no issue exists yet. That
  allowlist is measured rather than asked for: `prd.py check` reads the dirty
  set against HEAD and refuses every path outside it, along with a section
  missing one of its own kind's bullets, a STATUS or ROADMAP id that resolves
  nowhere, and a heading that gained a `(#<iid>)` the epic grill has not
  bound. `prd.py commit` re-runs all of it, stages the allowlist, and writes
  the `PRD-Project:` / `PRD-Section:` / `PRD-Unbound:` trailers that make the
  commit findable; a PRD commit written by hand carries none of them. A
  section is bound to its epic — the heading gains ` (#<iid>)`, `Tracking:` gains the
  link — by `grill-me-to-epic`, in the same run that opens the epic, and by
  nothing else.
- A `grill-*-to-td` writes only under the owning project's `docs/technical/`,
  one `## <title> (#<iid>)` section per work item, into a file named for its
  subsystem. Without a work item there is no section to write.
- `go-tdd-for-epic` reads the order out of the epic's own `Depends On`
  column. A line beginning `!` means there is no order to follow: report it and
  stop. An epic body you edited to make the graph parse is an epic whose
  dependencies you decided.
- `prepare-goal` prints text and sets nothing. `/goal` is a Claude Code
  built-in that nothing here implements, and a goal exists only once the human
  pastes one of the printed lines — which starts a turn on the spot, and
  supersedes whatever goal was already running.
- `check-meta` reads and never writes. Its baseline was 103 findings over 182
  documents, nearly all of them markers left behind by the deleted CLI. Those
  are cleared, and so are the three rules added on 2026-08-20 — `M5` a gate
  whose test-name filter cargo exits 0 on, `M6` a gate naming a package or
  target that is not in the checkout, `M7` a self-graded `Status:`/`Maturity:`
  field — which landed at 151, 5 and 526 findings and reached zero the same day
  the 60 project READMEs carrying them were rewritten. It reports `=> CLEAN`
  over 184 tracked META-docs and 64 project READMEs; the count was 185 until
  `4a30ca3097` deleted `apps/lumen`'s retired trees.
- All seven rules are ratcheted to zero by
  `.claude/aw/verification/check_meta_clean.py`, which `run_all.py` runs with a
  negative control. That is weaker than it sounds: nothing in this repository
  calls `run_all.py` — no CI workflow, no git hook, no phase script — so the
  ratchet is one a human runs, and a finding in a file you edited is still the
  signal.

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
`agy-operator` subagent. Its model is Sonnet at low reasoning. Create it only
after the user authorizes the exact headless AGY payload. Make it directly
inherit that user turn. Do not reuse an older operator. Do not forward
authorization through a controller message.

For more than one task in the same round, use `/dispatch-to-agy` instead of
hand-driving each operator: it classifies every task `measure-only` or
`bounded-write`, then fans out. Any number of `measure-only` tasks may run
concurrently. `bounded-write` tasks may run concurrently only across distinct
persistent AGY Projects — AGY has not proven per-conversation worktree
confinement for two concurrent bounded writes in one Project, so those queue
one at a time regardless of how disjoint their write ownership looks.

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

A phase's green must be attributable to a named red measured immediately before
it in the same tree. That is what the commit trailers `E2E-Red:`, `Unit-Red:`,
and `Logic-Contract:` carry, and what the next phase's predecessor check reads.

### Work-item terminal states

The closed work-item enum lives at `.claude/aw/scripts/workitem.py`:

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
- `docs/product/`, `docs/technical/` and `docs/technical/adr/` exist under every
  `apps/*` and `libs/*` since 2026-08-26. `docs/product/` is where
  `grill-me-to-prd` writes, before any work item exists, one `## <title>`
  section per promise; the heading gains its ` (#<iid>)` when
  `grill-me-to-epic` opens the epic. `docs/technical/` and `adr/` are where the
  two `grill-*-to-td` skills write, one `(#<iid>)` section at a time. None of
  them is `tech-design/` under a new name: nothing generates from them, no
  source header points at them, and they are prose a human was interviewed
  into. Nor are they ladder write roots — `C0` refuses a dirty `docs/` path
  like any other path outside a phase's write root, so land a PRD or TD before
  `e2e.py start`, not during.

## Test Layout

Every `apps/<p>` and `libs/<p>` crate owes an `e2e/` tree. Externally
observable behavior goes in `{apps,libs}/<p>/e2e/*.rs` — Rust, one file per
case, run by `cargo test -p <crate>`. Declare each one: `autotests = false`
plus a `[[test]]` stanza per file, so the `Cargo.toml` manifest is the
inventory and nothing starts or stops running without showing up in a diff.
Rules observable only inside the implementation stay as colocated unit tests
under their semantic `src/**` owner (`cargo test -p <crate> --lib`).

Write the `e2e/*.rs` case before the `src/**` change it judges.

`tech-design/`, `external-contracts/`, and `tests/` are superseded: no authored
source belongs in them. Never create one, never add a file to one, and migrate
a surviving case into `e2e/` rather than editing it in place. Python spec
models and `src/cases/*.py` verifiers are retired — never author a new one.
Generated EC evidence under `external-contracts/` is output, not contract.
