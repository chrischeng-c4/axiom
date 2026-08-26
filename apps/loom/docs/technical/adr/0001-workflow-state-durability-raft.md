# ADR 0001 — Workflow-state durability: per-shard Raft over `RunStore` (#110 / #123)

Status: **Accepted** (design decision; single-node crash recovery shipped, multi-node HA is the implementation follow-on).

## Context

loom owns its workflow/DAG state and is the one component that must not lose it:
a lost `WorkflowRun` means orphaned in-flight work on relay/keep. State is
naturally **sharded by `wf_id`** (each run is independent; the [`RunStore`]
trait already hides the backend). We need (a) crash recovery on a single node
and (b) strongly-consistent HA across replicas, without coupling the scheduler
or control API to the storage mechanism.

`libs/raftcore` is the in-house consensus core, already powering relay and
lumen. It is **step-driven**: `RaftNode::{tick, handle(RaftMsg), take_outgoing,
take_committed, compact, take_installed_snapshot}` — no timers/threads of its
own; a driver advances it. `auto_membership(n)` yields the largest **odd**
voter prefix (1,1,3,3,5… → clean majorities); `n=1` is a single-voter group.
relay's file-backed `RaftStore` persists `PersistedState` (term/votedFor/log,
fsync per policy).

## Decision

1. **Durability lives behind `RunStore`.** The scheduler, control API, and
   completion consumer keep using `RunStore::{put,get,list}`; nothing above the
   trait knows whether the backend is memory, a file, or Raft.

2. **Shard by `wf_id`; one Raft group per shard.** Each shard is an independent
   raft group whose replicated state machine is that shard's
   `BTreeMap<WorkflowRunId, WorkflowRun>`. Cross-shard coordination is never
   needed (runs are independent), so this scales horizontally and keeps each
   group small.

3. **Single-voter is the degenerate case, not a special path.** `n=1` uses
   `auto_membership(1)` → one voter → every write commits locally and is fsynced
   to the Raft log: that **is** crash recovery (#123). The shipped
   [`store::FileStore`] is the pragmatic single-node form (atomic file + reload
   on open); the Raft single-voter group supersedes it once the driver lands,
   with the same durability guarantee and a path to HA by adding voters.

4. **`RaftRunStore` is a Raft state machine over raftcore.**
   - `put(run)` → serialize a `PutRun(run)` command, `propose` it to the shard's
     `RaftNode`.
   - A step-driven **driver** (mirroring relay's `raft_driver`) loops:
     `tick` on a logical clock, `handle` inbound `RaftMsg` from peers, drain
     `take_outgoing` to the transport, **persist via relay-style `RaftStore`
     before sending** (hard state always durable), and `take_committed` →
     apply each `PutRun` to the in-memory shard map. Snapshots: `compact` past
     the applied index; ship the shard map as the snapshot payload.
   - `get/list` read the applied in-memory map (linearizable reads can route
     through the leader / a read-index later; eventually-consistent local reads
     are fine for status).
   - Transport: HTTP/2 (h2c) peer endpoints, consistent with the rest of the
     stack; identity/peers from the k8s downward API (as lumen/relay do).

5. **HA membership** uses `auto_membership(replica_count)`; a `loom` StatefulSet
   gives stable peer identities. Leadership + redelivery already tolerate
   restarts because relay owns lease/redelivery and loom rebuilds the DAG from
   committed state on leader change.

## Consequences

- One mechanism (`RunStore`) spans dev (Mem), single-node durable (File / Raft
  n=1), and HA (Raft n≥3) — no scheduler/API change between them.
- raftcore's step-driven model keeps loom testable (drive ticks/messages in a
  test, no real time/threads), matching how relay/lumen test their raft paths.
- The implementation cost is the driver + transport + snapshot wiring (the
  relay `raft_driver`/`raft_store`/`replication` trio is the reference); it is
  the remaining work, not a new decision.

## Status of work

- ✅ #123 single-node crash recovery — `FileStore` (atomic write + reload).
- ▶ #110 Raft `RunStore` — implement `RaftRunStore` + driver per this ADR
  (single-voter first = same guarantee as FileStore, then HA via membership).
