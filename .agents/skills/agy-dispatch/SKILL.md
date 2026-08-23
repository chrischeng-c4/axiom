---
name: agy-dispatch
description: Safely dispatch one bounded ticketed or one-shot task to headless AGY using persistent AGY Project permissions, a task-local command/write contract, and a revocable user-local standing-consent registry, then independently verify its report. Use whenever Codex delegates audits, measurements, investigation, transcription, or tightly scoped implementation to AGY; ticketed tasks reuse one conversation per live issue, unticketed tasks run once without resume, implementation requires frozen design inputs, and acceptance remains controller-only.
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

Before first use, disclose that the task contract, oracle, and any repository
files the worker reads are sent to headless AGY. For bounded-write mode, also
disclose that the generated repository diff is handled by that service. The
controller may then either preserve that per-task approval in
`external_payload_consent` or record an explicit, revocable all-bounded-WI
approval in `~/.codex/agy-dispatch/standing-consent.json`. A standing record
is user-local—not an AGY Global permission, credential, or permission-policy
change—and covers a new Project only while its payload classes still match.
Never infer approval from a general instruction to use AGY.

The supplied templates reference `all-bounded-work-items-v1`. On each
preflight, the script resolves the local record and binds its complete content
digest into the profile snapshot. Deleting it, setting `revoked: true`, or
altering it fails closed and voids any later resume/verification identity.
When no matching standing record exists, use a fresh concrete disclosure and a
per-task explicit approval; do not ask again merely because the task uses a
new clean worktree or persistent AGY Project.

1. Create a ticketed profile from [references/profile-template.json](references/profile-template.json) or an unticketed profile from [references/one-shot-profile-template.json](references/one-shot-profile-template.json). Keep it outside the repository if it contains local binary paths or mutable pins. Set `agy_project_root` to the existing persistent app/repository worktree and `root` to a distinct clean linked Git worktree physically nested beneath that root, for example `<agy_project_root>/.agy-worktrees/<task-key>`. Both paths must be exact worktree top levels of the same Git repository. Set `agy_project_id` to the one existing AGY Project registered for the persistent root. Keep the hard adapter settings `model=gemini-3.7-flash-high`, `worktree_layout=in-project`, and `launch_cwd=task-worktree`. `doctor` fails closed until every transmitted payload class is covered by a current per-task or standing explicit approval.
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

For Agentic Workflow, an external-contract (EC) authoring round is a
`bounded-write` implementation task: the controller first freezes the issue
contract, EC brief, independent oracle, exact EC source/fixture write paths,
and path budgets. AGY may materialize only that candidate EC artifact. The
controller still runs `aw ec` lifecycle commands, obtains independent semantic
EC review, verifies the result, owns Git/tracker operations, and decides
acceptance or closure. An AGY EC report is never EC approval.

## Dispatch protocol

Change to the repository root before every adapter verb. The only active
dispatcher entry point is `scripts/agy_dispatch.py`. Do not use an installed,
skill-local, or legacy dispatcher copy.

```bash
python3 scripts/agy_dispatch.py doctor profile.json
python3 scripts/agy_dispatch.py snapshot profile.json ISSUE
python3 scripts/agy_dispatch.py dispatch profile.json ISSUE
python3 scripts/agy_dispatch.py resume profile.json ISSUE
python3 scripts/agy_dispatch.py status profile.json
python3 scripts/agy_dispatch.py verify profile.json ISSUE
```

For an unticketed one-shot profile, use its explicit unique `RUN_ID` in place
of `ISSUE` for `snapshot`, `dispatch`, and `verify`. Never call `resume` for a
one-shot run. The dispatcher rejects both a resumed one-shot and a second
dispatch under the same run id.

Run `dispatch` and `resume` as direct long-lived host processes. Do not wrap
them in an orchestration helper that can return before the nested subprocess
finishes. When the host yields a process/session id, poll that same session
until terminal output is captured; an early empty wrapper result is not an AGY
report.

If `status` is `EMPTY`, run `verify` and inspect the stored AGY log. `verify`
still audits repository, Project, sibling, command, and protected baselines.
When the attempt has an auditable conversation and the envelope is intact, it
returns `DELIVERY_FAILED_ISOLATION_VERIFIED`, not VOID; this permits a ticketed
retry but never semantic acceptance. Missing conversation lineage, malformed
attempt evidence, or any isolation failure remains VOID. A log saying the user
is not logged into Antigravity is an authentication delivery failure: confirm
that no commands or writes occurred, do not change permission policy, and
require the user to sign in through AGY. Explicit permission-file authority
never authorizes creating, copying, or editing login credentials.

