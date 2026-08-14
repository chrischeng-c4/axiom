# Verification

Gates for the `aw` plugin and the two work-item schemas its scripts enforce.

```
uv run --python 3.13 --no-project plugins/aw/verification/run_all.py                          # ~31s
uv run --python 3.13 --no-project plugins/aw/verification/run_all.py --with-negative-controls  # ~62s
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
`check_plugin_negative_control.py` is eleven such rounds, ~24s of the ~31s the
flag adds. The other big number is in the default run and is cargo:
`check_tdd_flow.py` builds and tests a synthetic crate through all three phases,
~25s of the ~31s there.

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

The plugin's shape:

```
plugins/aw/
  scripts/        epic.py, change.py — the type-bound facades — and workitem.py, the engine
                  leg.py, and the three phases it is shared by: e2e.py, unit.py, logic.py
  skills/         codex-code-review, codex-e2e-review,
                  wi-change-grill, wi-epic-grill, wi-epic-reconcile, wi-tdd
  verification/
```

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
| `check_plugin.py` | a manifest, bundled path, or skill cross-reference that does not resolve — a skill that has grown its own copy of a shared script, or one that reaches past its facade to `aw` or `gh` |
| `check_plugin_negative_control.py` | a checker that cannot be seen to fail |
| `check_coverage_rule.py` | a requirement with no `## Verification Inventory` row — and a rule that reddens epics which were already green |
| `check_coverage_rule_negative_control.py` | a coverage gate that measures the population instead of the rule |
| `check_engine_split.py` | an engine that has learned which work-item type it is serving |
| `check_engine_split_negative_control.py` | a split gate whose extractor reports "clean" because it found nothing |
| `check_change_schema.py` | a change facade whose reading of the GHAN schema has drifted from the crate that owns it |
| `check_change_schema_negative_control.py` | a port gate that stays green while one ported rule quietly stops firing |
| `check_epic_order.py` | an epic sequence that was guessed — a cycle answered with an arbitrary order, a child appended to the end because nothing placed it, or a declared dependency dropped because it could not be parsed |
| `probe_plugin_root.py` | a script that only resolves the repository when it happens to live inside one |
| `probe_local_verbs.py` | an `adopt` that overwrites, or an id parser that invents a number |
| `check_review_flow.py` | a verdict that outlives the bytes it was given — or one assembled from a transcript that never carried it |
| `check_tdd_flow.py` | an `e2e → unit → logic` phase whose green is not attributable to a red the phase before it named |

`check_manifests_cli.py` is the only gate here whose oracle this repository does
not own: it shells out to `claude plugin validate`, so it stays correct when
Claude Code's schema moves without telling us. Its warning assertion is the
load-bearing one — measured against v2.1.227, a plugin named `aw:epic` **passes**
validation with exit 0 and warns only that the Claude.ai marketplace sync
requires kebab-case. The negative control prints that exit code under the
mutation, so "the exit code cannot see this" is a number in the output rather
than a claim in a comment.

The last two carry their negative controls inside themselves rather than in a
sibling file. Each row of those gates *is* a declared mutation: the gate stages
a throwaway tree, breaks one thing in it, and requires the ladder to refuse for
the named reason — so `None` in `run_all.py`'s control column means "already
controlled", not "uncontrolled". `check_tdd_flow.py`'s fixture is a real cargo
crate, which is why it runs last and costs the most; `check_review_flow.py`'s
is deliberately cargo-free, which is why it is a fifth of the cost for two
thirds as many rows.

Four of these encode defects that actually shipped and were caught late:

- **`probe_plugin_root.py`.** `_repo_root()` walked up from `__file__`. A
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

## The ladder, and what makes a phase's green mean anything

`e2e → unit → logic` replaces `ec → td → cb`. The rule the three phases exist to
enforce is that a green is only evidence when a **named** red was measured
immediately before it, in the same tree, by the phase that had reason to
predict it. `check_tdd_flow.py` is 25 declared mutations against that rule.

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

## The two semantic reviews

Two of the three phases end in a question no exit code answers, and they are
different questions, so they are two reviews with two rubrics and two skills:

