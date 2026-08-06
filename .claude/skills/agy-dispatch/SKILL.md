---
name: agy:dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless AGY using persistent AGY Project permissions plus a task-local command/write contract, then independently verify its report. Use whenever Claude delegates audits, measurements, investigation, transcription, or tightly scoped implementation to AGY; ticketed tasks reuse one conversation per live issue, unticketed tasks run once without resume, implementation requires frozen design inputs, and acceptance remains controller-only.
user-invocable: true
---

# AGY Dispatch

Treat headless AGY as a bounded worker, never as an autonomous owner. The
controller defines scope and oracle, dispatches one task per process, verifies
the result, and alone decides acceptance, publication, and closure.

The lifecycle is a worker-independent dispatcher core with an AGY adapter.
Read [references/lifecycle.md](references/lifecycle.md) completely before the
first dispatch or takeover, and whenever resuming from another controller's
handoff. It is the normative state machine for workspace binding, permissions,
contract freezing, snapshot, dispatch/resume, isolation verification,
controller acceptance, publication, and cleanup. A future worker adapter may
replace AGY only by preserving those states and evidence contracts.

For an explicitly requested AGY `/teamwork-preview` session, use the interactive-terminal path below instead. It is a separate execution mode, not a headless dispatch.

## Setup

1. Generate the profile rather than hand-writing it:

   ```bash
   python3 .claude/skills/agy-dispatch/scripts/make_profile.py \
       --root /abs/controller-checkout --repo owner/name --project-id <id> \
       --scope libs/thing --issue 1234 --inject /abs/delta-round.md \
       --design-input libs/thing/CAPABILITIES.md \
       --write libs/thing/src/a.py:40 --write libs/thing/tests/b.py:12 \
       --out /abs/profile.json
   ```

   `--root` is *your* checkout, recorded as `controller_root`; the round's own
   `root` is filled in later by `worktree`. For a one-shot round, `--run-id`
   also requires `--intent`.

   It freezes the **complement**: every regular file under `--scope` that the
   round may not write is protected with its current sha256, recorded
   repo-relative so the frozen set follows the round into its derived worktree.
   That is the direction that stays correct as a tree grows — an enumerated
   protected list silently stops covering files added since the last round, and
   `verify` then cannot distinguish a stray write from an intended one. A
   `--write` path need not exist yet, so greenfield and edit rounds are
   expressed identically. The
   templates ([references/profile-template.json](references/profile-template.json),
   [references/one-shot-profile-template.json](references/one-shot-profile-template.json))
   remain the field reference for anything the generator does not emit. Keep the
   profile outside the repository if it carries local paths or mutable pins.
2. Set `mode` to `measure-only` by default. Use `bounded-write` only with explicit `allowed_repo_writes`, `task_contract.kind=implementation`, and at least one frozen `design_inputs` artifact. For a ticket, set `session_policy=ticketed` plus `issue`; for an unticketed task, set `session_policy=one-shot` plus a unique `run_id` and frozen `intent`.
3. Put project-specific selectors, denominators, expected bands, and fabrication tells in the ticket or one-shot oracle; the dispatcher injects the full oracle into AGY's prompt.
4. For a revision or cleanup wave, write the exact delta contract to a file and set `inject_prompt_file`. Protect already accepted implementation paths and narrow `allowed_repo_writes` to only the files that round may change.
5. Review [references/report-verification.md](references/report-verification.md)
   before accepting any report. For inventory, classification, denominator, or
   regional-footprint work, also review
   [references/inventory-verification.md](references/inventory-verification.md).
6. Size the ticket so every required witness fits in the terminal report. Split
   a large exact-set inventory before dispatch if admitted plus discarded rows
   cannot be printed completely; a digest and total do not replace the rows
   needed to recompute them.

Task-class minimums are controller-owned; design-artifact selection is
caller-owned:

- `bounded-write` implementation: explicit authority + at least one
  caller-selected frozen design input + oracle; ticketed work additionally
  requires a live issue.
- `measure-only` measurement/investigation/review/audit: frozen ticket or
  one-shot intent + oracle; no design input required.
- The caller/profile chooses each design input and path. Project-specific
  conventions belong in that caller/profile or a separate project policy,
  never in this global skill.

## Dispatch protocol

Run the controller script from any directory:

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py worktree profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py doctor profile.json
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py scaffold profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py lint profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py snapshot profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py dispatch profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py verify profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py review profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py resume profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py abandon profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py accept profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py discard profile.json ISSUE
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py status profile.json
```

The loop is `worktree → doctor → scaffold → (fill) → lint → snapshot → dispatch
→ verify → review`, then either `accept` or a `resume` revision and back to
`verify`. `discard` releases the round. Exit codes are `0` clean, `1` VOID, `2`
findings for the controller to adjudicate.

Every verb returns in seconds except `dispatch` and `resume`, which run for the
profile's whole `timeout`. Start those two with the Bash tool's
`run_in_background`, which keeps the process tracked and wakes the controller
when it exits. Do not background them by hand with `nohup ... &`: that returns
immediately, orphans the run from the harness, and then needs a second polling
watcher to notice an exit the harness would have reported for free. Never wrap
them in an orchestration helper that can return before the nested subprocess
finishes — an early empty wrapper result is not an AGY report.

Size `timeout` to the round rather than reusing the previous one. A worker cut
off at its deadline exits non-zero with its work already on disk and no
`## EXEC REPORT`, so the round reads as a failure while the candidate is
complete. `dispatch` names that case separately from a denial; when it does,
read the worktree diff and run the gate before deciding whether to redispatch.

For an unticketed one-shot profile, use its explicit unique `RUN_ID` in place
of `ISSUE` for `snapshot`, `dispatch`, and `verify`. Never call `resume` for a
one-shot run. The dispatcher rejects both a resumed one-shot and a second
dispatch under the same run id.