Before `dispatch`, write the oracle to the profile's `state_dir/oracles/<task-key>.md`. Record its SHA-256 in the issue comment for ticketed work or the controller log for one-shot work. It must name an independently derivable expected result (or hard floor) and at least one fabrication tell.

## Project, worktree, and permission contract

Use one persistent AGY Project per repository/persistent app worktree. A
ticket, controller, and clean linked worktree never receive a Project of their
own. The profile binds `agy_project_root` to that persistent root, binds `root`
to one clean linked task worktree physically nested beneath it and in the same
Git common directory, and launches `agy --project <id>` with the subprocess
`cwd` set exactly to `<task-root>`. This is one Project with in-Project task
worktrees, not one Project per worktree and not an external sibling attached
with `--add-dir`. Keep the persistent root read-only and controller state under
the exact `/tmp/agy-dispatch/<agy_project_id>/<task-key>/` namespace; profiles
using another Project or task component are rejected.

Before enabling this topology for normal work in a Project, run one activation
canary. First dispatch a `measure-only` one-shot whose only shell commands are
`pwd`, `git rev-parse --show-toplevel`, and `git rev-parse --git-common-dir`;
the recorded command `Cwd` and top level must equal `root`. Then run one
serialized `bounded-write` canary limited to a disposable file under `root`,
with sentinel files in the persistent root and a synthetic sibling worktree
listed as protected artifacts. `verify` must accept only the task-root change
and preserve both sentinels. Until that canary passes, do not treat process
`cwd` as AGY-native filesystem isolation and do not run parallel bounded-write
AGY processes in the Project.

AGY CLI 1.1.15 formally exposes `--project` and `--new-project`, but not a
machine-readable Project list, Project-root lookup, or effective Project-policy
API. Do not infer these facts from `~/.gemini` registry/cache files. `doctor`
therefore requires `project_policy_observation`, created by a human from the
official Project selector, Project Settings gear, and `/permissions` **Project**
scope. With no observation—or zero/ambiguous matching persistent-root Projects—
it stops as `PROJECT_SETUP_REQUIRED`; it never selects, creates, deletes, or
repairs a Project. See [references/permissions.md](references/permissions.md).

Use exactly three layers:

- `global_permissions`: the reusable official **Global** baseline. Configure it
  once through AGY `/permissions` Global scope or the documented CLI Settings
  file `~/.gemini/antigravity-cli/settings.json`. It allows narrow repository
  discovery (`git status/log/diff/show/rev-parse/ls-files/merge-base`, `rg`,
  `sed`, `pwd`, `uv`, `python3`) and denies Git/tracker/publication mutation.
- `project_permissions`: only repo-specific exceptions. Usually empty. Use the
  Project Settings gear or `/permissions` Project scope; set **Outside of Folder
  File Access** to **Always Deny**. It does not replace the Global baseline.
- `task_commands`: the controller's byte-exact per-ticket shell contract. It
  is stricter than AGY: every observed command not copied exactly from
  `task_commands.allow` is VOID even when a Global or Project rule allows it.
  `allowed_repo_writes`, protected artifacts, snapshots, and budgets separately
  constrain file writes.

AGY merges inherited Global and Project rules; `Deny > Ask > Allow`. The
dispatcher reports the matched rule and source (`global`, `project`, or
`task_contract`) for each task command. It never calls a broader AGY allow an
authorization for controller-only Git, publication, tracker, or closure. Do
not use `command(git)` or `git *` deny rules: token-prefix matching would block
safe discovery as well as mutation. Use the explicit mutation denies in the
canonical Global template instead.

Do not use Computer Use, AppleScript, UI automation, `--dangerously-skip-permissions`,
or direct registry/cache JSON mutation. If an official CLI/API cannot perform
the needed discovery or policy operation, return `PROJECT_SETUP_REQUIRED` and
print the manual official-UI checklist. Do not silently switch workers or
weaken a policy. A `--sandbox` is a per-run terminal restriction, not the
Project file-access policy; templates default it to `false` because macOS may
reject task-worktree reads under the terminal sandbox. Enable it only after a
same-Project in-Project-worktree read-only probe succeeds.

If AGY soft-denies a command, run:

```bash
python3 scripts/agy_dispatch.py denied profile.json ISSUE
```

If the command is unnecessary, tighten the prompt and resume. If it is common
across repositories, add a narrow Global rule through `/permissions` Global
scope and update `global_permissions`; if it is genuinely repository-specific,
add a narrow Project exception through Project scope and update
`project_permissions`. Record a fresh official Project observation, rerun
`doctor`, and snapshot before resuming. Never grant a temporary ticket
permission or use a bypass-permissions flag.