| phase | skill | what it is shown | what it is asked |
|---|---|---|---|
| `e2e` | `/aw:codex-e2e-review` | the work item, the cases, the exception each currently dies on | does this case pin what was asked for, and would it refuse a wrong implementation? |
| `logic` | `/aw:codex-code-review` | the work item, the tests as `unit` committed them, the source each sits beside | does this code satisfy the work item, or only its own tests? |

`unit` has none, and the reason is structural rather than a saving: at `unit`
only half the pair exists, so the question the code review asks has no
implementation to ask it about yet.

The review lands as a record under `.aw/review/<phase>-wi-<iid>.json`, and the
commit gate's `C7` row reads it. Three things are asserted about that record and
each is the answer to a way the review could be theatre:

- **It binds the bytes.** `change_digest` is sha256 over the work item's body
  *and* every changed file, so editing either side after the verdict makes the
  record describe bytes that no longer exist, and `C7` says so by name. The work
  item is inside the digest because the question was a comparison.
- **It is derived, not written.** `verdict` parses the raw transcript itself —
  verdicts must agree, one must be the final non-empty line, and a `rejected`
  with no `FINDING:` is refused. The stored transcript is a byte copy of the
  file that was parsed.
- **Its absence is not silence.** A commit with no verdict at all is a `FAIL` on
  `C7`, not a missing row.

`codex exec` is the subcommand, not `codex review`: the latter emits a fixed
`[P1]`/`[P2]` schema against a diff and ignores the prompt's output contract, so
it produced correct findings and zero `VERDICT:` lines, and the parser refused
the transcript.

Which skill a phase routes to is a **module constant in the phase script**, and
that is a deliberate move away from where it used to live. It was a `[review]`
key in each project's `aw.toml`, read at runtime, which made "the configured
reviewer is a skill that exists" a claim nothing could check until someone
invoked a reviewer against a project that had been misconfigured. As a constant
it is resolvable without running anything, so `check_plugin.py` asserts both
that it names a bundled skill and that it names *the one declared for that
phase* — the second because both reviewer names are real, and a `logic.py`
pointing at the contract reviewer passes an existence check while handing the
implementation to a rubric that never mentions it. Two negative-control
mutations, `reviewer-swap` and `reviewer-gone`, are what keep those two rows
distinguishable.

The gate split is a cost decision. `check_review_flow.py` owns the transcript
parser, the record, and both whole-surface prompt forms, and needs no cargo at
all — both reviewed phases call the same `leg.py` code, so driving every shape
of it through one phase measures the shared implementation once instead of once
per phase. What stays in `check_tdd_flow.py` is each phase's own `C7` wiring,
because a commit gate is the thing a verdict has to be able to stop, and that
costs a compile.

Omitting the iid switches both `review-prompt` verbs to a whole-surface,
advisory review of the project. `verdict` refuses that form rather than writing
an unbound record: a file shaped like the one a commit gate reads, holding an
approval of nothing, is worse than no file.

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
Acceptance / Never"), which is a fourth reading of a schema owned by
`ghan.rs`, ported by `change.py`, and enforced by `aw wi validate`. Routing
creation through `/aw:wi-change-grill` means every child passes
`change.py validate` before it is reported, and the prose summary disappears
rather than being kept correct.

The handoff is two rounds, and the order is the reasoning: the whole set of
missing children is settled in one `AskUserQuestion` round, because completeness
and duplication are judgements about the *set* and asking child-by-child hides
both; then each accepted child is grilled and landed before the next begins, so
an interrupted reconcile leaves whole work items behind rather than fragments.

## The change schema is ported, not authored

The two schemas here have different owners, and that difference decides how each
one is verified. The epic schema is this plugin's own invention, so `epic.py`
holds it as declarative `Section` data and the gates check that data against
live epics. The change schema is not ours:
`apps/agentic-workflow/src/issues/ghan.rs` owns it, `aw wi validate` enforces
it, and 640 live work items are already judged by it. A hand-written second
reading of those rules would not be a schema — it would be a fork with a delay
fuse, invisible for exactly as long as both sides happen to agree.

So `change.py` is a **port**, and every gate on it reads the crate as the
oracle. Nothing about the change schema is authored plugin-side:

| channel | oracle | catches |
|---|---|---|
| constants | the four H2s, six H3s, 15 hedge words and 7 failure assertions extracted from `ghan.rs` | a re-typed word list that silently narrowed |
| template | the 987B empty body extracted from `issues.rs` | two surfaces handing a human two different forms |
| corpus | all 18 `#[test]` functions in `ghan.rs`, replayed against the port | a rule the port reads differently from its author |

