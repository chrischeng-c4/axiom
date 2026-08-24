# test-harness — architecture

Mamba's test harness turns external commands into auditable oracle evidence.
The bounded context owns process execution, deadlines, output capture, and
terminal classification. It does not own runtime semantics or the contents of
the probes it executes.

## Domain model

- **CommandSpec** is a validated argv vector plus working directory. Repository
  EC commands are data, not shell programs; parsing must reject control
  operators and must not route trusted rows through `sh -c`.
- **ExecutionAttempt** is the aggregate root. It owns one child process, its
  monotonic deadline, stdout/stderr readers, terminal status, and resource
  reaping. Exactly one terminal transition is permitted:
  `Exited(status)` or `TimedOut(signal)`.
- **ProcessIdentity** identifies the spawned process group, not only its first
  wrapper PID. Timeout cleanup owns every descendant created for the attempt.
- **CommandEvidence** is the immutable result: exit code or signal, timeout
  flag, stdout/stderr bytes and digests, elapsed time, and the command identity
  that produced them.
- **Classification** reconciles CommandEvidence with an oracle row only after
  the process aggregate has reached a reaped terminal state.

## Aggregate invariants

1. A child that exits before its deadline cannot later be classified timed out.
2. A timed-out attempt kills and reaps its whole process group; no descendant
   may retain captured pipes or survive into a later row.
3. Stdout and stderr are drained without waiting for an unrelated descendant,
   and their reader threads are joined before evidence is returned.
4. The watchdog and waiter share one terminal transition. Deadline expiry and
   normal exit race through that transition instead of independently setting
   contradictory flags.
5. The evidence runner never changes the subprocess's semantic environment
   relative to executing the same argv from the repository root.
6. Timeout values are contract data. Increasing a timeout is not a repair
   unless a measured successful command legitimately exceeds the old bound.

## Oracle-hierarchy execution flow

1. Parse and validate `oracle_command` and `sut_command` into CommandSpec.
2. Spawn the command in a fresh process group with piped stdout/stderr.
3. Concurrently drain both pipes while waiting on the child and the monotonic
   deadline.
4. Commit exactly one terminal state; on timeout terminate the group, then
   reap it.
5. Join readers, calculate digests, and create CommandEvidence.
6. Reconcile green or intentional-red only from that evidence.

The current #2010 regression is at this boundary: the behavior probe completes
from the repository shell in under one second after warm build, while
`run_command_with_evidence` repeatedly marks the same SUT row timed out at 30
seconds. The repair belongs to the runner, not to `asyncio.to_thread`.

## Repair scope and proof

The implementation seam is
`tests/external_contracts/mamba_core_semantics_ec.rs`, principally
`run_command_with_evidence`. `src/**`, probes, oracle classifications, and
timeouts are read-only for this repair.

Acceptance requires:

- a fast-command canary that cannot receive a late timeout;
- a deliberate direct-child timeout canary;
- a descendant-process timeout canary proving group cleanup and pipe closure;
- the direct `to_thread_gather_behavior_green.py` SUT command remains green;
- the exact `oracle_hierarchy_and_result_identity` release EC passes twice
  consecutively with all eight rows;
- no live probe process remains after either success or timeout.
