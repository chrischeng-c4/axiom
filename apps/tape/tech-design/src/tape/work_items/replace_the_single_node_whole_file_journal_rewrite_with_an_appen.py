"""Group-commit append-only WAL for tape's single-node durable event journal.

DDD framing:

- Bounded context: Durable Event Journal (the single-node `tape serve
  --data-dir` storage path).
- Domain invariant: every acknowledged mutation (append, checkpoint,
  subscription create/delete/ack, retention update) survives a process
  crash with RPO=0 -- an ack is never returned before the fsync that
  covers it has returned.
- Application operation: commit one batch of pending `TapeCommand`s behind
  one fsync barrier (group commit), then apply the whole batch to the
  in-memory `TapeJournal` under one lock acquisition.
- Infrastructure policy: on-disk representation becomes an append-only,
  frame-encoded write-ahead log plus periodic snapshots, replacing the
  current whole-file JSON rewrite in `AppState::persist`
  (apps/tape/src/server.rs).

Defect this design replaces: `AppState::persist` re-serializes and
rewrites the *entire* journal file on every single mutating request, then
fsyncs the whole file. Load testing measured a flat ~85-89 ops/s ceiling
regardless of concurrency -- the bottleneck is the per-request fsync
barrier on a whole-file rewrite, not lock contention (the in-process
journal lock is held only briefly; the fsync call dominates wall time).
Group commit amortizes one fsync over many queued commands instead of one
fsync per command.

Key insight this design is built on: the WAL must log `TapeCommand`
values, not `TapeJournal` snapshots or post-mutation state. `TapeJournal::
append_at` (apps/tape/src/lib.rs) calls `enforce_retention`, which can
*delete* events as a side effect of appending. Logging post-state would
silently discard the deleted-event history a replay needs in order to
reconstruct consumer checkpoints deterministically; logging the *command*
and replaying it through the same `apply_command` path used by
`TapeStateMachine::apply` (apps/tape/src/raft.rs) reproduces retention
enforcement identically on every replay.

Explicit non-goals / boundaries (unchanged by this design):

- `libs/storage-durable` is read-only reference infrastructure; this work
  reuses `FramedLogWriter`, `FramedLogReader`, and `SnapshotFileStore` as
  they exist today. No changes land under `libs/`.
- Snapshot and backup wire formats are unchanged.
- The Raft-replicated multi-node path (`TapeRaft`, `TapeStateMachine::
  apply`) is untouched; it gains a shared `apply_command` free function via
  extraction, but its call site and on-wire replication behavior do not
  change.
- `FsyncPolicy::Always` is never downgraded by this design; group commit
  changes *how many* commands one fsync covers, never whether a covering
  fsync happens before ack.
- The legacy `--store <file>` whole-file JSON path (used by roughly ten
  offline CLI verbs in apps/tape/src/bin/tape.rs) is unchanged; only the
  `serve --data-dir` path moves to the WAL.

@spec #3052
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import StrEnum

__aw_artifact_id__ = "artifact:durable-event-journal/replace-the-single-node-whole-file-journal-rewrite-with-an-appen-wi-3052"
__aw_work_item__ = "3052"


class TapeCommandKind(StrEnum):
    """The existing reusable command vocabulary (`TapeCommand` in
    apps/tape/src/raft.rs). The WAL logs these values, not journal state;
    values are the exact Rust enum variant names for traceability."""

    APPEND = "Append"
    CHECKPOINT_PUT = "CheckpointPut"
    SUBSCRIPTION_CREATE = "SubscriptionCreate"
    SUBSCRIPTION_DELETE = "SubscriptionDelete"
    SUBSCRIPTION_ACK = "SubscriptionAck"
    RETENTION_PUT = "RetentionPut"


@dataclass(frozen=True)
class ApplyCommandExtractionContract:
    """The prerequisite refactor this design depends on."""

    source_module: str
    source_type: str
    source_method: str
    extracted_function_name: str
    commands_covered: tuple[TapeCommandKind, ...]
    invariant: str


def apply_command_extraction_contract() -> ApplyCommandExtractionContract:
    """Extract a free `apply_command(journal, command) -> TapeOutcome`
    function out of `TapeStateMachine::apply`'s match arms so the new
    single-node WAL replay/coordinator path and the existing
    Raft-replicated apply path share one mutation implementation."""

    return ApplyCommandExtractionContract(
        source_module="apps/tape/src/raft.rs",
        source_type="TapeStateMachine",
        source_method="apply",
        extracted_function_name="apply_command",
        commands_covered=tuple(TapeCommandKind),
        invariant=(
            "both callers (Raft apply and the single-node WAL commit "
            "coordinator) must produce identical retention enforcement and "
            "identical TapeOutcome values for the same TapeCommand; the "
            "extraction itself must not change TapeStateMachine::apply's "
            "observable behavior"
        ),
    )


@dataclass(frozen=True)
class WalFrameContract:
    """What one WAL frame encodes, and why it cannot be journal state."""

    encodes: str
    reason: str


def wal_frame_contract() -> WalFrameContract:
    """The WAL logs TapeCommand values; it never logs TapeJournal
    snapshots or other post-mutation state."""

    return WalFrameContract(
        encodes="TapeCommand",
        reason=(
            "TapeJournal::append_at (apps/tape/src/lib.rs) calls "
            "enforce_retention as part of appending, which can delete "
            "events; logging post-state would lose the deleted-event "
            "history a deterministic replay needs, while logging the "
            "command and replaying it through apply_command reproduces "
            "retention enforcement identically every time"
        ),
    )


class JournalStoreKind(StrEnum):
    """Which on-disk journal store a `tape serve` invocation resolves to."""

    LEGACY_FILE = "legacy_file"
    WAL = "wal"
    NONE = "none"


def resolve_journal_store_kind(
    explicit_store: bool, data_dir: bool, replica_mode: bool
) -> JournalStoreKind:
    """Reference decision table for the JournalStore-enum refactor of
    `resolve_journal_store` (apps/tape/src/bin/tape.rs).

    Today, `--data-dir` without an explicit `--store` resolves to a legacy
    `journal.json` whole-file path. This design changes only that one
    branch: `--data-dir` now resolves to the new WAL store. An explicit
    `--store <file>` always keeps the unchanged legacy whole-file JSON
    path (used by the offline CLI verbs); replica mode keeps resolving to
    no local store.
    """

    if explicit_store:
        return JournalStoreKind.LEGACY_FILE
    if replica_mode:
        return JournalStoreKind.NONE
    if data_dir:
        return JournalStoreKind.WAL
    return JournalStoreKind.NONE


@dataclass(frozen=True)
class CommitRequest:
    """One caller's pending mutation waiting on the commit coordinator."""

    request_id: int
    command: TapeCommandKind


