# lumen — high availability & replication

## Decision: lumen owns replication in multi-pod mode

lumen is a **log-replicated, derived, rebuildable search index**: the caller
still owns the source of truth, and lumen indexes the caller's `external_id`s`.
The deployment boundary changed now that `libs/raft-core` exists: multi-pod
lumen should own its own write ordering and replica synchronization instead of
requiring an external broker as the default HA path.

The mode split is:

- **standalone**: one pod, embedded WAL, direct apply.
- **primary-replica**: multiple lumen pods, `raft-core` elects a leader, the
  leader owns the ordered write log, and followers replicate/apply the same raw
  `WalRecord::encode()` bytes.

lumen remains rebuildable: the upstream source of truth can replay documents.
But in primary-replica mode, a write acknowledged by lumen must be ordered by
lumen's own replication layer, not by Relay/NATS.

### Current HA runtime shape

`lumen serve --wal auto` is the production default. It starts embedded when no
k8s replica topology is present, and switches to raft when
`REPLICAS_PER_SHARD > 1` is injected by the operator/StatefulSet. Operators do
not pass special cluster flags; the topology comes from the downward API:
`POD_NAME`, `POD_NAMESPACE`, `SHARD_COUNT`, `REPLICAS_PER_SHARD`,
`VOTER_COUNT`, and `LUMEN_HEADLESS_SERVICE`.

For local multi-node work, `LUMEN_PEERS=host:port,...` overrides headless DNS so
several `lumen serve --wal raft` processes can run on one machine.

The raft implementation is split by responsibility:

- `libs/raft-core`: consensus state machine and log semantics.
- `libs/raft-host`: h2c peer transport, leader forwarding, snapshot install,
  and log compaction.
- `projects/lumen/src/raft_sm.rs`: converts committed Lumen write records into
  engine mutations and produces/restores snapshots.
- `projects/lumen/src/raft.rs`: API-facing cluster/debug DTOs and read
  consistency parsing.

Legacy broker-backed write logs are not part of the Lumen deployment archetype.
The old NATS backend remains compatibility/test surface; Relay WAL support was
removed from Lumen.

## Status

- ✅ Multi-pod Lumen owns replication through `libs/raft-core` +
  `libs/raft-host`.
- ✅ Operator auto mode: one serving pod renders standalone Deployment + HPA;
  `replicasPerShard > 1` renders a stable serving StatefulSet + headless
  Service.
- ✅ Snapshot/compaction is owned by `raft-host`; upload policy/sinks live in
  `libs/service-backup`.
