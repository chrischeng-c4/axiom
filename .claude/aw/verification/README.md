# Verification

Gates for the two ten-skill AW mirrors, the release Milestone contract, and
the remaining work-item schemas.

```
uv run --python 3.13 --no-project .claude/aw/verification/run_all.py                          # ~38s
uv run --python 3.13 --no-project .claude/aw/verification/run_all.py --with-negative-controls  # ~77s
```

One interpreter, the same launcher the skills use to run the scripts. The gates
do not need 3.11 themselves — they spawn the scripts under test through
`_paths.pinned_interpreter()` — but naming a second interpreter here is how a
gate suite ends up documented under one and only ever run under another, which
is the shape that let a `list[str] | None` at module scope sit in `_paths.py`
red under `python3` (3.9) and green under everything anyone actually typed.

The default runs the checkers — "is this tree admissible?", the question a
working session asks. The flag adds the negative controls, which answer a
different question: can each checker be seen to fail at all? That one is about
the gate rather than the tree, so its answer only changes when a gate changes,
and it is expensive by construction — a control mutates the thing under test
once per declared defect and re-runs the whole checker for each mutation.
`check_plugin_negative_control.py` plants five mirror and contract defects.
The other big number in the default run is cargo:
`check_tdd_flow.py` builds and tests a synthetic crate through both phases,
~29s of the ~38s there.

Run the full suite whenever a gate itself changes, and before reporting any
claim of the form "this is verified". The default mode is not allowed to sound
like the full one: it names the controls it did not run and prints
`CHECKERS GREEN`, never `ALL GREEN`, because the second string is the one that
gets pasted as evidence.

Nothing in this repository calls `run_all.py` — no CI workflow, no git hook, no
phase script. It is a ratchet a human runs, not a gate anything here enforces
on its own; a finding in a file you touched is still the signal, whether or not
anyone runs this suite over it.

Each gate resolves the checkout through `_paths.py`, which walks up to the
outermost `aw.toml` — the same rule the scripts use, so a gate and the script it
measures can never disagree about which tree is under test. `_paths.py` is also
the single place any of them spells a bundled location; a gate that recomputes
one is a second reading of a path, and the next time that path moves only one of
the two readings gets updated.

The shape under test:

```
apps/aw/src/aw/
  scripts/        milestone.py — release epic, version, ownership, and order
                  change.py — typed delivery facade; epic.py — legacy read facade
                  wi_types.py — closed type and flow registry
                  type_migration.py — manifest-led legacy cutover
                  workitem.py — shared issue engine
                  leg.py, and the two phases it is shared by: e2e.py, impl.py
                  maint.py — the maintenance phase and its four profiles
                  meta.py    — read-only META-doc check (M1..M7), not on the ladder
                  metadoc.py — the allowlisted README/STATUS/ROADMAP/docs write + commit
                  wis.py     — read-only work-item/promise gap reader (G1..G7)
.claude/aw/
  verification/   this directory
.claude/skills/
  aw-ask-user/                aw-build/                    aw-e2e-for/
  aw-grill-me-to-meta/        aw-grill-meta-to-milestone/  aw-grill-milestone-to-issue/
  aw-impl-for/                aw-prepare-goal/             aw-review/
  aw-test-for/
.agents/skills/
  the same ten SKILL.md files, byte-identical for Codex
```

This was a Claude Code plugin at `plugins/aw/` until 2026-08-21, which is why
several sections below are written about one. That tree is deleted — scripts
and this directory moved under `.claude/aw/`, the skills load as project
skills out of `.claude/skills/`, with byte-identical Codex mirrors under
`.agents/skills/`. `plugin.json`, `marketplace.json` and the
`enabledPlugins` entry in `.claude/settings.json` went with it. Two sections
are kept as measurement rather than as instruction, and each says so where it
starts: **Registration is the directory name** and **The installed copy is a
copy**. What they measured is a property of Claude Code, and the conclusion
that outlived the plugin is the one `check_plugin.py` still asserts — a
directory name registers as itself. Until 2026-08-26 that meant the `aw:`
namespace survived only because it was literally in each directory's name;
since the rename it means the directory and frontmatter name are both
`aw-<skill>`.

The thirteen scripts cannot be split across the ten skill directories, and that
is not a preference: `e2e.py` and `impl.py` each load `leg.py` by
`Path(__file__).parent / "leg.py"`, and `leg.change_module()` loads `change.py`
the same way. One directory is load-bearing.

`ec.py`, `td.py`, `cb.py`, their three gates, and the twelve `wi-{ec,td,cb}-*`
wrappers were **deleted**, not archived — the `ec → td → cb` ladder this
plugin replaced. `unit.py` and `logic.py` went the same way on 2026-08-27,
merged into the single `impl.py` above: in Rust a colocated test and the code
under it are the same tree and are edited together, so the boundary a second
commit used to draw between them cost an honest TDD loop more than it bought.
What the boundary used to buy — a named red measured before anything could
satisfy it — did not go with it; it moved onto `impl.py`'s `red` verb (see
"The ladder" below). An archive of instructions for scripts that no longer
exist is not history: it is a set of commands that fail with "no such file"
for a reader who cannot tell that from a broken checkout. What each retired
script was for survives where it is load-bearing — in the docstrings of the
phases that replaced them, and in `run_all.py`'s note on why rows left the
suite.

