# Verification

Gates for the `aw` plugin and the two work-item schemas its scripts enforce.

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
`check_plugin_negative_control.py` is eleven such rounds, ~23s of the ~39s the
flag adds. The other big number is in the default run and is cargo:
`check_tdd_flow.py` builds and tests a synthetic crate through all three phases,
~29s of the ~38s there.

Run the full suite whenever a gate itself changes, and before reporting any
claim of the form "this is verified". The default mode is not allowed to sound
like the full one: it names the controls it did not run and prints
`CHECKERS GREEN`, never `ALL GREEN`, because the second string is the one that
gets pasted as evidence.

Each gate resolves the checkout through `_paths.py`, which walks up to the
outermost `aw.toml` — the same rule the scripts use, so a gate and the script it
measures can never disagree about which tree is under test. `_paths.py` is also
the single place any of them spells a bundled location; a gate that recomputes
one is a second reading of a path, and the next time that path moves only one of
the two readings gets updated.

The shape under test:

```
.claude/aw/
  scripts/        epic.py, change.py — the type-bound facades — and workitem.py, the engine
                  leg.py, and the three phases it is shared by: e2e.py, unit.py, logic.py
  verification/   this directory
.claude/skills/
  aw-check-meta/           aw-go-tdd-for-change/   aw-go-tdd-for-epic/
  aw-grill-change-to-prd/  aw-grill-change-to-td/  aw-grill-epic-to-changes/
  aw-grill-epic-to-prd/    aw-grill-epic-to-td/    aw-grill-me-to-change/
  aw-grill-me-to-epic/     aw-prepare-goal/
```

This was a Claude Code plugin at `plugins/aw/` until 2026-08-21, which is why
several sections below are written about one. That tree is deleted — scripts
and this directory moved under `.claude/aw/`, the skills load as project
skills out of `.claude/skills/`, and `plugin.json`, `marketplace.json` and the
`enabledPlugins` entry in `.claude/settings.json` went with it. Two sections
are kept as measurement rather than as instruction, and each says so where it
starts: **Registration is the directory name** and **The installed copy is a
copy**. What they measured is a property of Claude Code, and the conclusion
that outlived the plugin is the one `check_plugin.py` still asserts — a
directory name registers as itself. Until 2026-08-26 that meant the `aw:`
namespace survived only because it was literally in each directory's name;
since the rename it means the directory `aw-<skill>` *is* the command
`/aw-<skill>`, and the frontmatter `name: aw:<skill>` is only the label the
skill list shows.

The eight scripts cannot be split across the eleven skill directories, and that
is not a preference: `e2e.py`, `unit.py` and `logic.py` each load `leg.py` by
`Path(__file__).parent / "leg.py"`, and `leg.change_module()` loads `change.py`
the same way. One directory is load-bearing.

`ec.py`, `td.py`, `cb.py`, their three gates, and the twelve `wi-{ec,td,cb}-*`
wrappers were **deleted**, not archived. An archive of instructions for scripts
that no longer exist is not history: it is a set of commands that fail with "no
such file" for a reader who cannot tell that from a broken checkout. What each
one was for survives where it is load-bearing — in the docstrings of the phases
that replaced them, and in `run_all.py`'s note on why three rows left the suite.

The scripts sit beside the skills rather than inside one. They lived under
`skills/wi-epic-grill/scripts/` while that was the only skill running them,
which read as ownership it never had: reconcile already reached across into it,
and the change grill would have made two skills reaching into a third one's
directory for a file none of them owns.

## Gates

| Script | What it refuses |
|---|---|
| `check_manifests_cli.py` | a manifest Claude Code's own validator rejects — or merely warns about |
| `check_manifests_cli_negative_control.py` | a manifest gate that reads only the exit code |
| `check_next_command.py` | a phase that ends by printing the command that follows it, when the parser it names exits 2 on that line |
| `check_next_command_negative_control.py` | a cross-check that is green because it stopped finding the commands it compares |
| `check_plugin.py` | a manifest, bundled path, or skill cross-reference that does not resolve — a skill that has grown its own copy of a shared script, or one that reaches past its facade to `aw` or `gh` |
| `check_plugin_negative_control.py` | a checker that cannot be seen to fail |
| `check_coverage_rule.py` | a requirement with no `## Verification Inventory` row — and a rule that reddens epics which were already green |
| `check_coverage_rule_negative_control.py` | a coverage gate that measures the population instead of the rule |
| `check_engine_split.py` | an engine that has learned which work-item type it is serving |
| `check_engine_split_negative_control.py` | a split gate whose extractor reports "clean" because it found nothing |
| `check_change_schema.py` | a change schema that has narrowed, widened, or grown a rule nothing is ever seen refusing |
| `check_change_schema_negative_control.py` | a schema gate that stays green while one rule quietly stops firing |
| `check_epic_order.py` | an epic sequence that was guessed — a cycle answered with an arbitrary order, a child appended to the end because nothing placed it, or a declared dependency dropped because it could not be parsed |
| `probe_offtree_root.py` | a script that only resolves the repository when it happens to live inside one |
| `probe_local_verbs.py` | an `adopt` that overwrites, or an id parser that invents a number |
| `check_meta_flow.py` | a META-doc rule that fires on nothing, or on everything — a marker whose producer is gone, a command for a binary that is gone, a link to a file that is gone, a project README missing the section a reader goes there for |
| `check_meta_clean.py` | a META-doc in *this* checkout that has rotted — and a certification issued over a population that was never read |
| `check_meta_clean_negative_control.py` | a ratchet that reports zero because a rule stopped running |
| `check_tdd_flow.py` | an `e2e → unit → logic` phase whose green is not attributable to a red the phase before it named |

