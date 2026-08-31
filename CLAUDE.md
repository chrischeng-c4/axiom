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

## `aw` is `.claude/aw` plus both `skills/aw-*` mirrors

`aw` is the scripts under `.claude/aw/scripts/` and the seven skills under
`.agents/skills/aw-*/` and `.claude/skills/aw-*/`. There is no binary behind
them — the protocol is their
stdout and their exit codes.

It was a Claude Code plugin at `plugins/aw/` until 2026-08-21. That tree is
deleted: the scripts moved to `.claude/aw/scripts/`, the verification suite to
`.claude/aw/verification/`, and `plugins/aw/skills/`, `plugins/aw/.claude-plugin/plugin.json`
and `.claude-plugin/marketplace.json` were removed outright, along with the
`enabledPlugins` entry in `.claude/settings.json`.

AW now has seven project skills in two runtime roots. Codex reads
`.agents/skills/aw-*/SKILL.md`. Claude Code reads
`.claude/skills/aw-*/SKILL.md`. Each matching pair must be byte-identical.
Both runtimes call the one engine under `.claude/aw/scripts/`. The retired
plugin root and `${CLAUDE_PLUGIN_ROOT}` still resolve nowhere.

Launch every AW script through `uv run --python 3.13 --no-project`. They read
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

Eight entry points: the seven `aw-*` skills below, plus the non-`aw`
`/project-readme-check`. Each is invoked by a human. `grill-me-to-meta`
interviews a human and writes prose under one project's `README.md`,
`STATUS.md`, `ROADMAP.md` and `docs/**`, and hands the run to `metadoc.py`,
which refuses a commit that wrote anywhere else and is the only writer of its
own commit; `grill-meta-to-wis` measures the gap with `wis.py gap` and
reorganises the tracker through `milestone.py`/`change.py`;
`e2e-for-wi`, `impl-for-wi`, and `maint-for-wi` hand off to their matching
phase scripts, which can refuse them; `ask-user` writes nothing; the
project-document checker uses its own read-only validators and a clean-context
reader.

There is no technical-design step. Two old grills — one per issue epic, one per change —
wrote per-subsystem technical-design sections and ADRs beside them, under a
`technical/` tree nested inside each project's `docs` directory, from
2026-08-26 until 2026-08-27; both skills and that whole tree are deleted.
Over that lifetime they produced no section at all and five ADRs, and the rule
that already governed this — `.claude/rules/authoring/source-carries-its-own-design.md`
— says the `.rs` file is the authoring surface. A design decision goes in the
`//!` or `///` block of the module or type that owns it.

They load as project skills from the runtime-specific root above, not as the
retired `aw` plugin. Each directory and frontmatter name is `aw-<skill>`.
`check_plugin.py` refuses a missing skill, a mismatched
pair, or a skill that calls the legacy issue-epic writer.

| Skill | Reach for it when | It does |
|---|---|---|
| `/aw-grill-me-to-meta` | a project's `README.md`, `STATUS.md`, `ROADMAP.md`, or a `docs/**` section is missing, stale, wrong, or no longer wanted | interviews you, then writes those four paths for one project through the landing sequence `metadoc.py check` → `meta.py check` → `metadoc.py commit`; absorbs the retired standalone meta-checking skill as that sequence's second step |
| `/aw-grill-meta-to-wis` | a future `docs/**` promise has no release Milestone, or its issue set or order is wrong | runs `wis.py gap`, then uses `milestone.py` and `change.py`; the Milestone title owns the version, its assigned issues own the work set, and its `## Development Order` owns the sequence; the promise heading gains `(Milestone #<number>)` |
| `/aw-e2e-for-wi` | the queue head is `type:feat`, `type:fix`, or `type:perf` | drives e2e for that one behavior issue only |
| `/aw-impl-for-wi` | the behavior queue head has landed e2e evidence | drives impl for that one behavior issue only |
| `/aw-maint-for-wi` | the queue head is `type:refactor`, `type:test`, `type:docs`, or `type:chore` | drives maint for that one maintenance issue only |
| `/aw-prepare-goal` | a project, typed issue, release Milestone, or bare intent must become decidable conditions | reads the project or tracker, selects the next route by type, and sets no goal unless the human explicitly asks |
| `/aw-ask-user` | the session has been deciding for you — stated assumptions, a route picked alone, `Open:` lines it read past | walks the context for every pending question, asks each through AskUserQuestion, and prints a decision table; writes nothing |
| `/project-readme-check` | after creating or editing an app or library README, STATUS, ROADMAP, protocol, generated-client, indexing, querying, GKE, client-integration, or migration guide | validates the adopted product-document set and cross-file references, then asks a context-free reader to restate the current, future, and interface contract |

The usual behavior sequence is `grill-me-to-meta` → `grill-meta-to-wis` →
`e2e-for-wi` → `impl-for-wi`. Maintenance uses `maint-for-wi`. These are
machine-checked handoffs. A release Milestone must bind to a product promise.
`milestone.py next` selects one queue head. `change.py fetch` records its type
and flow. Each phase commit is recorded by `change.py lifecycle`, and
`change.py close` advances the queue only after all required evidence matches
the commit. `prepare-goal` and `ask-user` stand outside the lifecycle.

Every `grill-*` skill enters Plan mode as step 1. It does not read, ask, or
write until the runtime confirms Plan mode. If the runtime cannot switch modes
from a skill, it stops and asks the human to enter Plan mode first.

- A grill never writes product source and never invents an answer you did not
  give. It offers only gates the repository already runs, and it stops asking
  once the body's own sections are answered.