`docs/technical/` and the two `aw-grill-{change,epic}-to-td` skills that wrote
into it are deleted too, along with `prd.py`, renamed to `metadoc.py` and
widened from one write root (`docs/product/`) to four
(`README.md`/`STATUS.md`/`ROADMAP.md`/`docs/**`) in the same change. There is
no technical-design step: a design decision lives in the `//!` or `///` block
of the module or type that owns it
(`.claude/rules/authoring/source-carries-its-own-design.md`). The three grills
that used to open and shape work items — `grill-me-to-epic`, `grill-me-to-change`
and `grill-epic-to-changes` — folded first into a single `grill-meta-to-wis`,
which split again on 2026-09-02 into `grill-meta-to-milestone` (the
promise↔Milestone structure) and `grill-milestone-to-issue` (one Milestone's
typed issue set and order). Both run `wis.py gap <project>` for the seven
`G1..G7` rows of what a project's META-docs promise and its work-item set
disagree about, then close their half of the gap through
`milestone.py create|update` and `change.py create|update`.
`check-meta` folded into `grill-me-to-meta`'s three-step landing sequence
instead of surviving as its own skill.

The scripts sit beside the skills rather than inside one. They lived under
`skills/wi-epic-grill/scripts/` while that was the only skill running them,
which read as ownership it never had: reconcile already reached across into it,
and the change grill would have made two skills reaching into a third one's
directory for a file none of them owns.

## Gates

| Script | What it refuses |
|---|---|
| `check_next_command.py` | a phase that ends by printing the command that follows it, when the parser it names exits 2 on that line |
| `check_next_command_negative_control.py` | a cross-check that is green because it stopped finding the commands it compares |
| `check_plugin.py` | a missing or drifted ten-skill mirror, missing script, legacy issue-epic writer, or incomplete type and Milestone contract |
| `check_plugin_negative_control.py` | a mirror checker that misses a removed file, byte drift, restored issue-epic writer, missing queue-head rule, or a grill that skips Plan mode |
| `check_milestone.py` | a malformed SemVer-core title, wrong next-version bump, duplicate identity, malformed or lingering draft, ambiguous reference, incomplete pagination, wrong child type or project, unsafe assignment write, failed readback, or an order that does not equal native Milestone membership |
| `check_type_registry.py` | a missing, duplicate, unknown, intake, or legacy executable type; wrong flow; unsafe retype; or lifecycle close without matching commit evidence |
| `check_type_migration.py` | an incomplete or drifted manifest, wrong fixed mapping, partial replacement hidden as complete, unsafe readback, or resume without the same receipt |
| `check_maint_flow.py` | a maintenance profile that writes outside its type boundary, accepts incomplete gate evidence, executes issue text, or commits without its evidence trailers |
| `check_maint_flow_negative_control.py` | a maintenance gate that cannot see one of those boundary or evidence defects |
| `check_coverage_rule.py` | a requirement with no `## Verification Inventory` row — and a rule that reddens epics which were already green |
| `check_coverage_rule_negative_control.py` | a coverage gate that measures the population instead of the rule |
| `check_engine_split.py` | an engine that copies or branches on type literals instead of using `wi_types.py` |
| `check_engine_split_negative_control.py` | a split gate whose extractor reports "clean" because it found nothing |
| `check_change_schema.py` | a change schema that has narrowed, widened, or grown a rule nothing is ever seen refusing |
| `check_change_schema_negative_control.py` | a schema gate that stays green while one rule quietly stops firing |
| `check_epic_order.py` | an epic sequence that was guessed — a cycle answered with an arbitrary order, a child appended to the end because nothing placed it, or a declared dependency dropped because it could not be parsed |
| `probe_offtree_root.py` | a script that only resolves the repository when it happens to live inside one |
| `probe_local_verbs.py` | an `adopt` that overwrites, or an id parser that invents a number |
| `check_meta_flow.py` | a META-doc rule that fires on nothing, or on everything — a marker whose producer is gone, a command for a binary that is gone, a link to a file that is gone, a project README missing the section a reader goes there for |
| `check_meta_clean.py` | a META-doc in *this* checkout that has rotted — and a certification issued over a population that was never read |
| `check_meta_clean_negative_control.py` | a ratchet that reports zero because a rule stopped running |
| `check_metadoc_scope.py` | a `metadoc.py` run that wrote outside its four-path allowlist, or a rule that fires on everything or nothing |
| `check_tdd_flow.py` | an `e2e → impl` phase whose green is not attributable to a named red measured before it, or whose fetch receipt selects the wrong flow |