`check_manifests_cli.py` is the only gate here whose oracle this repository does
not own: it shells out to `claude plugin validate`, so it stays correct when
Claude Code's schema moves without telling us. Its warning assertion is the
load-bearing one — measured against v2.1.227, a plugin named `aw:epic` **passes**
validation with exit 0 and warns only that the Claude.ai marketplace sync
requires kebab-case. The negative control prints that exit code under the
mutation, so "the exit code cannot see this" is a number in the output rather
than a claim in a comment.

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
  verbatim. `workitem.LEGS` still read `("ec", "td", "cb")` one commit past the
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
  two phase-keyed tables name the phases `workitem.LEGS` declares, so the
  vocabulary cannot drift at all. What the gate still owns is the drift no table
  can see — a phase script's own `PHASE`, a flag the receiver requires, a script
  name that is not on disk.

## The ladder, and what makes a phase's green mean anything

`e2e → unit → logic` replaces `ec → td → cb`. The rule the three phases exist to
enforce is that a green is only evidence when a **named** red was measured
immediately before it, in the same tree, by the phase that had reason to
predict it. `check_tdd_flow.py` is 22 declared mutations against that rule.

Four things it refuses that a simpler reading lets through:

- **A build failure is not a red.** `cargo test` exits non-zero both when
  nothing compiles and when an assertion fails, so `unit` declares `[unit]
  build` and `[unit] test` as separate commands and requires the first to pass
  before the second can count.
- **An exit code is not a measurement.** A selector matching nothing exits 0
  with 0 tests. `unit` records failing test *names*, and set-subtracts what was
  already failing at HEAD, so a pre-existing red cannot be claimed as the one
  this phase produced.
- **A deleted test and a passing test look alike.** `logic` requires each
  recorded test to be *present* as well as passing, and separately requires the
  rest of the suite to be whole — nothing else newly red, and nothing else
  silently unwired.
- **A row that vanished reads like a row that passed.** Every unrun row prints
  `PENDING`.

The `unit`/`logic` boundary is drawn by **filename**: colocated tests live in
`src/**/tests.rs`, and the `todo!()` skeleton `unit` writes goes outside it so
`logic` is free to replace it. Drawing the line by parsing `#[cfg(test)]` spans
instead has been got wrong here twice — item-level `#[cfg(test)] fn` and `use`
read as production, and a brace scanner that does not strip `r#"…"#` counts
fixture text as code. A filename cannot be got wrong that way.

Evidence lives in commit trailers, not a state file: `E2E-Red`, `Unit-Red`,
`Logic-Contract`, each beside a `*-Change-Digest`. HEAD comparisons use
`git worktree add --detach`, never a stash — a stash mutates the tree it is
supposed to be measuring against.

## Where an epic's order comes from

`epic.py order` composes two sections that were already in every epic body and
had no consumer: `## Verification Inventory` partially orders the requirements
through its `Depends On` column, and `## Child Work Items` maps each child to
the requirements it covers. A child inherits the position of the *deepest*
requirement it covers — taking the shallowest would place it before work it
needs — and equal positions break by `priority:` then by issue number.

Measured over the 255-epic snapshot: 45 epics carry the `Depends On` column and
14 do not, so its absence is a shape rather than a defect; 32 have at least one
real `R → R` edge; and there are **0 cycles and 0 dangling references** today.

A baseline of zero is also what a detector that never fires reports, so
`check_epic_order.py` seeds one cycle and one dangling reference into a copy of
the corpus and requires both counts to move to one. The row that carries real
weight is the fourth finding: **10 epics fill a `Depends On` with an issue
number (`#2403`) or with prose (`shared harness WI`)**, and on 4 of them there
are real edges alongside — so reading those cells as "no dependency" produced
an order that looked complete and had silently lost a constraint. They are
reported as `unreadable-dependency` rather than interpreted, because reading
`#2403` as an edge means guessing whether the author meant that issue or the
requirement it covers.

