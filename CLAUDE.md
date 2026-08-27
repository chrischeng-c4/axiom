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

`aw` is the phase scripts under `.claude/aw/scripts/` and the six skills
under `.claude/skills/aw-*/`. There is no CLI behind them — the protocol is their
stdout and their exit codes.

It was a Claude Code plugin at `plugins/aw/` until 2026-08-21. That tree is
deleted: the scripts moved to `.claude/aw/scripts/`, the verification suite to
`.claude/aw/verification/`, and `plugins/aw/skills/`, `plugins/aw/.claude-plugin/plugin.json`
and `.claude-plugin/marketplace.json` were removed outright, along with the
`enabledPlugins` entry in `.claude/settings.json`. The skills in
`.claude/skills/` — eight originally, eleven from 2026-08-26 — were already
the copy every session read, so nothing about what loads changed at the
collapse of the plugin — what changed is that there is now one copy of each
file instead of two, and `${CLAUDE_PLUGIN_ROOT}` resolves nowhere. The
2026-08-27 restructure described in "Skills" below folds into that same day:
it retires nine of those eleven — the one that wrote product-promise prose,
the one that checked META-docs standalone, the two that wrote technical-design
sections and ADRs, the one that opened an epic from an unbound promise, the
one that opened a change from an interview, the one that opened an epic's
children, and the two that drove the phase ladder — and adds four, landing on
the six named there.

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

Seven entry points: the six `aw-*` skills below, plus the non-`aw`
`/project-readme-check`. Each is invoked by a human. `grill-me-to-meta`
interviews a human and writes prose under one project's `README.md`,
`STATUS.md`, `ROADMAP.md` and `docs/**`, and hands the run to `metadoc.py`,
which refuses a commit that wrote anywhere else and is the only writer of its
own commit; `grill-meta-to-wis` measures the gap with `wis.py gap` and
reorganises the tracker to close it through `epic.py`/`change.py`;
`e2e-for-wi` and `impl-for-wi` hand off to `e2e.py` and `impl.py`, phase
scripts that can refuse them; `ask-user` writes nothing; the project-document
checker uses its own read-only validators and a clean-context reader.

There is no technical-design step. Two grills — one per epic, one per change —
wrote per-subsystem technical-design sections and ADRs beside them, under a
`technical/` tree nested inside each project's `docs` directory, from
2026-08-26 until 2026-08-27; both skills and that whole tree are deleted.
Over that lifetime they produced no section at all and five ADRs, and the rule
that already governed this — `.claude/rules/authoring/source-carries-its-own-design.md`
— says the `.rs` file is the authoring surface. A design decision goes in the
`//!` or `///` block of the module or type that owns it.

They load as project skills out of `.claude/skills/`, not as the `aw` plugin,
and that copy is what a session actually reads. Each directory is named
`aw-<skill>`, and the directory name is the command — `/aw-<skill>` — because
the loader keys off the path and ignores the frontmatter `name:`, measured with
a probe whose two names disagreed, and the directory won. The frontmatter
`name:` is `aw:<skill>` and is only the label the skill list shows; since the
rename on 2026-08-26 no `/aw:<skill>` resolves, and `check_plugin.py` refuses a
body that still writes one. These six are the only copy. A second set lived
under
`plugins/aw/skills/` with `${CLAUDE_PLUGIN_ROOT}` paths where these write
`.claude/aw/scripts/`, nothing detected drift between them, and that pair was
collapsed on 2026-08-21 by deleting the plugin.