`check_manifests_cli.py` and its control are historical: they shelled out to
`claude plugin validate`, the only oracle here this repository did not own, and
were deleted with the plugin on 2026-08-21 along with `plugin.json` and
`marketplace.json`. What they measured — v2.1.227 passes a plugin named
`aw:epic` with exit 0 and warns only that the marketplace sync requires
kebab-case, a warning the exit code alone cannot see — is kept below under
"Five of these encode defects that actually shipped", the same way "Registration
is the directory name" and "The installed copy is a copy" are kept: as a
measurement of Claude Code, not as an active gate.

The last one carries its negative controls inside itself rather than in a
sibling file. Each of its rows *is* a declared mutation: the gate stages a
throwaway tree, breaks one thing in it, and requires the ladder to refuse for
the named reason — so `None` in `run_all.py`'s control column means "already
controlled", not "uncontrolled". `check_tdd_flow.py`'s fixture is a real cargo
crate, which is why it runs last and costs the most. `check_review_flow.py`,
which shared that shape cargo-free, was deleted with the review on 2026-08-26.

Five of these encode defects that actually shipped and were caught late:

- **`probe_offtree_root.py`.** `_repo_root()` walked up from `__file__`. A
  git-marketplace install puts the plugin under `~/.claude/plugins/`, where no
  `aw.toml` exists on any parent, so the script would have died on import with
  a message blaming the user's checkout. Installing from a local directory
  hides this completely, because there the plugin root *is* the checkout —
  which is why the probe stages a copy outside every checkout instead of
  trusting the local install.
- **`check_plugin.py`'s skill-reference assertions.** Claude Code names a
  plugin skill `plugin:directory` and ignores the frontmatter `name:` outright.
  With the directories named `aw-epic-grill`/`aw-epic-reconcile` under plugin
  `aw-epic`, the skills registered as `aw-epic:aw-epic-grill`, while every
  cross-reference the two bodies carried — including reconcile's handoff back
  to grill — pointed at a skill that did not exist. Nothing compared a body's
  invocation names against the directories that produce them.
- **`check_plugin.py`'s "survives registration unchanged" assertion.** A later
  attempt to reach a `/aw:wi:epic:grill` invocation named the directories
  `wi:epic:grill`. That is worse than an illegal name, because it is not
  refused: registration rewrites the colons and the skill loads as
  `aw:wi-epic-grill`. The plugin works, the paths lie, and every body reference
  written to the colon form points at nothing.
- **`check_manifests_cli.py`'s warning assertion.** Nothing in this repository
  knows Claude Code's naming rules well enough to have caught a name that is
  accepted locally and rejected by the marketplace sync. Asking the tool is the
  only reading of that rule that cannot drift from it.
- **`check_next_command.py`.** The ladder is driven by its own output — every
  verb ends by printing the command that follows it, and an agent runs that line
  verbatim. The lifecycle enum still read `("ec", "td", "cb")` one commit past the
  changeover that deleted those three scripts, and `change.py` takes its `--leg`
  choices from there, so all three phases ended `commit` by printing a line
  `change.py` exits 2 on: the step that records a landed commit on the work item
  was unreachable from every phase that is supposed to reach it. `leg.py` ended
  an accepted review — that path left with the review on 2026-08-26 — by
  printing `<phase>.py commit <wi>` without the `--project` all three require,
  which broke the review-to-commit handoff the same way. The eighteen gates in
  this directory were green over both, because
  each half reads as consistent on its own and the flow gates construct argv
  themselves rather than reading the line that gets printed. The first half of
  that is now refused earlier than any gate: `leg.py` asserts at import that its
  phase-keyed tables name the behavior phases `wi_types.FLOW_LEGS` declares,
  so the vocabulary cannot drift at all. What the gate still owns is the drift no table
  can see — a phase script's own `PHASE`, a flag the receiver requires, a script
  name that is not on disk.

## The ladder, and what makes a phase's green mean anything

`e2e → impl` replaces `e2e → unit → logic`, which replaced `ec → td → cb`. The
rule the ladder exists to enforce is unchanged across both collapses: a green
is only evidence when a **named** red was measured immediately before it, in
the same tree, by the phase that had reason to predict it. `check_tdd_flow.py`
is 26 declared controls against that rule and the type receipt, run against
`e2e.py` and `impl.py` directly.

Three-phase ladder had two commits to prove that with — the `unit` commit's
`Unit-Red:` trailer was the evidence `logic` read back. `impl` is one phase, so
there is no earlier commit inside it to read; the evidence moved to a
mid-phase record instead. `impl.py red` runs `[impl] build` then `[impl] test`,
writes the names that fail in the tree and did not fail at HEAD to
`.aw/impl-red/<iid>.json` together with the HEAD sha and a sha256 of every test
file it measured, and `C2` refuses `verify`/`test`/`commit` when the tree has
drifted from that record — a test file edited after `red` measured it, or HEAD
having moved. The record is scratch, gitignored, and cleared on `commit`; the
names travel from there onto the `Impl-Red:` trailer, which is where the
measurement becomes history.

