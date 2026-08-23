# Dispatcher Lifecycle

This document defines the worker-independent lifecycle behind `agy-dispatch`.
AGY is the worker adapter shipped today. A future Copilot, Claude, Codex, or
other worker adapter may replace AGY without changing the controller's task,
evidence, acceptance, publication, or handoff semantics.

The current `agy_dispatch.py` implementation is AGY-specific; this document is
the normative protocol that its behavior implements. A new worker is not
equivalent merely because it can edit files. Its adapter must produce the same
normalized state and evidence, or fail closed at the unsupported phase.

## Roles and trust boundary

- **User**: supplies authority, product intent, and any required HITL decision.
- **Controller**: owns the task contract, oracle, worker selection,
  independent verification, integration, publication, and closure.
- **Dispatcher core**: records the normalized lifecycle state and enforces the
  task, workspace, baseline, command, write, and evidence contracts.
- **Worker adapter**: translates the core protocol into one worker's project,
  permission, session, execution, audit, and report APIs.
- **Worker**: investigates or implements only the admitted task and returns
  a candidate evidence bundle. It never accepts or closes its own work.

The controller and worker are deliberately asymmetric. Worker output is
untrusted evidence until the dispatcher verifies isolation and the controller
independently verifies product semantics.

## What is shared and what is adapter-specific

| Concern | Dispatcher core | Worker adapter |
|---|---|---|
| Task identity, session policy, and optional live tracker state | Shared | No |
| Frozen design inputs and digests | Shared | No |
| Oracle and fabrication tells | Shared | No |
| Controller root, derived worker root, and baseline | Shared | Binds the worker scope to the derived root |
| Allowed writes, protected paths, budgets | Shared | Enforces or audits them |
| Exact task command contract | Shared | Maps it to worker permissions/tools |
| Persistent permission representation | No | AGY Project, Copilot policy, etc. |
| Starting/resuming a session | No | Worker-specific transport |
| Command requests, denials, and tool audit | Normalized | Collected worker-specifically |
| Terminal report schema | Shared | Normalizes native output |
| Isolation verification | Shared verdict | Supplies audit evidence |
| Semantic tests and negative controls | Controller-owned | No |
| Commit, push, durable verdict, tracker mutation | Controller-owned | No |
| Durable handoff record | Shared | Includes adapter-specific resume data |

Changing the worker changes only the right-hand column. It must not weaken the
left-hand contract or silently reinterpret a lifecycle state.

## Canonical records

The core lifecycle is based on these logical records. A concrete adapter may
store them differently, but must preserve their meaning.

### Workspace binding

- Controller root: the work area the controller itself occupies.
- Derived worker root: the round's own checkout and branch, plus the base
  revision it was cut from.
- Repository identity.
- Worker adapter name and version.
- Worker isolation-scope id for the work area.
- Permission-policy digest and inherited-policy status.
- Controller state directory.

The **work area** is the stable shared identity: one work area, one worker
scope (for AGY, one Gemini Project id), one state directory. The **worker root**
is per round — a derived worktree on a namespaced branch, cut from the
controller's current `HEAD` and destroyed when the round is released.

The worker scope is bound to one absolute root at a time, and the adapter
forces the worker's working directory to that root regardless of how the
controller invokes it. Reaching a derived worktree therefore means *moving* the
existing binding for the duration of the round, not registering a second scope.
Moving is preferred over cloning: the reviewed permission surface stays a single
object that cannot drift from a stale copy, and the registry does not grow one
entry per round. The binding move is recorded with the prior root so release can
restore it.

Two invariants follow. Rounds inside one work area are serial, because one scope
addresses one root at a time. And release must restore the home root *before* it
removes anything, so an interrupted round leaves at worst a stale pointer that
re-binding heals.

Physical separation is not a substitute for the write contract; it changes what
a violation costs. The exact write contract, protected baseline, and
snapshot/verify still decide what is in scope, but a write outside that scope
now lands in a disposable tree the controller reads, instead of damaging the
tree the controller is working in.

### Task contract

- Session policy: `ticketed` or `one-shot`.
- Ticketed identity: one live issue id and task kind.
- One-shot identity: one unique run id, frozen intent, and task kind.
- Frozen design inputs and SHA-256 digests for implementation.
- Oracle, expected result or hard floor, and fabrication tells.
- Exact allowed repository writes and per-path budgets.
- Protected artifacts and their expected bytes.
- Exact task-local shell commands and explicit deny probes.
- Required report witnesses and controller-owned gates.
- Optional revision delta that supersedes the prior prompt while preserving
  already accepted facts.

