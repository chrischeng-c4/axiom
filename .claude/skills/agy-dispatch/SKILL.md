---
name: agy:dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless AGY using persistent AGY Project permissions plus a task-local command/write contract, then independently verify its report. Use whenever Claude delegates audits, measurements, investigation, transcription, or tightly scoped implementation to AGY; ticketed tasks reuse one conversation per live issue, unticketed tasks run once without resume, implementation requires frozen design inputs, and acceptance remains controller-only.
user-invocable: true
---

# AGY Dispatch

Headless AGY is a bounded worker, never an autonomous owner. You own three
things: **what the round claims**, **what the worker is told**, and
**acceptance**. Everything else is a verb — the write contract, the frozen
complement, the permission install, the freeze, the audit, the diff, the
commit.

**This file is about driving the verbs.** It is not about how to author the two
documents. Those rules live in the forms `scaffold` writes, as `<!-- fill -->`
comments in the slot each one governs, and `lint` refuses a form that still
holds one. Read the form when authoring. Read this file when a verb refuses.

Each verb below states what it changes, what it **refuses** and the recovery
for each refusal, and what it **does not refuse** — the ways a round goes
silently wrong with every exit code zero. That last row is why this file
exists; the refusals announce themselves.

[references/lifecycle.md](references/lifecycle.md) is the normative state
machine. Read it completely before your first dispatch or takeover, and
whenever resuming from another controller's handoff. A future worker adapter
may replace AGY only by preserving its states and evidence contracts.

## The round

```
worktree → grant → doctor → scaffold → (fill, capture) → lint → snapshot → dispatch
        → verify → review → prove ×2 → sweep → accept | revise | discard
```

```bash
S=.claude/skills/agy-dispatch/scripts/agy_dispatch.py
python3 $S worktree profile.json KEY     # derive the worker's checkout
python3 $S grant    profile.json         # install the Project permission set
python3 $S doctor   profile.json         # read-only permission preflight
python3 $S scaffold profile.json KEY     # write both documents as blank forms
python3 $S capture  profile.json KEY CMD # run CMD, store what it printed
python3 $S lint     profile.json KEY     # structural checks on both documents
python3 $S snapshot profile.json KEY     # freeze contract, tree, permissions
python3 $S dispatch profile.json KEY     # run the worker (long)
python3 $S resume   profile.json KEY     # revision round, ticketed only (long)
python3 $S revise   profile.json KEY NEXT_KEY DELTA.md   # one-shot revision
python3 $S abandon  profile.json KEY     # release a run that produced nothing
python3 $S denied   profile.json KEY     # triage a soft-denied command
python3 $S verify   profile.json KEY     # integrity + scope audit
python3 $S review   profile.json KEY     # print the diff to adjudicate
python3 $S prove    profile.json KEY mutant|candidate
python3 $S sweep    profile.json KEY SCRIPT   # record the mutation sweep
python3 $S accept   profile.json KEY     # commit the candidate on its branch
python3 $S discard  profile.json KEY     # release worktree, branch, binding
python3 $S status   profile.json
```

Every verb takes the profile; those taking a task key take the issue id for
ticketed work, the explicit `run_id` for a one-shot.

**Exit codes.** `0` clean, `1` VOID — the evidence itself cannot be trusted,
`2` findings for you to adjudicate. Never read `2` as `1`: a VOID discards the
round, a finding is a question about a candidate that still exists.

## `make_profile.py` — generate the round's contract

Declares scope, permissions, the write contract, and the gate. Generating
freezes the **complement**: every regular file under `--scope` the round may
not write, protected with its current sha256, recorded repo-relative so the
frozen set follows the round into its derived worktree.

This is the section with the most prose left, because the profile is the one
form that is generated but not *slotted* — its fields carry no rule, so the
rules are here. #3440 moves them onto the fields; this section shrinks to the
flag table when it lands.

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
so greenfield and edit rounds are expressed identically. A one-shot round needs
`--run-id` plus `--intent`.

**Does not refuse.** A profile you produced any other way. Hand-writing one, or
editing the last round's, is the single most expensive mistake available here,
and nothing stops you:

