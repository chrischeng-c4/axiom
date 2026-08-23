# Dispatcher Lifecycle

This document defines the worker-independent lifecycle behind `agy-dispatch`.
AGY is the worker adapter shipped today. A future Copilot, Claude, Codex, or
other worker adapter may replace AGY without changing the controller's task,
evidence, acceptance, publication, or handoff semantics.

The current `scripts/agy_dispatch.py` implementation is AGY-specific. This
document is the normative protocol that its behavior implements. A new worker
is not equivalent merely because it can edit files. Its adapter must produce the same
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
| Canonical worktree root and baseline | Shared | Maps root to worker scope |
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

- Canonical absolute worktree root.
- Repository identity and current revision.
- Stable Git admin/control bytes, every shared object byte, and semantic index
  entries/flags for every registered worktree.
- Worker adapter name and version.
- Worker isolation-scope id and its durable scope root.
- Permission-policy digest and inherited-policy status.
- Controller state directory.

The worktree root is the task-local baseline identity. The worker scope is an
adapter-specific binding. For AGY it is a Gemini Project id registered once
for a persistent app/repository worktree. A profile binds one clean linked
task worktree physically nested beneath that persistent root, launches AGY
from the task worktree as its process `cwd`, preserves a read-only baseline of
the persistent Project root, and uses distinct task state for every worktree.

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

### Baseline snapshot

- Worktree status and revision before the run.
- Bytes or hashes of protected artifacts.
- Frozen task-contract digest.
- Design and oracle digests.
- Worker scope and permission-policy digest.
- Conversation event floor from which new commands and writes are audited.
- Immutable snapshot id plus a controller marker proving that every event in a
  reused conversation before a new floor was independently verified. The
  marker hashes the steps schema and every column of every row through that
  exact ceiling, not only the visible command payload.

A snapshot is evidence about a moment, not a reusable token. Any change to the
contract, oracle, design, permissions, protected baseline, worktree binding, or
accepted implementation boundary requires a fresh snapshot.

### Normalized run record

For AGY this is a linked evidence set rather than one synthetic mega-record:

- Dispatcher evidence binds the task key, canonical attempt ordinal/suffix,
  snapshot id, conversation id, model, effort, exact launch cwd, terminal exit
  code, delivery classification, evidence schema, and command-audit contract
  version. A version change invalidates older reuse markers.
- File digests bind the rendered prompt, round contract, AGY log, raw output,
  and, only for a reported attempt, its normalized terminal report.
- The conversation `steps` database supplies post-snapshot shell requests and
  lifecycle replicas; verification separately derives changed paths and write
  budgets from the frozen repository baseline.
- The controller verdict records denials, independently executed checks,
  changed-path disposition, and semantic acceptance. Those facts are not
  invented as fields in the dispatcher evidence file.

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
WORKSPACE_BOUND
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

Any permission drift, protected-path mutation, unlisted command, escaped write,
task mismatch, or unrecoverable audit gap moves the run to VOID. VOID runs
are never resumed or accepted.