A ticket maps to one worker conversation. A revision resumes that conversation;
it does not silently create a second worker interpretation of the same ticket.
An unticketed task maps to one one-shot conversation that is retained for audit
but never resumed. A follow-up one-shot receives a new run id and conversation.

Minting that new run id is a revision, not a fresh round. The prior round's
candidate is uncommitted in the worker's checkout, so re-deriving the worktree
would discard it. `revise` carries the checkout, the write contract, the
protected set, and the budgets forward and changes only the run id and the
injected delta, so the revision is judged against the same tree under the same
ceiling. The oracle is inherited unchanged: a revision exists to satisfy the
sealed claim, not to move it.

### Baseline snapshot

- Worktree status and revision before the run.
- Bytes or hashes of protected artifacts.
- Frozen task-contract digest.
- Design and oracle digests.
- Worker scope and permission-policy digest.
- Conversation event floor from which new commands and writes are audited.

A snapshot is evidence about a moment, not a reusable token. Any change to the
contract, oracle, design, permissions, protected baseline, worktree binding, or
accepted implementation boundary requires a fresh snapshot.

### Normalized run record

- Task key, optional ticket id, worker adapter, worker scope, process/run id,
  and conversation id.
- Start/end times and terminal process status.
- Rendered prompt digest.
- All post-snapshot command requests, denials, and tool events available from
  the worker.
- Changed paths and write counts.
- Raw output plus one normalized terminal report.
- Resume lineage and failure classification.

### Controller verdict

- Dispatcher isolation result.
- Semantic review findings.
- Independently run commands and their observed results.
- Negative-control and false-green findings.
- Accepted commit, issue comment or controller record, publication state, and
  remaining work.

The controller verdict is the only acceptance record. A worker report may
claim `PASS`; that claim never advances the lifecycle by itself.

## State machine

```text
DISCOVERED
    |
    v
WORKTREE_DERIVED        (branch cut, base frozen)
    |
    v
WORKSPACE_BOUND         (worker scope moved to the derived root)
    |
    v
CONTRACT_FROZEN
    |
    v
PREFLIGHTED
    |
    v
SNAPSHOTTED
    |
    v
RUNNING -----------------------> FAILED_RETRYABLE
    |                                  |
    v                                  | ticketed resume, if baseline remains valid
REPORTED                               |
    |                                  |
    v                                  |
ISOLATION_VERIFIED <-------------------+
    |
    +---- semantic defect ----> REVISION_REQUIRED
    |                                  |
    |                    exact delta + fresh snapshot when required
    |                                  |
    +<----------------------------- RESUMED
    |
    v
CONTROLLER_ACCEPTED
    |
    v
PUBLISHED
    |
    v
CLOSED

Every path out of WORKTREE_DERIVED — CLOSED, PUBLISHED, VOID, or abandonment —
ends at RELEASED: home root restored first, then worktree and branch removed.

Permission drift, an altered dispatch contract, a swapped worker scope, an
unlisted command, a task mismatch, a missing oracle, or an unrecoverable audit
gap moves the run to VOID. VOID runs are never resumed or accepted.

Nothing the worker did to its own checkout is in that list. Every scope
violation — an unauthorized write, a protected-path mutation, a budget
overrun, a moved branch head — is a *finding*: the round still reaches
`REPORTED`, `ISOLATION_VERIFIED` reports the overrun rather than failing, and
the controller adjudicates it at `CONTROLLER_ACCEPTED` by reading the diff.

The distinction is what the violation costs. An integrity failure means the
evidence cannot be trusted at all; a scope overrun in a disposable tree costs
one read. Voiding on scope was correct when the worker shared the controller's
tree; with a derived worktree it throws away complete in-scope work over a
recoverable mistake. The protected set is the *complement* of the write scope,
so voiding on it would reinstate that behavior under another name.

`FAILED_RETRYABLE -> RESUMED` applies only to ticketed work. Retry a one-shot
task as a new run id with a new snapshot; never attach it to the prior session.
```

`CLOSED` is optional. A user or outer lifecycle may require accepted work to
remain open for a later batch review. In that case `PUBLISHED` is terminal for
the dispatch without implying ticket closure.

## Lifecycle phases

### 0. Establish authority and route the task

1. Read the user's current instruction and, for ticketed work, the live issue.
2. Confirm that delegation is authorized and identify what remains
   controller-only.