Four things the ladder refuses that a simpler reading lets through:

- **A build failure is not a red.** `cargo test` exits non-zero both when
  nothing compiles and when an assertion fails, so a project's `aw.toml`
  declares `[impl] build` and `[impl] test` as separate commands and requires
  the first to pass before the second can count.
- **An exit code is not a measurement.** A selector matching nothing exits 0
  with 0 tests. `red` records failing test *names*, and set-subtracts what was
  already failing at HEAD, so a pre-existing red cannot be claimed as the one
  this phase produced — and an implementation written before `red` ever ran
  leaves nothing failing to record, which `red` refuses rather than reporting
  an empty red.
- **A deleted test and a passing test look alike.** `test` requires each
  recorded name to be *present* as well as passing, and separately requires the
  rest of the suite to be whole — nothing else newly red, and nothing else
  silently unwired.
- **A row that vanished reads like a row that passed.** Every unrun row prints
  `PENDING`.

The old `unit`/`logic` **boundary** is gone along with the second commit it
needed to draw a line across. What survives is an **existence** rule drawn by
the same means — filename, not a `#[cfg(test)]` span reader: colocated tests
live in `src/**/tests.rs`, and `C0` refuses an `impl` phase whose dirty set
touches none of them, because a phase that wrote no test file wrote nothing for
`red` to measure. Parsing `#[cfg(test)]` spans instead has been got wrong here
twice — item-level `#[cfg(test)] fn` and `use` read as production, and a brace
scanner that does not strip `r#"…"#` counts fixture text as code. A filename
cannot be got wrong that way, and what the retrofit the boundary used to refuse
outright — editing a test into agreement with the implementation after the
fact — now goes through `C2`'s byte comparison instead.

Evidence lives in commit trailers, not only in the scratch record: `e2e`
writes `E2E-Red` and `E2E-Change-Digest`; `impl` writes `Impl-Red`,
`Impl-Contract` and `Impl-Change-Digest`. HEAD comparisons use
`git worktree add --detach`, never a stash — a stash mutates the tree it is
supposed to be measuring against.

Maintenance uses a separate one-phase gate. It does not manufacture a red.
`maint.py start` freezes a clean baseline, the typed fetch receipt, and the
GHAN change points. The controller reviews and runs each accepted gate outside
the script. `maint.py record` stores only the exact command, exit code, and
output digest. Refactor requires the same gates before and after. Test, docs,
and chore require after evidence. Scope is checked independently for all four
profiles. The commit carries `Maint-Type:`, `Maint-Base:`, `Maint-Gates:`,
`Maint-Contract:`, and `Maint-Change-Digest:`. `check_maint_flow.py` and its
negative control hold these rules.

## Where a release order comes from

One GitHub Milestone is one versioned epic. Its title is
`<project>@<major>.<minor>.<patch>`. The three SemVer core fields are
non-negative integers without leading zeroes. GitHub's native issue
`milestone` field is the only membership relation.

`milestone.py next-version <project>` reads all open and closed release
Milestones for the project. It increments minor and resets patch to zero by
default. Major and patch bumps are explicit overrides. An exact version is
also a human choice. If no prior release Milestone exists, the command refuses
to guess the initial version. This Milestone planning rule does not change a
project's build or release version rules.

The Milestone description has exactly three H2 sections: `## Goal`,
`## Development Order`, and `## Acceptance`. The order section contains only
contiguous rows such as `1. #2403`. Each assigned issue appears exactly once.
Every assigned delivery issue must carry exactly one of `type:feat`,
`type:fix`, `type:refactor`, `type:perf`, `type:test`, `type:docs`, or
`type:chore`, and the Milestone project's one `app:*` or `lib:*` label.
`type:spike` and `type:report` are intake. `type:change` and every other legacy
type are rejected. Creation uses `--draft` and the skeleton's exact draft line.
A draft cannot survive after the first issue is assigned. The first non-draft
update must equal native membership.

`milestone.py order` compares the numbered list with native membership.
`milestone.py next` runs that full comparison before it emits the first open
row, its type, flow, and next phase. It refuses missing, extra, duplicate,
wrong-type, and wrong-project issues. Execution skills must process only that
queue head. It also refuses a bare number because issue and
Milestone numbers are separate namespaces. `change.py` assigns only to an open
release Milestone, checks the project label before writing, and reads the issue
back after the write. `wis.py` reads the complete paginated label population;
it does not stop at 200 issues. `check_milestone.py` holds these rules as pure
cases and isolated writer fakes.

`epic.py order` remains only for reading and measuring the legacy issue-epic
population. New skills do not call it. Its historical graph checks remain in
the suite until the legacy records are migrated or retired.

## How the legacy type cutover stays resumable