When a dispatch dies on infrastructure — an upstream 5xx, a killed host process
— run `abandon` to release the run id, then `snapshot` and `dispatch` again.
It succeeds only when the run provably produced nothing: no path changed in the
worker checkout, no command requested after the snapshot floor, no
`## EXEC REPORT` filed. A round that produced any of those is judged with
`review`/`verify`, never released, so `abandon` cannot be used to retry away a
result the controller dislikes. The dead attempt's logs are parked under
`runs/abandoned/` rather than deleted — they are evidence, and leaving them in
place would let the id lookup recover the dead conversation.

### Author the round from the scaffold, not from memory

`scaffold` writes the round's two documents as blank forms and prefills what
the profile already knows — the gate command and the frozen design inputs. Fill
the slots, then `lint`. It never overwrites an authored file, so re-running it
mid-round is safe.

- **Oracle** — `state_dir/oracles/<task-key>.md`. `## Claim`,
  `## Measurements`, `## Gate`, `## Fabrication tells`.
- **Injection** — the profile's `inject_prompt_file`. `## Task`,
  `## Current behavior`, `## Required change`, `## Shape to follow`,
  `## Reference`, `## Out of scope`, `## Definition of done`.

`snapshot` records the sha256 of both, and `dispatch`, `resume`, and `verify`
each VOID on a mismatch. Finish authoring before snapshotting: an edit after
that point is refused, including one that adds an injection the round did not
have. The controller is the only party able to edit these two files while a
worker runs, which is exactly why they are frozen — an oracle that can be
softened to fit the answer it received is not an oracle, and a hash that is
printed but never compared reads like a freeze without being one.

The injection answers four questions in order: what to do, from what starting
point, against what constraint, and how it will be judged. It deliberately does
not answer *how*. A round is only worth dispatching if the worker still has the
design left to do, so `## Shape to follow` is capped and may only point at a
convention already in the tree — an existing function or error shape to match
rather than a second one to invent. The `Current behavior` quote is the other
end of the same discipline: it grounds the round in what was read instead of
what was remembered. Everything the worker needs but the round does not choose —
the write allowlist, the command allowlist, the stop-and-report rule, the report
shape — is already in the static prompt and must not be restated here, where the
two copies would drift.

Record the oracle's SHA-256 in the issue comment for ticketed work or the
controller log for one-shot work.

`lint` and `dispatch` both enforce the structure; `dispatch` refuses on any
finding. The checks are structural and carry no project knowledge:

| Check | What it prevents |
|---|---|
| `## Measurements` has ≥2 rows, ≥1 marked `negative control` | a table an unchanged implementation also satisfies |
| `## Gate` commands ⊆ the task allowlist | a gate the worker is not authorized to run, so nobody runs it |
| `## Definition of done` ≡ the oracle's `## Gate` | instruction and judgement drifting apart, each satisfiable alone |
| `## Current behavior` has a non-empty fenced quote | a round authored from memory rather than from the checkout |
| no fenced block outside `Current behavior` / `Definition of done` | pasting the implementation, which leaves the worker nothing to derive |
| no numbered steps in `Required change` / `Shape to follow` | a recipe to retype instead of a requirement to satisfy |
| `## Shape to follow` names a backticked symbol, within its line budget | a free-prose slot growing into the design the round was meant to buy |
| `## Definition of done` names where the check lands | a correct diff arriving in the wrong module |
| backticked paths resolve under `root` | stale coordinates reaching the worker |
| no `<!-- fill -->` slots remain | a form dispatched before it was written |

The gate cross-check is skipped when the round grants no shell: a measure-only
oracle names what the *controller* will run.

An injection is optional — a measure-only round can carry its instruction in
the oracle — but a declared `inject_prompt_file` must exist and conform, since
that is the half through which the last false green entered.

AGY Projects are the persistent permission unit. Register the work area once
with `agy --new-project`, then use `/permissions` with **Project** scope to
configure its reusable allow/deny/ask policy. Do not edit AGY's JSON files by
hand and do not rewrite permissions per ticket. The one sanctioned write is
`worktree`/`discard` moving that Project's `projectResources` between the work
area and the round's checkout; grants are never touched, which is why the
permission digest is stable across a round.

### Derive the worker's worktree from your own

**The worker gets its own checkout on its own branch. Run `worktree` before
`doctor`, and never dispatch into the tree you are working in.**