@dataclass(frozen=True)
class CommitBatch:
    """One fsync barrier's worth of commit requests, in submission order."""

    requests: tuple[CommitRequest, ...]


def drain_pending_commit_batch(
    pending: tuple[CommitRequest, ...], max_batch_size: int
) -> CommitBatch:
    """Drain up to `max_batch_size` pending requests as one batch.

    Batch membership preserves submission (FIFO) order; anything queued
    after this drain snapshot is taken joins the *next* batch. This is the
    "batch drain" step of group commit.
    """

    return CommitBatch(requests=tuple(pending[:max_batch_size]))


def apply_order_for_batch(batch: CommitBatch) -> tuple[int, ...]:
    """Return request ids in the exact order the single lock-scope apply
    step must apply them: unchanged submission order. Group commit changes
    fsync cardinality; it never changes apply ordering.
    """

    return tuple(request.request_id for request in batch.requests)


class CommitOutcome(StrEnum):
    """Terminal result of committing one batch."""

    ACKED = "acked"
    FAILED_FSYNC = "failed_fsync"
    FAILED_APPLY = "failed_apply"


def commit_batch_outcome(
    write_ok: bool, fsync_ok: bool, apply_ok: bool
) -> CommitOutcome:
    """Fail-closed commit decision: never ack before the covering fsync.

    RPO=0 invariant: if the batch write or its covering fsync did not
    succeed, no request in the batch may be acknowledged and none of its
    commands may be applied to the in-memory journal -- the coordinator
    fails that batch closed, matching today's `FsyncPolicy::Always`
    guarantee, only amortized over a batch instead of one command.
    """

    if not write_ok or not fsync_ok:
        return CommitOutcome.FAILED_FSYNC
    if not apply_ok:
        return CommitOutcome.FAILED_APPLY
    return CommitOutcome.ACKED