`snapshot` requires a clean isolated in-Project task worktree and records its Git state,
ignored path baseline, protected bytes, frozen dispatch contract, AGY Project
identity, exact pre-launch task baseline, and read-only byte baselines of the
persistent Project root plus every nested sibling worktree, along with the
conversation step floor and a digest of Project plus Global permissions.
`dispatch` rechecks those frozen boundaries before AGY can launch, passes
only `--project <id>` and launches the AGY process with
`cwd=<task-worktree>`; it does not mutate settings or create a Project/worktree.
`verify` fails closed if permissions drift, the conversation changes, an
unlisted command is requested, a newly generated ignored path appears, a
protected artifact or sibling worktree changes, or repository writes escape
the exact path/budget contract. The ignored-path exceptions are reproducible compiler/interpreter
caches (`target/`, `__pycache__/`, and task-local `.venv/` directories),
because controller-owned build and EC gates may create them between same-ticket
AGY revisions. This never permits source, permission, or tracked
virtual-environment writes; manifest and lock paths remain snapshot and
write-contract protected.

Fresh profile templates set `sandbox: false`. The persistent Project file
boundary is the documented **Outside of Folder File Access: Always Deny**
setting, and `verify` still audits every
actual shell command against the exact task allowlist. `sandbox: true` adds an
optional AGY per-process terminal restriction, is part of the frozen dispatch
contract, and requires a fresh snapshot if changed. Use it only after a
same-Project in-Project-worktree read-only probe succeeds; a log containing
`SANDBOX_COMMAND_BLOCKED` plus `Operation not permitted` is a VOID, not a
reason to widen Global or Project permissions.

`dispatch` and `resume` store a snapshot-bound attempt record, rendered prompt,
round contract, AGY log, and raw output under `state_dir/runs/` for every
terminal process result. A successful delivery additionally stores the
normalized final `## EXEC REPORT`. Progress chatter may precede the final
report, but empty output, nonzero exit, or a missing terminal report remains a
failed delivery. Process exit is never acceptance.

Attempt identity is ordinal, never mtime: the initial files have no suffix,
the first retry uses `.resume`, and later retries use `.resume.N`. Ordinals
must be contiguous and every attempt must have one canonical prompt, contract,
AGY log, raw output, and evidence file; a reported attempt must also have one
normalized report. Resume verification requires the AGY log to name the exact
stored conversation id. Missing, extra, renamed, reclassified, or digest-drifted
artifacts fail closed. Run evidence and verified markers carry an explicit
schema plus command-audit contract version; older parser semantics never inherit
reuse authority after an audit-contract revision.

Snapshot, launch, and verify hold one nonblocking OS lock for the task, so two
controllers cannot reuse one verified predecessor concurrently. The marker
binds the exact schema and every recorded conversation-row column through its
step ceiling plus the complete ordinal attempt-artifact lineage; later row or
artifact edits, new steps, conflicting conversation ids, or a new floor over
unaudited events fail closed. Shell requests are audited from the authoritative
`steps` table: type-15 request rows are counted once, while later lifecycle
rows may only replay an identical command/Cwd identity. Any truthy
`has_subtrajectory` is VOID until the adapter can traverse and audit the child
trajectory. Verification also freezes stable
Git admin bytes, reflogs/refs/hooks/config, semantic indexes for every
registered worktree, and every shared Git object byte—not only ordinary Git
status.

If a failed ticketed run created a conversation for the same ticket, first run
`verify`. Resume is refused until the latest attempt and exact conversation
step ceiling have a controller verification marker. After
`DELIVERY_FAILED_ISOLATION_VERIFIED`, correct the delta prompt or Project
policy, take a fresh snapshot when either contract changed, and run `resume`
with the same profile/state. A transient backend 5xx may resume without a
contract change when the snapshot remains valid. Taking a new snapshot is also
refused while any prior conversation step remains unverified, so a new floor
cannot hide old commands. Do not resume after a VOID result, without a stored
conversation id, or for another ticket.

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

## Interactive teamwork preview

Use this path only when the user explicitly requests `/teamwork-preview`. It requires a real TTY; `--print/-p` and `--prompt-interactive/-i` are mutually exclusive, so do not attempt to trigger it through headless mode.

First detect support, then launch a macOS Terminal session with the prompt injected into AGY's interactive TTY:

```bash
python3 ~/.codex/skills/agy-dispatch/scripts/teamwork_terminal.py detect
python3 ~/.codex/skills/agy-dispatch/scripts/teamwork_terminal.py launch profile.json teamwork-prompt.md
```