The versioned manifest freezes every open legacy issue before a write. Each
row names the issue, source and target type, classification reason, evidence,
complete labels digest, live `updatedAt`, Milestone number, and state.
`type:bug` maps only to `type:fix`. `type:enhancement` maps only to
`type:feat`. Each `type:change` row is classified from its body, Milestone, and
product promise.

`type_migration.py` verifies the complete live legacy cohort and every frozen
field before its first write. Apply uses one complete-label replacement per
issue. It reads back title, body, state, Milestone, target type, and every
non-type label. The receipt starts as `INCOMPLETE` and is updated after each
readback. A retry uses only `--resume <receipt>`. It never starts a new batch
over partial state. `check_type_migration.py` injects a mid-batch failure and
proves that the same receipt resumes to `COMPLETE`.

The cutover order is fixed:

1. Prepare the scripts, ten mirrored skills, versioned manifest, and green
   full verification suite in one reviewed change. Do not publish it yet.
2. Create every missing canonical GitHub label, then verify that all seven
   delivery labels and both intake labels exist.
3. Stop every AW session that can write tracker state. This is an operator
   pause. A local file cannot lock writers on another host.
4. Land the strict legacy-type refusals before changing any live issue label.
5. Run one apply with a durable receipt path:

   ```text
   uv run --python 3.13 --no-project "apps/aw/src/aw/scripts/type_migration.py" --repo chrischeng-c4/axiom --manifest .claude/aw/migrations/open-legacy-types-2026-08-31.json --apply --receipt <durable-receipt.json>
   ```

6. If it stops, keep the pause and run only
   `type_migration.py --resume <durable-receipt.json>`. Never start a second
   apply over partial state.
7. Release the pause only after the receipt is `COMPLETE`, live readback finds
   no open legacy type, and every migrated issue still has its frozen non-type
   labels, Milestone, title, body, and state.

## What the META-docs are measured against

`meta.py check` reads every tracked `CLAUDE.md`, `README.md` and
`CONTRIBUTING.md` — **182** of them — and asks of each fact whether the thing
that owns it still exists. Four rules, and each resolves against the filesystem
rather than against a judgement: `M1` a generator marker whose producer is gone,
`M2` a command naming the deleted CLI, `M3` a relative link whose target is not
in the checkout, `M4` a project README missing `## Brief` or `## Capabilities`.
The population is `git ls-files`, so it is the checkout you are standing in and
never a scratch file under an ignored build directory.

The first run reported **103 findings across 80 files** — `M1` 66, `M2` 31,
`M3` 6, `M4` 0 — over 182 documents and the 62 directories that hold both a
`README.md` and a `CONTRIBUTING.md`. That is **0** now, and the difference
between those two states is why there are two gates rather than one.

`check_meta_flow.py` measures the **detector** and deliberately pins none of
those numbers: they were meant to fall, and a gate that pins them goes red on
the fix. What it pins is that each rule fires on its own defect and on nothing
else, in a `tempfile` git repository holding one rotten project, one clean one
as the shared negative control, and one untracked directory that must not be
scanned at all.

`check_meta_clean.py` measures the **tree**, and it exists only because the
count reached zero. A ratchet authored while its subject is red is a ratchet
with a tolerated-failure list, and a tolerated-failure list is where a genuine
regression goes to hide; this one has none and the tolerated set stays empty.
Its own false-green modes are the two that a zero cannot distinguish itself
from. A population that collapsed — `git ls-files` returning nothing prints
`=> CLEAN` and exits 0 having read no document at all — is refused by
cross-checking the reported count against a second listing and against a floor.
A rule that stopped running is refused by proving, in the same invocation and
before the tree is certified, that all four still reach a throwaway repository
carrying one defect apiece. Its sibling control then rots two real `apps/cube`
documents, one rule at a time, and requires exactly that rule's row plus the
exit-code row to go red, restoring each by sha256.

`M1`'s population is the whole rule. `PRODUCERS` is empty — not as a stub, but
because the verb that wrote those blocks was deleted with the crate that carried
it — so all 66 marker pairs were orphaned *by derivation*. The gate declares a
producer into that table and requires `M1` to stop firing for that name, which
is the only way to tell "no producer exists" apart from "the lookup is broken".
`M4`'s live population was 0 even before the cleanup, so its discrimination
comes entirely from the fixture; a rule measured only against a repository that
does not exhibit it is a rule nobody has watched fire.

Three of the four rules were wrong when first run, and each was caught by
running it rather than by reading it. All three are `M3`- and `M2`-shaped —
a detector that reads syntax cannot tell a command from a sentence about one:

- **`M3` reported markdown links that were Python.** `_b = _Box[int](42)` in a
  fenced example matches a `[...](...)`  link exactly. The fix is a fence walker
  the rules share, and `M3` fell from 8 to 6 — every one of the two removed was
  a code sample.
