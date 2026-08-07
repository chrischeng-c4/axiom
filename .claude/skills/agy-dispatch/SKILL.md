---
name: agy:dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless AGY using persistent AGY Project permissions plus a task-local command/write contract, then independently verify its report. Use whenever Claude delegates audits, measurements, investigation, transcription, or tightly scoped implementation to AGY; ticketed tasks reuse one conversation per live issue, unticketed tasks run once without resume, implementation requires frozen design inputs, and acceptance remains controller-only.
user-invocable: true
---

# AGY Dispatch

Treat headless AGY as a bounded worker, never as an autonomous owner. You
define scope and oracle, dispatch one task per process, verify the result, and
alone decide acceptance, publication, and closure.

Each phase below is stated the same way the round's own injection form is
stated — what it does, the command verbatim, what not to do, and what counts as
done. Read the phase you are in; do not read the file end to end to run a
verb.

[references/lifecycle.md](references/lifecycle.md) is the normative state
machine. Read it completely before your first dispatch or takeover, and
whenever resuming from another controller's handoff. A future worker adapter
may replace AGY only by preserving its states and evidence contracts.

## The round

```
worktree → grant → doctor → scaffold → (fill) → lint → snapshot → dispatch
        → verify → review → prove ×2 → accept | resume | discard
```

Every verb takes the profile; those marked below also take the task key — the
issue id for ticketed work, the explicit `run_id` for a one-shot.

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py
python3 $S worktree profile.json KEY     # derive the worker's checkout
python3 $S grant    profile.json         # install the Project permission set
python3 $S doctor   profile.json         # read-only permission preflight
python3 $S scaffold profile.json KEY     # write both documents as blank forms
python3 $S lint     profile.json KEY     # structural checks on both documents
python3 $S snapshot profile.json KEY     # freeze contract, tree, permissions
python3 $S dispatch profile.json KEY     # run the worker (long)
python3 $S resume   profile.json KEY     # revision round, ticketed only (long)
python3 $S abandon  profile.json KEY     # release a run that produced nothing
python3 $S denied   profile.json KEY     # triage a soft-denied command
python3 $S verify   profile.json KEY     # integrity + scope audit
python3 $S review   profile.json KEY     # print the diff to adjudicate
python3 $S prove    profile.json KEY mutant|candidate
python3 $S accept   profile.json KEY     # commit the candidate on its branch
python3 $S discard  profile.json KEY     # release worktree, branch, binding
python3 $S status   profile.json
```

Exit codes: `0` clean, `1` VOID, `2` findings for you to adjudicate.

## Before the first round — the profile

**Does.** Declares the round's scope, permissions, write contract, and gate.
Generating it freezes the **complement**: every regular file under `--scope`
the round may not write is protected with its current sha256, recorded
repo-relative so the frozen set follows the round into its derived worktree.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/make_profile.py \
    --root /abs/controller-checkout --repo owner/name --project-id <id> \
    --scope libs/thing --issue 1234 --inject /abs/delta-round.md \
    --design-input libs/thing/CAPABILITIES.md \
    --write libs/thing/src/a.py:40 --write libs/thing/tests/b.py:12 \
    --out /abs/profile.json
```

`--root` is *your* checkout, recorded as `controller_root`; the round's own
`root` is filled in later by `worktree`. A `--write` path need not exist yet,
so greenfield and edit rounds are expressed identically. For a one-shot round,
`--run-id` also requires `--intent`.

**Never** hand-write the profile, and never enumerate the protected list
instead of freezing the complement — an enumerated list silently stops covering
files added since the last round, and `verify` then cannot tell a stray write
from an intended one. Never keep a profile carrying local paths or mutable pins
inside the repository. Never put project-specific selectors, denominators,
expected bands, or fabrication tells here or anywhere else in this skill; they
belong in the ticket or the round's oracle, which the dispatcher injects whole.

**Done when** the class minimums hold:

| Class | Requires |
|---|---|
| `bounded-write` implementation | explicit `allowed_repo_writes`, `task_contract.kind=implementation`, ≥1 frozen `design_inputs`, oracle; ticketed work also a live issue |
| `measure-only` measurement / investigation / review / audit | frozen ticket or one-shot `intent` + oracle; no design input required |

The profile states the command surface twice, deliberately:

- `project_permissions` is the complete, persistent Project policy. Keep it
  stable and broad enough for normal work on that project, but exclude
  destructive capabilities. `grant` installs it and `doctor` compares it
  against the live surface.