3. Classify the task:
   - implementation: bounded write, frozen design required;
   - measurement, investigation, review, or audit: measure-only, oracle
     required;
   - planning or HITL: do not disguise it as worker implementation.
4. Record whether issue closure is authorized. Default to no closure.
5. Inspect the controller root. Uncommitted work is preserved, but note that
   the round is cut from `HEAD`, so uncommitted changes are invisible to the
   worker. Commit or stash anything the task depends on before deriving.

The dispatcher does not broaden authority. Permission to implement one task
does not imply permission to publish, close, change infrastructure, or repair
unrelated dirty files.

### 1. Derive the worker root and bind the worker scope

1. Resolve the controller root and confirm it is a checkout.
2. Select a worker adapter.
3. Derive the round's worker root: a new branch under the reserved namespace,
   cut from the controller's current `HEAD`, checked out at a path outside the
   controller root. Freeze the base revision.
4. Move the work area's single worker isolation scope to the derived root and
   record the prior root for release.
5. Verify that the scope points to the exact derived root, not merely the same
   repository or branch name.
6. Choose one controller state directory shared by controllers using that
   worker scope.

For AGY, the isolation scope is an AGY Project, the reserved branch namespace
is `agy/`, and the shared state directory is `/tmp/agy-dispatch/<project-id>/`.
Codex and Claude reuse the same Project and state directory, and therefore must
not run rounds against one work area concurrently.

Moving the scope rewrites only its root binding. Its permission grants are the
same object before and after, so the digest validated in phase 2 is unaffected
by deriving.

The dispatcher core itself does not require a per-task permission lease when
the worker's persistent policy is stable. If an outer controller uses a
logical ownership lease to prevent concurrent mutation, acquire it before the
first write and release it during handoff. That lease is separate from the
worker permission model.

### 2. Validate the worker permission policy

The adapter must expose enough information for the controller to prove:

- the effective project/workspace policy matches the reviewed policy;
- inherited or global permissions are absent or explicitly admitted;
- destructive capabilities are denied;
- the worker cannot silently escape the exact worktree;
- task-local commands can be distinguished and audited;
- permission changes during a run are detectable.

Persistent worker policy and task contract are different layers:

- Persistent policy is reusable for normal safe work in the bound worktree.
- The task contract is narrower and exact for one ticket or one-shot run.

Do not rewrite persistent permissions per task. If a genuinely reusable
capability is missing, change it through the worker's supported permission UI
or API, update the reviewed profile, run preflight again, and take a fresh
snapshot. Never edit a worker's internal permission database directly.

AGY implements this phase with Project-scoped `/permissions`, an empty Global
surface by default, and `doctor`. Another adapter may use a different
permission system, but it must establish equivalent evidence. If it cannot,
it must declare the phase unsupported rather than pretending isolation.

### 3. Freeze the task contract and oracle

1. Select the session policy. Bind ticketed work to one live issue; bind
   unticketed work to one unique run id and frozen intent.
2. Select and freeze implementation design inputs.
3. Write the independent oracle under controller state.
4. Define exact admitted writes, budgets, protected artifacts, commands, deny
   probes, required witnesses, and verification gates.
5. Compute design and oracle digests.
6. Record those digests durably in the issue for ticketed work or the
   controller log for one-shot work.

The oracle must be independently derivable. It names expected exact sets,
denominators, invariants, hard floors, or negative controls plus at least one
fabrication tell. Restating the worker's expected report is not an oracle.

The tracker comment and accepted commit are durable. Files under `/tmp` are
ephemeral working state. If ephemeral state is lost, reconstruct the contract
from durable records, create a fresh snapshot, and start a new auditable run.
Never invent missing resume state.

### 4. Preflight and snapshot

1. Re-read live ticket state for ticketed work; revalidate the frozen intent
   and run id for one-shot work.
2. Run the adapter's read-only preflight.
3. Reject root mismatch, policy drift, unresolved commands, missing design,
   missing oracle, dirty protected paths, or an ineligible task.
4. Capture the baseline snapshot.
5. Verify the snapshot contains the current contract, oracle, worker-scope,
   permission, protected-artifact, and conversation-floor digests.

For AGY the sequence is:

```bash
python3 agy_dispatch.py doctor PROFILE
python3 agy_dispatch.py snapshot PROFILE ISSUE
```

Always take a fresh snapshot after a contract, oracle, design, permission,
baseline, or workspace-binding change.