- **`M2` was blind to the most copy-and-run shape there is.** The detector was
  anchored on backticks, which cannot see a ```` ```bash ```` block whose lines
  begin bare. Measuring `^\s*aw [a-z]` across all 182 documents found exactly
  two such lines, both fenced, both in `apps/jet/README.md`. The bare form is
  now matched *only* inside a fence — outside one, a line starting "aw " is
  English far more often than it is a command — and `M2` rose from 30 to 32.
  The 30 it had before are the cross-check: they plus the 4 exempted lines are
  exactly the 34 an independent hand count reached, so the scope did not shift
  while the detector was widened. The 2 the widening added are beyond anything
  that hand count could have reached, because it was grepping for backticks
  too.
- **A `--path` that matched nothing reported clean.** A mistyped path that
  certifies the whole repository is worse than no run, so an empty selection is
  now a usage error. That is also why exit `2` exists at all: before it, "three
  files have rotted" and "you passed a rule that does not exist" were the same
  exit code.

Writing the two paragraphs above then produced a fourth. An *inline* code span
is the same exemption a fence is — no renderer turns `` `[a](b)` `` into a link
— and `M3` did not know that, so quoting its own false positives made it report
them again, in this file. The live count did not move, which is the point worth
recording: the only document in the repository that exercises the case is the
one describing the case, so the three fixture rows are the entire measurement.
They are one link written three times — spanned, spanned in a longer backtick
run, and unspanned beside a span — because asserting only the suppression would
also pass if the rule had learned to blank the whole line.

The gate then caught a fifth defect in the script itself. `DEAD_COMMAND_EXEMPT`
audits itself — an exemption that stops matching its line is reported, the same
discipline `UNUSED` and `ADOPT_WHY` carry in `check_plugin.py` — but those
fragments quote sentences in *this* checkout, so running that audit against the
fixture produced four findings about the script's own data. It is now skipped
when the scanned repository is not the one shipping the script, and a gate row
asserts the skip.

One assertion reaches across files: `meta.py` and `check_plugin.py` must share
one `aw`-invocation detector, compared as source text. Two regexes for one
question drift, and the drift is invisible — each file stays green against its
own reading of what a dead command looks like.

## The engine/facade split

`workitem.py` is the shared issue engine. `change.py` binds the seven delivery
types to the GHAN schema. `epic.py` keeps the retired issue-epic schema readable
but refuses every write. `milestone.py` is the active release facade; it owns
version identity, native membership, development order, and the queue head.

`check_engine_split.py` keeps the shared engine below those facades. Type and
flow vocabulary comes from `wi_types.py`; the engine may consume that registry,
but it may not copy delivery, intake, or legacy literals into a second private
registry. Docstrings and comments are excluded on purpose. Explaining a legacy
label is documentation; branching on a copied label is behavior.

The extraction was accepted by pinning the acceptance before writing it: every
gate's output byte-identical before and after. Seven of the eight were, and the
eighth differed in exactly two lines — `check_coverage_rule_negative_control.py`
prints the sha256 of `epic.py`, which cannot survive changing `epic.py`. That
difference was checked rather than normalized away: both printed digests had to
equal each other and equal the file's real digest.

### What each facade must expose, and what it may leave unused

`check_plugin.py` holds a required-verb set per script and resolves every verb
a skill names against the real script, so a documented verb that no longer
exists is refused rather than discovered at use. The active sets differ on
purpose. A release Milestone exposes `children`, `order`, `next`, `reconcile`,
and release closure. A delivery issue exposes typed body and lifecycle verbs.
The legacy epic parser still exposes its old command names for compatibility,
but `epic.py` refuses `create`, `update`, and `close` before tracker access.

The interesting half is the gap in the other direction — a verb the script
exposes that no skill drives. Left silent, that is how a verb rots: nothing
documents it, nothing runs it, and it stays in the file looking supported. Any
such recovery verb therefore needs a focused probe even when it is absent from
the normal skill path.

`adopt` remains a recovery verb for a staged body whose issue number arrived
outside `change.py create`. No skill uses that path in the normal flow.
`probe_local_verbs.py` still exercises it, so the recovery behavior cannot rot
silently.

## Who opens a delivery issue

`aw-grill-milestone-to-issue` settles the complete issue set, each issue type,
and the global order with the human in Plan mode. It then uses `milestone.py`
and `change.py`; no SKILL.md may name a direct
`gh issue|pr create|edit|close|comment|delete|reopen` command. The positive
control for that detector is the former hand-written `gh issue create` block,
so the assertion stays pinned to a defect that actually existed.

The defect it refuses is not only a shortcut. A hand-opened issue can have a
real number and plausible labels while carrying a body no validator has seen.
Routing creation through `change.py create --type <delivery-type>` binds the
canonical type and validates the one GHAN schema. Native Milestone assignment
then supplies the only parent relation. The final `milestone.py reconcile`
compares the complete assigned set with `## Development Order` instead of
trusting creation order or a filtered child list.

## The change schema was ported, and now it is owned