STORAGE_DEGRADED_REPROBE_INTERVAL_SECONDS = 30


def should_enter_storage_degraded_mode(
    outcome: CommitOutcome, is_enospc: bool, is_eio: bool
) -> bool:
    """WI R6: a durability failure caused by ENOSPC *or* EIO latches sticky
    degraded mode via the existing `TapeMetrics::mark_storage_degraded`
    (apps/tape/src/metrics.rs) and its 507 `storage_full` response
    envelope, re-probed every `STORAGE_DEGRADED_REPROBE_INTERVAL_SECONDS`
    seconds by the existing `spawn_storage_full_reprobe` (apps/tape/src/
    bin/tape.rs). Narrowing this predicate to ENOSPC alone would route a
    durable EIO into a plain per-batch failure -- exactly the "durability
    failure treated as an ordinary retryable request" R6 forbids. A batch
    failure that is neither ENOSPC nor EIO still fails that one batch
    closed but does not flip the server into sticky read-only degraded
    mode.
    """

    return outcome is not CommitOutcome.ACKED and (is_enospc or is_eio)


class RecoveryAction(StrEnum):
    """Startup decision after scanning the on-disk WAL."""

    START_EMPTY = "start_empty"
    REPLAY = "replay"
    TRUNCATE_TORN_TAIL = "truncate_torn_tail"


def recovery_action_for_wal_scan(
    has_existing_frames: bool, tail_is_torn: bool
) -> RecoveryAction:
    """Startup recovery over the WAL never hard-fails on a torn tail.

    Contrast with today's `load_journal` (apps/tape/src/bin/tape.rs), which
    treats any undecodable file content as a fatal error. The WAL path
    instead scans to the last good frame boundary and truncates a torn
    tail written by a crash mid-frame; it never refuses to start.
    """

    if not has_existing_frames:
        return RecoveryAction.START_EMPTY
    if tail_is_torn:
        return RecoveryAction.TRUNCATE_TORN_TAIL
    return RecoveryAction.REPLAY


def legacy_store_corruption_is_fatal() -> bool:
    """The unchanged `--store <file>` legacy JSON path keeps today's
    hard-fail-on-corruption behavior; this design does not touch it."""

    return True


def wal_tail_corruption_is_fatal() -> bool:
    """The new WAL path truncates a torn tail instead of refusing to
    start; see `recovery_action_for_wal_scan`."""

    return False


def should_snapshot_and_truncate(frames_since_last_snapshot: int, threshold: int) -> bool:
    """Bound WAL growth with a periodic snapshot + truncate cycle built on
    the existing `storage-durable` snapshot-store and log-truncation
    primitives. Snapshot content and the on-disk snapshot/backup wire
    format are unchanged by this design.
    """

    return frames_since_last_snapshot >= threshold


@dataclass(frozen=True)
class OutOfScopeBoundary:
    """One thing this design explicitly does not change."""

    area: str
    must_remain: str


def out_of_scope_boundaries() -> tuple[OutOfScopeBoundary, ...]:
    """The fixed boundary of this bounded change; tape-dev must not expand
    scope to cover these areas."""

    return (
        OutOfScopeBoundary(
            area="libs/storage-durable",
            must_remain="unmodified; consumed read-only via its existing public API",
        ),
        OutOfScopeBoundary(
            area="snapshot/backup wire format",
            must_remain="byte-for-byte unchanged",
        ),
        OutOfScopeBoundary(
            area="Raft-replicated multi-node path (TapeRaft, TapeStateMachine::apply)",
            must_remain=(
                "untouched at the call site; only gains the shared "
                "apply_command extraction"
            ),
        ),
        OutOfScopeBoundary(
            area="FsyncPolicy::Always",
            must_remain=(
                "never downgraded; group commit changes fsync cardinality, "
                "not the ack-after-fsync guarantee"
            ),
        ),
    )


