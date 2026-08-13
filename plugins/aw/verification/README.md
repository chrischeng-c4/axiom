# Verification

Gates for the `aw` plugin and the two work-item schemas its scripts enforce.
Run everything with:

```
python3 plugins/aw/verification/run_all.py
```

Each gate resolves the checkout through `_paths.py`, which walks up to the
outermost `aw.toml` — the same rule the scripts use, so a gate and the script it
measures can never disagree about which tree is under test. `_paths.py` is also
the single place any of them spells a bundled location; a gate that recomputes
one is a second reading of a path, and the next time that path moves only one of
the two readings gets updated.

The plugin's shape:

```
plugins/aw/
  scripts/     epic.py, change.py — the type-bound facades — and workitem.py, the engine
  skills/      wi-epic-grill, wi-change-grill, wi-epic-reconcile
  verification/
```

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
| `probe_plugin_root.py` | a script that only resolves the repository when it happens to live inside one |
| `probe_local_verbs.py` | an `adopt` that overwrites, or an id parser that invents a number |

`check_manifests_cli.py` is the only gate here whose oracle this repository does
not own: it shells out to `claude plugin validate`, so it stays correct when
Claude Code's schema moves without telling us. Its warning assertion is the
load-bearing one — measured against v2.1.227, a plugin named `aw:epic` **passes**
validation with exit 0 and warns only that the Claude.ai marketplace sync
requires kebab-case. The negative control prints that exit code under the
mutation, so "the exit code cannot see this" is a number in the output rather
than a claim in a comment.

Three of these encode defects that actually shipped and were caught late:

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