The two historical issue schemas arrived by different routes. The retired epic
schema was this plugin's own invention, so `epic.py` still holds it as
declarative `Section` data for read compatibility. It no longer authorizes an
executable flow. The change schema was not ours: it was written in
`apps/agentic-workflow/src/issues/ghan.rs`, `aw wi validate` enforced it, and
640 live work items were judged by it. While that crate existed, `change.py` was
a **port**, and every gate on it read the crate as the oracle through three
channels — constants extracted from `ghan.rs`, the empty template extracted from
`issues.rs`, and a replay of all 18 `#[test]` functions.

The crate is gone. There is no upstream left, so the port is the original and
`change.py` owns the schema outright. Deleting an oracle costs discrimination
unless something replaces it, and what the crate actually caught was *narrowing*
— a word list re-typed one entry shorter, invisible because both sides get
edited together. Three replacements carry that load, and each catches something
a crate channel did not:

| replacement | what it is | catches |
|---|---|---|
| declared inventory | the four H2s, six H3s, 15 hedges and 7 failure assertions written as literals in `check_change_schema.py` and compared to `change.py` | narrowing **and** widening: the gate states the schema, so a diff has to go through it either way |
| liveness | every declared entry is then used — each hedge refuses a premise, each failure assertion is accepted, each heading is load-bearing | an entry that sits in both lists doing nothing. Two identical dead lists agree perfectly, so the crate oracle never checked this |
| refusal coverage | every `errors.append` site in the five `validate_*` functions must be reached by some case, found by AST walk and measured with `sys.settrace` | a rule that exists but is never observed firing. This replaces "every crate test has a replay": coverage of the rules that exist, not of a test list in another language |

The inventory and the liveness probe travel together and neither is sufficient.
A probe loop derived from the list cannot see the list shrink — delete a hedge
and its probe deletes itself — which is the exact failure being defended
against. The declared literals are what make the loop honest, and the
`narrowed-vocab` mutation in the negative control is what proves the loop reads
them: it must red **twice**, once for the inventory and once for that word's
probe. Generate the loop from `change.py` instead and it reds once and looks
fine.

Refusal coverage was not decoration on arrival. Its first run reported 19 of 24
sites reached; six cases were added to close the gap, each naming a rule
`validate_*` had always enforced and nothing had ever observed firing.

The fixtures under `_fixtures/` — the sample body and the 987B skeleton — were
lifted out of the crate before it was removed, at a point where crate and port
were measured to agree on all five channels. They are the plugin's own now.

The Rust-vs-Python near misses stay written as Rust primitives even though there
is no longer a Rust side to match, because they are the behaviour 640 live
bodies were judged by. `str::lines` splits on `\n` alone, while `splitlines`
also breaks on `\v`, `\f`, `\x1c`-`\x1e`, `\x85`, U+2028 and U+2029 — so a body
containing one of them would be read as having more lines, and a line is what a
section boundary is made of. `to_ascii_lowercase` touches `A-Z` and nothing
else, while `.lower()` folds U+212A KELVIN SIGN to `k` and U+0130 to `i` plus a
combining dot — either of which could make a hedge word match where the original
found none. `_lines`, `_ascii_lower` and `_split_on` each hold that behaviour on
exactly those inputs. Changing any of them now is a schema change, not a
cleanup: it re-judges the existing population.

### What the live differential reached, and what it never did

`measure_change_agreement.py` compiled the crate's own rules and ran them
against the port over every live change body: 640 bodies, 6,280 error strings,
zero divergence. It died with the crate, but its finding outlives it and is the
reason the cases in `check_change_schema.py` are the whole of the per-section
evidence.

Validation short-circuits. A body missing an H2 or carrying an unexpected one is
refused structurally and never reaches `validate_goal` and its three siblings.
Live, that was 619 of 640. The 21 that got through passed every per-section
rule, so **the live population exercises the per-section rules almost entirely
on their non-firing path** and contributed zero per-section error strings. That
five-figure agreement was breadth, never depth. The gate's cases are the only
place those rules are observed to fire at all — which was equally true while the
crate existed; the crate was never what made it true.

## What decides the invocation

Not a gate — it is a property of the tool, so it is measured and recorded here
rather than asserted against this machine's install.

**The oracle is a live session, not `claude plugin details`.** That subcommand
enumerates the skills *directory* and prints the raw directory names back, so
it reports names that registration will not produce. It displayed
`wi:epic:grill` for a skill that loads as `aw:wi-epic-grill`, and a
`/reload-plugins` reporting `0 skills` alongside it is what exposed the
disagreement. The reading that cannot drift is to load the plugin into a
session and ask it what it has:

```
claude --plugin-dir "$PWD/plugins/aw" -p "list every skill whose name contains epic"
  -> aw:wi-epic-grill
     aw:wi-epic-reconcile
```

That command is not runnable now — `plugins/aw/` was deleted on 2026-08-21 —
and it is kept verbatim because it is the transcript of the measurement, not an
instruction. The finding it produced is what `check_plugin.py` asserts.

