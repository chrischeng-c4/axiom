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

## `aw` is `.claude/aw` plus `.claude/skills/aw:*`

`aw` is the phase scripts under `.claude/aw/scripts/` and the eight skills under
`.claude/skills/aw:*/`. There is no CLI behind them — the protocol is their
stdout and their exit codes.

It was a Claude Code plugin at `plugins/aw/` until 2026-08-21. That tree is
deleted: the scripts moved to `.claude/aw/scripts/`, the verification suite to
`.claude/aw/verification/`, and `plugins/aw/skills/`, `plugins/aw/.claude-plugin/plugin.json`
and `.claude-plugin/marketplace.json` were removed outright, along with the
`enabledPlugins` entry in `.claude/settings.json`. The eight skills in
`.claude/skills/` were already the copy every session read, so nothing about
what loads changed — what changed is that there is now one copy of each file
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

Nine entry points. Each is invoked by a human. Lifecycle skills hand off to a
phase script that can refuse them; the project-document checker uses its own
read-only validators and a clean-context reader.

They load as project skills out of `.claude/skills/`, not as the `aw` plugin,
and that copy is what a session actually reads. Each directory is named
`aw:<skill>`, so the invocation keeps the namespace the plugin gave it and
every `/aw:…` in this repository still resolves. The prefix has to be in the
directory name because the loader keys off the path and ignores the frontmatter
`name:` — measured with a probe whose two names disagreed, and the directory
won. These eight are the only copy. A second set lived under
`plugins/aw/skills/` with `${CLAUDE_PLUGIN_ROOT}` paths where these write
`.claude/aw/scripts/`, nothing detected drift between them, and that pair was
collapsed on 2026-08-21 by deleting the plugin.

| Skill | Reach for it when | It does |
|---|---|---|
| `/aw:wi-change-grill` | a change must be opened, or its body is thin, stale, or unvalidatable | interviews you, then `change.py create\|update` writes the body |
| `/aw:wi-epic-grill` | the same, for an epic | interviews you, then `epic.py create\|update` |
| `/aw:wi-epic-reconcile` | before driving an epic, or whenever its child set is suspect | opens what the epic promises but never opened, and resolves duplicates and misfiled children |
| `/aw:wi-tdd` | the work item is ready to implement | drives one change — or an epic's children in dependency order — through the ladder below |
| `/aw:codex-e2e-review` | the `e2e` phase printed it as the next command | routes the case to the other model and binds its verdict to the reviewed bytes |
| `/aw:codex-code-review` | the `logic` phase printed it as the next command | the same, for the implementation and its colocated tests |
| `/aw:meta-check` | before trusting a `CLAUDE.md`, `README.md` or `CONTRIBUTING.md`, and after editing one | `meta.py check` reports every doc fact whose owner is gone |
| `/project-readme-check` | after creating or editing an app or library README, STATUS, ROADMAP, protocol, generated-client, indexing, querying, GKE, client-integration, or migration guide | validates the adopted product-document set and cross-file references, then asks a context-free reader to restate the current, future, and interface contract |
| `/aw:prepare-goal` | you want an intent as `/goal` conditions the session's evaluator can decide | reads the tracker through `epic.py`/`change.py`, or interviews you, and prints the conditions — it sets no goal itself |

The usual sequence is grill → reconcile (epics only) → `wi-tdd`; the two review
skills are reached from the phase that prints them, not chosen, and
`meta-check` and `prepare-goal` stand outside the lifecycle entirely.

- A grill never writes product source and never invents an answer you did not
  give. It offers only gates the repository already runs, and it stops asking
  once the body's own sections are answered.
- On reconcile, the label is what makes a child a child. An unlabelled issue is
  not one, whatever its body claims.
- `wi-tdd` on an epic reads the order out of the epic's own `Depends On`
  column. A line beginning `!` means there is no order to follow: report it and
  stop. An epic body you edited to make the graph parse is an epic whose
  dependencies you decided.
- `meta-check` reads and never writes. Its baseline was 103 findings over 182
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

`.claude/skills/agy-dispatch/` delegates one frozen, bounded task to an external
worker from a clean isolated worktree. Send only task-required repository
material, never secrets.

The controller keeps the issue contract, the design, the oracle, the independent
review, the tests, git integration, tracker mutation, and acceptance. A worker
does not commit, push, approve itself, comment on an issue, or close one.
Ticketed work reuses one worker conversation for that issue and its bounded
corrections; unticketed work is one-shot and cannot resume. Do not run workers
with overlapping write ownership in parallel.

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