The launcher uses `expect` to start `agy --project <id>
--prompt-interactive`, sends the prompt, and hands the TTY to the user. On
macOS it prefers splitting a pane in the current iTerm2 tab and otherwise
falls back to Terminal.app. The prompt must start with `/teamwork-preview` and
name the coordination scope. The launcher validates the same registered,
nested in-Project task-worktree and exact state namespace as headless mode,
then starts from that task root. It does not use `--add-dir` or
`--dangerously-skip-permissions`. Interactive teamwork is not eligible for
automatic issue closure or the headless report-verification contract.

## Rules

- Never fabricate `external_payload_consent.approval_record`, a standing
  registry, or an approval scope. A matching current standing record is an
  explicit authorization and must not trigger a repeat approval just because a
  task has a new Project or worktree; an absent, revoked, or insufficient
  record requires a fresh concrete payload-risk disclosure.
- Treat an outer execution or escalation denial of the AGY launch as an egress
  denial, even when the digest-bound profile validates. It proves no AGY
  payload was sent. Do not retry, proxy the launch through another tool, or
  change Project policy to work around it. Preserve the clean worktree and
  snapshot, report the exact denial, and require a new ordinary user message
  naming the ticket(s), headless AGY, the transmitted private-context classes,
  and controller-only acceptance. After that message, run doctor and take a
  fresh snapshot before one launch.
- One task per AGY process. Measure-only work may increase 1 → 2 → 4 only after
  the preceding batch verifies cleanly. Serialize bounded-write AGY processes
  within one persistent Project: process `cwd` and post-run diff verification
  do not make sibling in-Project worktrees unreadable. Parallel writes require
  a separately proven AGY-native worktree confinement boundary.
- AGY may comment but never closes a ticket. A GitHub comment is unverified input, not a result.
- Prohibit git mutations, branch switching, worktrees, and unscoped writes. Permit read-only git commands only when the round profile explicitly lists their prefixes.
- Treat a changed protected artifact hash or an unexpected tracked/untracked repo change as a void result.
- Triage logs and the mandatory local report, not process exit alone: an
  unlisted AGY command can abort while returning zero. Resume only after
  verifying the failed run and correcting the exact denied command or injected
  contract when one exists. A same-ticket transient 5xx may resume without a
  contract change if its snapshot remains valid; do not blindly re-dispatch.
- Treat every report PASS as provisional. A claim that formatting is clean or
  an excluded form is unchanged requires controller evidence such as
  `git diff --check` and focused negative controls; report prose is not a
  witness.
- In a bounded-write report, label each requirement `implemented; controller
  gate pending` unless the worker actually ran the specific permitted command
  that proves it. Never label a runtime requirement PASS merely because the
  source was edited or because a later controller build/EC command is named.
- For bounded implementation, issue `REVISION_REQUIRED` only for a material
  finding: an isolation/scope breach, a frozen oracle or acceptance-criterion
  mismatch, a reproducible required-gate failure, or a false-green evidence
  gap. Do not reopen correct work for naming, style, report prose, optional
  coverage, or another acceptable implementation shape. When a report is
  merely incomplete, prefer controller re-execution of the named gate and
  record that evidence rather than making the worker rewrite prose.
- Before a semantic revision, collect every currently reproducible material
  finding from the frozen oracle and controller gates into one replacement
  delta. Do not turn independently discovered preferences into serial
  one-finding revision waves. If the source or a required gate contradicts
  the frozen task itself, stop and correct or split the task contract; never
  ask the worker to guess the missing product semantics.
- A same-ticket semantic correction may replace only the controller round
  prompt after the clean baseline. Preserve the frozen oracle, task contract,
  Project policy, task commands, protected hashes, allowed writes, and
  budgets. Record the new prompt hash in the round report (and tracker when
  ticketed); `verify` compares those immutable boundaries rather than treating
  a logged controller correction as a reason to void the candidate.
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
  the same conversation only after
  `DELIVERY_FAILED_ISOLATION_VERIFIED`, with an explicit character budget and
  the exact evidence surfaces that must remain. Never accept the raw log as
  the report.
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

For `bounded-write`, enumerate exact writable paths in `allowed_repo_writes`; do not use directory globs. Set `path_change_budgets` when a path should receive a small localized change. A generated lockfile that records a changed source digest normally replaces at least one line: give that exact lock path an explicit deletion allowance. For a source-only hand edit, `max_deleted: 4` may suffice; for an authoritative EC/IR lock regeneration, reserve at least `max_deleted: 12` (and a matching bounded addition allowance), because regenerated aggregate and source digests can replace several lines. A budget is an anti-explosion guard, not a reason to VOID a semantically expected generator delta. If the ticket does not need repository writes, keep `mode: measure-only`.