- `task_commands` is the ticket-local exact shell allowlist plus explicit deny
  probes. It narrows the prompt without changing Project settings, and `verify`
  audits every post-snapshot request against it.

`mode` is `measure-only` by default. For a ticket set
`session_policy=ticketed` plus `issue`; for an unticketed task set
`session_policy=one-shot` plus a unique `run_id` and frozen `intent`. Enumerate
exact writable paths — never directory globs — and set `path_change_budgets`
when a path should receive only a small localized change. Design-input
selection is caller-owned; the task-class minimums are not.

Size the ticket so every required witness fits in the terminal report. Split a
large exact-set inventory before dispatch if admitted plus discarded rows
cannot be printed completely: a digest and total do not replace the rows needed
to recompute them.

The templates
([references/profile-template.json](references/profile-template.json),
[references/one-shot-profile-template.json](references/one-shot-profile-template.json))
are the field reference for anything the generator does not emit.

## `worktree` — the worker's own checkout

**Does.** Creates `agy/KEY` at `<parent>/.agy-worktrees/<repo>-KEY`, rebinds
the AGY Project to it, and writes `root`, `agy_project_id`, and the frozen
`worktree.base_sha` back into the profile. Every later verb then operates on
the worker's checkout with no further arguments.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py worktree profile.json KEY
```

**Never** dispatch into the tree you are working in, and never hand-create the
worktree with `git worktree add` — ad-hoc creation is what produced hundreds of
stranded checkouts and orphaned branches. Never clone the AGY Project per
round: `agy --project <id>` forces the worker's working directory to the
Project's registered folder and ignores the caller's cwd, so reaching a derived
worktree means *moving* that binding. `worktree` moves it and records the home
root; `discard` moves it back. Only `projectResources` changes, so the reviewed
permission surface is the same object before and after. Cloning would leave one
registry entry and one un-updatable permission copy per round.

Two consequences follow from moving rather than cloning: rounds inside a work
area are **serial**, since one Project points at one root at a time — raise
throughput with work areas that have their own registered Project — and you
must **always `discard`**, since an interrupted round leaves the shared Project
pointing at the round's worktree and an unrelated `agy` session would open
there.

**Done when** the profile carries a derived `root` and the branch exists. The
unit is one project → one persistent AGY Project → one `state_dir` → one
disposable worktree per round.

The round is cut from your **`HEAD`**, so uncommitted work is invisible to the
worker: commit or stash anything the task depends on — a design input, a
fixture, the file being extended — before running this. Otherwise `doctor`
fails on a missing design input, or worse, the worker silently builds against
the committed version of a file you had already changed.

Standing constraints: register each work area once with `agy --new-project` and
never change `agy_project_id` per round; worker branches are namespaced `agy/*`
and worker checkouts live outside `controller_root`, both enforced; Claude and
Codex share one `~/.gemini` Project registry and are controllers, not separate
permission namespaces, so two controllers must not run rounds against one work
area concurrently; keep one stable `/tmp/agy-dispatch/<project-id>/`
`state_dir` per Project and treat it as transient — publish durable verdicts to
the issue, and start from a fresh oracle and snapshot rather than resuming if
`/tmp` was cleaned.

## `grant` — install the Project permission set

**Does.** Installs `project_permissions` as the live Project policy.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py grant profile.json
```

**Never** skip this after a `discard`. `discard` restores the Project's grants
baseline, so the next round starts without the previous round's grants and
`doctor` reports every task command as `expected allow but resolves ask`.

**Done when** `grant` reports the live grants matching the profile. It refuses
to install a set that would still leave any `task_commands` allow entry
resolving to `ask`, and names those commands — without that check, a profile
copied from the previous round (where the gate command is the field that always
changes) could report "nothing to change" with the *previous* round's gate
granted and this round's not.

## `doctor` — permission preflight

**Does.** Compares the live Project surface exactly with `project_permissions`,
read-only. Blocks on live Project drift, an inherited Global rule that widens
the worker past the declared surface, an unresolved ticket command, an inert
sandbox escape, or a project/root mismatch.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py doctor profile.json
```

**Never** treat its `unsandboxed` column as advisory for a build gate. A
command runs sandboxed unless a paired `unsandboxed(...)` allow rule matches
it; escape is consulted only after the command already resolved to `allow`, so
an `unsandboxed(P)` rule with no `command(P)` twin can never fire and `doctor`
blocks on it. Whether a command *needs* the escape is a judgement the profile
cannot state, so `doctor` reports it per command instead of guessing: `cargo`
needs it, `rustfmt` on a file in the worktree does not, and a sandboxed `cargo`
fails in a way that reads like a product defect.

**Done when** it prints `dispatch_ready: true, blockers: []`. Project, Shared,
and Global rules are additive and deny/ask can override an allow, so an
unnoticed Global rule destroys project isolation. Add
`project_permissions.require_empty_global=true` by hand for a round that must
refuse *any* inherited Global rule; the generator leaves it off, because a flag
you switch off every round teaches you to click past the finding standing next
to it.

Project rules use AGY's token-prefix matching, so a reusable `command(rg)`
Project allow can cover multiple tickets. Ticket commands stay full, byte-exact
lines such as `rg -n 'EXACT_SELECTOR' src/file.rs`; a pipeline, reordered flag,
alternate quote form, or narrower command is a *different* ticket command.
Prefer shell-safe selectors, one command per tool call, and the built-in
read-file tool for additional inspection.

## `scaffold` + fill — author the round's two documents

**Does.** Writes both documents as blank forms and prefills what the profile
already knows — the gate command and the frozen design inputs.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py scaffold profile.json KEY
```

- **Oracle** — `state_dir/oracles/<task-key>.md`. `## Claim`,
  `## Measurements`, `## Gate`, `## Fabrication tells`.
- **Injection** — the profile's `inject_prompt_file`. `## Task`,
  `## Current behavior`, `## Required change`, `## Shape to follow`,
  `## Reference`, `## Out of scope`, `## Definition of done`.

**Never** author from memory, and never restate in a slot anything the
dispatcher already sends. The seven injection slots are the whole of what a
round decides: what to do, what the code does today, what must become true,
which convention to follow, what to read, what not to touch, what counts as
done. The write allowlist, the command allowlist, the stop-and-report rule, the
report shape, and the session policy are hardcoded in `render_prompt` and are
not yours to choose; a second copy is the one that drifts.

**Done when** no `<!-- fill -->` slot remains and `lint` is clean. Each slot's
rule lives *in the slot*, as a `<!-- fill -->` comment the scaffold writes —
read the form, not this file, when authoring. Those comments are
controller-facing and never reach the worker: `render_prompt` strips every HTML
comment from both documents, so a rule's rationale cannot arrive as an
instruction, and a hint can be as long as it needs to be at no cost to the
worker.

`scaffold` never overwrites an authored file, so re-running it mid-round is
safe. An injection is optional — a measure-only round can carry its instruction
in the oracle — but a *declared* `inject_prompt_file` must exist and conform,
since that is the half through which the last false green entered.

## `lint` — structural checks

**Does.** Enforces both documents' structure, carrying no project knowledge.
`dispatch` runs the same checks and refuses on any finding.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py lint profile.json KEY
```

| Check | What it prevents |
|---|---|
| `## Measurements` has ≥2 rows, ≥1 marked `negative control` | a table an unchanged implementation also satisfies |
| the control is marked in a row's input or observation, not its rationale | prose *about* a control counted as *having* one |
| `## Gate` commands ⊆ the task allowlist | a gate the worker is not authorized to run, so nobody runs it |
| `## Gate` names only `task_contract.gate_command` | a second command that reads as judged when it is only authorized |
| `## Definition of done` ≡ the oracle's `## Gate` | instruction and judgement drifting apart, each satisfiable alone |
| `## Current behavior` has a non-empty fenced quote | a round authored from memory rather than from the checkout |
| every quoted line appears in one of the round's files | a quote that was true at an earlier base and is now fiction |
| no fenced block outside `Current behavior` / `Definition of done` | pasting the implementation, which leaves the worker nothing to derive |
| no numbered steps in `Required change` / `Shape to follow` | a recipe to retype instead of a requirement to satisfy |
| `## Shape to follow` names a backticked symbol, within its line budget | a free-prose slot growing into the design the round was meant to buy |
| `## Definition of done` names where the check lands | a correct diff arriving in the wrong module |
| backticked paths resolve under `root` | stale coordinates reaching the worker |
| no `<!-- fill -->` slots remain | a form dispatched before it was written |

**Never** repair a finding from the wrong half of a pair. Three checks come in
pairs: the **gate** pair asks separately whether the worker *may* run a command
and whether `prove` *will*, since `prove` runs `task_contract.gate_command`
alone — the remedy for a compound gate is to name it in the profile, not to
widen the fence. The **control** pair asks whether the table has a control at
all and whether the marked row *is* one. The **`Current behavior`** pair asks
whether you opened the file and whether you opened it *at this round's base* —
rounds get re-based constantly here, so a quote goes stale with nobody editing
it.

**Done when** it exits `0`. The gate cross-check is skipped when the round
grants no shell: a measure-only oracle names what *you* will run.

## `snapshot` — freeze

**Does.** Records the sha256 of both documents plus Git state, protected bytes,
the frozen dispatch contract, AGY project identity, the conversation step
floor, and a digest of Project plus Global permissions.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py snapshot profile.json KEY
```

**Never** edit either document afterwards. `dispatch`, `resume`, and `verify`
each VOID on a mismatch, including one that adds an injection the round did not
have. You are the only party able to edit these two files while a worker runs,
which is exactly why they are frozen: an oracle that can be softened to fit the
answer it received is not an oracle, and a hash that is printed but never
compared reads like a freeze without being one.

**Done when** the snapshot is written. Record the oracle's SHA-256 in the issue
comment for ticketed work or the controller log for one-shot work.

Git state is captured with `--untracked-files=all` at both `snapshot` and
`verify`. Without it, porcelain collapses a never-tracked directory to a single
`dir/` entry, which can never match an exact `allowed_repo_writes` path and
misreports every greenfield dispatch. Because of that flag a round creating a
brand-new source tree is verified path by path exactly like one editing tracked
files: keep listing exact files, never a directory.

## Baseline check — no verb, still mandatory

**Does.** Proves the worker's checkout is green *before* the worker touches it,
and leaves a warm `target/` behind so the round's timeout pays for reasoning
rather than a cold build. A worker dispatched onto a red base spends its whole
budget on a failure it did not cause, and then reports it as one it could not
fix.

**Run.** From the worker worktree, one gate that already passes at this base —
not the round's own gate, which does not exist yet. Take it verbatim from the
profile's `project_permissions.allow`:

```bash
cd <worktree> && <one landed gate from the allowlist>
```

**Never** run the round's whole allowlist here. One landed gate proves the tree
builds and the harness works; the rest is the worker's job and its evidence.
Never widen it past the allowlist either — the allowlist is narrow by
construction, and a broader suite can hang or fail for reasons this round does
not own, so a controller stuck there has burned the round's wall-clock before
it started.

**Done when** that gate reports `test result: ok` and `target/` exists in the
worktree.

## `dispatch` / `resume` — run the worker

**Does.** Runs AGY against the frozen contract, passing only `--project <id>`
— no `--add-dir`, no settings mutation. Stores the rendered prompt, AGY log,
raw output, and normalized final `## EXEC REPORT` under `state_dir/runs/`.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py dispatch profile.json KEY
```

**Never** background these two by hand with `nohup ... &`: that returns
immediately, orphans the run from the harness, and then needs a second polling
watcher to notice an exit the harness would have reported for free. Use the
Bash tool's `run_in_background` instead — every other verb returns in seconds,
these two run for the profile's whole `timeout`. Never wrap them in an
orchestration helper that can return before the nested subprocess finishes; an
early empty wrapper result is not an AGY report. Never call `resume` for a
one-shot run, and never silently promote a one-shot into a ticketed session —
for a one-shot profile use its unique `RUN_ID` in place of the issue id for
`snapshot`, `dispatch`, and `verify`, and on a retryable transport failure
create a new run id, oracle, and snapshot instead.

**Done when** a terminal `## EXEC REPORT` was filed. Process exit is never
acceptance: empty output, nonzero exit, or a missing terminal report is a
failed delivery, and progress chatter may precede the real report. Size
`timeout` to the round rather than reusing the previous one — a worker cut off
at its deadline exits non-zero with its work already on disk and no report, so
the round reads as a failure while the candidate is complete. `dispatch` names
that case separately from a denial; when it does, read the worktree diff and
run the gate before deciding whether to redispatch.

To send a ticketed round back, first run `verify`; after confirming the
snapshot and protected artifacts, correct the delta prompt or Project policy,
take a fresh snapshot when either contract changed, and `resume` with the same
profile and state. A transient backend 5xx may resume without a contract change
while the snapshot remains valid. Do not resume after a VOID result, without a
stored conversation id, or for another ticket.

A revision report is a self-contained replacement, not a delta fragment.
Require it to preserve every already accepted identity, row, matrix, invariant,
lifecycle boundary, planned path, forbidden change, and test seam while
applying the correction, and compare the new normalized report against the
prior accepted sections — reject a compact re-emission that silently drops
evidence. Make list-edit intent explicit in the delta: say `append` when a new
test or invariant must coexist with accepted entries, `replace <exact entry>`
only when the old entry must disappear. "Add if needed" is not a contract when
count or coverage is an acceptance surface.

For a hard report-size limit, leave operational headroom instead of asking for
the exact ceiling: on the first oversize revision set a target at least 10%
below the acceptance maximum and name both the removable structures (executive
checklists, repeated prose, decorative separators) and the exact tables and
lists that must survive. If AGY exceeds it again, give a section budget or a
compact replacement template — size compliance is an acceptance criterion, not
a controller-side truncation step. The frozen maximum remains the hard limit:
do not turn advisory headroom into a second, stricter ceiling, and do not keep
revising a semantically complete report already below the frozen maximum.

## `denied` / `abandon` — the two side exits

**Does.** `denied` triages a soft-denied command. `abandon` releases a run id
after an infrastructure death — an upstream 5xx, a killed host process — so
`snapshot` and `dispatch` can run again.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py denied  profile.json KEY
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py abandon profile.json KEY
```

**Never** grant a temporary ticket permission or reach for a
bypass-permissions flag to clear a denial. If the command is unnecessary,
tighten the prompt and resume; if it is a reusable project capability, add one
narrow Project-scope rule through `/permissions`, update the profile's exact
`project_permissions`, rerun `doctor`, and take a fresh snapshot before
resuming. Never use `abandon` to retry away a result you dislike.

**Done when** `abandon` accepts — which it does only if the run provably
produced nothing: no path changed in the worker checkout, no command requested
after the snapshot floor, no `## EXEC REPORT` filed. A round that produced any
of those is judged with `review` and `verify`, never released. The dead
attempt's logs are parked under `runs/abandoned/` rather than deleted; they are
evidence, and leaving them in place would let the id lookup recover the dead
conversation.

## `verify` — integrity and scope audit

**Does.** Compares post-run Git state and protected artifact hashes against the
pre-dispatch snapshot, audits AGY's post-snapshot command requests from the
conversation database against the exact `task_commands` allowlist, and applies
`path_change_budgets` to bound added plus removed lines per writable path.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py verify profile.json KEY
```

**Never** read exit `2` as exit `1`. The two outcomes are different claims:

- **Integrity failures exit `1` (VOID)** — permission drift, a command outside
  the audited allowlist, a changed dispatch contract, a swapped Project, a
  missing oracle. The evidence cannot be trusted at all.
- **Scope problems exit `2` (findings)** — a write outside
  `allowed_repo_writes`, a changed protected artifact, a declared path left
  unwritten, a budget overrun, a moved branch `HEAD`. These are handed to
  `review` for you to adjudicate against the diff.

That split is sound only because the worker's tree is disposable; a profile
with no `worktree` key keeps the old behavior of voiding on scope.

**Done when** you have read every finding it names. The worker never runs a git
mutation — a moved branch `HEAD` is how a worker commit surfaces.

## `review` — adjudicate the diff

**Does.** Prints the round's branch and base, the touched paths with `!` on
anything outside `allowed_repo_writes`, every finding, `git diff --stat`, the
full diff, and the body of each new untracked file; exits `2` if there is a
finding.

**Run.**

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py review profile.json KEY
```

**Never** decide anything before reading it: the worker's report is a claim
about the diff, not the diff. Never accept a finding without reading the path
it names — a finding is a question, not a verdict. An out-of-contract write can
be a worker that misread the scope, which you send back, or a contract too
narrow for the work, in which case widen `allowed_repo_writes` next round
rather than blaming the diff.

**Done when** every finding has an adjudication and the diff conforms to the
oracle row by row. Treat every report `PASS` as provisional: a claim that
formatting is clean or an excluded form is unchanged needs controller evidence
such as `git diff --check` plus focused negative controls. Report prose is not
a witness.

Read [references/report-verification.md](references/report-verification.md)
before accepting any report, and
[references/inventory-verification.md](references/inventory-verification.md)
too for inventory, classification, denominator, or regional-footprint work.
[references/report-review-vocabulary.md](references/report-review-vocabulary.md)
holds the distinctions a report must hold; put the ones this round turns on
into its oracle's `## Fabrication tells`, where they are frozen and checkable.

Keep an oracle's claim about *current* source behavior separate from its design
authority. Accepted sole-owner, boundary, and target-lifecycle decisions remain
authoritative, but a claim about present call order, key contents, or ownership
is a hypothesis until the admitted source proves it. If source evidence
contradicts it, AGY must name the exact witness and report the contradiction,
and you record an explicit correction in the next round's injection — never
force the report to repeat a disproved claim, and never silently rewrite the
sealed oracle.

## `prove` ×2 — measure the gate before trusting it

**Does.** Runs the round's gate over the worktree exactly as it stands and
files the result under the label you give it. **`prove` reverts nothing** — the
reverting is yours.

**Run.**

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py
# restore the product change to the round's baseline, keeping the worker's tests
python3 $S prove profile.json KEY mutant
# restore the candidate
python3 $S prove profile.json KEY candidate
```

**Never** run the two back to back without reverting in between; `accept`
refuses a mutant that passed, a candidate that failed, two proofs over an
identical tree, and a candidate proof taken before the tree moved again. Never
let a build failure read as a behavioural kill: when the round introduces a new
symbol the reverted tree cannot compile, so the gate goes red because the
function it names does not exist yet and nothing about behaviour was measured.
That is the only answer the revert can give for such a round, so it does not
block — `prove` and `accept` both say so.

Guard the revert with assertions, and anchor them on the declaration the round
extends rather than on a bare identifier — a file large enough to be worth a
round usually already carries that name for something else. When such a guard
fires anyway, read the namesake before loosening it. Two things in one file
answering to one name is either the duplication the round was supposed to avoid
or a second implementation of the same rule that will now drift from the first,
and both are worth a work item before this round lands.

**Done when** the pair discriminates *and* a mutation sweep backs it. A gate
nobody has seen fail proves nothing: a test written against the implementation
just produced passes by construction. The pair is a floor, not a ceiling.

Discrimination comes from a sweep whose mutants keep the product compiling: one
single-defect mutant per rule the oracle claims, each expected to be killed by
a named row.

- A mutant that fails to build is a badly written mutant, not a kill. Repair it
  by making that same defect representable — naming a type restores a build
  that inference lost — never by reaching for a different defect that happens
  to compile. Substituting quietly changes which row is under test, so the
  sweep reports a kill for a rule nobody claimed while the rule that failed to
  compile stays unmeasured.
- Apply *and* restore by writing the whole file. A copy that preserves an older
  mtime lets cargo skip the rebuild, which turns the mutant into a false kill
  and the next candidate run into a false green.
- A survivor is a finding against the round's **evidence**, not the product:
  the code may be right and simply have no row that would notice if it stopped
  being right. Close it by authoring the missing row in a round, never by
  editing the test yourself — you writing the evidence you then judge is the
  same failure the oracle freeze exists to prevent.
- A round that only *adds* evidence has nothing to revert, so the pair
  degenerates into two runs over one tree, which `accept` refuses as identical
  digests. Take the mutant from the **unchanged** product instead: break the
  rule the new rows are supposed to pin. That is also the proof that catches
  the round which performs its deletions and skips its additions — the gate is
  green whether or not the missing rows exist, so the report reads as done, and
  only a surviving mutant says otherwise.

## `accept` / `resume` / `discard` — the three outcomes

**Does.** `accept` stages exactly the touched paths and commits them on the
round's branch (with `Refs #<issue>` for ticketed work), then prints the
`git cherry-pick <sha>` for you to run from `controller_root`. `resume` sends
the round back. `discard` restores the Project's home root and removes the
worktree and branch.

**Run.**

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py
python3 $S accept  profile.json KEY
python3 $S discard profile.json KEY [--keep-branch]
```

**Never** expect `accept` to merge. Integration and gates are yours and they
run against *your* branch, not the worker's checkout. To send a round back,
write the exact delta contract to a file, point `inject_prompt_file` at it,
re-`snapshot` if the contract changed, and `resume` (ticketed only) — the
worker's checkout persists across the revision, so the next round builds on the
same tree instead of starting over. Never leave a round undiscarded; use
`--keep-branch` to throw away the checkout but keep the candidate commit.

**Done when** the candidate is cherry-picked into your branch and `discard` has
restored the Project's home root. `discard` also restores the grants baseline,
so the next round must start at `grant` again.

When a delta round exists because *your own* contract was wrong, say so in the
delta, quote the superseded instruction, and give the corrected rule with its
authority. A worker that faithfully implemented a bad instruction has not
erred, and a delta implying otherwise teaches it to distrust the frozen
contract it is supposed to follow.

## Rounds with no shell

**Does.** For a pure authoring round — write these files, to this design — an
empty `task_commands.allow` and empty `project_permissions.allow` give the
worker no shell at all. `doctor` still passes, and the dispatcher swaps its
appended report contract for the no-shell variant, which forbids `PASS`/`FAIL`
and any claimed observation and asks instead for a description of what was
written.

**Never** leave the verdict-shaped contract in place for such a round. The
trailer is appended *after* the injected delta, so a trailer demanding "PASS or
FAIL per criterion" overrides an injection saying "do not write PASS" and
manufactures the exact fabrication the oracle then has to catch. Match the
contract to the worker's actual capability and the tell becomes real: with an
empty allowlist, any test result, build outcome, or per-criterion verdict in
the report is fabricated by construction, and the audited command list in
`verify` proves it.

**Done when** a controller-side recompute passes. Acceptance is never a read of
the report: write a script that imports the symbols, inspects the annotations,
calls the functions on the oracle's witness inputs, and runs the project gate,
and accept on its exit code. Keep it outside the worker's readable material
together with any seeded-defect or answer-key file. A report that agrees with a
recompute adds nothing; a report that disagrees is the finding.

A freeze is a claim about the whole tree, so check it before making it.
Whenever a round changes a value other files could restate — a declared count,
an arity, a version, an enum's membership — grep the entire scope for that
value before deciding what to freeze. A second copy left frozen turns a correct
round red and costs a whole extra dispatch, and the worker will have been right
to leave it alone. Having found a duplicate, resist merging it: two tables that
must agree are a check, one table two places read is an assumption. Keep them
independent when they are derived differently — one counted syntactically, the
other read out of a real run's output — and say so in the delta, or the next
worker will helpfully de-duplicate them and delete the check.

## Interactive teamwork preview

**Does.** Launches an interactive AGY `/teamwork-preview` session. This is a
separate execution mode, not a headless dispatch.

**Run.** Only when the user explicitly requests it.

```bash
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py detect
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py launch profile.json teamwork-prompt.md
```

**Never** attempt it headless: it requires a real TTY, and `--print/-p` and
`--prompt-interactive/-i` are mutually exclusive. It does not use `--add-dir`
or `--dangerously-skip-permissions`.

**Done when** the TTY is handed to the user. The launcher uses `expect` to
start `agy --project <id> --prompt-interactive` and sends the prompt; on macOS
it prefers splitting a pane in the current iTerm2 tab and otherwise falls back
to Terminal.app. The prompt must start with `/teamwork-preview` and name the
coordination scope. Interactive teamwork is not eligible for automatic issue
closure or the headless report-verification contract.

## Standing rules

- One task per AGY process, one round per work area. Rounds inside a project
  are serial by construction; raise throughput with distinct work areas that
  have their own registered Project.
- AGY may comment but never closes a ticket. A GitHub comment is unverified
  input, not a result.
- Prohibit git mutations, branch switching, worktrees, and unscoped writes.
  Permit read-only git commands only when the profile lists their prefixes.
- A permission-surface change, a changed dispatch contract, a swapped Project,
  an unaudited command, or a missing oracle is a **void**: the evidence itself
  is untrustworthy.
- Everything the worker did to its own checkout is a **finding**, not a void —
  including a changed protected artifact. `make_profile.py` protects the whole
  complement of the write scope, so voiding on it would reinstate exactly the
  behavior this design removes. A protected-artifact finding is still the most
  serious class: read that path before anything else.
- Keep working while a round is in flight. The worker has its own checkout, so
  controller-side edits cannot dirty its snapshot. What you must not do is edit
  *the worker's* checkout under `.agy-worktrees/` while the round runs.
- Triage logs and the mandatory local report, not process exit alone: an
  unlisted AGY command can abort while returning zero.
- The profile supplies project-specific build/test commands, exact binary
  paths, caller-selected design inputs, and the verification contract. This
  skill supplies the task-class minimums and permission isolation and nothing
  project-specific. Do not add a toolchain denial such as `cargo` unless that
  ticket must not use it.