class TrapKind(StrEnum):
    """Which direction one implementation trap points tape-dev in."""

    MUST_HANDLE = "must_handle"
    CONFIRM_DO_NOT_CHANGE = "confirm_do_not_change"


@dataclass(frozen=True)
class ImplementationTrap:
    """One named hazard tape-dev must resolve explicitly, not rediscover.
    `kind` distinguishes "this must change" from "this is already correct;
    the hazard is editing it anyway"."""

    name: str
    location: str
    kind: TrapKind
    hazard: str


def implementation_traps() -> tuple[ImplementationTrap, ...]:
    """Traps found while grounding this design against current source;
    each must be resolved in the direction `kind` names, not silently
    reintroduced or "fixed" in the wrong direction."""

    return (
        ImplementationTrap(
            name="data_dir_has_existing_state",
            location="apps/tape/src/raft.rs",
            kind=TrapKind.CONFIRM_DO_NOT_CHANGE,
            hazard=(
                "reads backwards on a skim: `data_dir_has_existing_state` "
                "(raft.rs:155-172) already returns true on the first "
                "directory entry that is not `lost+found` -- it recognizes "
                "no markers at all, so a new WAL/snapshot file already "
                "trips it with zero code changes needed. Its only caller is "
                "`prepare_bootstrap_seed`'s replica-mode cold-start refusal "
                "(raft.rs:192), and replica mode never has a WAL "
                "(JournalStoreKind.NONE), so 'refuse to seed over existing "
                "single-node data' stays correct as written. Do not edit "
                "this function: 'teaching' it to recognize WAL files would "
                "be narrowing a check that today refuses on anything to "
                "one that refuses only on recognized markers -- a "
                "regression dressed as a fix"
            ),
        ),
        ImplementationTrap(
            name="storage_full_probe filename collision",
            location="apps/tape/src/bin/tape.rs (spawn_storage_full_reprobe)",
            kind=TrapKind.MUST_HANDLE,
            hazard=(
                "the ENOSPC reprobe already writes a `.storage_full_probe` "
                "file under --data-dir; new WAL segment and snapshot "
                "filenames must be chosen so they cannot collide with it"
            ),
        ),
        ImplementationTrap(
            name="in-crate persist tests must be rewritten, not deleted",
            location="apps/tape/src/server.rs",
            kind=TrapKind.MUST_HANDLE,
            hazard=(
                "persist_failure_leaves_the_previous_journal_intact, "
                "persist_commits_by_rename_without_temp_residue, "
                "enospc_latches_degraded_mode_fast_fails_mutations_and_keeps_reads_serving, "
                "and leaving_degraded_mode_restores_mutations_without_a_restart "
                "each assert on whole-file-rewrite behavior; each must be "
                "re-expressed against the WAL/group-commit path, not "
                "dropped, or a durability regression ships silently"
            ),
        ),
    )


@dataclass(frozen=True)
class PerformanceGateContract:
    """The performance gate this design requires; today's `bench.rs` is
    vacuous (it benchmarks an in-memory-only journal and never touches
    disk, so it cannot regress-test durability throughput).

    The required ratio is a *scaling* assertion -- throughput at
    `scaled_connections` versus `baseline_connections` on the same build --
    not an old-versus-new improvement assertion; a bare "improvement
    ratio" name would read as the latter, which is a different and much
    easier property (it is satisfiable by making the 1-connection baseline
    worse, not by amortising the barrier). This matches accepted EC
    `ec-3052-scaling`'s `BASELINE_CONNECTIONS`/`SCALED_CONNECTIONS`/
    `REQUIRED_RATIO`.
    """

    baseline_ops_per_sec_low: float
    baseline_ops_per_sec_high: float
    baseline_connections: int
    scaled_connections: int
    required_scaled_over_baseline_connection_throughput_ratio: float