The corpus assertion is what makes the corpus non-optional: the replay count
must equal the crate's `#[test]` count, so a rule added upstream turns this gate
red until the port learns it — the one moment a drift is still cheap to fix.

The port's Rust-vs-Python near misses are where a transliteration actually
fails, so they are written as those Rust primitives rather than as the
nearest Python idiom. `str::lines` splits on `\n` alone, while `splitlines`
also breaks on `\v`, `\f`, `\x1c`-`\x1e`, `\x85`, U+2028 and U+2029 — so a
body containing one of them would be read as having more lines than the crate
sees, and a line is what a section boundary is made of. `to_ascii_lowercase`
touches `A-Z` and nothing else, while `.lower()` folds U+212A KELVIN SIGN to
`k` and U+0130 to `i` plus a combining dot — either of which could make a
hedge word match where the crate finds none. These differences are measured
rather than assumed: `_lines`, `_ascii_lower` and `_split_on` each hold the
Rust behaviour on exactly those inputs.

### What the live differential does and does not reach

`measure_change_agreement.py` runs the crate's own compiled rules against the
port over every live change body. The result is 640 bodies, 6,280 error strings,
zero divergence — but that number is breadth, not depth, and reading it as depth
is the trap the gate now prints its way out of.

Validation short-circuits: a body missing an H2 or carrying an unexpected one is
refused structurally and never reaches `validate_goal` and its three siblings.
Live, that is 619 of 640. The 21 that do get through pass every per-section rule,
so **the live population compares the per-section rules only on their non-firing
path** and contributes zero per-section error strings. A ported rule that is too
strict surfaces there as a spurious error; one that is too lax is invisible
there, and catchable only in the crate's own tests. The two gates cover
different halves, and the differential asserts it reached the per-section tier
at all rather than letting a five-figure count imply it.

### Why the oracle is `rustc` and not `aw wi validate`

The obvious oracle is the CLI the crate already ships, and it is disqualified:
`ValidateArgs` (`cli/issues.rs:890`) has no `--body-file` mode, and its failure
path calls `backend.update()` to write `validation_errors` back. Pointing it at
640 work items is a write sweep wearing a measurement's clothes.

So the differential extracts the rule half of `ghan.rs` mechanically — dropping
only `use super::Issue` and `validate_ghan_body`, the two items that reach into
the crate — compiles it with `rustc`, and runs it over bodies fetched with a
plain `gh issue list` GET. That is both read-only *and* strictly stronger: both
sides compute the same function, so the differential carries no excluded error
class at all. Read-only is measured rather than asserted — the run is bracketed
by an `updatedAt` census over all 640 items.

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
  before the colon.
- **The frontmatter `name:` is inert.** `zeta-mismatch` declared `zeta-other`
  and registered under its directory anyway. This is why `check_plugin.py`
  asserts the frontmatter name *equals* the directory: the field cannot change
  the invocation, so its only remaining job is to not lie about it — which is
  precisely what it did when this shipped broken.

The registration probe is deliberately not a gate: it costs an API call per
run, and what it measures is a property of Claude Code rather than of this
tree. `check_plugin.py` carries its conclusion instead, as a directory-name
assertion with a positive control that refuses a colon.

## The installed copy is a copy

`plugin install` copies the plugin into
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
| `measure_change_agreement.py` | do the crate's compiled rules and `change.py` return the same errors for every live change body? |

`measure_change_agreement.py` also needs `rustc` on PATH, and re-derives its
oracle from `ghan.rs` on every run, so it cannot go stale the way a transcribed
copy would. Its extractor asserts what it removed and what survived — an
extraction that silently produced an empty file would otherwise agree with
everything.

`measure_population.py` writes `_snapshots/`, which is gitignored: it is live
tracker state, and a committed copy would let the regression assertion in
`check_coverage_rule.py` pass against a population that no longer exists. When
the snapshot is absent that assertion fails and says so rather than skipping,
because a silent skip turns the strongest gate here into a no-op that still
prints green.

The blast-radius measurement is why `_requirement_refs` expands ranges and
lists at all: the bare-equality reading turned 8 of 54 valid epics red, every
one of them on spelling rather than on missing coverage.