### 5. Dispatch or resume one worker conversation

Use `dispatch` only when the task key has no existing worker conversation. Use
`resume` only for revisions and retryable failures of the same ticket. A
one-shot run never resumes; a follow-up must use a new run id and snapshot.

1. Launch the adapter as one direct long-lived host process.
2. Keep polling the same process/session until terminal output arrives.
3. Do not treat wrapper completion, progress chatter, or exit code zero as a
   report.
4. Do not dispatch a second conversation for the same ticket or one-shot run
   id.
5. Do not change permissions or the task contract while the worker runs.

The worker receives only the normalized contract plus adapter-required
orientation. Product acceptance criteria remain identical across adapters.

### 6. Normalize and triage the worker report

The adapter preserves raw output and emits one self-contained normalized
terminal report containing:

- task key, optional ticket id, and run identity;
- claimed acceptance-criterion disposition;
- exact changed paths;
- exact commands/tool actions performed;
- required witnesses and evidence locations;
- denied actions, skipped gates, and unfinished steps;
- error or retry classification when delivery failed.

Missing report, empty output, nonzero exit, truncated evidence, or a worker
claim that hides skipped verification is a failed delivery. A report with
`PASS` remains only `REPORTED`.

Secrets, private keys, bearer tokens, and credential bodies must never be
copied into prompts, logs, or reports. Evidence should use redacted metadata,
digests, and independently reproducible selectors.

### 7. Verify isolation and contract integrity

Run dispatcher verification before semantic review. It must check:

- task, design, oracle, prompt, and baseline digests;
- worker scope and permission digest;
- protected artifacts;
- every post-snapshot command request and denial;
- exact allowed-write paths and write budgets;
- conversation lineage and terminal report identity.

For AGY:

```bash
python3 agy_dispatch.py verify PROFILE ISSUE
```

`ISOLATION_VERIFIED` means only that the worker stayed inside the frozen
envelope and produced an auditable report. It does not mean the code, analysis,
tests, or claimed acceptance criteria are correct.

Verification separates two outcomes:

- **Integrity failure → `VOID`.** Unlisted command, permission drift,
  conversation mismatch, task mismatch, swapped worker scope, missing oracle,
  or a missing audit trail. The evidence itself cannot be trusted, so nothing
  about the round is salvageable. Never resume, accept, or publish a VOID run.
- **Scope finding → report and continue.** A write outside
  `allowed_repo_writes`, a changed protected artifact, a declared path left
  unwritten, a per-path budget overrun, or a moved branch `HEAD`. The worker's
  checkout is disposable, so these are inputs to phase 8, surfaced with the
  diff for the controller to adjudicate. Distinct exit statuses must keep the
  two apart.

Report scope findings even when the round is otherwise clean. Silently
tolerating an overrun is how a contract stops describing the work.

### 8. Perform controller-owned semantic acceptance

The controller now verifies the work without trusting the report:

1. Inspect the exact diff and every admitted new file.
2. Compare behavior with the frozen task, live issue when present, and frozen
   design.
3. Recompute exact sets, denominators, classifications, and invariants.
4. Run focused tests and required integration gates independently.
5. Prove negative controls and failure paths, not only the happy path.
6. Distinguish executed, skipped, unavailable, and simulated gates.
7. Check for false-green tests whose fixtures or assertions bypass production.
8. Check interactions with unrelated dirty work without modifying it.
9. Record precise findings and an acceptance or revision verdict.

A green worker test, report, snapshot, structural match, or process exit is
never a substitute for this phase.

### 9. Revise or retry

For a semantic defect in ticketed work:

1. Keep the same ticket and worker conversation.
2. Write an exact delta naming observed evidence, required correction, protected
   accepted facts, and controller gates still pending.
3. Narrow the write envelope when possible.
4. Update the profile and take a fresh snapshot if any frozen contract field
   changed.
5. Resume, normalize the replacement report, verify isolation again, and
   repeat controller acceptance.

For a transient backend or transport failure, resume unchanged only when the
adapter classifies it retryable and the snapshot remains valid.

For one-shot work, record the failed run and its conversation id for audit,
then create a new run id, oracle, and snapshot if the controller chooses to
retry. Never invoke the adapter's resume operation.

For a denied command, decide whether it is unnecessary or a genuinely reusable
project capability. Remove unnecessary work from the prompt. For a reusable
capability, change persistent permissions through the worker-supported
surface, update the reviewed profile, rerun preflight, and take a fresh
snapshot.