| Skill | Reach for it when | It does |
|---|---|---|
| `/aw-grill-me-to-meta` | a project's `README.md`, `STATUS.md`, `ROADMAP.md`, or a `docs/**` section is missing, stale, wrong, or no longer wanted | interviews you, then writes those four paths for one project through the landing sequence `metadoc.py check` → `meta.py check` → `metadoc.py commit`; absorbs the retired standalone meta-checking skill as that sequence's second step |
| `/aw-grill-meta-to-wis` | a `docs/**` section has no epic yet, an epic's child set is suspect, or a change must be opened, or its body is thin, stale, or unvalidatable | runs `wis.py gap` for the seven-row table, then closes what it shows through `epic.py create\|update` and `change.py create\|update`; binds a section to the epic it opens in the same run — the heading gains ` (#<iid>)`, `Tracking:` gains the link |
| `/aw-e2e-for-wi` | one work item's black-box contract is ready to write | drives the e2e phase's four verbs on a change, or on each child of an epic in the order `epic.py order --open-only` prints |
| `/aw-impl-for-wi` | a work item's e2e phase has landed | drives the impl phase's five verbs — `start`, `red`, `verify`, `test`, `commit` — on a change, or on each child of an epic in that same order |
| `/aw-prepare-goal` | a work item, or a bare intent, has to become a `/goal` the session's evaluator can actually decide | interviews you or reads the tracker through `epic.py`/`change.py`, then prints conditions for you to paste |
| `/aw-ask-user` | the session has been deciding for you — stated assumptions, a route picked alone, `Open:` lines it read past | walks the context for every pending question, asks each through AskUserQuestion, and prints a decision table; writes nothing |
| `/project-readme-check` | after creating or editing an app or library README, STATUS, ROADMAP, protocol, generated-client, indexing, querying, GKE, client-integration, or migration guide | validates the adopted product-document set and cross-file references, then asks a context-free reader to restate the current, future, and interface contract |

The usual sequence is `grill-me-to-meta` → `grill-meta-to-wis` →
`e2e-for-wi` → `impl-for-wi`. The first step is a hard stop rather than a
convention: `grill-meta-to-wis` refuses to open an epic that no `docs/**`
section promises. The rest of the order is convention — nothing downstream
reads what came before it. `prepare-goal` and `ask-user` stand outside the
lifecycle entirely.

- A grill never writes product source and never invents an answer you did not
  give. It offers only gates the repository already runs, and it stops asking
  once the body's own sections are answered.
- On `grill-meta-to-wis`, the label is what makes a child a child. An
  unlabelled issue is not one, whatever its body claims.
- `grill-me-to-meta` writes only `README.md`, `STATUS.md`, `ROADMAP.md` and
  `docs/**` for the one project it names, one `## <title>` section per
  promise — never for an issue number, because a promise can predate the
  work item that will carry it. That allowlist is measured rather than asked
  for: `metadoc.py check` reads the dirty set against HEAD and refuses every
  path outside it, along with a section missing one of its own kind's
  bullets (a section is exactly one of `Outcome:`-shaped or
  `Status rows:`-shaped), a STATUS or ROADMAP id that resolves nowhere, and a
  heading that gained a `(#<iid>)` — binding a section to an epic is
  `grill-meta-to-wis`'s job, not this one's, and this check refuses a run
  that did it anyway. `metadoc.py commit` re-runs all of it, stages the
  allowlist, and writes the `Meta-Project:` / `Meta-Top:` / `Meta-Index:` /
  `Meta-Section:` / `Meta-Unbound:` trailers that make the commit findable; a
  META-doc commit written by hand carries none of them. A section is bound to
  its epic — the heading gains ` (#<iid>)`, `Tracking:` gains the link — by
  `grill-meta-to-wis`, in the same run that opens the epic, and by nothing
  else.
- `wis.py gap` prints seven rows, each with the size of what it read printed
  beside the count (`3 / 31`), not a bare count: G1 a future promise no epic
  is opened for, G2 an open work item no promise reaches, G3 a promise bound
  to an issue that cannot carry it, G4 a ROADMAP outcome no promise claims,
  G5 a STATUS row no promise claims, G6 an e2e case the crate manifest does
  not run, G7 a README gate that names no cargo target. A row it could not
  read — a missing `docs/` area file, an absent `STATUS.md`, `gh` exiting
  non-zero outside a git directory — prints `?` UNMEASURED with the reason
  instead of `0`, and is counted separately, so a run that could not reach
  the tracker never reports `=> ALIGNED`. It writes nothing; every write
  `grill-meta-to-wis` makes goes through `epic.py` / `change.py`.