def performance_gate_contract() -> PerformanceGateContract:
    """Recorded single-node baseline and the required durable-path gate."""

    return PerformanceGateContract(
        baseline_ops_per_sec_low=85.0,
        baseline_ops_per_sec_high=89.0,
        baseline_connections=1,
        scaled_connections=16,
        required_scaled_over_baseline_connection_throughput_ratio=4.0,
    )


def benchmark_is_durable_and_not_vacuous(
    uses_real_disk_io: bool, bypasses_persist: bool
) -> bool:
    """The rewritten performance gate (apps/tape/tests/tape_perf_gate.rs,
    apps/tape/src/bench.rs) must exercise real durable I/O through the
    group-commit path; a benchmark that bypasses `AppState::persist` (or
    its WAL successor) cannot prove the fix."""

    return uses_real_disk_io and not bypasses_persist


@dataclass(frozen=True)
class DesignInvariant:
    """One structural invariant this design restates for legibility; NOT
    an acceptance criterion. Kept under its own name and function so it
    cannot be confused with WI #3052's actual, numbered acceptance surface
    below (`AcceptanceCriterion` / `acceptance_criteria`)."""

    id: str
    statement: str


def design_invariants() -> tuple[DesignInvariant, ...]:
    """Design-level restatements, useful as a reading aid; the bounded
    *acceptance* surface for this change is `acceptance_criteria()`."""

    return (
        DesignInvariant(
            id="DI1",
            statement=(
                "`tape serve --data-dir` mutations commit through an "
                "append-only WAL + group commit; `--store <file>` keeps "
                "today's whole-file JSON behavior unchanged"
            ),
        ),
        DesignInvariant(
            id="DI2",
            statement=(
                "WAL frames encode TapeCommand values and replay through "
                "the shared apply_command extraction, not raw journal state"
            ),
        ),
        DesignInvariant(
            id="DI3",
            statement=(
                "one fsync covers one batch of commands (group commit); no "
                "request is acknowledged before its covering fsync returns"
            ),
        ),
        DesignInvariant(
            id="DI4",
            statement=(
                "FIFO submission order is preserved end-to-end through "
                "batch drain, on-disk commit, and single-lock-scope apply"
            ),
        ),
        DesignInvariant(
            id="DI5",
            statement=(
                "an ENOSPC or EIO durability failure latches the existing "
                "sticky storage-degraded mode and 507 storage_full "
                "response; a batch failure that is neither fails closed "
                "without a silent ack"
            ),
        ),
        DesignInvariant(
            id="DI6",
            statement=(
                "startup recovery truncates a torn WAL tail instead of "
                "hard-failing, unlike today's load_journal"
            ),
        ),
        DesignInvariant(
            id="DI7",
            statement=(
                "periodic snapshot + truncate bounds WAL growth using "
                "existing storage-durable primitives; snapshot/backup wire "
                "format is unchanged"
            ),
        ),
        DesignInvariant(
            id="DI8",
            statement=(
                "the performance gate exercises real durable I/O and "
                "asserts a throughput ratio between scaled and baseline "
                "connection counts, replacing the vacuous in-memory bench"
            ),
        ),
    )


@dataclass(frozen=True)
class AcceptanceCriterion:
    """One of WI #3052's own acceptance criteria, carried with the WI's
    own id, the WI's own measurable statement, and where it is verified.
    Distinct from `DesignInvariant`: these are outcome measurements the
    implementation is graded against, not a restatement of the design."""

    id: str
    statement: str
    verified_by: str