The order refuses rather than guesses. A cycle yields *no* order at all, not a
declaration-ordered fallback — a fallback would be indistinguishable from a
computed answer, and the finding printed beside it would read as advisory.

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

`epic.py` is the epic-bound facade; `workitem.py` beside it is the engine that
does not know which type it is serving. The split is what makes a second type —
change, spike, report — a thin facade rather than a copied file, and
`check_engine_split.py` is what keeps it that way: the engine's *code* may not
name a work-item type, in a string literal or an identifier.

Docstrings and comments are excluded on purpose. Explaining what the epic
facade does with a label is documentation; embedding `type:epic` in a branch is
behavior, and a gate that cannot tell them apart forces the engine to be
undocumented in order to stay green. The one exemption inside code is the
closed enum `WORK_ITEM_TYPES`, which is the axis itself — and the exemption is
measured, not trusted: widening the enum by one member turns the gate red, so a
leak cannot walk in through the exit.

The extraction was accepted by pinning the acceptance before writing it: every
gate's output byte-identical before and after. Seven of the eight were, and the
eighth differed in exactly two lines — `check_coverage_rule_negative_control.py`
prints the sha256 of `epic.py`, which cannot survive changing `epic.py`. That
difference was checked rather than normalized away: both printed digests had to
equal each other and equal the file's real digest.

### What each facade must expose, and what it may leave unused

`check_plugin.py` holds a required-verb set per script and resolves every verb
a skill names against the real script, so a documented verb that no longer
exists is refused rather than discovered at use. The two sets differ on purpose:
an epic owns children and can be closed against them, a change has neither, so a
change facade exposing `children` would mean the engine's epic shape had leaked
into the wrong type.

The interesting half is the gap in the other direction — a verb the script
exposes that no skill drives. Left silent, that is how a verb rots: nothing
documents it, nothing runs it, and it stays in the file looking supported. So
each one is declared with its reason, and the declaration is itself checked: the
verb must still resolve, **and** no skill may have quietly started naming it.

`adopt` is the declaration on both facades, and it got there by being used and
then stopping. `create` renames the staged body itself, so `adopt` only ever
answered the case of an iid arriving from outside the script — which is what
reconcile's hand-rolled `gh issue create` produced. Once child creation moved to
the change grill, nothing named it. That is the moment a verb usually rots
quietly; here it became a written claim instead, and `probe_local_verbs.py` still
exercises the behaviour. A declaration exempts a verb from being *named*, never
from being tested, which is the difference between a documented recovery path
and dead code with an alibi.

## Who opens a child

Reconcile decides **which** children an epic is missing; the change grill decides
what each one says. That line is enforced, not merely described: no SKILL.md may
name a `gh issue|pr create|edit|close|comment|delete|reopen` command, and the
positive control for that detector is the literal block reconcile carried until
this split — `gh issue create` with four `--label` flags — so the assertion is
pinned to the defect that was really there rather than to a caricature of it.

The defect it refuses is not "a shortcut". A hand-opened child gets a real issue
number and the right labels; what it does not get is a body any validator has
seen. Reconcile described that body in prose ("its body is Goal / How /
Acceptance / Never"), which is a second reading of a schema `change.py` owns
outright and is the sole enforcement of. Routing
creation through `/aw-grill-me-to-change` means every child passes
`change.py validate` before it is reported, and the prose summary disappears
rather than being kept correct.

The handoff is two rounds, and the order is the reasoning: the whole set of
missing children is settled in one `AskUserQuestion` round, because completeness
and duplication are judgements about the *set* and asking child-by-child hides
both; then each accepted child is grilled and landed before the next begins, so
an interrupted reconcile leaves whole work items behind rather than fragments.

## The change schema was ported, and now it is owned

The two schemas here arrived by different routes, and the difference still
decides how each is verified. The epic schema is this plugin's own invention, so
`epic.py` holds it as declarative `Section` data and the gates check that data
against live epics. The change schema was not ours: it was written in
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
  command `/aw-<skill>`, and the `aw:` form survives only as the frontmatter
  label.
- **The frontmatter `name:` is inert.** `zeta-mismatch` declared `zeta-other`
  and registered under its directory anyway. The field cannot change the
  invocation, so its only remaining job is to be the label the skill list
  shows without lying about the command — which is precisely what it did when
  this shipped broken. `check_plugin.py` pinned it to the directory name until
  2026-08-26; since the rename it pins it to `aw:<skill>` beside a directory
  `aw-<skill>`, so the two deliberately differ and each is checked against
  the one thing it decides.

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

These hit the tracker and produce evidence, not a verdict. Run them in order:

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