Repeated semantic corrections keep concurrency at one. Increase parallelism
only after the worker repeatedly returns independently accepted bounded work.

### 10. Integrate, publish, and optionally close

Only after `CONTROLLER_ACCEPTED`:

1. Stage the exact accepted paths on the round's branch; never use a broad
   staging sweep.
2. Inspect the staged path list and diff.
3. Commit on the round's branch with required issue provenance for ticketed
   work or an explicit controller provenance record for one-shot work. This
   commit is a candidate, not integration.
4. Integrate that commit into the controller's branch — cherry-pick or merge —
   from the controller root.
5. Run the final controller gates against the integrated state, not against
   the worker's checkout.
6. Push through the repository's authorized workflow.
7. Write a durable issue comment for ticketed work, or a controller record for
   one-shot work, with commit, gates, digests, skipped external gates, and
   remaining work.
8. Close a ticket only when the user or outer lifecycle explicitly authorizes
   closure and every required gate is satisfied.

The worker does not run Git mutation, merge, push, tracker mutation, or
closure commands. These remain controller responsibilities even if a future
worker technically has those capabilities. Committing the accepted candidate
onto the round's own branch is a controller action performed after acceptance;
it is not the worker touching Git.

### 11. Release the round, then clean up or hand off

1. Restore the worker scope's home root **before** removing anything. An
   interrupted release must leave a stale pointer, never a scope pointing at a
   deleted tree.
2. Remove the round's worktree and delete its branch, refusing any branch
   outside the reserved namespace. Keep the branch when the candidate is still
   under review.
3. Confirm the live worker permission surface still matches the reviewed
   persistent policy.
4. Do not "restore" permissions per task when no stable policy changed.
5. Release any outer logical ownership lease.
6. Preserve unrelated controller-root changes; the controller's tree was never
   in scope for the round.
7. Retain ephemeral state while a same-ticket conversation revision is likely,
   or until a one-shot audit is complete.
8. If cleaning ephemeral state, first publish every durable verdict and
   remaining-step record.
9. Hand off the exact task, commit, accepted/unaccepted state, remaining
   gates, state paths, permission status, worker adapter, scope id, and safe
   next command.

Release is mandatory, not hygiene. A round left unreleased strands a checkout
and a branch, and leaves the shared worker scope pointing somewhere no other
session expects.

A handoff is a snapshot, not authority to repeat stale claims. The next
controller rechecks the task identity, live issue when present, branch,
worktree, permissions, and baseline before resuming.

## Terminal and nonterminal states

| State | Meaning | May publish? |
|---|---|---|
| `REPORTED` | Worker returned a candidate report | No |
| `ISOLATION_VERIFIED` | Envelope and audit checks passed | No |
| `REVISION_REQUIRED` | Controller found a semantic or evidence defect | No |
| `FAILED_RETRYABLE` | Delivery failed without invalidating the snapshot | No |
| `BLOCKED_HITL` | A real user decision or new authority is required | No |
| `VOID` | Integrity or isolation cannot be proven | Never |
| `CONTROLLER_ACCEPTED` | Independent semantic gates passed | Yes |
| `PUBLISHED` | Accepted change/report is durably committed or recorded | Already |
| `CLOSED` | Authorized tracker closure is complete | Already |

`REPORTED`, `ISOLATION_VERIFIED`, and `CONTROLLER_ACCEPTED` must remain
separate in logs, status output, and handoffs.

## Concurrency across projects

- At most one round runs against one work area at a time. This is enforced by
  construction: the work area's single worker scope addresses one root at a
  time, so a second concurrent round would have to steal the first one's
  binding.
- Measure-only tasks may overlap only when they cannot observe a mutating
  baseline and the oracle remains valid.
- Parallelism comes from work areas that already have their own registered
  worker scope. Two concurrent bounded-write dispatches require distinct
  scopes, distinct controller roots, disjoint task/file ownership, and
  independent snapshots.
- Deriving a worktree raises *safety*, not concurrency. It removes the
  controller-vs-worker diff ambiguity and makes a scope overrun readable
  instead of destructive; it does not let one work area run two rounds.
- Controllers serialize integration, resolve merge order explicitly, and
  rerun affected gates after each merge. Accepted rounds land as a commit on
  the round's branch and are integrated by the controller, never by the worker.
- A worker or adapter that cannot prove root isolation is not eligible for
  parallel bounded-write dispatch.