- On `grill-meta-to-wis`, GitHub's native issue `milestone` field is the only
  ownership relation. Milestone creation uses the exact draft order line.
  Assignment requires an open Milestone and the same one `app:*` or `lib:*`
  label. The first final update must match all assigned issues. An
  `epic:<iid>` label is legacy data, not ownership.
- A delivery issue has exactly one type: `type:feat`, `type:fix`,
  `type:refactor`, `type:perf`, `type:test`, `type:docs`, or `type:chore`.
  Behavior types are `feat`, `fix`, and `perf`; they route through e2e then
  impl. Maintenance types are `refactor`, `test`, `docs`, and `chore`; they
  route through maint. `spike` and `report` are intake. `type:change` and all
  other legacy types are rejected.
- `grill-me-to-meta` writes only `README.md`, `STATUS.md`, `ROADMAP.md` and
  `docs/**` for the one project it names, one `## <title>` section per
  promise — never for an issue number, because a promise can predate the
  work item that will carry it. That allowlist is measured rather than asked
  for: `metadoc.py check` reads the dirty set against HEAD and refuses every
  path outside it, along with a section missing one of its own kind's
  bullets (a section is exactly one of `Outcome:`-shaped or
  `Status rows:`-shaped), a STATUS or ROADMAP id that resolves nowhere, and a
  heading that gained a `(Milestone #<number>)` — binding a section to a
  release Milestone is `grill-meta-to-wis`'s job, not this one's, and this check refuses a run
  that did it anyway. `metadoc.py commit` re-runs all of it, stages the
  allowlist, and writes the `Meta-Project:` / `Meta-Top:` / `Meta-Index:` /
  `Meta-Section:` / `Meta-Unbound:` trailers that make the commit findable; a
  META-doc commit written by hand carries none of them. A section is bound to
  its release Milestone — the heading gains `(Milestone #<number>)` and
  `Tracking:` gains the Milestone link — by `grill-meta-to-wis`, and by
  nothing else.
- `wis.py gap` prints seven rows, each with the size of what it read printed
  beside the count (`3 / 31`), not a bare count: G1 a future promise no release
  Milestone owns, G2 an open release Milestone or unmilestoned delivery issue no
  promise reaches, G3 a promise bound to a Milestone that cannot carry it,
  G4 a ROADMAP outcome no promise claims,
  G5 a STATUS row no promise claims, G6 an e2e case the crate manifest does
  not run, G7 a README gate that names no cargo target. A row it could not
  read — a missing `docs/` area file, an absent `STATUS.md`, `gh` exiting
  non-zero outside a git directory — prints `?` UNMEASURED with the reason
  instead of `0`, and is counted separately, so a run that could not reach
  the tracker never reports `=> ALIGNED`. It writes nothing; every write
  `grill-meta-to-wis` makes goes through `milestone.py` / `change.py`.
- Execution skills treat `#<iid>` as one typed issue. They treat only
  `milestone:<number>` or an exact `<project>@<version>` title as a Milestone.
  A bare number never means a Milestone. A Milestone run first calls
  `milestone.py next <ref> --json`. This command checks the complete assigned
  child set before it returns the first open row. Any structural error stops
  the run. The returned issue is the only executable queue head.
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

The delivery lifecycle has two flows. Behavior is `wi → e2e → impl → close`.
Maintenance is `wi → maint → close`. Each phase prints the command that follows
it. `e2e` runs four verbs — `start`, `verify`, `test`, `commit`; `impl` runs
five — `start`, `red`, `verify`, `test`, `commit`. The extra behavior verb
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

Maintenance does not invent a red. `maint.py start` freezes the type, baseline,
GHAN change points, and clean tree. A controller reads each declared command,
checks its paths, runs it outside `maint.py`, and records its exact exit and
output digest with `maint.py record`. Refactor records the same gates before
and after. Test, docs, and chore record their after gates. The commit carries
`Maint-Contract:` and `Maint-Change-Digest:`.

### Work-item terminal states

The closed type registry lives at `.claude/aw/scripts/wi_types.py`:

| Type | Terminal state |
|---|---|
| release Milestone | `milestone.py reconcile` is clean and all assigned delivery issues are terminal |
| `feat`, `fix`, `perf` | e2e then impl evidence is terminal |
| `refactor`, `test`, `docs`, `chore` | maint evidence is terminal |
| `spike` | an ADR-style decision records spawned WI refs or explicit no-action; expiry converges to `gave_up` |
| `report` | typed `triage` accepts and links a spawned change, or closes as `duplicate`, `invalid`, or `by-design` |

`workitem.py` keeps the legacy `epic` enum for read compatibility. New writes
must not create or update issue-based epics. Only typed delivery issues enter
the executable backlog. A `spike` never lands
investigation code in product source. A `report` remains in the intake queue
until triage, and both converge by spawn-and-link instead of changing type in
place.

## Artifact write order

Every behavior change to a project under `apps/<name>/` is authored one phase
at a time, red first. A phase does not start until its predecessor has landed
its commit.

| Phase | Writes | What lands there |
|---|---|---|
| 1 `e2e` | `apps/<name>/e2e/` | The black-box case, written to fail against the current tree |
| 2 `impl` | `apps/<name>/src/` | The skeleton, its colocated tests, and the implementation that turns them green; at least one test file is required |

Maintenance has one `maint` phase. Its type selects its write boundary:

| Type | Allowed change |
|---|---|
| `refactor` | product code whose public behavior is unchanged; the same named gates run before and after |
| `test` | test files or test-only sections; product behavior stays unchanged |
| `docs` | product documents or documentation-only comments; executable code stays unchanged |
| `chore` | only build, config, dependency, or tooling paths listed in the issue GHAN |

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
  `## <title>` section per promise; the heading gains its
  `(Milestone #<number>)` when `grill-meta-to-wis` binds the release. The
  technical-design tree that landed
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