- Editing `--write` leaves the frozen complement describing the round it came
  from, so the new write target arrives **already frozen**. Writing it is a
  finding and not writing it is a finding. `doctor` has a check for exactly
  this and it cannot fire (#3439), so the contradiction surfaces as a
  protected-artifact hash mismatch *after* the worker has been editing for
  minutes, and on a one-shot the killed dispatch has already spent the run id.
  **Regenerate. Never derive** (#3428, #3440).
- Enumerating the protected list instead of freezing the complement silently
  stops covering files added since the last round, and `verify` then cannot
  tell a stray write from an intended one.

Keep no profile carrying local paths or mutable pins inside the repository, and
keep project-specific selectors, denominators, expected bands, and fabrication
tells out of the profile and out of this skill — they belong in the ticket or
the round's oracle, which the dispatcher injects whole.

**Done when** the class minimums hold:

| Class | Requires |
|---|---|
| `bounded-write` implementation | explicit `allowed_repo_writes`, `task_contract.kind=implementation`, ≥1 frozen `design_inputs`, oracle; ticketed work also a live issue |
| `measure-only` measurement / investigation / review / audit | frozen ticket or one-shot `intent` + oracle; no design input required |

The profile states the command surface twice, deliberately. `project_permissions`
is the complete persistent Project policy — stable, broad enough for normal work
on that project, excluding destructive capabilities; `grant` installs it and
`doctor` compares it against the live surface. `task_commands` is the
ticket-local exact shell allowlist plus explicit deny probes; it narrows the
prompt without changing Project settings, and `verify` audits every
post-snapshot request against it.

`mode` is `measure-only` by default. Enumerate exact writable paths — never
directory globs — and set `path_change_budgets` when a path should receive only
a small localized change. `intent` reaches the worker verbatim alongside the
oracle, so it is a third document able to disagree with the other two: write it
after the oracle and as a summary of it. An intent drafted first survives the
edits that fixed the oracle, and the worker then satisfies the contradiction.

Size the ticket so every required witness fits in the terminal report. Split a
large exact-set inventory before dispatch if admitted plus discarded rows
cannot be printed completely: a digest and total do not replace the rows needed
to recompute them.

[references/profile-template.json](references/profile-template.json) and
[references/one-shot-profile-template.json](references/one-shot-profile-template.json)
are the field reference for anything the generator does not emit.

## `worktree` — derive the worker's checkout

Creates `agy/KEY` at `<parent>/.agy-worktrees/<repo>-KEY`, rebinds the AGY
Project to it, and writes `root`, `agy_project_id`, and the frozen
`worktree.base_sha` back into the profile. Every later verb then operates on
the worker's checkout with no further arguments.

```bash
python3 $S worktree profile.json KEY
```

| Refuses | Recover by |
|---|---|
| invalid task key | use the issue id, or the profile's `run_id` |
| profile missing `controller_root` | regenerate the profile with `--root` |
| `controller_root` is not a git checkout | point `--root` at the checkout, not its parent |
| branch outside the `agy/` namespace | leave the branch name to the verb |
| worktree path inside `controller_root` | leave the path to the verb |
| path exists on another branch | `discard` the previous round first |
| no single AGY project for this root | register the work area once with `agy --new-project`, then set `agy_project_id` |

**Does not refuse.** A round cut from a `HEAD` that is missing your uncommitted
work. The worktree is cut from **`HEAD`**, so anything uncommitted is invisible
to the worker: commit or stash the design input, the fixture, the file being
extended, before running this. Otherwise `doctor` fails on a missing design
input — or worse, the worker silently builds against the committed version of a
file you had already changed.

**Done when** the profile carries a derived `root` and the branch exists.

Two consequences of *moving* the Project binding rather than cloning it:
`agy --project <id>` forces the worker's working directory to the Project's
registered folder and ignores the caller's cwd, so reaching a derived worktree
means moving that binding. Rounds inside a work area are therefore **serial** —
raise throughput with work areas that have their own registered Project — and
you must **always `discard`**, since an interrupted round leaves the shared
Project pointing at the round's worktree and an unrelated `agy` session would
open there. Cloning would leave one registry entry and one un-updatable
permission copy per round; only `projectResources` changes, so the reviewed
permission surface is the same object before and after.

## `grant` — install the Project permission set

```bash
python3 $S grant profile.json
```

| Refuses | Recover by |
|---|---|
| no grants baseline recorded | run `worktree` first, so `discard` can restore the Project |
| `project_permissions` do not cover the round's own `task_commands` | widen `project_permissions`, regenerate, re-run |

**Does not refuse.** Being skipped. `discard` restores the Project's grants
baseline, so the next round starts without the previous round's grants and
`doctor` reports every task command as `expected allow but resolves ask`.
`grant` after every `discard`.

**Done when** the live grants match the profile.

## `doctor` — permission preflight

Compares the live Project surface exactly with `project_permissions`,
read-only. Blocks on live Project drift, an inherited Global rule that widens
the worker past the declared surface, an unresolved ticket command, an inert
sandbox escape, or a project/root mismatch.

```bash
python3 $S doctor profile.json
```

**Does not refuse.** A profile that both authorizes and freezes the same path.
The check exists and cannot fire — `load_profile` absolutizes the protected
paths and leaves the write list repo-relative, so the intersection is empty for
every profile (#3439). Until that lands, the only defence is regenerating
rather than deriving.

Also read the `unsandboxed` column as a decision, not a note. A command runs
sandboxed unless a paired `unsandboxed(...)` allow rule matches it, and escape
is consulted only after the command already resolved to `allow` — so an
`unsandboxed(P)` rule with no `command(P)` twin can never fire, and `doctor`
blocks on it. Whether a command *needs* the escape is a judgement the profile
cannot state, so it is reported per command instead of guessed: `cargo` needs
it, `rustfmt` on a file in the worktree does not, and a sandboxed `cargo` fails
in a way that reads like a product defect.

**Done when** it prints `dispatch_ready: true, blockers: []`.

Project, Shared, and Global rules are additive and deny/ask can override an
allow, so an unnoticed Global rule destroys project isolation. Add
`project_permissions.require_empty_global=true` by hand for a round that must
refuse *any* inherited Global rule; the generator leaves it off, because a flag
you switch off every round teaches you to click past the finding standing next
to it. Project rules use AGY's token-prefix matching, so a reusable
`command(rg)` Project allow can cover multiple tickets, while ticket commands
stay full, byte-exact lines — a pipeline, reordered flag, alternate quote form,
or narrower command is a *different* ticket command.

## `scaffold` — write both documents as forms

Writes the oracle and the injection as blank forms and prefills what the
profile already knows: the gate command and the frozen design inputs.

```bash
python3 $S scaffold profile.json KEY
```

- **Oracle** — `state_dir/oracles/<task-key>.md`
- **Injection** — the profile's `inject_prompt_file`

**Each slot carries its own rule**, as a `<!-- fill -->` comment. Those
comments are controller-facing and never reach the worker — `render_prompt`
strips every HTML comment from both documents — so a rule's rationale cannot
arrive as an instruction, and a hint can be as long as it needs to be at no
cost to the worker. **Read the form, not this file, when authoring.** Nothing
about how to fill a slot is repeated here; a second copy is the one that
drifts.

Re-running mid-round is safe: `scaffold` never overwrites an authored file.

**Does not refuse.** A form filled from memory. Every structural check `lint`
has still passes on a document whose content is invented, and the two failure
modes it cannot see are named in the slots that own them.

**Done when** no `<!-- fill -->` slot remains and `lint` is clean. An injection
is optional — a measure-only round can carry its instruction in the oracle —
but a *declared* `inject_prompt_file` must exist and conform.

## `capture` — ground a transcript in a run

Runs a command, stores what it printed under the round's task key, and prints
the ```console block to paste into `## Current behavior`. `lint` compares every
transcript in that section against these records.

```bash
python3 $S capture profile.json KEY \
  'AW_FIXTURE_LOCAL_BACKEND=1 aw health --project probe | tail -1' [--cwd DIR]
```

Runs in the profile's `root` unless `--cwd` names somewhere else — a temporary
fixture usually lives outside the checkout. Stores command, directory, exit
code, and output in `state_dir/transcripts/<task-key>.json`; re-capturing the
same command replaces its record rather than appending. Nothing about the
command is authorized by this: `capture` is a controller tool, and the round's
`task_commands.allow` still decides what the *worker* may run.

| Refuses | Recover by |
|---|---|
| capture directory does not exist | create the fixture first, or drop `--cwd` |

**Does not refuse — and these cost a `lint` round each:**

- **A command written across several lines.** `lint` reads every `$ ` line of a
  fence as a command; continuation lines are read as output and never match the
  record. Capture one single-line command per fence.
- **A fence missing the run's trailing noise.** The comparison is byte-exact
  against what was recorded, including a shell's `Alarm clock:` line. Paste the
  stored record, do not retype the interesting part.
- **A capture whose command differs from what the block shows** — an explicit
  `target/debug/aw` path pasted as `aw`, an absolute fixture path shortened.
  The record is what actually ran; the block is what the worker reads. State
  the difference in the round evidence.

This is the injection's one section the source-quote rule cannot reach — there
is no file to find those lines in — so a paraphrase there passes every other
check. #3426 is the round where two shipped: one naming a flag the verb does
not accept, one whose behaviour existed only in a build newer than the
installed binary. Both were true *observations*; neither was a run.

**Done when** `lint` reports no transcript finding.

## `lint` — structural checks

Enforces both documents' structure, carrying no project knowledge. `dispatch`
runs the same checks and refuses on any finding.

```bash
python3 $S lint profile.json KEY
```

The table below is a catalogue of what fails, not of how to write the slot —
when a check fires, the rule for repairing it is in the slot comment the
finding names. The second column is the part that is only here: what each check
is buying, which is what tells you whether a repair kept it.

| Check | What it prevents |
|---|---|
| `## Measurements` has ≥2 rows, ≥1 marked `negative control` | a table an unchanged implementation also satisfies |
| the control is marked in a row's input or observation, not its rationale | prose *about* a control counted as *having* one |
| `## Gate` commands ⊆ the task allowlist | a gate the worker is not authorized to run, so nobody runs it |
| `## Gate` names only `task_contract.gate_command` | a second command that reads as judged when it is only authorized |
| `## Definition of done` ≡ the oracle's `## Gate` | instruction and judgement drifting apart, each satisfiable alone |
| `## Current behavior` has a non-empty fenced quote | a round authored from memory rather than from the checkout |
| every quoted line appears in one of the round's files | a quote that was true at an earlier base and is now fiction |
| every ```console block was produced by `capture` | a transcript nobody ran, or one edited after its run |
| no fenced block outside `Current behavior` / `Definition of done` | pasting the implementation, which leaves the worker nothing to derive |
| no numbered steps in `Required change` / `Shape to follow` | a recipe to retype instead of a requirement to satisfy |
| `## Shape to follow` names a backticked symbol, within its line budget | a free-prose slot growing into the design the round was meant to buy |
| `## Definition of done` names where the check lands | a correct diff arriving in the wrong module |
| backticked paths resolve under `root` | stale coordinates reaching the worker |
| no `<!-- fill -->` slots remain | a form dispatched before it was written |

**Repair from the right half of a pair.** Three checks come in pairs and the
wrong repair satisfies the check while destroying what it protects:

- The **gate** pair asks separately whether the worker *may* run a command and
  whether `prove` *will*, since `prove` runs `task_contract.gate_command`
  alone. The remedy for a compound gate is to name it in the profile, not to
  widen the fence.
- The **control** pair asks whether the table has a control at all and whether
  the marked row *is* one.
- The **`Current behavior`** pair asks whether you opened the file and whether
  you opened it *at this round's base*. Rounds get re-based constantly here, so
  a quote goes stale with nobody editing it. A console block answers to
  `capture` instead: the remedy for a transcript finding is to run the command,
  never to reword the block.

**Does not refuse.** A row stating what you assumed the code does rather than
what it does. Structure and truth are indistinguishable to `lint` and fail
differently at dispatch — a true row the worker satisfies; a false one it can
neither satisfy nor refuse, and what comes back is that row asserted against
something that does hold.

**Done when** it exits `0`. The gate cross-check is skipped when the round
grants no shell: a measure-only oracle names what *you* will run.

## `snapshot` — freeze

Records the sha256 of both documents plus Git state, protected bytes, the
frozen dispatch contract, AGY project identity, the conversation step floor,
and a digest of Project plus Global permissions.

```bash
python3 $S snapshot profile.json KEY
```

**Does not refuse.** Being taken before the documents were finished. After it,
`dispatch`, `resume`, and `verify` each VOID on a mismatch — including one that
*adds* an injection the round did not have. You are the only party able to edit
these two files while a worker runs, which is exactly why they are frozen: an
oracle that can be softened to fit the answer it received is not an oracle.

Git state is captured with `--untracked-files=all` at both `snapshot` and
`verify`. Without it, porcelain collapses a never-tracked directory to a single
`dir/` entry, which can never match an exact `allowed_repo_writes` path and
misreports every greenfield dispatch. Because of that flag a round creating a
brand-new source tree is verified path by path exactly like one editing tracked
files: keep listing exact files, never a directory.

**Done when** the snapshot is written. Record the oracle's SHA-256 in the issue
comment for ticketed work or the controller log for one-shot work.

## Baseline check — no verb, still mandatory

Proves the worker's checkout is green *before* the worker touches it, and
leaves a warm `target/` behind so the round's timeout pays for reasoning rather
than a cold build. A worker dispatched onto a red base spends its whole budget
on a failure it did not cause, then reports it as one it could not fix.

```bash
cd <worktree> && <one landed gate from the allowlist>
```

One gate that already passes at this base — not the round's own gate, which
does not exist yet. Take it verbatim from `project_permissions.allow`. Running
the whole allowlist here burns the round's wall-clock before it starts, and
widening past the allowlist invites a hang this round does not own.

**Done when** that gate reports `test result: ok` and `target/` exists in the
worktree.

## `dispatch` / `resume` — run the worker

Runs AGY against the frozen contract, passing only `--project <id>` — no
`--add-dir`, no settings mutation. Stores the rendered prompt, AGY log, raw
output, and normalized final `## EXEC REPORT` under `state_dir/runs/`.

```bash
python3 $S dispatch profile.json KEY
```

| Refuses | Recover by |
|---|---|
| no oracle | `scaffold`, then fill |
| the injection does not satisfy its contract | run `lint` and repair each finding |
| **the task key already has a conversation** | ticketed: `resume`. One-shot: `revise` — **not** a fresh round, see below |
| AGY exited non-zero | `denied`, verify the snapshot, and change persistent Project policy only when the command is a reusable capability |
| empty local report | inspect the AGY log and the repository diff |
| no valid terminal `## EXEC REPORT` | same; progress chatter may precede the real report |
| missing conversation id | the run cannot be audited; treat as a failed delivery |
| **timed out at the profile's `timeout`** | not a denial — the worker was cut off with its work on disk. Read the worktree diff and run the gate before deciding to redispatch |

The conversation refusal is the one that strands a controller. Its text says
only "create a new one-shot run id", which reads as an instruction to author a
fresh round. **Do not**: the candidate is uncommitted in the very worktree
`worktree` would re-create from `HEAD`, so that path silently deletes it. Use
`revise`, which mints the id and carries the tree.

**Does not refuse.** Being backgrounded wrongly. Use the Bash tool's
`run_in_background` — never `nohup ... &`, which returns immediately, orphans
the run from the harness, and then needs a second polling watcher to notice an
exit the harness would have reported for free. Never wrap these in an
orchestration helper that can return before the nested subprocess finishes; an
early empty wrapper result is not an AGY report. Every other verb returns in
seconds; these two run for the profile's whole `timeout`, so size it to the
round rather than reusing the last one.

Before any `resume`, run `verify` and confirm the snapshot and protected
artifacts; re-`snapshot` if either contract changed. A transient backend 5xx may
resume without a contract change. Never resume after a VOID, without a stored
conversation id, or for another ticket.

**Done when** a terminal `## EXEC REPORT` was filed. Process exit is never
acceptance.

## `status` — which failure a run hit

Prints one verdict per stored run log: `DENIED` a command was auto-denied,
`EMPTY` the log has nothing in it, `REPORTED` a terminal `## EXEC REPORT`
parsed, `INVALID REPORT` there is output but no parseable report. It is the
router for a failed `dispatch` — it says which row of that verb's refusal table
you are in before you go reading logs.

```bash
python3 $S status profile.json
```

**Does not refuse.** Anything, and `REPORTED` is the one to distrust. The
verdict is a scan of the log text, so it says a well-formed report exists — not
that anything in it is true. `verify` and `review` are what judge a
`REPORTED` run.

## `denied` / `abandon` — the two side exits

`denied` triages a soft-denied command. `abandon` releases a run id after an
infrastructure death — an upstream 5xx, a killed host process — so `snapshot`
and `dispatch` can run again.

```bash
python3 $S denied  profile.json KEY
python3 $S abandon profile.json KEY
```

| Refuses | Recover by |
|---|---|
| `denied`: no conversation id, or missing database | there is nothing to triage; read the AGY log |
| `denied`: no denied `run_command` payload | the denial was not a soft one; read the AGY log |
| `abandon`: no conversation recorded | the id is already free — just `dispatch` |
| `abandon`: the worker changed paths | judge it with `review` and `verify`; a candidate is never released |
| `abandon`: the worker ran commands | same — effects may reach outside the checkout |
| `abandon`: an `## EXEC REPORT` was filed | run `status` and judge what it claims |

**Does not refuse.** Being reached for as a retry. Never grant a temporary
ticket permission or a bypass-permissions flag to clear a denial: if the
command is unnecessary, tighten the prompt and resume; if it is a reusable
project capability, add one narrow Project-scope rule through `/permissions`,
update the profile's exact `project_permissions`, rerun `doctor`, and take a
fresh snapshot. Never `abandon` a result you dislike.

**Done when** `abandon` accepts — which it does only if the run provably
produced nothing. A refused request started no process, so it does not strand
the run id. The dead attempt's logs are parked under `runs/abandoned/` rather
than deleted: they are evidence, and deleting them would let the id lookup
recover the dead conversation.

## `verify` — integrity and scope audit

Compares post-run Git state and protected artifact hashes against the
pre-dispatch snapshot, audits AGY's post-snapshot command requests from the
conversation database against the exact `task_commands` allowlist, and applies
`path_change_budgets` to bound added plus removed lines per writable path.

```bash
python3 $S verify profile.json KEY
```

**Integrity failures exit `1` (VOID)** — permission drift, a command that *ran*
outside the audited allowlist, a request for a command the profile names in
`deny`, a changed dispatch contract, a swapped Project, a missing oracle. The
evidence cannot be trusted at all.

**Scope problems exit `2` (findings)** — a write outside `allowed_repo_writes`,
a changed protected artifact, a declared path left unwritten, a budget overrun,
a moved branch `HEAD`, a command the permission layer *refused*. These go to
`review` for you to adjudicate against the diff.

An unlisted command splits on whether it ran. The VOID protects the evidence:
if an unaudited command executed, the tree may hold state nobody authorized and
nobody can reconstruct. A refused request left none, so it is a finding — the
guard held, and discarding would punish the intention while throwing away the
candidate. The audit reads the outcome row, not the request row: every request
carries the same status, so the request alone cannot tell the two apart
(#3427). An outcome never written — a process killed mid-command — reads as
*ran*, because the conservative direction for an unknown is the fatal one. That
split is sound only because the worker's tree is disposable; a profile with no
`worktree` key keeps the old behavior of voiding on scope.

**Does not refuse.** Anything about what the diff *says*. `verify` answers
"were the rules kept", never "is the change right", and three gaps follow from
that:

- **A generated mirror left stale.** A `SPEC-MANAGED` block edited in the `.rs`
  and not in its tech-design source compiles, tests green, and passes every
  hash check here, because both files are writable in the contract. The next
  generation silently reverts the round. Diff the pair at `review`.
- **An allowed command with effects outside the checkout.** The audit asks
  whether a command was on the allowlist, not what it touched. Keep the
  allowlist to commands whose blast radius is the worktree.
- **A declared path written with something irrelevant.** Budgets bound how many
  lines moved, never which.

**Done when** you have read every finding it names. The worker never runs a git
mutation — a moved branch `HEAD` is how a worker commit surfaces.

## `review` — adjudicate the diff

Prints the round's branch and base, the paths this revision wrote with `!` on
anything outside `allowed_repo_writes`, the paths carried forward from an
earlier revision, every finding, `git diff --stat`, the full diff, and the body
of each new untracked file; exits `2` if there is a finding.

```bash
python3 $S review profile.json KEY
```

On a revised round `touched` and the diff answer different questions.
`touched` is what *this* worker wrote, which is what a scope finding is about;
`carried` is the rest of the candidate, which `revise` kept uncommitted across
the new run id. The diff below both is always the whole candidate against the
frozen base, and that is what `accept` commits.

**Does not refuse.** Anything. It prints and exits — every judgement here is
yours. The worker's report is a claim *about* the diff, not the diff. A finding
is a question, not a verdict: an out-of-contract write can be a worker that
misread the scope, which you send back, or a contract too narrow for the work,
in which case widen `allowed_repo_writes` next round rather than blaming the
diff. Treat every report `PASS` as provisional — a claim that formatting is
clean or an excluded form is unchanged needs controller evidence such as
`git diff --check` plus focused negative controls.

**Done when** every finding has an adjudication and the diff conforms to the
oracle row by row.

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

Runs the round's gate over the worktree exactly as it stands and files the
result under the label you give it. **`prove` reverts nothing** — the reverting
is yours.

```bash
# restore the product change to the round's baseline, keeping the worker's tests
python3 $S prove profile.json KEY mutant
# restore the candidate
python3 $S prove profile.json KEY candidate
```

| Refuses | Recover by |
|---|---|
| a label other than `mutant` / `candidate` | use one of the two |
| (at `accept`) a mutant that passed, a candidate that failed, two proofs over an identical tree, a candidate proof taken before the tree moved | revert properly between the two runs |

**Does not refuse.** A revert that silently reverted the wrong thing. Guard it
with assertions anchored on the declaration the round extends rather than a
bare identifier — a file large enough to be worth a round usually already
carries that name for something else. When such a guard fires, read the
namesake before loosening it: two things in one file answering to one name is
either the duplication the round was supposed to avoid or a second
implementation of the same rule that will drift from the first, and both are
worth a work item before this round lands.

A build failure is not a behavioural kill. When the round introduces a new
symbol, the reverted tree cannot compile, so the gate goes red because the
function it names does not exist yet and nothing about behaviour was measured.
That is the only answer the revert can give for such a round, so it does not
block — `prove` and `accept` both say so.

**Done when** the pair discriminates *and* a mutation sweep backs it. A gate
nobody has seen fail proves nothing: a test written against the implementation
just produced passes by construction. The pair is a floor, not a ceiling.

## `sweep` — put the sweep in the round record

Runs your mutation script over the worktree and files its text, its output, its
exit code, and whether the tree digest survived it. Run it after
`prove candidate`, so a script that fails to restore what it mutated shows up
as a moved digest rather than as the next round's mystery.

```bash
python3 $S sweep profile.json KEY /path/to/mutate.py
```

| Refuses | Recover by |
|---|---|
| the script does not exist | pass an absolute path |
| the tree digest changed across the sweep | the script did not restore what it mutated; every result after the first was measured against a corrupted tree — repair and re-run |
| (at `accept`) a recorded sweep that exited non-zero, or none at all | write the script to exit non-zero when a mutant misses its expected verdict |

Discrimination comes from one single-defect mutant per rule the oracle claims,
each expected to be killed by a named row, each keeping the product compiling.
Writing that script is the part with no verb behind it:

- **A mutant that fails to build is a badly written mutant, not a kill.** Repair
  it by making that same defect representable — naming a type restores a build
  that inference lost — never by reaching for a different defect that happens
  to compile. Substituting quietly changes which row is under test.
- **Apply *and* restore by writing the whole file.** A copy that preserves an
  older mtime lets cargo skip the rebuild, which turns the mutant into a false
  kill and the next candidate run into a false green.
- **Classify by whether the harness ran**, not by searching output for the word
  "error". Build tools announce an ordinary assertion failure in the same
  vocabulary they use for a broken build — cargo prints `error: test failed` —
  so a scan for that word reports every real kill as malformed, and a sweep of
  nine kills reads as a sweep that measured nothing.
- **A survivor is a finding against the round's evidence, not the product.**
  The code may be right and simply have no row that would notice if it stopped
  being right. Close it by authoring the missing row in a round, never by
  editing the test yourself.
- **A round that only adds evidence has nothing to revert**, so the pair
  degenerates into two runs over one tree, which `accept` refuses as identical
  digests. Take the mutant from the **unchanged** product instead: break the
  rule the new rows are supposed to pin. That is also the proof that catches
  the round which performs its deletions and skips its additions.
- **An expected survivor is a legitimate row** when it probes redundancy the
  product genuinely has. Declare it in the script rather than dropping it, so
  the sweep's own count stays honest.

**Does not refuse.** A sweep kept in a scratch directory. It would be the
strongest claim in the round's evidence and the only one nobody else can
re-run, which inverts exactly the wrong way.

## `accept` / `revise` / `discard` — the three outcomes

`accept` stages the whole candidate — every path the worktree changed since its
frozen base, across every revision — and commits it on the round's branch (with
`Refs #<issue>` for ticketed work), then prints the `git cherry-pick <sha>` for
you to run from `controller_root`. `revise` sends a one-shot round back.
`discard` restores the Project's home root and removes the worktree and branch.

```bash
python3 $S accept  profile.json KEY
python3 $S revise  profile.json KEY NEXT_KEY delta.md
python3 $S discard profile.json KEY [--keep-branch]
```

| Refuses | Recover by |
|---|---|
| `accept`: the worker changed no files | there is nothing to accept; `discard` |
| `accept`: the gate is not shown to discriminate | run the proof pair and a `sweep` |
| `revise`: the same run id | pass a new `NEXT_KEY`; `revise` mints nothing for you |
| `revise`: profile root is not the worker checkout | there is no round in progress — author a fresh one |
| `revise`: the worker changed nothing | nothing to carry forward; a fresh round costs no more |
| `revise`: the delta injection does not exist | write it first |
| `revise`: the target run id is already taken | choose one not in flight |
| `revise`: the round has no oracle | there is no sealed claim to inherit |
| `discard`: no derived worktree | nothing to release |

To send a round back, write the delta contract to a file (next section), then:

- **ticketed** — point `inject_prompt_file` at it, re-`snapshot` if the
  contract changed, and `resume`.
- **one-shot** — `revise`. It changes two things and carries everything else —
  root, worktree, policy, protected artifacts, budgets — so the revision is
  measured against the same tree under the same ceiling, and it copies the
  oracle unchanged, because a revision exists to satisfy the sealed claim, not
  to move it. Then `lint`, `grant`, `doctor`, `snapshot`, `dispatch` on the new
  key.

Either way the worker's checkout persists across the revision, so the next
round builds on the same tree instead of starting over. Never leave a round
undiscarded; `--keep-branch` throws away the checkout but keeps the candidate
commit.

**Does not refuse.** Expecting `accept` to merge. It commits on the worker's
branch and stops; integration and your gates run against *your* branch, and
nothing here reaches it until you run the printed `git cherry-pick`.

**Done when** the candidate is cherry-picked into your branch and `discard` has
restored the Project's home root. `discard` also restores the grants baseline,
so the next round must start at `grant` again.

## Writing the delta — the one form nothing scaffolds

`scaffold` emits the oracle and the injection, so their rules live in their
slots and this file says nothing about filling them. The delta has no skeleton:
`resume` reads whatever `inject_prompt_file` points at and `revise` takes the
path as an argument, so these four rules have nowhere else to live yet and are
stated here until a `DELTA_SKELETON` exists to carry them (#3442).

- **A revision report is a self-contained replacement, not a fragment.**
  Require it to preserve every already accepted identity, row, matrix,
  invariant, lifecycle boundary, planned path, forbidden change, and test seam
  while applying the correction, and diff the new normalized report against the
  prior accepted sections. A compact re-emission that silently drops evidence
  is a rejection.
- **Make list-edit intent explicit.** `append` when a new test or invariant
  must coexist with accepted entries; `replace <exact entry>` only when the old
  entry must disappear. "Add if needed" is not a contract when count or
  coverage is an acceptance surface.
- **Leave headroom on a size limit** rather than naming the exact ceiling: on
  the first oversize revision set a target at least 10% below the acceptance
  maximum and name both the removable structures and the exact tables and lists
  that must survive. Size compliance is an acceptance criterion, not a
  controller-side truncation step.
- **Own your own bad contract.** When the delta exists because *your*
  instruction was wrong, say so, quote the superseded line, and give the
  corrected rule with its authority. A worker that faithfully implemented a bad
  instruction has not erred, and a delta implying otherwise teaches it to
  distrust the frozen contract it is supposed to follow.

## Rounds with no shell

For a pure authoring round — write these files, to this design — an empty
`task_commands.allow` and empty `project_permissions.allow` give the worker no
shell at all. `doctor` still passes, and the dispatcher swaps its appended
report contract for the no-shell variant, which forbids `PASS`/`FAIL` and any
claimed observation and asks instead for a description of what was written.

**Does not refuse.** Leaving the verdict-shaped contract in place. The trailer
is appended *after* the injected delta, so a trailer demanding "PASS or FAIL
per criterion" overrides an injection saying "do not write PASS" and
manufactures the exact fabrication the oracle then has to catch. Match the
contract to the worker's actual capability and the tell becomes real: with an
empty allowlist, any test result, build outcome, or per-criterion verdict is
fabricated by construction, and `verify`'s audited command list proves it.

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
must agree are a check, one table two places read is an assumption.

## Interactive teamwork preview

A separate execution mode, not a headless dispatch. Run only when the user
explicitly requests it.

```bash
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py detect
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py launch profile.json teamwork-prompt.md
```

Never attempt it headless: it requires a real TTY, and `--print/-p` and
`--prompt-interactive/-i` are mutually exclusive. It does not use `--add-dir`
or `--dangerously-skip-permissions`. The launcher uses `expect` to start
`agy --project <id> --prompt-interactive` and sends the prompt; on macOS it
prefers splitting a pane in the current iTerm2 tab and otherwise falls back to
Terminal.app. The prompt must start with `/teamwork-preview` and name the
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
- Claude and Codex share one `~/.gemini` Project registry and are controllers,
  not separate permission namespaces, so two controllers must not run rounds
  against one work area concurrently.
- Keep one stable `/tmp/agy-dispatch/<project-id>/` `state_dir` per Project and
  treat it as transient — publish durable verdicts to the issue, and start from
  a fresh oracle and snapshot rather than resuming if `/tmp` was cleaned.
- The profile supplies project-specific build/test commands, exact binary
  paths, caller-selected design inputs, and the verification contract. This
  skill supplies the task-class minimums and permission isolation and nothing
  project-specific. Do not add a toolchain denial such as `cargo` unless that
  ticket must not use it.