def acceptance_criteria() -> tuple[AcceptanceCriterion, ...]:
    """WI #3052's AC1-AC8, unchanged from the work item. Two are only
    partially covered by the two accepted EC cases (`ec-3052-scaling`,
    `ec-3052-durability`); the rest have no EC case at all and must be
    added as new Rust-level tests -- `ec-3052-durability-under-sigkill`'s
    own docstring hands AC7 to the TD explicitly ("the TD owes it a new
    home ... in scope for #3052 and is not verified by any contract in
    this file"), and AC2/AC3/AC6/AC8 are simply outside both EC cases'
    scope."""

    return (
        AcceptanceCriterion(
            id="AC1",
            statement=(
                "durable append throughput rises with connection count "
                "instead of staying flat, measured with the same harness "
                "that produced the 85-89 ops/s flat line"
            ),
            verified_by=(
                "EC ec-3052-scaling: scaled/baseline ratio >= "
                "REQUIRED_RATIO plus the lone-writer floor and the "
                "barrier-cost ceiling"
            ),
        ),
        AcceptanceCriterion(
            id="AC2",
            statement=(
                "median read latency under 4 concurrent durable writers is "
                "within one order of magnitude of the 0.11 ms idle "
                "baseline (currently 45.13 ms, 400x)"
            ),
            verified_by=(
                "new Rust-level read-latency-under-concurrent-writers "
                "test; not covered by either accepted EC case"
            ),
        ),
        AcceptanceCriterion(
            id="AC3",
            statement=(
                "RSS / logical payload on the durable path is within the "
                "same order of magnitude as the in-memory path's 1.6x "
                "(currently 43.4x)"
            ),
            verified_by=(
                "new Rust-level memory-amplification test; not covered by "
                "either accepted EC case"
            ),
        ),
        AcceptanceCriterion(
            id="AC4",
            statement=(
                "a SIGKILL-then-restart test recovers every acknowledged "
                "append and no unacknowledged one"
            ),
            verified_by=(
                "EC ec-3052-durability, on the achievable half of the "
                "wording: every acknowledged append survives, and no "
                "append the server explicitly refused comes back; that "
                "EC's own docstring explains why the literal 'no "
                "unacknowledged one' half is not achievable by any "
                "correct implementation (a barrier can succeed after the "
                "response fails to write)"
            ),
        ),
        AcceptanceCriterion(
            id="AC5",
            statement=(
                "a test that appends a deliberately truncated final frame "
                "recovers all prior records and drops only the torn one"
            ),
            verified_by=(
                "new Rust-level torn-tail recovery test exercising "
                "recovery_action_for_wal_scan's TRUNCATE_TORN_TAIL branch "
                "against a real on-disk WAL"
            ),
        ),
        AcceptanceCriterion(
            id="AC6",
            statement=(
                "GET /admin/backup on a journal built through the new "
                "path is byte-identical to the same journal built through "
                "the old path"
            ),
            verified_by=(
                "new Rust-level round-trip test; this is WI #3052's "
                "Required Closure (R7) and is not covered by either "
                "accepted EC case"
            ),
        ),
        AcceptanceCriterion(
            id="AC7",
            statement=(
                "a fault-injected ENOSPC on the durable path yields 507 "
                "and sticky degraded read-only, not a silent success"
            ),
            verified_by=(
                "new Rust-level fault-injection test at the commit-"
                "coordinator boundary, replacing today's injection inside "
                "the synchronous persist this design deletes "
                "(server.rs:1671-1714); EC ec-3052-durability's docstring "
                "explicitly excludes AC7 and hands it to the TD, and R6 "
                "additionally requires the same treatment for EIO"
            ),
        ),
        AcceptanceCriterion(
            id="AC8",
            statement="cargo test -p tape passes",
            verified_by="cargo test -p tape (aw.toml test_cmd)",
        ),
    )


def design_contract() -> str:
    """Name this bounded change and its non-expansion boundary."""

    return (
        "Replace AppState::persist's per-request whole-file journal "
        "rewrite with an append-only, TapeCommand-encoded write-ahead log "
        "plus group commit, reusing the existing TapeCommand/apply_command "
        "vocabulary and storage-durable primitives; keep the legacy --store "
        "path, the Raft-replicated path, and the snapshot/backup wire "
        "format unchanged."
    )