The unit is: one project → one persistent AGY Project → one `state_dir` → and
one *disposable* worktree per round, branched from the controller's current
`HEAD`.

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py worktree profile.json 3348
```

That single command creates `agy/3348` at
`<parent>/.agy-worktrees/<repo>-3348`, rebinds the AGY Project to it, and
writes `root`, `agy_project_id`, and the frozen `worktree.base_sha` back into
the profile. Every later verb then operates on the worker's checkout without
further arguments.

The profile needs `controller_root` (your checkout). `agy_project_id` is
resolved from the registry when exactly one Project maps to that root, and must
be authored when more than one does. `root` is derived, never authored.

**The round is cut from `HEAD`, so your uncommitted work is invisible to the
worker.** Commit or stash anything the task depends on — a design input, a
fixture, the file being extended — before running `worktree`. Otherwise
`doctor` fails on a missing design input, or worse, the worker silently builds
against the committed version of a file you had already changed.

**One AGY Project per work area, moved — never cloned.** `agy --project <id>`
forces the worker's working directory to the Project's registered folder and
ignores the caller's cwd, so reaching a derived worktree means moving that
binding. `worktree` moves it and records the home root; `discard` moves it
back. Only `projectResources` changes, so the reviewed permission surface is
the same object before and after and cannot drift from a stale copy. Cloning a
Project per round would leave one registry entry and one un-updatable
permission copy per round; do not do it.

Two consequences follow from moving rather than cloning:

- **Serial rounds per work area.** One Project points at one root at a time, so
  a project runs one round at a time. Parallelism comes from work areas that
  already have their own Project. This matches the pre-existing bounded-write
  rule; it is not a new restriction.
- **Always `discard`.** An interrupted round leaves the shared Project pointing
  at the round's worktree, so an unrelated `agy` session would open there.
  `discard` restores the home root first, before it removes anything, and
  re-running `worktree` re-binds cleanly, so a stale pointer self-heals.

Apply these rules:

- Register each work area once with `agy --new-project`, then configure its
  reusable allow/deny/ask policy through `/permissions` at **Project** scope.
  Put that id in `agy_project_id` and never change it per round.
- Worker branches are namespaced `agy/*` and worker checkouts live outside
  `controller_root`. Both are enforced: `worktree` refuses any other branch
  prefix, and refuses a path nested inside the controller's tree.
- Claude and Codex share the same `~/.gemini` Project registry; they are
  controllers, not separate permission namespaces. Because the Project's root
  now moves per round, two controllers must not run rounds against one work
  area concurrently.
- Keep one stable `/tmp/agy-dispatch/<project-id>/` `state_dir` per Project.
  Treat it as transient controller state: publish durable verdicts to the
  issue, and start from a fresh oracle/snapshot instead of resuming if `/tmp`
  was cleaned.
- The worker still never runs a git mutation. `verify` reports a moved branch
  `HEAD` as a finding, which is how a worker commit surfaces.
- Do not hand-create worktrees outside this command. Ad-hoc `git worktree add`
  is what produced hundreds of stranded checkouts and orphaned branches; the
  round-scoped `worktree`/`discard` pair exists to keep that bounded.

The profile separates two contracts:

- `project_permissions` is the complete, persistent Project policy. Keep it
  stable and broad enough for normal work on that project, but exclude
  destructive capabilities. `doctor` compares the live Project surface
  exactly with this policy and is read-only.
- `task_commands` is the ticket-local exact shell allowlist plus explicit
  deny probes. It narrows the prompt without changing Project settings.
  `verify` audits AGY's post-snapshot command requests from the conversation
  database and voids any command not copied exactly from this allowlist.

Project, Shared, and Global rules are additive, and deny/ask rules can override
an allow, so an unnoticed Global rule destroys project isolation. `doctor`
blocks on live Project drift, an inherited Global rule that widens the worker
past the declared surface, an unresolved ticket command, an inert sandbox
escape, or a project/root mismatch. It never changes any permission surface.
Add `project_permissions.require_empty_global=true` by hand for a round that
wants to refuse *any* inherited Global rule, including the harmless denies most
setups carry; the generator leaves it off, because a flag the controller flips
off every round teaches it to flip past the finding standing next to it.

A command runs sandboxed unless a paired `unsandboxed(...)` allow rule matches
it. Escape is consulted only after the command has already resolved to `allow`,
so an `unsandboxed(P)` rule with no `command(P)` twin can never fire and
`doctor` blocks on it. The opposite direction is a judgement the profile cannot
state — whether a command needs the network or writes outside the worktree —
so `doctor` reports `unsandboxed` per ticket command instead of guessing.
`cargo` needs it; `rustfmt` on a file in the worktree does not. A sandboxed
`cargo` fails in a way that reads like a product defect, so check that column
before dispatching a round with a build gate.

Project rules use AGY's token-prefix matching, so a reusable
`command(rg)` Project allow can cover multiple tickets. Ticket commands remain
full, byte-exact lines such as `rg -n 'EXACT_SELECTOR' src/file.rs`. Prefer
shell-safe selectors, one command per tool call, and the built-in read-file
tool for additional inspection. A pipeline, reordered flag, alternate quote
form, or narrower command is a different ticket command.

If AGY soft-denies a command, run:

```bash
python3 .claude/skills/agy-dispatch/scripts/agy_dispatch.py denied profile.json ISSUE
```

If the command is unnecessary, tighten the prompt and resume. If it is a
reusable project capability, add one narrow Project-scope rule through
`/permissions`, update the profile's exact `project_permissions`, rerun
`doctor`, and create a fresh snapshot before resuming. Never grant a temporary
ticket permission and never use a bypass-permissions flag.

`snapshot` records Git state, protected bytes, frozen dispatch contract, AGY
project identity, conversation step floor, and a digest of Project plus Global
permissions. `dispatch` passes only `--project <id>`; it does not use
`--add-dir` or mutate settings. `verify` fails closed if permissions drift,
the conversation changes, an unlisted command is requested, a protected
artifact changes, or repository writes escape the exact path/budget contract.

`dispatch` and `resume` store the rendered prompt, AGY log, raw output, and
normalized final `## EXEC REPORT` under `state_dir/runs/`. Progress chatter
may precede the final report, but empty output, nonzero exit, or a missing
terminal report remains a failed delivery. Process exit is never acceptance.

If a failed ticketed run created a conversation for the same ticket, first run
`verify`. After confirming the snapshot and protected artifacts, correct the
delta prompt or Project policy, take a fresh snapshot when either contract
changed, and run `resume` with the same profile/state. A transient backend 5xx
may resume without a contract change when the snapshot remains valid. Do not
resume after a VOID result, without a stored conversation id, or for another
ticket.

A one-shot run records its conversation id only for audit. It never resumes,
even after a retryable transport failure; create a new run id, oracle, and
snapshot instead. Do not silently promote a one-shot into a ticketed session.

A revision report is a self-contained replacement, not a delta fragment.
Require it to preserve every already accepted identity, row, matrix,
invariant, lifecycle boundary, planned path, forbidden change, and test seam
while applying the correction. Compare the new normalized report with the
prior accepted sections; reject a compact re-emission that silently drops
evidence. Make list-edit intent explicit in the delta contract: say `append`
when a new test/invariant must coexist with accepted entries, and say
`replace <exact entry>` only when the old entry must disappear. Do not rely on
“add if needed” wording when count or coverage is an acceptance surface.

For a hard report-size limit, leave operational headroom instead of asking for
the exact ceiling. On the first oversize revision, set a target at least 10%
below the acceptance maximum and name the removable structures (executive
checklists, repeated prose, decorative separators) plus the exact tables/lists
that must survive. If AGY exceeds the limit again, provide a section budget or
compact replacement template; size compliance is an acceptance criterion, not
a controller-side truncation step. The frozen acceptance maximum remains the
hard limit: do not turn an advisory headroom target into a second, stricter
acceptance ceiling or keep revising a semantically complete report that is
already below the frozen maximum.

`verify` compares post-run Git state and protected artifact hashes with the pre-dispatch snapshot. Optional `path_change_budgets` bound added+removed lines per writable path and catch formatting explosions inside an otherwise allowed file.

It separates two outcomes. **Integrity failures exit `1` (VOID)** — permission
drift, a command outside the audited allowlist, a changed dispatch contract, a
swapped Project, a missing oracle — and mean the evidence cannot be trusted at
all. **Scope problems exit `2` (findings)** — a write outside
`allowed_repo_writes`, a changed protected artifact, a declared path left
unwritten, a budget overrun, a moved branch `HEAD` — and are handed to `review`
for the controller to adjudicate against the diff. That split is only sound
because the worker's tree is disposable; a profile with no `worktree` key keeps
the old behavior of voiding on scope.

Git state is captured with `--untracked-files=all` at both `snapshot` and
`verify`. Without it, porcelain collapses a never-tracked directory to a single
`dir/` entry, which can never match an exact `allowed_repo_writes` path and
misreports every greenfield dispatch. Because of that flag, a round that
creates a brand-new source tree is verified path by path exactly like a round
that edits tracked files: keep listing exact files, never a directory.

## Adjudicate the diff, then take it or send it back

`review` is the acceptance surface. It prints the round's branch and base, the
touched paths with `!` on anything outside `allowed_repo_writes`, every finding,
`git diff --stat`, the full diff, and the body of each new untracked file — then
exits `2` if there is a finding. Read it before deciding anything; the worker's
report is a claim about the diff, not the diff.

### Measure the gate before you trust it

A gate nobody has seen fail proves nothing: a test written against the
implementation that was just produced passes by construction. So a
bounded-write round cannot be accepted without a proof pair, and `accept`
names what is missing.

`prove` reverts nothing. It runs the round's gate over the worktree exactly as
it stands and files the result under the label you give it, which means the
reverting is yours: restore the product change to the round's baseline while
keeping the worker's tests, record `mutant`, restore the candidate, record
`candidate`. `accept` then refuses a mutant that passed, a candidate that
failed, two proofs over an identical tree, and a candidate proof taken before
the tree moved again.

That pair is a floor, not a ceiling. When the round introduces a new symbol,
the reverted tree cannot compile, so the gate goes red because the function it
names does not exist yet — nothing about behaviour was measured. That is the
only answer the revert can give for such a round, so it does not block; `prove`
and `accept` both say so instead of letting a build failure read like a
behavioural kill.

Where discrimination actually comes from is a sweep whose mutants keep the
product compiling: one single-defect mutant per rule the oracle claims, each
expected to be killed by a named row. A mutant that fails to build is a badly
written mutant, not a kill. Apply and restore by writing the whole file — a
copy that preserves an older mtime lets cargo skip the rebuild, which turns the
mutant into a false kill and the next candidate run into a false green. A
survivor is a finding against the round's *evidence*, not against the product:
the code may be right and simply have no row that would notice if it stopped
being right. Close it by authoring the missing row in a round, never by editing
the test yourself — the controller writing the evidence it then judges is the
same failure the oracle freeze exists to prevent.

Three outcomes:

- **Take it.** `accept` stages exactly the touched paths and commits them on the
  round's branch (with `Refs #<issue>` for ticketed work), then prints the
  `git cherry-pick <sha>` for you to run from `controller_root`. It never
  merges — integration and gates are yours, and they run against your branch,
  not the worker's checkout.
- **Send it back.** Write the exact delta contract to a file, point
  `inject_prompt_file` at it, re-`snapshot` if the contract changed, and
  `resume` (ticketed only). The worker's checkout persists across the revision,
  so the next round builds on the same tree instead of starting over.
- **Drop it.** `discard` restores the Project's home root and removes the
  worktree and branch. Use `--keep-branch` to throw away the checkout but keep
  the candidate commit for later.

A finding is a question, not a verdict. An out-of-contract write can be a
worker that misread the scope — send it back — or a contract that was too
narrow for the work, in which case widen `allowed_repo_writes` in the next
round rather than blaming the diff. What you must not do is accept a finding
without reading the path it names.

## Authoring dispatches with no shell

For a pure authoring round — write these files, to this design — give the worker
no shell at all: `task_commands.allow` empty and `project_permissions.allow`
empty. `doctor` still passes, and the dispatcher swaps its appended report
contract for the no-shell variant, which forbids `PASS`/`FAIL` and any claimed
observation and asks instead for a description of what was written.

Do not leave the verdict-shaped contract in place for such a round. The trailer
is appended *after* the injected delta, so a trailer demanding "PASS or FAIL per
criterion" overrides an injection saying "do not write PASS" and manufactures
the exact fabrication the oracle then has to catch. Match the contract to the
worker's actual capability and the tell becomes real: with an empty allowlist,
any test result, build outcome, or per-criterion verdict in the report is
fabricated by construction, and the audited command list in `verify` proves it.

The corollary is that acceptance is never a read of the report. Write a
controller-side recompute — a script that imports the symbols, inspects the
annotations, calls the functions on the oracle's witness inputs, and runs the
project gate — and accept on its exit code. Keep it outside the worker's
readable material together with any seeded-defect or answer-key file. A report
that agrees with a recompute adds nothing; a report that disagrees is the
finding.

A freeze is a claim about the whole tree, so check it before making it. Whenever
a round changes a value that other files could restate — a declared count, an
arity, a version, an enum's membership — grep the entire scope for that value
before deciding which files to freeze. A second copy left frozen turns a correct
round red and costs a whole extra dispatch, and the worker will have been right
to leave it alone.

Having found a duplicate, resist merging it. Two tables that must agree are a
check; one table two places read is an assumption. Keep them independent when
they are derived differently — one counted syntactically, the other read out of
a real run's output — and say so in the delta, or the next worker will
helpfully de-duplicate them and delete the check.

When a delta round exists because the *controller's own* contract was wrong,
say so in the delta, quote the superseded instruction, and give the corrected
rule with its authority. A worker that faithfully implemented a bad instruction
has not erred, and a delta that implies otherwise teaches it to distrust the
frozen contract it is supposed to follow.

## Interactive teamwork preview

Use this path only when the user explicitly requests `/teamwork-preview`. It requires a real TTY; `--print/-p` and `--prompt-interactive/-i` are mutually exclusive, so do not attempt to trigger it through headless mode.

First detect support, then launch a macOS Terminal session with the prompt injected into AGY's interactive TTY:

```bash
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py detect
python3 .claude/skills/agy-dispatch/scripts/teamwork_terminal.py launch profile.json teamwork-prompt.md
```

The launcher uses `expect` to start `agy --project <id>
--prompt-interactive`, sends the prompt, and hands the TTY to the user. On
macOS it prefers splitting a pane in the current iTerm2 tab and otherwise
falls back to Terminal.app. The prompt must start with `/teamwork-preview` and
name the coordination scope. It does not use `--add-dir` or
`--dangerously-skip-permissions`. Interactive teamwork is not eligible for
automatic issue closure or the headless report-verification contract.

## Rules

- One task per AGY process, and one round per work area. The work area's single
  Project points at one root at a time, so rounds inside a project are serial by
  construction. Raise throughput with distinct work areas that have their own
  registered Project, never by pointing two rounds at one Project.
- AGY may comment but never closes a ticket. A GitHub comment is unverified input, not a result.
- Prohibit git mutations, branch switching, worktrees, and unscoped writes. Permit read-only git commands only when the round profile explicitly lists their prefixes.
- Treat a permission-surface change, a changed dispatch contract, a swapped
  Project, an unaudited command, or a missing oracle as a void result: the
  evidence itself is untrustworthy.
- Everything the worker did to its own checkout is a finding, not a void —
  including a changed protected artifact. `make_profile.py` protects the whole
  complement of the write scope, so voiding on it would reinstate exactly the
  behavior this design removes. A protected-artifact finding is still the most
  serious class: read that path before anything else.
- Keep working while a round is in flight. The worker has its own checkout, so
  controller-side edits cannot dirty the worker's snapshot and the dispatcher
  never has to tell the two apart. This is the whole point of deriving the
  worktree; the previous rule requiring a frozen controller tree is retired.
  What you must not do is edit *the worker's* checkout under
  `.agy-worktrees/` while the round runs.
- Triage logs and the mandatory local report, not process exit alone: an
  unlisted AGY command can abort while returning zero. Resume only after
  verifying the failed run and correcting the exact denied command or injected
  contract when one exists. A same-ticket transient 5xx may resume without a
  contract change if its snapshot remains valid; do not blindly re-dispatch.
- Treat every report PASS as provisional. A claim that formatting is clean or
  an excluded form is unchanged requires controller evidence such as
  `git diff --check` and focused negative controls; report prose is not a
  witness.
- Enforce frozen semantic distinctions in every reported row. A summary-level
  disclaimer does not cure a row-local contradiction (for example, declaring
  a key opaque globally but later treating its reuse as necessarily pointer or
  address reuse). Reject the report or record an explicit controller
  normalization before acceptance; never silently let the summary override the
  detailed evidence.
- Distinguish frozen design authority from frozen empirical expectations.
  Accepted sole-owner, boundary, and target-lifecycle decisions remain
  authoritative, but an oracle's claim about current source behavior, key
  contents, call order, or ownership is a hypothesis until the admitted source
  proves it. If source evidence contradicts that hypothesis, AGY must name the
  exact witness and report the contradiction; the controller records an
  explicit correction in the next-round injection instead of forcing the
  report to repeat a disproved claim or silently rewriting the sealed oracle.
- Require one unambiguous target semantic owner per reported row. Current
  storage, target ownership, and service dependencies are separate fields; an
  `A / B` owner label or a process service named as a co-owner is not an
  ownership decision. Labels such as `candidate A or B`, `remove or retain`,
  and `optional compatibility owner` are unresolved choices and must be
  rejected or resolved by the controller before acceptance.
- Preserve sole-owner decisions from frozen design inputs. A new slice may
  add a typed binding or coordinator edge to an accepted aggregate, but must
  not silently nest that aggregate's records under a second owner or replace
  its key/value and ownership contract. Treat such a topology rewrite as a
  conflict requiring controller resolution, not harmless diagram shorthand.
- Keep current behavior separate from proposed target behavior. A stored
  configuration value is not evidence that the execution path consumes it,
  and a function name such as `*_all_threads` is not evidence of propagation.
  Verify the producer-to-consumer path before accepting the claimed effect.
- Do not project current broad invalidation onto a target authoritative
  version boundary. A current helper may clear caches after not-found,
  invalid, or no-op calls; a target publication generation advances only for a
  proved semantic visibility change. Preserve current reachability and target
  commit semantics as separate decisions.
- Keep a source-representable partial state or race window separate from an
  observed event. Separate calls without an atomic rollback protocol prove
  that skewed state is representable; they do not prove an exception, crash,
  callback, or failed publication actually occurs between those calls. Name
  the concrete event witness or report only the representable hazard.
- When the admitted state is read only through a helper, trace the helper's
  production callers before declaring its semantic consumers complete. Name
  each caller's actual enclosing function and branch; a helper labeled
  “general inquiry” can hide multiple behavior policies outside the selector
  file.
- For same-key or same-name replacement claims, distinguish paths that invoke
  the admitted writer from paths that do not. A second writer call may replace
  an entry, while an ordinary rebinding that skips that writer can leave the
  old entry stale. Do not summarize both as generic “rebind overwrites.”
- Keep entity identity separate from immutable publication version. Updating
  or decorating an existing entity normally preserves its typed runtime key
  while publishing a new definition/configuration version; only creation of a
  genuinely new entity allocates a new key. Require the report to name both
  transitions explicitly when same-display-name replacement is also in scope.
- If admitted evidence exposes a semantically duplicate owner outside the
  frozen selector, do not silently expand the denominator or pretend the
  duplicate was audited. Name its exact identity as an out-of-scope sibling
  inventory dependency, preserve the current denominator, and block a target
  “single owner” implementation until that sibling decision is accepted.
- Separate authoritative state failure from derived-projection maintenance.
  A publication/version update that defines visibility must fail closed, while
  an opportunistic cache insert, prune, or old-generation clear may safely
  skip only when a typed authoritative generation makes the projection
  correctness-neutral and lookup falls back to the owner.
- Helper names such as `set_field`, `replace`, `remove`, `clear`, or `drop`
  do not prove retain/release behavior. Inspect the helper body and record the
  exact old-value return/drop and new-value retain/transfer edges before
  calling an ownership ledger balanced.
- An ownership ledger for a registry is not complete until it also accounts
  for the fields of every object retained by that registry and for returned
  aliases. Require one row per stored field/alias with its incoming ownership
  contract, explicit retains, replacements, removed-value handling, and final
  retirement. Container-level `retain(entry)` evidence cannot authenticate
  `_target`, callback, payload, or child-handle ownership inside the entry.
- Distinguish an aggregate-owned cache/registry claim from a caller-owned
  alias or lease. A registry record such as `OwnedValue` is not itself a
  caller lease; report the registry's installed claim, each returned retain or
  `Arc` clone, and their independent retirement edges with the actual types.
- Do not invent a nested lease for an immutable value already owned by a
  leased aggregate. If `Arc<Aggregate>` keeps all aggregate fields alive, a
  proposed `Arc<Field>`, field-specific generation, or second retirement
  protocol needs a distinct independently-lived use case in the frozen
  contract. Otherwise require the field to remain a direct aggregate value and
  the aggregate lease to be the sole lifetime authority.
- A copied raw value, function parameter, vector lookup, or map insertion does
  not prove an incoming ownership transfer. Require the caller/container
  retirement edge too. If that edge is outside the allowed evidence surface,
  mark it unresolved rather than guessing `borrowed` or `transferred`.
- When one public constructor has wrapper, alias, or compatibility branches,
  audit each concrete variant. Do not project fields or registration behavior
  from one branch onto another; a branch that returns the input value may have
  no registry entry at all and a different retain ledger.
- Verify every claimed cleanup bypass against actual control flow. A
  side-channel exception flag is not a Rust early return or unwind, and a
  timeout path is not stale-state evidence when it still passes through the
  common cleanup block. Name the exact branch, return, panic, cancellation, or
  missing retirement edge that bypasses cleanup.
- Do not infer Rust guard lifetime from indentation or the apparent end of a
  method call. `if let`/`match` scrutinee temporaries can keep a `RefMut`,
  mutex, or container guard live through the branch body. Inspect the exact
  statement and workspace Rust edition; when release/reentry safety depends on
  it, use a same-shaped minimal probe or MIR evidence. Also distinguish the
  caller's registry guard from locks acquired inside helpers such as dict
  insertion: “no outer borrow” does not prove “release is guard-free.”
- Separate cleanup registration from cleanup effect. A central reset may call
  a named cleanup function whose body intentionally leaves state unchanged;
  that is an invoked no-op, not an absent call. Verify both the caller edge and
  the callee body before reporting either omission or successful cleanup.
- Include implicit storage retirement in lifecycle claims. Thread-local maps
  may survive object drop and runtime reset yet still be dropped by the OS
  thread's TLS destructor. Report each boundary separately; do not call such
  entries process-permanent, and check whether longer-lived objects lose their
  metadata when the creating worker exits.
- For callback/reentrancy claims, name the exact invocation site and the exact
  guards still live at that point. Distinguish a registry borrow/lock from an
  object-deallocation phase, collector phase flag, or graph traversal. Do not
  say all callbacks run on a notification path when that path only marks some
  entry kinds and invokes others.
- Keep stale map entries separate from leaked ownership after removal. Prove
  whether the key was removed on the concrete path before claiming address
  reuse; a missed cross-worker notification may leave a stale key, while a
  successful removal that skips `release` leaks an owned object without
  leaving a reusable map entry.
- Keep outer-container retirement separate from retained-object field
  retirement. Draining a registry normally retires its registry claim only;
  manually releasing all fields and then releasing a still-live entry can
  double-retire its contents. Require atomic replace-with-owned-return evidence
  for early field clearing and a separate account of external entry claims.
- Cleanup and reset evidence must include conditional acquisition and failure
  branches. `try_lock`, `try_borrow`, ignored `Result`, poison recovery, or an
  early return can make a cleanup silently partial or a no-op even when the
  nominal body clears every field.
- For concurrency claims, inspect synchronization inside every traversed
  helper and variant. Per-object read locks do not prove a graph-wide stable
  snapshot, but their presence also means the path must not be reported as
  wholly unlocked.
- A synchronization primitive's type name proves only its documented semantic
  contract, not a performance property. For example, `OnceLock::get_or_init`
  supports race-safe single publication; it does not by itself prove that
  concurrent initialization is lock-free. Separate initialization behavior
  from any post-initialization fast path and require evidence for both.
- A disabled or early-return branch can be a semantic no-op without being
  zero-overhead. Trace entry, first-use initialization, steady-state reads,
  allocation, and synchronization separately. Reserve "zero overhead" for a
  compiled-out or otherwise proven absent path.
- Do not equate debug-only with test-only. A `cfg(debug_assertions)` symbol can
  be production diagnostic state and can be consumed by integration harnesses
  or debug binaries. Classify a state identity as test-only only from its
  compilation/reachability boundary, not from a test-oriented name or caller.
- State synchronization internals only when the public contract or inspected
  implementation proves them. A public API may say concurrent callers wait or
  block without proving a mutex, futex, parking implementation, fairness, or
  another internal mechanism.
- When a report promises an exact implementation shape, require syntactically
  complete code: statics need initializers, types and visibility must be
  coherent, and the snippet must preserve the stated initial state. Conceptual
  pseudocode must be labeled as such and cannot satisfy an exact-path contract.
- Keep `changed paths` separate from `planned implementation paths`.
  `changed paths: none` is correct for a measure-only run but does not satisfy
  a ticket asking which files a later implementation must change. Require both
  fields when both are in the contract.
- Before accepting the final report, reconcile every acceptance verb that says
  print, list, enumerate, or provide a matrix with an actual report section and
  its complete members. A checklist `PASS`, `fully mapped`, or `unfinished:
  none` cannot substitute for omitted paths, rows, invariants, or test seams.
- Do not introduce a second selector denominator such as "matching lines",
  "unique lines", or "occurrence rows" unless the ticket defines it and the
  report recomputes it. If every appendix member has a distinct path:line, its
  physical-row count and matching-line count must agree.
- A complete state/operation matrix includes non-mutating error branches,
  read-only operations, RAII/destructor transitions, and reset/recovery paths,
  not only successful mutations. Likewise, `exact invariants` means an
  enumerated invariant set; two summary bullets cannot satisfy a larger frozen
  lifecycle/ownership contract.
- When target ownership moves state out of TLS, a process static, or a side
  table, recompute target retirement from the new owner. Do not copy a current
  storage defect such as TLS evidence loss at OS-thread exit into the target
  invariant; prove how the owning child/context record survives, publishes,
  joins, quiesces, and retires.
- Distinguish scoped child bindings from transferable child state. An active
  invocation/frame binding guarded by RAII normally belongs to one execution
  child and must be restored on guard drop; it is not automatically payload for
  thread snapshot/replace or worker inheritance. Likewise, compatibility
  cleanup must not clear a live scoped binding behind its guard. Require an
  explicit quiescence/fail-closed rule before accepting either transfer or
  cleanup of active child bindings.
- Separate a low-level helper's transition from its RAII wrapper's later drop.
  If an imbalanced helper returns `Err` while state remains active and the
  wrapper's destructor then marks it incomplete, report both steps. Do not
  attribute the destructor transition directly to the helper.
- When a ticket audits multiple reset, cleanup, or retirement entry points,
  require a row-by-path matrix over the whole admitted denominator. A prose
  description of the primary registry does not prove what each sibling cell,
  cache, handle, or retained value resets or leaves behind.
- Do not accept a number without a runnable selector, witnesses, and independent recomputation. Audit both admitted and discarded members.
- For an exact-set row table, independently compare every reported identity
  and its path/line/type fields with current selector output. A correct total
  or digest can coexist with fabricated, shifted, or reconstructed row
  evidence; reject those rows even when the arithmetic reconciles.
- For dense call-site matrices, resolve every row to its actual enclosing
  function boundary before naming the operation; do not infer owners by call
  order or copy a nearby sequence. Also prove guard lifetime at the exact call:
  a temporary `lock.write().method();` guard is normally dropped at the end of
  that statement, while a named guard remains live until its last use/scope.
  Reject a row that reports either form from visual proximity alone.
- Apply enclosing-function resolution to test rows too. A nearby test topic,
  filename, or later assertion does not authenticate the `#[test] fn` that
  owns a selector line; report the actual enclosing test name.
- A mutation/cleanup helper call does not prove that the surrounding operation
  changed state or succeeded. For every matrix row, inspect whether the call is
  outside a `match`/error branch, runs on empty/no-op inputs, or still runs
  after an exception is raised. Preserve these call-site reachability facts
  separately from the desired target semantics.
- Apply the same current-selector check to every exhaustive evidence inventory,
  not only the primary exact-set table. Caller/acquisition families can be
  omitted while the admitted digest stays correct, and path:line witnesses can
  be stale, snippet-relative, or reconstructed. Compare the complete reported
  evidence surface to fresh selector output and reject any missing family or
  mismatched path/line/type claim.
- For selector family breakdowns, every counted row must actually match that
  selector. Adjacent declarations or mutation statements may be useful context
  but must be explicitly excluded from the selector count; never let a correct
  headline total authenticate a fabricated family subtotal.
- A selector family subtotal used for acceptance must expose its complete
  member path:line list in the report or a protected machine-readable sidecar.
  Representative witnesses can illustrate a family but cannot authenticate
  the subtotal or prove that every headline member was assigned exactly once.
- Reconcile every declared numeric count with the enumerated surface in the
  same report. Section headings, terminal summaries, and acceptance checklists
  must update when a revision splits or appends rows; a correct list with a
  stale headline count is not self-reconciling evidence.
- For an alternation selector, the default denominator is the selector's
  physical output rows, not one row per matched alternative. A source line
  mentioning multiple identities appears once and records all matched
  identities (for example `symbols=[A,B]`). If the ticket instead requires a
  symbol-occurrence denominator, it must say so explicitly and the controller
  must independently recompute that different denominator.
- State whether an exact selector is comment-inclusive or code-reference-only.
  When the ticket asks for actual state accesses, freeze a selector that
  excludes comments by construction and count only declaration/read/write
  code rows. Do not count a test-name comment as a test access while excluding
  an equivalent production comment. If comments are intentionally admitted,
  list them as a separate comment-only category and reconcile them explicitly.
- Validate every cited representative witness too. Correct aggregate counts do
  not authenticate sample rows: each cited path/line/type and the semantic fact
  attributed to it must match fresh current source. Reject stale or invented
  samples even when the selector total and headline conclusion are correct.
- Bind every exact-set row to its actual enclosing function/module from fresh
  source, preferably with a mechanically derived row-to-owner map when the
  language shape permits it. Do not infer the owner from a stale line number,
  nearby comment, expected behavior, or previous revision. A correct row count
  with wrong enclosing owners or policies is not reconciled.
- Reject placeholders such as “remaining rows cataloged”, “same schema”, or
  ellipses in an exact-set report. If complete witnesses do not fit, keep the
  roll-up open and create smaller owner slices.
- Keep the rendered report compact enough for the dispatcher to normalize.
  When a raw log contains `## EXEC REPORT` but `status` is `EMPTY` and no
  normalized report artifact exists, verify the snapshot, then resume
  the same conversation with an explicit character budget and the exact
  evidence surfaces that must remain. Never accept the raw log as the report.
- A PASS checklist or sentence saying an audit was completed is not the
  required audit surface. When the ticket requires a path matrix, ownership
  ledger, caller lineage, or retain/release accounting, the report must print
  those rows with evidence; reject a summary that merely asserts completion.
- Apply that rule to implementation-boundary deliverables too. If acceptance
  requires the smallest safe slice, exact source paths, invariants, forbidden
  changes, or focused tests, the report must print each of those surfaces.
  Naming a target struct and then marking the criterion PASS is not a
  substitute for the requested path/test matrix.
- Verify every claimed existing path and fixture leaf against the fresh
  workspace. A semantically plausible but nonexistent filename is a failed
  witness. Keep planned new paths explicitly labeled as planned so they are
  not subjected to the existing-path check.
- Treat numeric test/invariant minimums as floors, never as permission to
  truncate an enumerated coverage contract. Map every explicitly required
  seam to a distinct planned test or state exactly which test covers multiple
  seams and how. If the oracle lists more seams than its numeric minimum, the
  full seam list controls acceptance.
- A future verification gate is not an executed result. If the profile forbids
  tests/builds, the report may name an exact test or command as pending
  controller verification, but it must not say that test passes.
- Preserve the oracle's epistemic boundary. A source comment, issue history,
  design rationale, or suspected mechanism can explain why code exists, but it
  does not prove the mechanism, minimal critical section, safety property, or
  recovery policy. If the ticket reserves that proof for later work, reject a
  report that upgrades the motivation into a present fact.
- Failure evidence must cover every owner that can be partially advanced, not
  only the headline cache or registry. An insertion that occurs after a
  fallible operation can prove the map stayed clean while the retained module,
  allocator, service, or external resource became unsafe to reuse. Require the
  retry, abandonment, poison, or fail-closed decision for that retained owner;
  absence of a cache entry is not integrity evidence.
- Keep the failing synchronization attempt separate from later poison
  handling. A panic while a mutex guard is live drops that guard during unwind
  and poisons the mutex; `PoisonError::into_inner()` can describe a subsequent
  acquisition policy, not recovery of the already failed operation.
- An `Err` from a mutating API proves neither atomic rollback nor partial
  mutation unless its contract or implementation says so. Report the bounded
  facts (which later publications did not occur, which owner remains live, and
  what atomicity is unproven) instead of asserting that a declaration,
  allocation, or registry mutation definitely survived the error.
- Reconcile symbol-occurrence subtotals at row granularity. When one physical
  row contains two different alternatives, list both once; do not attribute a
  duplicate occurrence to the next line or say one token appears twice unless
  the source line actually contains it twice.
- On revision, propagate a corrected fact into every report surface that
  repeats it. A fixed criterion summary does not cure a stale appendix,
  occurrence label, matrix cell, invariant, or conclusion that still states
  the rejected version. Cross-check the replacement report internally before
  acceptance.
- Keep explicit retirement, language/runtime destructors, and OS process
  reclamation separate. A process-global static is not proven to run a Rust
  destructor at process exit; when there is no explicit drain, state that the
  address space is reclaimed without claiming Python retains or fields were
  retired through their normal ownership protocol.
- Avoid using “lock-free” to mean only “outside another lock.” Say
  `registry-guard-free`, `outer-lock-free`, or name the exact absent guard while
  still listing any object/inner lock that remains required.
- Keep owner visibility separate from leased-object lifetime. Removing a
  registry entry, handle, or lookup edge can make new resolution fail
  immediately while an already-acquired `Arc`, refcount, epoch pin, or other
  lease keeps the detached object alive. Reject reports that delay
  deregistration until lease count reaches zero, or that claim deregistration
  invalidates existing leases, unless the implementation proves that coupling.
- A raw callback, vtable slot, JIT address, or FFI function pointer proves
  callable identity, not that its code remains mapped. Require explicit
  code-lifetime authority: a proved process-image lifetime or an owned module
  lease. An optional authority field does not close the lifetime invariant when
  unloadable producers are in scope.
- Do not call a multi-step fallible publication “atomic” merely because its
  target invariant is all-or-nothing. Require the concrete commit point,
  provisional visibility rules, and rollback for every acquired claim and
  partially inserted entry. Use `transactional publication` unless one actual
  atomic primitive spans the entire visible transition.
- Reconcile the visibility set with the phase ordering. Every alias, binding,
  record, policy, and cache generation claimed to become visible together must
  be staged before and activated by the named commit point. A supposedly
  coordinated alias or index update cannot first happen after that commit;
  post-commit work is limited to prior-generation detachment and retirement.
- Do not infer ancestry or propagation from local variable names such as
  `parent`, `snapshot`, or `context_snapshot`. Trace where the value was
  captured and on which OS/logical thread before claiming inheritance.

## Profile boundaries

The generic controller owns the task-class minimums and project permission
isolation. The profile supplies project-specific build/test commands, exact
binary paths, caller-selected design inputs, and verification contract. Do not
include a toolchain denial such as `cargo` unless that ticket must not use it.

For `bounded-write`, enumerate exact writable paths in `allowed_repo_writes`; do not use directory globs. Set `path_change_budgets` when a path should receive a small localized change. If the ticket does not need repository writes, keep `mode: measure-only`.