`FAILED_RETRYABLE -> RESUMED` applies only to ticketed work. Retry a one-shot
task as a new run id with a new snapshot; never attach it to the prior session.
```

`CLOSED` is optional. A user or outer lifecycle may require accepted work to
remain open for a later batch review. In that case `PUBLISHED` is terminal for
the dispatch without implying ticket closure.

## Lifecycle phases

### 0. Establish authority and route the task

1. Read the user's current instruction and, for ticketed work, the live issue.
2. Disclose that the task contract, oracle, and repository files viewed by the
   worker are transmitted to headless AGY; disclose repository diff handling
   for bounded-write mode. Resolve either a per-task explicit approval or the
   controller user's matching, revocable standing record at
   `~/.codex/agy-dispatch/standing-consent.json`. The standing record covers
   all bounded WIs only when its approved payload classes match this task; its
   full digest is bound into the profile snapshot. Never infer or fabricate
   consent, but do not repeat a matching standing approval for a new Project
   or clean worktree.
3. Confirm that delegation is authorized and identify what remains
   controller-only.
4. Classify the task:
   - implementation: bounded write, frozen design required;
   - measurement, investigation, review, or audit: measure-only, oracle
     required;
   - planning or HITL: do not disguise it as worker implementation.
5. Record whether issue closure is authorized. Default to no closure.
6. Inspect the worktree and preserve unrelated user or agent changes.

The dispatcher does not broaden authority. Permission to implement one task
does not imply permission to publish, close, change infrastructure, or repair
unrelated dirty files.

### 1. Bind the workspace to a worker scope

1. Resolve the canonical absolute worktree root.
2. Select a worker adapter.
3. Resolve one durable worker isolation scope for the persistent app/repo
   worktree.
4. Verify that the task root is a distinct clean linked worktree physically
   nested beneath the persistent Project root, ignored by that root, that both
   paths are exact worktree top levels of the same Git repository, and that
   controller state is outside the scope.
5. Choose a controller state directory scoped to that task under the stable
   worker scope.

For AGY, the isolation scope is an AGY Project registered for the persistent
app/repository root. The process is launched with its `cwd` equal to the active
in-Project task root, the persistent root is snapshotted read-only, and state lives below
`/tmp/agy-dispatch/<project-id>/<task-key>/`. Codex and Claude reuse the same
Project policy but never share a ticket's state directory.

One persistent app/repository worktree has one AGY Project. A task worktree,
controller, or ticket never creates a second Project: `root` is the clean
linked task worktree, `agy_project_root` is the persistent Project worktree,
`root` must be physically nested beneath and ignored by `agy_project_root`,
both paths must be exact worktree top levels, and both must resolve to the same
Git common directory. The adapter launches `agy --project <id>` with
`cwd=<task-root>`.

The dispatcher core itself does not require a per-task permission lease when
the worker's persistent policy is stable. If an outer controller uses a
logical ownership lease to prevent concurrent mutation, acquire it before the
first write and release it during handoff. That lease is separate from the
worker permission model.

For a newly migrated Project, prove this adapter mapping with a two-stage
activation canary: a measure-only root/common-dir probe whose recorded shell
`Cwd` equals the task root, followed by one serialized disposable-file write.
Protect sentinels in the persistent root and a synthetic sibling worktree in
the second snapshot. Do not claim AGY-native per-conversation worktree
confinement or enable parallel bounded writes merely because the outer process
was launched with `cwd=<task-root>`.

### 2. Validate the worker permission policy

The adapter must expose enough information for the controller to prove:

- the effective project/workspace policy matches the reviewed policy;
- inherited or global permissions are absent or explicitly admitted;
- destructive capabilities are denied;
- every shell-command `Cwd` and every repository mutation that escapes the
  exact task worktree can be detected and voided;
- task-local commands can be distinguished and audited;
- permission changes during a run are detectable.

Persistent worker policy and task contract are different layers:

- Persistent policy is reusable for normal safe work in the bound worktree.
- The task contract is narrower and exact for one ticket or one-shot run.

Do not rewrite persistent permissions per task. Put safe common capability in
the official **Global** baseline, add only repository-specific exceptions at
**Project** scope, and keep each task exact. Use `/permissions` or Project
Settings manually; never use Computer/UI automation or private registry/cache
files. Any policy change requires a fresh profile observation, `doctor`, and
snapshot. It never repairs AGY authentication.

For AGY, `doctor` reads the formally documented CLI Global settings file
`~/.gemini/antigravity-cli/settings.json` at `permissions`, compares it with
`global_permissions`, and binds its digest. AGY Projects inherit/augment that
baseline; AGY evaluates Deny > Ask > Allow. The current public CLI has no
machine-readable Project-list/root/effective-policy command, so doctor cannot
read a Project registry. It requires a dated `project_policy_observation` made
from the official Project selector/Settings or `/permissions` Project scope.
Absent, zero-match, or multi-match Project discovery is
`PROJECT_SETUP_REQUIRED`, never an automatic create/select/delete.

For the Project-local **Outside of Folder File Access** setting, the bounded
profile requires `always_deny`; `--sandbox` does not replace it. macOS AGY may
deny in-Project task-worktree terminal reads under sandboxing, so templates
default to `sandbox: false`; enable it only after a same-Project read-only probe. A
terminal sandbox access failure voids that run rather than widening policy.

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

1. Fail closed unless a current per-task or standing explicit approval covers
   every external payload class exposed by this mode. A missing, revoked, or
   changed standing record is a hard preflight/identity failure.
2. Re-read live ticket state for ticketed work; revalidate the frozen intent
   and run id for one-shot work.
3. Run the adapter's read-only preflight.
4. Reject root mismatch, policy drift, unresolved commands, missing design,
   missing oracle, dirty protected paths, or an ineligible task.
5. Capture the baseline snapshot.
6. Verify the snapshot contains the current contract, oracle, consent,
   worker-scope,
   permission, protected-artifact, persistent-root, sibling-worktree, and
   conversation-floor digests.

For AGY the sequence is:

Run each command from the repository root.

```bash
python3 scripts/agy_dispatch.py doctor PROFILE
python3 scripts/agy_dispatch.py snapshot PROFILE ISSUE
```

Always take a fresh snapshot after a contract, oracle, design, permission,
baseline, or workspace-binding change. When the task already has a
conversation, the dispatcher refuses a new snapshot until the latest attempt
evidence and exact conversation step ceiling have passed verification. A new
floor must never hide commands from an unverified predecessor.

### 5. Dispatch or resume one worker conversation

Use `dispatch` only when the task key has no existing worker conversation. Use
`resume` only for revisions and retryable failures of the same ticket. A
one-shot run never resumes; a follow-up must use a new run id and snapshot.
Every resume requires the controller verification marker for the latest
attempt and refuses if any newer conversation step appeared afterward.
Attempts are selected by their canonical contiguous ordinal (`initial`,
`.resume`, `.resume.N`), never by mutable filesystem timestamps. Every ordinal
must have a one-to-one canonical prompt, contract, AGY log, raw output, and
evidence set; reported attempts also require a normalized report. The resume
AGY log must identify the same stored conversation.

1. Launch the adapter as one direct long-lived host process.
2. Acquire the task-local nonblocking OS operation lock; refuse concurrent
   snapshot, launch, or verification against the same conversation.
3. Immediately before launch, recheck the snapshot-bound model, task root,
   worktree topology, Project identity, permission digest, persistent root,
   sibling worktrees, and initial task bytes. Any drift refuses the launch.
4. Keep polling the same process/session until terminal output arrives.
5. Do not treat wrapper completion, progress chatter, or exit code zero as a
   report.
6. Do not dispatch a second conversation for the same ticket or one-shot run
   id.
7. Do not change permissions or the task contract while the worker runs.

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

The adapter writes a snapshot-bound attempt record for every terminal process
result, including prompt, round-contract, AGY-log, raw-output, exit status,
conversation, model/effort/cwd, and failure-classification digests. Missing report, empty
output, nonzero exit, truncated evidence, or a worker claim that hides skipped
verification is a failed delivery. A report with `PASS` remains only
`REPORTED`.

Secrets, private keys, bearer tokens, and credential bodies must never be
copied into prompts, logs, or reports. Evidence should use redacted metadata,
digests, and independently reproducible selectors.

### 7. Verify isolation and contract integrity

Run dispatcher verification before semantic review. It must check:

- task, design, oracle, prompt, and baseline digests;
- worker scope and permission digest;
- protected artifacts;
- every post-snapshot shell request in the authoritative conversation `steps`
  table. Type-15 command requests are audited once; command-bearing lifecycle
  rows may only replay an already-seen identical command/Cwd, and mismatched or
  novel replica surfaces are VOID. `gen_metadata.data` is a cumulative display
  replica rather than an event authority and is deliberately excluded. A
  truthy `steps.has_subtrajectory` is VOID until child-trajectory traversal is
  implemented and independently audited;
- pre-floor conversation schema and full-row digests;
- stable Git admin/reflog/ref/hook/config bytes, registered-worktree semantic
  indexes, and shared Git object bytes;
- exact allowed-write paths and write budgets;
- conversation lineage and terminal report identity.

For AGY:

Run the command from the repository root.

```bash
python3 scripts/agy_dispatch.py verify PROFILE ISSUE
```

`ISOLATION_VERIFIED` means only that the worker stayed inside the frozen
envelope and produced an auditable report. It does not mean the code, analysis,
tests, or claimed acceptance criteria are correct.

For a nonzero, empty, or invalid-report attempt with an auditable conversation,
the same checks may produce `DELIVERY_FAILED_ISOLATION_VERIFIED`. This is a
non-acceptance retry state: it records the verified step ceiling and permits a
ticketed resume. It is not available when conversation lineage or attempt
evidence is missing.

Move the run to `VOID` when integrity cannot be proven. Typical causes are an
unlisted command, escaped write, protected mutation, permission drift,
conversation mismatch, task mismatch, or missing audit trail. Never resume,
accept, or publish a VOID run.

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
adapter classifies it retryable, verification returns
`DELIVERY_FAILED_ISOLATION_VERIFIED`, and the snapshot remains valid.

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

1. Stage the exact accepted paths; never use a broad staging sweep.
2. Inspect the staged path list and diff.
3. Run the final controller gates against the staged/integrated state.
4. Commit with required issue provenance for ticketed work or an explicit
   controller provenance record for one-shot work.
5. Push through the repository's authorized workflow.
6. Write a durable issue comment for ticketed work, or a controller record for
   one-shot work, with commit, gates, digests, skipped external gates, and
   remaining work.
7. Close a ticket only when the user or outer lifecycle explicitly authorizes
   closure and every required gate is satisfied.

The worker does not run Git mutation, merge, push, tracker mutation, or
closure commands. These remain controller responsibilities even if a future
worker technically has those capabilities.

### 11. Clean up or hand off

1. Confirm the live worker permission surface still matches the reviewed
   persistent policy.
2. Do not "restore" permissions per task when no stable policy changed.
3. Release any outer logical ownership lease.
4. Preserve unrelated worktree changes.
5. Retain ephemeral state while a same-ticket conversation revision is likely,
   or until a one-shot audit is complete.
6. If cleaning ephemeral state, first publish every durable verdict and
   remaining-step record.
7. Hand off the exact task, commit, accepted/unaccepted state, remaining
   gates, state paths, permission status, worker adapter, scope id, and safe
   next command.

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
| `DELIVERY_FAILED_ISOLATION_VERIFIED` | Failed delivery whose exact envelope and event ceiling were verified; ticketed resume only | No |
| `BLOCKED_HITL` | A real user decision or new authority is required | No |
| `VOID` | Integrity or isolation cannot be proven | Never |
| `CONTROLLER_ACCEPTED` | Independent semantic gates passed | Yes |
| `PUBLISHED` | Accepted change/report is durably committed or recorded | Already |
| `CLOSED` | Authorized tracker closure is complete | Already |

`REPORTED`, `ISOLATION_VERIFIED`, and `CONTROLLER_ACCEPTED` must remain
separate in logs, status output, and handoffs.

## Concurrency and worktrees

- At most one bounded-write AGY worker runs within one persistent Project at a
  time; a task-worktree `cwd` is not a sibling-worktree access boundary.
- Measure-only tasks may overlap only when they cannot observe a mutating
  baseline and the oracle remains valid.
- Parallel measure-only work uses distinct linked worktrees and independent
  snapshots under the same stable worker scope. Parallel bounded writes need a
  separately proven AGY-native per-conversation worktree confinement boundary.
- Controllers serialize integration, resolve merge order explicitly, and
  rerun affected gates after each merge.
- A worker or adapter that cannot prove root isolation is not eligible for
  parallel bounded-write dispatch.

This lets a future scheduler move from concurrency 1 to 2 to 4 while keeping
the verification contract constant. Throughput never weakens isolation.

## Adapter contract for another worker

A replacement adapter is eligible only if it can implement or explicitly
reject these operations:

1. `bind(root) -> worker_scope`
2. `inspect_effective_permissions(worker_scope) -> normalized_policy`
3. `preflight(workspace_binding, task_contract) -> findings`
4. `snapshot(workspace_binding, task_contract) -> baseline`
5. `start(task_contract, baseline) -> run_handle`
6. `resume(run_handle, revision_contract) -> run_handle` (ticketed only)
7. `poll(run_handle) -> progress_or_terminal`
8. `collect_audit(run_handle, event_floor) -> commands, denials, tools, writes`
9. `normalize_report(run_handle) -> normalized_report`
10. `classify_failure(run_handle) -> retryable | blocked | void`

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
| Worker scope | Gemini/AGY Project id for the persistent app/repository worktree |
| Effective permissions | documented CLI Global settings plus dated official Project UI observation; task contract is stricter |
| Per-run terminal confinement | Profile `sandbox: true` maps to AGY `--sandbox`; it is snapshot-bound but does not replace Project policy |
| Preflight | `python3 scripts/agy_dispatch.py doctor` |
| Baseline | `python3 scripts/agy_dispatch.py snapshot` |
| Start | `python3 scripts/agy_dispatch.py dispatch`; `agy --project <id>` with `cwd=<task-root>` |
| Resume | Ticketed-only `python3 scripts/agy_dispatch.py resume` using stored conversation id |
| Progress/terminal | Direct long-lived process and same host session polling |
| Audit | Conversation database command requests plus filesystem snapshot |
| Normalize | Raw log and final `## EXEC REPORT` under `state_dir/runs/` |
| Isolation verdict | `python3 scripts/agy_dispatch.py verify` |
| Ephemeral state | `/tmp/agy-dispatch/<project-id>/<task-key>/` |

AGY Project permissions are stable worker policy, not a task lease. Codex
and Claude are interchangeable controllers of the same AGY Project because
they share the Project registry and dispatcher state. They must not create
duplicate Projects or change policy during another controller's run.

## Non-negotiable invariants

- One bounded contract and one worker conversation per task key.
- One live ticket reuses exactly one conversation; one-shot work never resumes.
- Exact in-Project task worktree plus same-repository persistent Project binding and launch cwd are proven before dispatch.
- Implementation always has frozen design input; every task has an oracle.
- Permission policy and task-local command contract are separate.
- No unlisted command or escaped write is accepted.
- No worker report, exit code, or self-run test is controller acceptance.
- Skipped and unavailable gates are never reported as passing.
- Ticket revisions resume the same conversation unless the prior run is VOID.
- Worker output never contains secret material.
- Workers never commit, push, mutate trackers, or close work themselves.
- Only the controller records acceptance and publication.
- Changing workers changes adapter mechanics, not product or evidence
  standards.