- `e2e-for-wi` and `impl-for-wi` dispatch on a work item's type by running
  `epic.py order <iid> --open-only` first. On a change it refuses and names
  the actual type — that message, not the exit code, is the type answer,
  since "not an epic" and "epic whose graph has no solution" are both exit 1.
  On an epic, a line beginning `!` or `?` means there is no order to follow:
  quote it verbatim and stop. An epic body edited to make the graph parse is
  an epic whose dependencies you decided.
- `prepare-goal` prints text and sets nothing. `/goal` is a Claude Code
  built-in that nothing here implements, and a goal exists only once the human
  pastes one of the printed lines — which starts a turn on the spot, and
  supersedes whatever goal was already running.
- `meta.py check` reads and never writes; it is the second step of
  `grill-me-to-meta`'s landing sequence now, where the standalone skill it
  came from used to be run by a human who had to remember to. Its baseline
  was 103 findings over 182 documents, nearly all
  of them markers left behind by the deleted CLI. Those are cleared, and so
  are the three rules added on 2026-08-20 — `M5` a gate whose test-name
  filter cargo exits 0 on, `M6` a gate naming a package or target that is not
  in the checkout, `M7` a self-graded `Status:`/`Maturity:` field — which
  landed at 151, 5 and 526 findings and reached zero the same day the 60
  project READMEs carrying them were rewritten. It reports `=> CLEAN` over
  184 tracked META-docs and 64 project READMEs; the count was 185 until
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

The lifecycle is linear: `wi → e2e → impl`. Each phase prints the command
that follows it. `e2e` runs four verbs — `start`, `verify`, `test`, `commit`;
`impl` runs five — `start`, `red`, `verify`, `test`, `commit`. The extra verb
is load-bearing: the retired three-phase ladder proved attribution with two
commits — its middle phase's own trailer was the record the phase after it
read back — and a two-phase ladder's second phase has no earlier commit of
its own kind to read. `red` is where the evidence moves instead — it records the
named failing tests, the head sha, and a per-test-file sha256 to
`.aw/impl-red/<iid>.json`, and only passes at the moment the tests are
written and the implementation is not; write the implementation first and
`red` finds nothing failing to name, and refuses.

A phase's green must be attributable to a named red measured immediately before
it in the same tree. That is what the commit trailers `E2E-Red:`, `Impl-Red:`,
and `Impl-Contract:` carry: `e2e`'s own commit carries `E2E-Red:`; `impl`'s
`verify`/`test` refuse a tree that has drifted from the `red` record before
`impl`'s commit ever writes `Impl-Red:`/`Impl-Contract:` back onto it.

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
| 2 `impl` | `apps/<name>/src/` | The skeleton, its colocated tests, and the implementation that turns them green; at least one test file is required |

- This table is not advice. `C0` refuses any dirty path outside the phase's
  write root against `leg.LEG_ROOTS`. The retired ladder told `unit` from
  `logic` inside their shared `src/` root against `leg.LEG_TEST_FILES` — by
  filename, never by reading a `#[cfg(test)]` span; that boundary gate is
  gone with those two phases. What `leg.LEG_TEST_FILES` still gates is an
  existence requirement: an `impl` commit that touches no test file is
  refused by `C0`.
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
- `docs/product/` exists under every `apps/*` and `libs/*` since 2026-08-26,
  and as of 2026-08-27 `metadoc.py`'s write allowlist widens past it to four
  paths per project: `README.md`, `STATUS.md`, `ROADMAP.md`, and everything
  under `docs/**` — not `docs/product/` alone, because a promise and the
  STATUS row that measures it are one edit now, not two skills' worth. It is
  where `grill-me-to-meta` writes, before any work item exists, one
  `## <title>` section per promise; the heading gains its ` (#<iid>)` when
  `grill-meta-to-wis` opens the epic. The technical-design tree that landed
  beside `docs/product/` on the same day, under `docs`'s own `technical/`,
  is deleted as of 2026-08-27, ADRs included — a design decision lives in the
  `.rs` file that owns it.
  `docs/**` is not `tech-design/` under a new name: nothing generates from
  it, no source header points at it, and it is prose a human was interviewed
  into. Nor is it a ladder write root — `C0` refuses a dirty `docs/` path
  like any other path outside a phase's write root, so land a META-doc
  before `e2e.py start`, not during.

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
