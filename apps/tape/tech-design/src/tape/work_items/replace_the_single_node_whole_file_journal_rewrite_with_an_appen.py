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


def should_enter_storage_degraded_mode(outcome: CommitOutcome, is_enospc: bool) -> bool:
    """Only a durability failure caused by ENOSPC latches sticky degraded
    mode via the existing `TapeMetrics::mark_storage_degraded` (apps/tape/
    src/metrics.rs) and its 507 `storage_full` response envelope. A
    transient, non-ENOSPC batch failure fails that one batch closed but
    does not flip the server into sticky read-only degraded mode.
    """

    return outcome is not CommitOutcome.ACKED and is_enospc


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


@dataclass(frozen=True)
class ImplementationTrap:
    """One named hazard tape-dev must handle explicitly, not rediscover."""

    name: str
    location: str
    hazard: str


def implementation_traps() -> tuple[ImplementationTrap, ...]:
    """Traps found while grounding this design against current source;
    each must be handled, not silently reintroduced."""

    return (
        ImplementationTrap(
            name="data_dir_has_existing_state",
            location="apps/tape/src/raft.rs",
            hazard=(
                "bootstrap-vs-join detection currently only recognizes "
                "Raft/snapshot markers under --data-dir; it must also "
                "recognize a non-empty WAL/snapshot as existing state, or "
                "a restart re-bootstraps and silently discards it"
            ),
        ),
        ImplementationTrap(
            name="storage_full_probe filename collision",
            location="apps/tape/src/bin/tape.rs (spawn_storage_full_reprobe)",
            hazard=(
                "the ENOSPC reprobe already writes a `.storage_full_probe` "
                "file under --data-dir; new WAL segment and snapshot "
                "filenames must be chosen so they cannot collide with it"
            ),
        ),
        ImplementationTrap(
            name="in-crate persist tests must be rewritten, not deleted",
            location="apps/tape/src/server.rs",
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
    disk, so it cannot regress-test durability throughput)."""

    baseline_ops_per_sec_low: float
    baseline_ops_per_sec_high: float
    min_improvement_ratio: float


def performance_gate_contract() -> PerformanceGateContract:
    """Recorded single-node baseline and the required durable-path gate."""

    return PerformanceGateContract(
        baseline_ops_per_sec_low=85.0,
        baseline_ops_per_sec_high=89.0,
        min_improvement_ratio=3.0,
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
class AcceptanceCriterion:
    """One acceptance criterion for WI #3052, identified by a stable id."""

    id: str
    statement: str


def acceptance_criteria() -> tuple[AcceptanceCriterion, ...]:
    """The bounded acceptance surface for this change."""

    return (
        AcceptanceCriterion(
            id="AC1",
            statement=(
                "`tape serve --data-dir` mutations commit through an "
                "append-only WAL + group commit; `--store <file>` keeps "
                "today's whole-file JSON behavior unchanged"
            ),
        ),
        AcceptanceCriterion(
            id="AC2",
            statement=(
                "WAL frames encode TapeCommand values and replay through "
                "the shared apply_command extraction, not raw journal state"
            ),
        ),
        AcceptanceCriterion(
            id="AC3",
            statement=(
                "one fsync covers one batch of commands (group commit); no "
                "request is acknowledged before its covering fsync returns"
            ),
        ),
        AcceptanceCriterion(
            id="AC4",
            statement=(
                "FIFO submission order is preserved end-to-end through "
                "batch drain, on-disk commit, and single-lock-scope apply"
            ),
        ),
        AcceptanceCriterion(
            id="AC5",
            statement=(
                "an ENOSPC durability failure latches the existing sticky "
                "storage-degraded mode and 507 storage_full response; a "
                "non-ENOSPC batch failure fails closed without a silent ack"
            ),
        ),
        AcceptanceCriterion(
            id="AC6",
            statement=(
                "startup recovery truncates a torn WAL tail instead of "
                "hard-failing, unlike today's load_journal"
            ),
        ),
        AcceptanceCriterion(
            id="AC7",
            statement=(
                "periodic snapshot + truncate bounds WAL growth using "
                "existing storage-durable primitives; snapshot/backup wire "
                "format is unchanged"
            ),
        ),
        AcceptanceCriterion(
            id="AC8",
            statement=(
                "the performance gate exercises real durable I/O and "
                "asserts a measurable throughput improvement ratio over "
                "the recorded 85-89 ops/s baseline, replacing the vacuous "
                "in-memory bench"
            ),
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