This lets a future scheduler move from concurrency 1 to 2 to 4 while keeping
the verification contract constant. Throughput never weakens isolation.

## Adapter contract for another worker

A replacement adapter is eligible only if it can implement or explicitly
reject these operations:

1. `derive(controller_root, task_key) -> worker_root, branch, base_revision`
2. `bind(worker_scope, worker_root) -> previous_root`
3. `release(worker_scope, previous_root, worker_root)` — restores first,
   removes second
4. `inspect_effective_permissions(worker_scope) -> normalized_policy`
5. `preflight(workspace_binding, task_contract) -> findings`
6. `snapshot(workspace_binding, task_contract) -> baseline`
7. `start(task_contract, baseline) -> run_handle`
8. `resume(run_handle, revision_contract) -> run_handle` (ticketed only)
8a. `revise(run_handle, revision_contract) -> task_contract` (one-shot; mints
    the next run id on the same checkout)
9. `poll(run_handle) -> progress_or_terminal`
10. `collect_audit(run_handle, event_floor) -> commands, denials, tools, writes`
11. `normalize_report(run_handle) -> normalized_report`
12. `classify_failure(run_handle) -> retryable | blocked | void`
13. `classify_scope(baseline, writes, contract) -> findings`

An adapter whose `bind` cannot be moved and restored, or whose worker ignores
the requested working directory without exposing where it actually ran, is not
eligible for derived-worktree dispatch and must fall back to serial in-project
rounds with scope treated as VOID.

For example, a Copilot adapter may use a different workspace registration and
approval model. That is acceptable only if it binds the exact root, exposes
effective permissions and inherited rules, preserves session lineage, audits
post-snapshot commands and writes, and produces the same normalized report.
If Copilot cannot expose one of those proofs, the adapter must narrow its
supported modes or fail closed; the controller must not infer equivalence from
the UI.

## AGY adapter mapping

| Core concept | AGY implementation |
|---|---|
| Worker scope | One Gemini/AGY Project id per work area |
| Derive + bind | `agy_dispatch.py worktree` (branch `agy/<task-key>`) |
| Release | `agy_dispatch.py discard` (`--keep-branch` to retain the candidate) |
| Effective permissions | Project `/permissions` plus checked Global surface |
| Preflight | `agy_dispatch.py doctor` |
| Baseline | `agy_dispatch.py snapshot` |
| Start | `agy_dispatch.py dispatch` |
| Resume | Ticketed-only `agy_dispatch.py resume` using stored conversation id |
| Revise | One-shot `agy_dispatch.py revise` — new run id, same checkout and contract |
| Progress/terminal | Direct long-lived process and same host session polling |
| Audit | Conversation database command requests plus filesystem snapshot |
| Normalize | Raw log and final `## EXEC REPORT` under `state_dir/runs/` |
| Isolation verdict | `agy_dispatch.py verify` (exit `1` VOID, `2` findings) |
| Scope adjudication | `agy_dispatch.py review` then `accept` |
| Ephemeral state | `/tmp/agy-dispatch/<project-id>/` |

AGY Project permissions are stable worker policy, not a task lease. Codex
and Claude are interchangeable controllers of the same AGY Project because
they share the Project registry and dispatcher state. They must not create
duplicate Projects or change policy during another controller's run, and —
because a round moves the Project's root — must not run rounds against one work
area concurrently.

`agy --project <id>` forces the worker's working directory to the Project's
registered folder and ignores the caller's cwd. That measured behavior is why
`derive` and `bind` are separate operations here and why the binding moves
rather than being cloned.

## Non-negotiable invariants

- One bounded contract and one worker conversation per task key.
- One live ticket reuses exactly one conversation; one-shot work never resumes.
- The worker works in its own derived checkout, never in the controller's.
- Exact worker-root binding is proven before dispatch.
- The worker scope's home root is restored before any teardown.
- Implementation always has frozen design input; every task has an oracle.
- Permission policy and task-local command contract are separate.
- No unlisted command is accepted; every out-of-scope write is reported.
- No worker report, exit code, or self-run test is controller acceptance.
- Skipped and unavailable gates are never reported as passing.
- Ticket revisions resume the same conversation unless the prior run is VOID.
- Worker output never contains secret material.
- Workers never commit, push, mutate trackers, or close work themselves.
- Only the controller records acceptance and publication.
- Changing workers changes adapter mechanics, not product or evidence
  standards.