Two separate probes, each staged as one throwaway plugin with a matching
control so a null result could be told apart from a broken probe (v2.1.227):

| plugin | directory | frontmatter `name:` | registered as | isolates |
|---|---|---|---|---|
| `regprobe` | `probe-hyphen` | `probe-hyphen` | `probe-hyphen` | control |
| `regprobe` | `probe:colon` | `probe:colon` | **`probe-colon`** | the colon |
| `fmprobe` | `zeta-match` | `zeta-match` | `fmprobe:zeta-match` | control |
| `fmprobe` | `zeta-mismatch` | `zeta-other` | **`fmprobe:zeta-mismatch`** | the frontmatter |
| `fmprobe` | `zeta-nofm` | *(absent)* | `fmprobe:zeta-nofm` | its absence |

Two rules follow, and both are load-bearing:

- **A colon in a directory name is rewritten to a hyphen at registration.** It
  is not refused — the plugin loads, and every body reference written to the
  colon form points at nothing. So exactly one colon is reachable in an
  invocation: the `plugin:skill` separator Claude Code puts there itself.
  `/aw:wi:epic:grill` is not a name that can exist; `/aw:wi-epic-grill` is what
  that tree produces. An axis therefore lives in the *leaf*, hyphen-separated —
  `wi-epic-grill`, `wi-change-grill` — and the plugin name is the only segment
  before the colon. Without a plugin there is no separator at all: a project
  skill's command is its bare directory name, which is what made the
  2026-08-26 rename a naming decision — directories `aw-<skill>` give the
  command `/aw-<skill>`.
- **The frontmatter `name:` is inert.** `zeta-mismatch` declared `zeta-other`
  and registered under its directory anyway. The field cannot change the
  invocation, so its only remaining job is to be the label the skill list
  shows without lying about the command — which is precisely what it did when
  this shipped broken. The Codex skill schema also requires a hyphen-case
  frontmatter name. `check_plugin.py` therefore pins the field to the same
  `aw-<skill>` string as the directory in both runtime mirrors.

The registration probe is deliberately not a gate: it costs an API call per
run, and what it measures is a property of Claude Code rather than of this
tree. `check_plugin.py` carries its conclusion instead, as a directory-name
assertion with a positive control that refuses a colon.

## The installed copy is a copy

Historical, as of 2026-08-21: there is no installed copy, because there is no
plugin. Kept because it records *why* the plugin was worth deleting — an edit
here reached a session only after an uninstall/install cycle, and nothing
detected the gap. The skills in `.claude/skills/` are read from the
checkout directly, so this failure mode is gone rather than mitigated.

`plugin install` copied the plugin into
`~/.claude/plugins/cache/<marketplace>/<plugin>/<version>/`, and
`installed_plugins.json` points `installPath` at that copy. Nothing in a
session reads the checkout.

`plugin update` does **not** refresh it. Measured: with the checkout renamed
and the cache still holding the pre-rename directories, `claude plugin update
aw@axiom --scope project` reported *"aw is already at the latest version
(0.1.0)"* and copied nothing — it compares the version string, and a local-path
marketplace edit does not move that string. `marketplace update` does not
refresh it either.

So an edit here reaches a session only after `plugin uninstall` +
`plugin install` (or a version bump). Confirm with a diff rather than the
install's own output:

```
diff -r ~/.claude/plugins/cache/axiom/aw/0.1.0/ plugins/aw/    # silent == in sync
```

This is what a `/reload-plugins` reporting `0 skills` looked like from the
outside: the checkout was correct, the gates were green, and the loaded copy
was three renames behind.

## Measurement

These legacy issue-epic measurements hit the tracker and produce evidence, not
a verdict. They do not define the new Milestone flow. Run them only when
auditing the legacy population:

| Script | Question |
|---|---|
| `measure_population.py` | what do the two coupled sections contain across every live epic? |
| `measure_spelling_tail.py` | which first-column spellings would a naive rule refuse? |
| `measure_blast_radius.py` | how many currently-green epics would each candidate reading turn red? |

There was a fourth, `measure_change_agreement.py`, which compiled the crate's
own rules and ran them against `change.py` over all 640 live change bodies. It
was deleted with the crate — there is nothing left to differ from. What it
measured is recorded above under "What the live differential reached", because
the finding is what justifies the current gate's shape and would otherwise be
lost with the tool.

`measure_population.py` writes `_snapshots/`, which is gitignored: it is live
tracker state, and a committed copy would let the regression assertion in
`check_coverage_rule.py` pass against a population that no longer exists. When
the snapshot is absent that assertion fails and says so rather than skipping,
because a silent skip turns the strongest gate here into a no-op that still
prints green.

The blast-radius measurement is why `_requirement_refs` expands ranges and
lists at all: the bare-equality reading turned 8 of 54 valid epics red, every
one of them on spelling rather than on missing coverage.
